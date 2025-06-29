//! Core agent implementation.

use crate::{CoreError, CoreResult, State, Node, NodeId};
use crate::agent::builder::AgentConfig;
use crate::roles::AgentRole;
use crate::memory::{MemorySystem, MemoryConfig};
use crate::collaboration::CollaborationContext;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Core agent implementation
#[derive(Debug)]
pub struct Agent<S>
where
    S: State,
{
    /// Agent configuration
    config: AgentConfig,
    /// Memory system
    memory: Arc<RwLock<MemorySystem>>,
    /// Collaboration context
    collaboration: Arc<RwLock<CollaborationContext>>,
    /// Agent state
    state: Arc<RwLock<AgentState>>,
    /// Agent metrics
    metrics: Arc<RwLock<AgentMetrics>>,
    /// State type marker
    _phantom: std::marker::PhantomData<S>,
}

/// Agent internal state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    /// Current status
    pub status: AgentStatus,
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
    /// Number of tasks completed
    pub tasks_completed: u64,
    /// Current task ID if any
    pub current_task_id: Option<String>,
    /// Agent context data
    pub context: HashMap<String, serde_json::Value>,
}

/// Agent status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentStatus {
    /// Agent is idle and ready for tasks
    Idle,
    /// Agent is currently executing a task
    Busy,
    /// Agent is waiting for input or resources
    Waiting,
    /// Agent encountered an error
    Error,
    /// Agent is paused
    Paused,
    /// Agent is shutting down
    Shutdown,
}

/// Agent performance metrics
#[derive(Debug, Clone, Default)]
pub struct AgentMetrics {
    /// Total execution time in milliseconds
    pub total_execution_time_ms: u64,
    /// Number of successful executions
    pub successful_executions: u64,
    /// Number of failed executions
    pub failed_executions: u64,
    /// Average execution time in milliseconds
    pub avg_execution_time_ms: f64,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// Custom metrics
    pub custom_metrics: HashMap<String, f64>,
}

impl<S> Agent<S>
where
    S: State + Send + Sync + 'static,
{
    /// Create a new agent with the given configuration
    pub fn new(config: AgentConfig) -> CoreResult<Self> {
        let memory = Arc::new(RwLock::new(MemorySystem::new(config.memory_config.clone())?));
        let collaboration = Arc::new(RwLock::new(CollaborationContext::new()));
        
        let state = AgentState {
            status: AgentStatus::Idle,
            last_activity: Utc::now(),
            tasks_completed: 0,
            current_task_id: None,
            context: HashMap::new(),
        };

        Ok(Self {
            config,
            memory,
            collaboration,
            state: Arc::new(RwLock::new(state)),
            metrics: Arc::new(RwLock::new(AgentMetrics::default())),
            _phantom: std::marker::PhantomData,
        })
    }

    /// Get agent ID
    pub fn id(&self) -> &str {
        &self.config.id
    }

    /// Get agent name
    pub fn name(&self) -> &str {
        &self.config.name
    }

    /// Get agent role
    pub fn role(&self) -> &AgentRole {
        &self.config.role
    }

    /// Get agent configuration
    pub fn config(&self) -> &AgentConfig {
        &self.config
    }

    /// Get current agent status
    pub async fn status(&self) -> AgentStatus {
        self.state.read().await.status.clone()
    }

    /// Set agent status
    pub async fn set_status(&self, status: AgentStatus) {
        let mut state = self.state.write().await;
        state.status = status;
        state.last_activity = Utc::now();
    }

    /// Get agent metrics
    pub async fn metrics(&self) -> AgentMetrics {
        self.metrics.read().await.clone()
    }

    /// Update metrics after execution
    async fn update_metrics(&self, execution_time_ms: u64, success: bool) {
        let mut metrics = self.metrics.write().await;
        metrics.total_execution_time_ms += execution_time_ms;
        
        if success {
            metrics.successful_executions += 1;
        } else {
            metrics.failed_executions += 1;
        }

        let total_executions = metrics.successful_executions + metrics.failed_executions;
        if total_executions > 0 {
            metrics.avg_execution_time_ms = metrics.total_execution_time_ms as f64 / total_executions as f64;
        }
    }

    /// Store information in agent memory
    pub async fn remember(&self, key: String, value: serde_json::Value) -> CoreResult<()> {
        let mut memory = self.memory.write().await;
        memory.store(key, value).await
    }

    /// Retrieve information from agent memory
    pub async fn recall(&self, key: &str) -> CoreResult<Option<serde_json::Value>> {
        let memory = self.memory.read().await;
        memory.retrieve(key).await
    }

    /// Get collaboration context
    pub async fn collaboration_context(&self) -> CollaborationContext {
        self.collaboration.read().await.clone()
    }

    /// Update collaboration context
    pub async fn update_collaboration(&self, context: CollaborationContext) {
        let mut collaboration = self.collaboration.write().await;
        *collaboration = context;
    }

    /// Execute a task with the given state
    pub async fn execute_task(&self, state: &mut S, task_id: Option<String>) -> CoreResult<AgentExecutionResult> {
        let start_time = std::time::Instant::now();
        
        // Set status to busy
        self.set_status(AgentStatus::Busy).await;
        
        // Update current task
        if let Some(ref task_id) = task_id {
            let mut agent_state = self.state.write().await;
            agent_state.current_task_id = Some(task_id.clone());
        }

        let result = match self.config.role.execute(state, &self.config).await {
            Ok(output) => {
                self.set_status(AgentStatus::Idle).await;
                let execution_time = start_time.elapsed().as_millis() as u64;
                self.update_metrics(execution_time, true).await;
                
                // Update task completion count
                let mut agent_state = self.state.write().await;
                agent_state.tasks_completed += 1;
                agent_state.current_task_id = None;
                
                AgentExecutionResult {
                    success: true,
                    output: Some(output),
                    execution_time_ms: execution_time,
                    error: None,
                }
            }
            Err(error) => {
                self.set_status(AgentStatus::Error).await;
                let execution_time = start_time.elapsed().as_millis() as u64;
                self.update_metrics(execution_time, false).await;
                
                // Clear current task
                let mut agent_state = self.state.write().await;
                agent_state.current_task_id = None;
                
                AgentExecutionResult {
                    success: false,
                    output: None,
                    execution_time_ms: execution_time,
                    error: Some(error),
                }
            }
        };

        Ok(result)
    }

    /// Pause the agent
    pub async fn pause(&self) {
        self.set_status(AgentStatus::Paused).await;
    }

    /// Resume the agent
    pub async fn resume(&self) {
        self.set_status(AgentStatus::Idle).await;
    }

    /// Shutdown the agent
    pub async fn shutdown(&self) {
        self.set_status(AgentStatus::Shutdown).await;
    }

    /// Check if agent is available for new tasks
    pub async fn is_available(&self) -> bool {
        matches!(self.status().await, AgentStatus::Idle)
    }

    /// Get agent context value
    pub async fn get_context(&self, key: &str) -> Option<serde_json::Value> {
        let state = self.state.read().await;
        state.context.get(key).cloned()
    }

    /// Set agent context value
    pub async fn set_context(&self, key: String, value: serde_json::Value) {
        let mut state = self.state.write().await;
        state.context.insert(key, value);
        state.last_activity = Utc::now();
    }
}

#[async_trait]
impl<S> Node<S> for Agent<S>
where
    S: State + Send + Sync + 'static,
{
    async fn execute(&self, state: &mut S) -> CoreResult<crate::node::NodeOutput> {
        let result = self.execute_task(state, None).await?;
        
        let node_output = if result.success {
            crate::node::NodeOutput::success_with_data(
                serde_json::json!({
                    "agent_id": self.id(),
                    "execution_time_ms": result.execution_time_ms,
                    "output": result.output
                })
            )
        } else {
            crate::node::NodeOutput::failure(
                result.error.unwrap_or_else(|| CoreError::execution_error("Unknown agent error"))
            )
        };

        Ok(node_output)
    }

    fn id(&self) -> &str {
        &self.config.id
    }

    fn metadata(&self) -> &crate::node::NodeMetadata {
        // Create a static metadata for the agent
        // In a real implementation, this might be stored in the config
        static METADATA: std::sync::OnceLock<crate::node::NodeMetadata> = std::sync::OnceLock::new();
        METADATA.get_or_init(|| crate::node::NodeMetadata {
            name: "Agent Node".to_string(),
            description: Some("AI Agent execution node".to_string()),
            version: "1.0.0".to_string(),
            parallel_safe: true,
            expected_duration_ms: Some(1000),
            tags: vec!["agent".to_string(), "ai".to_string()],
            custom_properties: std::collections::HashMap::new(),
        })
    }
}

/// Result of agent execution
#[derive(Debug, Clone)]
pub struct AgentExecutionResult {
    /// Whether execution was successful
    pub success: bool,
    /// Output data if successful
    pub output: Option<serde_json::Value>,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Error if execution failed
    pub error: Option<CoreError>,
}

impl Default for AgentState {
    fn default() -> Self {
        Self {
            status: AgentStatus::Idle,
            last_activity: Utc::now(),
            tasks_completed: 0,
            current_task_id: None,
            context: HashMap::new(),
        }
    }
}