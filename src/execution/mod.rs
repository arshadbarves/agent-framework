// Advanced graph execution engine for AgentGraph
// Provides parallel processing, conditional routing, and state management

#![allow(missing_docs)]

use crate::graph::Graph;
use crate::node::{Node, NodeId};
use crate::state::StateManager;
use serde_json::Value as JsonValue;

// Type alias for execution state
pub type ExecutionState = JsonValue;
use crate::edge::{Edge, EdgeCondition};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use thiserror::Error;
use tokio::sync::{mpsc, RwLock, Semaphore};
use tokio::time::timeout;

pub mod parallel;
pub mod scheduler;
pub mod checkpoint;
pub mod streaming;

/// Execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    /// Maximum concurrent node executions
    pub max_concurrency: usize,
    /// Execution timeout per node
    pub node_timeout: Duration,
    /// Overall execution timeout
    pub total_timeout: Duration,
    /// Enable parallel execution
    pub parallel_execution: bool,
    /// Enable checkpointing
    pub checkpointing_enabled: bool,
    /// Checkpoint interval
    pub checkpoint_interval: Duration,
    /// Enable streaming results
    pub streaming_enabled: bool,
    /// Retry configuration
    pub retry_config: RetryConfig,
    /// Resource limits
    pub resource_limits: ResourceLimits,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            max_concurrency: 10,
            node_timeout: Duration::from_secs(300), // 5 minutes
            total_timeout: Duration::from_secs(3600), // 1 hour
            parallel_execution: true,
            checkpointing_enabled: true,
            checkpoint_interval: Duration::from_secs(60), // 1 minute
            streaming_enabled: false,
            retry_config: RetryConfig::default(),
            resource_limits: ResourceLimits::default(),
        }
    }
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum retry attempts
    pub max_attempts: u32,
    /// Base delay between retries
    pub base_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
    /// Jitter factor (0.0 - 1.0)
    pub jitter_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
        }
    }
}

/// Resource limits for execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage (bytes)
    pub max_memory: u64,
    /// Maximum CPU usage (percentage)
    pub max_cpu: f64,
    /// Maximum execution time per node
    pub max_node_time: Duration,
    /// Maximum total nodes in execution
    pub max_nodes: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory: 1024 * 1024 * 1024, // 1GB
            max_cpu: 80.0, // 80%
            max_node_time: Duration::from_secs(600), // 10 minutes
            max_nodes: 1000,
        }
    }
}

/// Execution context for a single run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    /// Execution ID
    pub execution_id: String,
    /// Start time
    pub started_at: SystemTime,
    /// Current status
    pub status: ExecutionStatus,
    /// Execution configuration
    pub config: ExecutionConfig,
    /// Input state
    pub input_state: ExecutionState,
    /// Current state
    pub current_state: ExecutionState,
    /// Execution metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Node execution history
    pub execution_history: Vec<NodeExecution>,
    /// Error information
    pub error: Option<ExecutionError>,
}

impl ExecutionContext {
    /// Create a new execution context
    pub fn new(config: ExecutionConfig, input_state: ExecutionState) -> Self {
        Self {
            execution_id: uuid::Uuid::new_v4().to_string(),
            started_at: SystemTime::now(),
            status: ExecutionStatus::Pending,
            config,
            input_state: input_state.clone(),
            current_state: input_state,
            metadata: HashMap::new(),
            execution_history: Vec::new(),
            error: None,
        }
    }
    
    /// Get execution duration
    pub fn duration(&self) -> Duration {
        SystemTime::now()
            .duration_since(self.started_at)
            .unwrap_or(Duration::ZERO)
    }
    
    /// Add node execution to history
    pub fn add_execution(&mut self, execution: NodeExecution) {
        self.execution_history.push(execution);
    }
    
    /// Get successful executions
    pub fn successful_executions(&self) -> Vec<&NodeExecution> {
        self.execution_history
            .iter()
            .filter(|e| e.status == NodeExecutionStatus::Completed)
            .collect()
    }
    
    /// Get failed executions
    pub fn failed_executions(&self) -> Vec<&NodeExecution> {
        self.execution_history
            .iter()
            .filter(|e| e.status == NodeExecutionStatus::Failed)
            .collect()
    }
}

/// Execution status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    /// Execution is pending
    Pending,
    /// Execution is running
    Running,
    /// Execution completed successfully
    Completed,
    /// Execution failed
    Failed,
    /// Execution was cancelled
    Cancelled,
    /// Execution timed out
    TimedOut,
    /// Execution is paused
    Paused,
}

/// Node execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeExecution {
    /// Node ID
    pub node_id: NodeId,
    /// Execution status
    pub status: NodeExecutionStatus,
    /// Start time
    pub started_at: SystemTime,
    /// End time
    pub ended_at: Option<SystemTime>,
    /// Input state
    pub input_state: ExecutionState,
    /// Output state
    pub output_state: Option<ExecutionState>,
    /// Error information
    pub error: Option<String>,
    /// Retry attempts
    pub retry_attempts: u32,
    /// Execution metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl NodeExecution {
    /// Create a new node execution
    pub fn new(node_id: NodeId, input_state: ExecutionState) -> Self {
        Self {
            node_id,
            status: NodeExecutionStatus::Pending,
            started_at: SystemTime::now(),
            ended_at: None,
            input_state,
            output_state: None,
            error: None,
            retry_attempts: 0,
            metadata: HashMap::new(),
        }
    }
    
    /// Mark execution as started
    pub fn start(&mut self) {
        self.status = NodeExecutionStatus::Running;
        self.started_at = SystemTime::now();
    }
    
    /// Mark execution as completed
    pub fn complete(&mut self, output_state: ExecutionState) {
        self.status = NodeExecutionStatus::Completed;
        self.ended_at = Some(SystemTime::now());
        self.output_state = Some(output_state);
    }
    
    /// Mark execution as failed
    pub fn fail(&mut self, error: String) {
        self.status = NodeExecutionStatus::Failed;
        self.ended_at = Some(SystemTime::now());
        self.error = Some(error);
    }
    
    /// Get execution duration
    pub fn duration(&self) -> Duration {
        let end_time = self.ended_at.unwrap_or_else(SystemTime::now);
        end_time
            .duration_since(self.started_at)
            .unwrap_or(Duration::ZERO)
    }
}

/// Node execution status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeExecutionStatus {
    /// Execution is pending
    Pending,
    /// Execution is running
    Running,
    /// Execution completed successfully
    Completed,
    /// Execution failed
    Failed,
    /// Execution was cancelled
    Cancelled,
    /// Execution timed out
    TimedOut,
}

/// Advanced graph execution engine
#[derive(Debug)]
pub struct ExecutionEngine {
    /// Configuration
    config: ExecutionConfig,
    /// State manager
    state_manager: Arc<StateManager>,
    /// Concurrency semaphore
    semaphore: Arc<Semaphore>,
    /// Active executions
    active_executions: Arc<RwLock<HashMap<String, ExecutionContext>>>,
    /// Execution scheduler
    scheduler: scheduler::ExecutionScheduler,
    /// Checkpoint manager
    checkpoint_manager: checkpoint::CheckpointManager,
    /// Streaming manager
    streaming_manager: streaming::StreamingManager,
}

impl ExecutionEngine {
    /// Create a new execution engine
    pub fn new(config: ExecutionConfig, state_manager: Arc<StateManager>) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrency));
        let scheduler = scheduler::ExecutionScheduler::new(config.clone());
        let checkpoint_manager = checkpoint::CheckpointManager::new(config.clone());
        let streaming_manager = streaming::StreamingManager::new(config.clone());
        
        Self {
            config,
            state_manager,
            semaphore,
            active_executions: Arc::new(RwLock::new(HashMap::new())),
            scheduler,
            checkpoint_manager,
            streaming_manager,
        }
    }
    
    /// Execute a graph
    pub async fn execute_graph<S>(
        &self,
        graph: &Graph<S>,
        input_state: ExecutionState,
    ) -> Result<ExecutionResult, ExecutionError>
    where
        S: crate::state::State,
    {
        let mut context = ExecutionContext::new(self.config.clone(), input_state);
        context.status = ExecutionStatus::Running;
        
        // Register execution
        {
            let mut executions = self.active_executions.write().await;
            executions.insert(context.execution_id.clone(), context.clone());
        }
        
        // Execute with timeout
        let result = timeout(
            self.config.total_timeout,
            self.execute_graph_internal(graph, &mut context),
        )
        .await;
        
        // Handle timeout
        let execution_result = match result {
            Ok(Ok(result)) => {
                context.status = ExecutionStatus::Completed;
                Ok(result)
            }
            Ok(Err(error)) => {
                context.status = ExecutionStatus::Failed;
                context.error = Some(error.clone());
                Err(error)
            }
            Err(_) => {
                context.status = ExecutionStatus::TimedOut;
                let error = ExecutionError::Timeout {
                    duration: self.config.total_timeout,
                };
                context.error = Some(error.clone());
                Err(error)
            }
        };
        
        // Update execution context
        {
            let mut executions = self.active_executions.write().await;
            executions.insert(context.execution_id.clone(), context);
        }
        
        execution_result
    }
    
    /// Internal graph execution
    async fn execute_graph_internal<S>(
        &self,
        graph: &Graph<S>,
        context: &mut ExecutionContext,
    ) -> Result<ExecutionResult, ExecutionError>
    where
        S: crate::state::State,
    {
        // Validate graph
        self.validate_graph(graph)?;
        
        // Create execution plan
        let execution_plan = self.create_execution_plan(graph)?;
        
        // Execute plan
        if self.config.parallel_execution {
            self.execute_parallel(graph, &execution_plan, context).await
        } else {
            self.execute_sequential(graph, &execution_plan, context).await
        }
    }
    
    /// Validate graph for execution
    fn validate_graph<S>(&self, graph: &Graph<S>) -> Result<(), ExecutionError>
    where
        S: crate::state::State,
    {
        // Check node count limit
        if graph.nodes().len() > self.config.resource_limits.max_nodes {
            return Err(ExecutionError::ResourceLimit {
                resource: "nodes".to_string(),
                limit: self.config.resource_limits.max_nodes as u64,
                actual: graph.nodes().len() as u64,
            });
        }
        
        // Check for cycles (if not allowed)
        if graph.has_cycles() {
            return Err(ExecutionError::InvalidGraph {
                reason: "Graph contains cycles".to_string(),
            });
        }
        
        // Validate all nodes have implementations
        for node in graph.nodes() {
            if node.node_type().is_empty() {
                return Err(ExecutionError::InvalidGraph {
                    reason: format!("Node {} has no type", node.id()),
                });
            }
        }
        
        Ok(())
    }
    
    /// Create execution plan
    fn create_execution_plan<S>(&self, graph: &Graph<S>) -> Result<ExecutionPlan, ExecutionError>
    where
        S: crate::state::State,
    {
        let mut plan = ExecutionPlan::new();
        
        // Topological sort for execution order
        let sorted_nodes = graph.topological_sort()
            .map_err(|e| ExecutionError::InvalidGraph {
                reason: format!("Failed to create execution order: {}", e),
            })?;
        
        // Group nodes by execution level (for parallel execution)
        let mut levels = Vec::new();
        let mut visited = HashSet::new();
        let mut current_level = Vec::new();
        
        for node_id in sorted_nodes {
            let node = graph.get_node(&node_id).unwrap();
            
            // Check if all dependencies are satisfied
            let dependencies_satisfied = graph.incoming_edges(&node_id)
                .iter()
                .all(|edge| visited.contains(&edge.from()));
            
            if dependencies_satisfied {
                current_level.push(node_id.clone());
                visited.insert(node_id);
            } else {
                // Start new level
                if !current_level.is_empty() {
                    levels.push(current_level);
                    current_level = Vec::new();
                }
                current_level.push(node_id.clone());
                visited.insert(node_id);
            }
        }
        
        if !current_level.is_empty() {
            levels.push(current_level);
        }
        
        plan.execution_levels = levels;
        Ok(plan)
    }
    
    /// Execute graph in parallel
    async fn execute_parallel<S>(
        &self,
        graph: &Graph<S>,
        plan: &ExecutionPlan,
        context: &mut ExecutionContext,
    ) -> Result<ExecutionResult, ExecutionError>
    where
        S: crate::state::State,
    {
        let mut current_state = context.current_state.clone();
        
        for level in &plan.execution_levels {
            // Execute all nodes in this level in parallel
            let mut tasks = Vec::new();
            
            for node_id in level {
                let node = graph.get_node(node_id).unwrap();
                let node_state = current_state.clone();
                let semaphore = Arc::clone(&self.semaphore);
                let config = self.config.clone();
                
                let task = tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    Self::execute_node_with_retry(node, node_state, &config).await
                });
                
                tasks.push((node_id.clone(), task));
            }
            
            // Wait for all tasks to complete
            for (node_id, task) in tasks {
                let result = task.await
                    .map_err(|e| ExecutionError::NodeExecution {
                        node_id: node_id.clone(),
                        error: e.to_string(),
                    })?;
                
                match result {
                    Ok(node_execution) => {
                        current_state = node_execution.output_state.clone().unwrap_or(current_state);
                        context.add_execution(node_execution);
                    }
                    Err(error) => {
                        return Err(ExecutionError::NodeExecution {
                            node_id,
                            error: error.to_string(),
                        });
                    }
                }
            }
        }
        
        context.current_state = current_state.clone();
        
        Ok(ExecutionResult {
            execution_id: context.execution_id.clone(),
            status: ExecutionStatus::Completed,
            final_state: current_state,
            execution_time: context.duration(),
            node_executions: context.execution_history.len(),
            successful_nodes: context.successful_executions().len(),
            failed_nodes: context.failed_executions().len(),
            metadata: context.metadata.clone(),
        })
    }
    
    /// Execute graph sequentially
    async fn execute_sequential<S>(
        &self,
        graph: &Graph<S>,
        plan: &ExecutionPlan,
        context: &mut ExecutionContext,
    ) -> Result<ExecutionResult, ExecutionError>
    where
        S: crate::state::State,
    {
        let mut current_state = context.current_state.clone();
        
        for level in &plan.execution_levels {
            for node_id in level {
                let node = graph.get_node(node_id).unwrap();
                
                let result = Self::execute_node_with_retry(node, current_state.clone(), &self.config).await;
                
                match result {
                    Ok(node_execution) => {
                        current_state = node_execution.output_state.clone().unwrap_or(current_state);
                        context.add_execution(node_execution);
                    }
                    Err(error) => {
                        return Err(ExecutionError::NodeExecution {
                            node_id: node_id.clone(),
                            error: error.to_string(),
                        });
                    }
                }
            }
        }
        
        context.current_state = current_state.clone();
        
        Ok(ExecutionResult {
            execution_id: context.execution_id.clone(),
            status: ExecutionStatus::Completed,
            final_state: current_state,
            execution_time: context.duration(),
            node_executions: context.execution_history.len(),
            successful_nodes: context.successful_executions().len(),
            failed_nodes: context.failed_executions().len(),
            metadata: context.metadata.clone(),
        })
    }
    
    /// Execute a single node with retry logic
    async fn execute_node_with_retry<S>(
        node: &dyn Node<S>,
        input_state: ExecutionState,
        config: &ExecutionConfig,
    ) -> Result<NodeExecution, ExecutionError>
    where
        S: crate::state::State,
    {
        let mut execution = NodeExecution::new(node.id().clone(), input_state.clone());
        execution.start();
        
        for attempt in 0..config.retry_config.max_attempts {
            execution.retry_attempts = attempt;
            
            // Execute node with timeout
            let result = timeout(
                config.node_timeout,
                node.execute(execution.input_state.clone()),
            )
            .await;
            
            match result {
                Ok(Ok(output_state)) => {
                    execution.complete(output_state);
                    return Ok(execution);
                }
                Ok(Err(error)) => {
                    if attempt == config.retry_config.max_attempts - 1 {
                        execution.fail(error.to_string());
                        return Ok(execution);
                    }
                    
                    // Calculate retry delay
                    let delay = Self::calculate_retry_delay(&config.retry_config, attempt);
                    tokio::time::sleep(delay).await;
                }
                Err(_) => {
                    execution.fail("Node execution timed out".to_string());
                    return Ok(execution);
                }
            }
        }
        
        execution.fail("Maximum retry attempts exceeded".to_string());
        Ok(execution)
    }
    
    /// Calculate retry delay with exponential backoff and jitter
    fn calculate_retry_delay(config: &RetryConfig, attempt: u32) -> Duration {
        let base_delay_ms = config.base_delay.as_millis() as f64;
        let delay_ms = base_delay_ms * config.backoff_multiplier.powi(attempt as i32);
        let max_delay_ms = config.max_delay.as_millis() as f64;
        
        let delay_ms = delay_ms.min(max_delay_ms);
        
        // Add jitter
        let jitter = delay_ms * config.jitter_factor * (rand::random::<f64>() - 0.5);
        let final_delay_ms = (delay_ms + jitter).max(0.0) as u64;
        
        Duration::from_millis(final_delay_ms)
    }
    
    /// Get execution status
    pub async fn get_execution_status(&self, execution_id: &str) -> Option<ExecutionContext> {
        let executions = self.active_executions.read().await;
        executions.get(execution_id).cloned()
    }
    
    /// Cancel execution
    pub async fn cancel_execution(&self, execution_id: &str) -> Result<(), ExecutionError> {
        let mut executions = self.active_executions.write().await;
        if let Some(mut context) = executions.get_mut(execution_id) {
            context.status = ExecutionStatus::Cancelled;
            Ok(())
        } else {
            Err(ExecutionError::ExecutionNotFound {
                execution_id: execution_id.to_string(),
            })
        }
    }
    
    /// Get configuration
    pub fn config(&self) -> &ExecutionConfig {
        &self.config
    }
}

/// Execution plan
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    /// Execution levels (for parallel execution)
    pub execution_levels: Vec<Vec<NodeId>>,
}

impl ExecutionPlan {
    /// Create a new execution plan
    pub fn new() -> Self {
        Self {
            execution_levels: Vec::new(),
        }
    }
}

/// Execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Execution ID
    pub execution_id: String,
    /// Final status
    pub status: ExecutionStatus,
    /// Final state
    pub final_state: ExecutionState,
    /// Total execution time
    pub execution_time: Duration,
    /// Number of node executions
    pub node_executions: usize,
    /// Number of successful nodes
    pub successful_nodes: usize,
    /// Number of failed nodes
    pub failed_nodes: usize,
    /// Execution metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Execution errors
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum ExecutionError {
    /// Invalid graph
    #[error("Invalid graph: {reason}")]
    InvalidGraph { reason: String },
    
    /// Node execution error
    #[error("Node execution error for {node_id}: {error}")]
    NodeExecution { node_id: NodeId, error: String },
    
    /// Execution timeout
    #[error("Execution timed out after {duration:?}")]
    Timeout { duration: Duration },
    
    /// Resource limit exceeded
    #[error("Resource limit exceeded for {resource}: {actual} > {limit}")]
    ResourceLimit { resource: String, limit: u64, actual: u64 },
    
    /// Execution not found
    #[error("Execution not found: {execution_id}")]
    ExecutionNotFound { execution_id: String },
    
    /// Configuration error
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },
    
    /// System error
    #[error("System error: {message}")]
    SystemError { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_config_default() {
        let config = ExecutionConfig::default();
        assert_eq!(config.max_concurrency, 10);
        assert!(config.parallel_execution);
        assert!(config.checkpointing_enabled);
    }

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.backoff_multiplier, 2.0);
        assert_eq!(config.jitter_factor, 0.1);
    }

    #[test]
    fn test_execution_context_creation() {
        let config = ExecutionConfig::default();
        let state = serde_json::json!({});
        let context = ExecutionContext::new(config, state);
        
        assert_eq!(context.status, ExecutionStatus::Pending);
        assert!(context.execution_history.is_empty());
        assert!(context.error.is_none());
    }

    #[test]
    fn test_node_execution_lifecycle() {
        let node_id = "test_node".to_string();
        let state = serde_json::json!({});
        let mut execution = NodeExecution::new(node_id, state.clone());
        
        assert_eq!(execution.status, NodeExecutionStatus::Pending);
        
        execution.start();
        assert_eq!(execution.status, NodeExecutionStatus::Running);
        
        execution.complete(state);
        assert_eq!(execution.status, NodeExecutionStatus::Completed);
        assert!(execution.output_state.is_some());
    }

    #[test]
    fn test_execution_plan_creation() {
        let plan = ExecutionPlan::new();
        assert!(plan.execution_levels.is_empty());
    }
}
