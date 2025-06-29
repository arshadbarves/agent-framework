//! Parallel execution engine for concurrent node processing.

use crate::{CoreError, CoreResult, State, Node, NodeId};
use agent_graph_core::{Graph, ExecutionContext};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{mpsc, Semaphore, RwLock};
use tokio::task::JoinHandle;
use tracing::{info, warn, error, debug};

/// Parallel execution engine for running nodes concurrently
#[derive(Debug)]
pub struct ParallelExecutor<S>
where
    S: State,
{
    config: ParallelConfig,
    semaphore: Arc<Semaphore>,
    active_tasks: Arc<RwLock<HashMap<NodeId, JoinHandle<CoreResult<ExecutionResult>>>>>,
}

impl<S> ParallelExecutor<S>
where
    S: State + Send + Sync + 'static,
{
    /// Create a new parallel executor
    pub fn new(config: ParallelConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_nodes));
        
        Self {
            config,
            semaphore,
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Execute a graph with parallel processing
    pub async fn execute_graph(
        &self,
        graph: &Graph<S>,
        context: ExecutionContext,
    ) -> CoreResult<ParallelExecutionResult> {
        info!("Starting parallel execution with max {} concurrent nodes", 
              self.config.max_concurrent_nodes);

        // Build dependency graph
        let dep_graph = super::dependency::DependencyGraph::from_graph(graph)?;
        
        // Find nodes ready for execution (no dependencies)
        let mut ready_nodes = dep_graph.get_ready_nodes();
        let mut completed_nodes = HashSet::new();
        let mut failed_nodes = HashSet::new();
        let mut execution_results = HashMap::new();
        
        let (result_tx, mut result_rx) = mpsc::unbounded_channel();
        
        // Execute nodes in parallel
        while !ready_nodes.is_empty() || !self.active_tasks.read().await.is_empty() {
            // Start execution for ready nodes
            for node_id in ready_nodes.drain(..) {
                if completed_nodes.contains(&node_id) || failed_nodes.contains(&node_id) {
                    continue;
                }
                
                self.execute_node_async(
                    graph,
                    &node_id,
                    &context,
                    result_tx.clone(),
                ).await?;
            }
            
            // Wait for at least one task to complete
            if let Some(result) = result_rx.recv().await {
                let node_id = result.node_id.clone();
                
                match result.result {
                    Ok(exec_result) => {
                        info!("Node '{}' completed successfully", node_id);
                        completed_nodes.insert(node_id.clone());
                        execution_results.insert(node_id.clone(), exec_result);
                        
                        // Find newly ready nodes
                        let newly_ready = dep_graph.mark_completed(&node_id);
                        ready_nodes.extend(newly_ready);
                    }
                    Err(error) => {
                        error!("Node '{}' failed: {}", node_id, error);
                        failed_nodes.insert(node_id.clone());
                        
                        if self.config.fail_fast {
                            // Cancel all running tasks
                            self.cancel_all_tasks().await;
                            return Err(error);
                        }
                    }
                }
                
                // Remove completed task
                self.active_tasks.write().await.remove(&node_id);
            }
        }
        
        // Check if execution was successful
        let success = failed_nodes.is_empty();
        let total_nodes = completed_nodes.len() + failed_nodes.len();
        
        info!("Parallel execution completed. Success: {}, Total nodes: {}, Failed: {}", 
              success, total_nodes, failed_nodes.len());
        
        Ok(ParallelExecutionResult {
            success,
            completed_nodes: completed_nodes.into_iter().collect(),
            failed_nodes: failed_nodes.into_iter().collect(),
            execution_results,
            total_execution_time: context.duration(),
        })
    }

    /// Execute a single node asynchronously
    async fn execute_node_async(
        &self,
        graph: &Graph<S>,
        node_id: &NodeId,
        context: &ExecutionContext,
        result_tx: mpsc::UnboundedSender<NodeExecutionResult>,
    ) -> CoreResult<()> {
        let permit = self.semaphore.clone().acquire_owned().await
            .map_err(|e| CoreError::internal(format!("Failed to acquire semaphore: {}", e)))?;
        
        let node = graph.nodes.get(node_id)
            .ok_or_else(|| CoreError::execution_error(format!("Node '{}' not found", node_id)))?;
        
        let state_manager = graph.state_manager.clone();
        let node_id_clone = node_id.clone();
        let context_clone = context.clone();
        
        let task = tokio::spawn(async move {
            let _permit = permit; // Keep permit alive
            
            debug!("Executing node '{}'", node_id_clone);
            let start_time = std::time::Instant::now();
            
            let result = state_manager.write_state(|state| {
                node.execute(state)
            }).await;
            
            let duration = start_time.elapsed();
            let success = result.is_ok();

            // Update context metrics
            context_clone.update_metrics(|metrics| {
                metrics.record_node_execution(duration, success);
            });
            
            match result {
                Ok(output) => {
                    Ok(ExecutionResult {
                        node_id: node_id_clone.clone(),
                        success: output.success,
                        duration,
                        output: Some(output),
                        error: None,
                    })
                }
                Err(error) => {
                    Err(error)
                }
            }
        });
        
        // Store the task handle
        self.active_tasks.write().await.insert(node_id.clone(), task);
        
        // Send result when task completes
        let node_id_for_result = node_id.clone();
        let active_tasks = self.active_tasks.clone();
        
        tokio::spawn(async move {
            if let Some(task) = active_tasks.write().await.remove(&node_id_for_result) {
                let result = task.await;
                
                let execution_result = match result {
                    Ok(Ok(exec_result)) => NodeExecutionResult {
                        node_id: node_id_for_result,
                        result: Ok(exec_result),
                    },
                    Ok(Err(error)) => NodeExecutionResult {
                        node_id: node_id_for_result,
                        result: Err(error),
                    },
                    Err(join_error) => NodeExecutionResult {
                        node_id: node_id_for_result,
                        result: Err(CoreError::internal(format!("Task join error: {}", join_error))),
                    },
                };
                
                let _ = result_tx.send(execution_result);
            }
        });
        
        Ok(())
    }

    /// Cancel all active tasks
    async fn cancel_all_tasks(&self) {
        let mut tasks = self.active_tasks.write().await;
        for (node_id, task) in tasks.drain() {
            warn!("Cancelling task for node '{}'", node_id);
            task.abort();
        }
    }
}

/// Configuration for parallel execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelConfig {
    /// Maximum number of nodes to execute concurrently
    pub max_concurrent_nodes: usize,
    /// Whether to stop execution on first failure
    pub fail_fast: bool,
    /// Timeout for individual node execution
    pub node_timeout: std::time::Duration,
    /// Enable resource monitoring
    pub monitor_resources: bool,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            max_concurrent_nodes: 10,
            fail_fast: true,
            node_timeout: std::time::Duration::from_secs(300), // 5 minutes
            monitor_resources: true,
        }
    }
}

/// Result of parallel execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelExecutionResult {
    /// Whether execution was successful
    pub success: bool,
    /// Nodes that completed successfully
    pub completed_nodes: Vec<NodeId>,
    /// Nodes that failed
    pub failed_nodes: Vec<NodeId>,
    /// Individual execution results
    pub execution_results: HashMap<NodeId, ExecutionResult>,
    /// Total execution time
    pub total_execution_time: std::time::Duration,
}

/// Result of individual node execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Node ID
    pub node_id: NodeId,
    /// Whether execution was successful
    pub success: bool,
    /// Execution duration
    pub duration: std::time::Duration,
    /// Node output (if successful)
    pub output: Option<agent_graph_core::node::NodeOutput>,
    /// Error message (if failed)
    pub error: Option<String>,
}

/// Internal result type for node execution
#[derive(Debug)]
struct NodeExecutionResult {
    node_id: NodeId,
    result: CoreResult<ExecutionResult>,
}