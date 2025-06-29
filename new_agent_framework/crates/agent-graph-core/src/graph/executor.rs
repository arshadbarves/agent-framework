//! Graph execution engine.

use crate::error::{CoreError, CoreResult};
use crate::graph::Graph;
use crate::node::NodeId;
use crate::runtime::{ExecutionContext, ExecutionConfig};
use crate::state::State;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::Duration;
use tracing::{info, debug, error, warn};
use uuid::Uuid;

/// Graph executor for running graph execution
#[derive(Debug)]
pub struct GraphExecutor<'a, S>
where
    S: State,
{
    graph: &'a Graph<S>,
    context: ExecutionContext,
}

impl<'a, S> GraphExecutor<'a, S>
where
    S: State,
{
    /// Create a new graph executor
    pub fn new(graph: &'a Graph<S>, context: ExecutionContext) -> Self {
        Self { graph, context }
    }

    /// Execute the graph
    pub async fn execute(self) -> CoreResult<ExecutionResult> {
        let start_time = std::time::Instant::now();
        let mut executed_nodes = Vec::new();
        let mut node_results = HashMap::new();

        info!("Starting graph execution: {}", self.context.execution_id);

        // Validate graph before execution
        self.graph.validate()?;

        // Check if we have entry points
        if self.graph.entry_points.is_empty() {
            return Err(CoreError::validation_error("Graph has no entry points"));
        }

        // Initialize execution queue with entry points
        let mut execution_queue = VecDeque::new();
        let mut completed_nodes = std::collections::HashSet::new();

        for entry_point in &self.graph.entry_points {
            execution_queue.push_back(entry_point.clone());
        }

        // Execute nodes sequentially (parallel execution is in the execution crate)
        while let Some(node_id) = execution_queue.pop_front() {
            // Skip if already executed
            if completed_nodes.contains(&node_id) {
                continue;
            }

            // Check timeout
            if self.context.should_timeout() {
                warn!("Graph execution timed out");
                let duration = start_time.elapsed();
                let metrics = self.context.get_metrics();
                
                return Ok(ExecutionResult::timeout(
                    self.context.execution_id,
                    duration,
                    metrics,
                    executed_nodes,
                ));
            }

            // Execute the node
            match self.execute_node(&node_id).await {
                Ok(output) => {
                    debug!("Node '{}' executed successfully", node_id);
                    executed_nodes.push(node_id.clone());
                    completed_nodes.insert(node_id.clone());
                    node_results.insert(node_id.clone(), output.clone());

                    // Update context metrics
                    self.context.update_metrics(|metrics| {
                        metrics.record_node_execution(Duration::from_millis(output.metrics.duration_ms), output.success);
                    });

                    // Determine next nodes
                    if output.continue_execution {
                        let next_nodes = if let Some(next_node) = output.next_node {
                            vec![next_node]
                        } else {
                            // Find outgoing edges and add target nodes
                            self.graph.edges.get_outgoing_edges(&node_id)
                                .iter()
                                .map(|edge| edge.to.clone())
                                .collect()
                        };

                        // Add next nodes to queue if not already completed
                        for next_node in next_nodes {
                            if !completed_nodes.contains(&next_node) && !execution_queue.contains(&next_node) {
                                execution_queue.push_back(next_node);
                            }
                        }
                    }
                }
                Err(error) => {
                    error!("Node '{}' execution failed: {}", node_id, error);
                    let duration = start_time.elapsed();
                    let metrics = self.context.get_metrics();
                    
                    return Ok(ExecutionResult::failure(
                        self.context.execution_id,
                        error.to_string(),
                        duration,
                        metrics,
                        executed_nodes,
                    ));
                }
            }
        }

        // Execution completed successfully
        let duration = start_time.elapsed();
        let metrics = self.context.get_metrics();
        
        // Get final state
        let final_state = self.graph.state_manager.read_state(|state| {
            state.to_json().ok()
        });

        info!("Graph execution completed successfully in {:?}", duration);

        Ok(ExecutionResult::success(
            self.context.execution_id,
            final_state,
            duration,
            metrics,
            executed_nodes,
            node_results,
        ))
    }

    /// Execute a single node
    async fn execute_node(&self, node_id: &NodeId) -> CoreResult<crate::node::NodeOutput> {
        debug!("Executing node: {}", node_id);

        // Get the node
        let node = self.graph.nodes.get(node_id)
            .ok_or_else(|| CoreError::execution_error(format!("Node '{}' not found", node_id)))?;

        // Pre-execution hook
        self.graph.state_manager.read_state(|state| {
            node.before_execute(state)
        }).await?;

        // Execute the node
        let result = self.graph.state_manager.write_state(|state| {
            node.execute(state)
        }).await?;

        // Post-execution hook
        self.graph.state_manager.read_state(|state| {
            node.after_execute(state, &result)
        }).await?;

        // Validate state if enabled
        if self.context.is_validation_enabled() {
            self.graph.state_manager.validate_state()?;
        }

        Ok(result)
    }
}

/// Result of graph execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Execution ID
    pub execution_id: Uuid,
    /// Whether execution was successful
    pub success: bool,
    /// Final state (if successful)
    pub final_state: Option<serde_json::Value>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Execution duration
    pub duration: Duration,
    /// Execution metrics
    pub metrics: crate::runtime::ExecutionMetrics,
    /// Nodes that were executed
    pub executed_nodes: Vec<NodeId>,
    /// Individual node results
    pub node_results: HashMap<NodeId, crate::node::NodeOutput>,
    /// Execution status
    pub status: ExecutionStatus,
}

impl ExecutionResult {
    /// Create a successful result
    pub fn success(
        execution_id: Uuid,
        final_state: Option<serde_json::Value>,
        duration: Duration,
        metrics: crate::runtime::ExecutionMetrics,
        executed_nodes: Vec<NodeId>,
        node_results: HashMap<NodeId, crate::node::NodeOutput>,
    ) -> Self {
        Self {
            execution_id,
            success: true,
            final_state,
            error: None,
            duration,
            metrics,
            executed_nodes,
            node_results,
            status: ExecutionStatus::Completed,
        }
    }

    /// Create a failed result
    pub fn failure(
        execution_id: Uuid,
        error: String,
        duration: Duration,
        metrics: crate::runtime::ExecutionMetrics,
        executed_nodes: Vec<NodeId>,
    ) -> Self {
        Self {
            execution_id,
            success: false,
            final_state: None,
            error: Some(error),
            duration,
            metrics,
            executed_nodes,
            node_results: HashMap::new(),
            status: ExecutionStatus::Failed,
        }
    }

    /// Create a timeout result
    pub fn timeout(
        execution_id: Uuid,
        duration: Duration,
        metrics: crate::runtime::ExecutionMetrics,
        executed_nodes: Vec<NodeId>,
    ) -> Self {
        Self {
            execution_id,
            success: false,
            final_state: None,
            error: Some("Execution timed out".to_string()),
            duration,
            metrics,
            executed_nodes,
            node_results: HashMap::new(),
            status: ExecutionStatus::TimedOut,
        }
    }

    /// Get execution summary
    pub fn summary(&self) -> ExecutionSummary {
        ExecutionSummary {
            execution_id: self.execution_id,
            success: self.success,
            duration: self.duration,
            nodes_executed: self.executed_nodes.len(),
            nodes_succeeded: self.metrics.nodes_succeeded,
            nodes_failed: self.metrics.nodes_failed,
            status: self.status.clone(),
        }
    }
}

/// Execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionStatus {
    /// Execution completed successfully
    Completed,
    /// Execution failed with an error
    Failed,
    /// Execution timed out
    TimedOut,
    /// Execution was cancelled
    Cancelled,
}

/// Execution summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSummary {
    /// Execution ID
    pub execution_id: Uuid,
    /// Whether execution was successful
    pub success: bool,
    /// Execution duration
    pub duration: Duration,
    /// Number of nodes executed
    pub nodes_executed: usize,
    /// Number of nodes that succeeded
    pub nodes_succeeded: usize,
    /// Number of nodes that failed
    pub nodes_failed: usize,
    /// Execution status
    pub status: ExecutionStatus,
}

impl ExecutionSummary {
    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        if self.nodes_executed == 0 {
            0.0
        } else {
            (self.nodes_succeeded as f64 / self.nodes_executed as f64) * 100.0
        }
    }
}

/// Extension trait for Graph to add execution methods
pub trait GraphExecution<S>
where
    S: State,
{
    /// Execute the graph with default configuration
    async fn execute(&self) -> CoreResult<ExecutionResult>;

    /// Execute the graph with custom configuration
    async fn execute_with_config(&self, config: ExecutionConfig) -> CoreResult<ExecutionResult>;

    /// Execute the graph with custom context
    async fn execute_with_context(&self, context: ExecutionContext) -> CoreResult<ExecutionResult>;
}

impl<S> GraphExecution<S> for Graph<S>
where
    S: State,
{
    async fn execute(&self) -> CoreResult<ExecutionResult> {
        let config = ExecutionConfig::default();
        self.execute_with_config(config).await
    }

    async fn execute_with_config(&self, config: ExecutionConfig) -> CoreResult<ExecutionResult> {
        let context = ExecutionContext::new(config);
        self.execute_with_context(context).await
    }

    async fn execute_with_context(&self, context: ExecutionContext) -> CoreResult<ExecutionResult> {
        let executor = GraphExecutor::new(self, context);
        executor.execute().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::GraphBuilder;
    use crate::node::{NodeOutput, NodeMetadata};
    use async_trait::async_trait;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestState {
        value: i32,
        log: Vec<String>,
    }

    struct TestNode {
        id: String,
        metadata: NodeMetadata,
        increment: i32,
    }

    #[async_trait]
    impl crate::node::Node<TestState> for TestNode {
        async fn execute(&self, state: &mut TestState) -> CoreResult<NodeOutput> {
            state.value += self.increment;
            state.log.push(format!("Executed {}", self.id));
            Ok(NodeOutput::success())
        }

        fn id(&self) -> &str {
            &self.id
        }

        fn metadata(&self) -> &NodeMetadata {
            &self.metadata
        }
    }

    #[tokio::test]
    async fn test_graph_execution() {
        let initial_state = TestState {
            value: 0,
            log: Vec::new(),
        };

        let node1 = TestNode {
            id: "node1".to_string(),
            metadata: NodeMetadata::new("Test Node 1".to_string()),
            increment: 5,
        };

        let graph = GraphBuilder::new()
            .with_initial_state(initial_state)
            .add_node("node1".to_string(), node1)
            .add_entry_point("node1".to_string())
            .build()
            .unwrap();

        let result = graph.execute().await.unwrap();

        assert!(result.success);
        assert_eq!(result.executed_nodes.len(), 1);
        assert_eq!(result.executed_nodes[0], "node1");
        assert_eq!(result.status, ExecutionStatus::Completed);

        // Check final state
        graph.state_manager.read_state(|state| {
            assert_eq!(state.value, 5);
            assert_eq!(state.log.len(), 1);
        });
    }
}