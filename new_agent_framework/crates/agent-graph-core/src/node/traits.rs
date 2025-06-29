//! Core node traits and abstractions.

use crate::error::{CoreError, CoreResult};
use crate::state::State;
use crate::node::NodeMetadata;
use async_trait::async_trait;
use std::fmt::Debug;

/// Unique identifier for a node
pub type NodeId = String;

/// Core trait that all nodes must implement
#[async_trait]
pub trait Node<S>: Send + Sync + Debug
where
    S: State,
{
    /// Execute the node with the given state
    async fn execute(&self, state: &mut S) -> CoreResult<NodeOutput>;

    /// Get the node's unique identifier
    fn id(&self) -> &str;

    /// Get the node's metadata
    fn metadata(&self) -> &NodeMetadata;

    /// Validate the node configuration
    fn validate(&self) -> CoreResult<()> {
        Ok(())
    }

    /// Check if this node can run in parallel with others
    fn is_parallel_safe(&self) -> bool {
        self.metadata().parallel_safe
    }

    /// Get estimated execution time in milliseconds
    fn estimated_duration_ms(&self) -> Option<u64> {
        self.metadata().expected_duration_ms
    }

    /// Pre-execution hook (optional)
    async fn before_execute(&self, _state: &S) -> CoreResult<()> {
        Ok(())
    }

    /// Post-execution hook (optional)
    async fn after_execute(&self, _state: &S, _output: &NodeOutput) -> CoreResult<()> {
        Ok(())
    }
}

/// Output from node execution
#[derive(Debug, Clone)]
pub struct NodeOutput {
    /// Whether the execution was successful
    pub success: bool,
    /// Optional result data
    pub data: Option<serde_json::Value>,
    /// Execution metrics
    pub metrics: ExecutionMetrics,
    /// Optional next node to execute
    pub next_node: Option<NodeId>,
    /// Whether to continue execution
    pub continue_execution: bool,
}

impl NodeOutput {
    /// Create a successful output
    pub fn success() -> Self {
        Self {
            success: true,
            data: None,
            metrics: ExecutionMetrics::default(),
            next_node: None,
            continue_execution: true,
        }
    }

    /// Create a successful output with data
    pub fn success_with_data(data: serde_json::Value) -> Self {
        Self {
            success: true,
            data: Some(data),
            metrics: ExecutionMetrics::default(),
            next_node: None,
            continue_execution: true,
        }
    }

    /// Create a failure output
    pub fn failure(error: CoreError) -> Self {
        Self {
            success: false,
            data: Some(serde_json::json!({
                "error": error.to_string(),
                "category": error.category()
            })),
            metrics: ExecutionMetrics::default(),
            next_node: None,
            continue_execution: false,
        }
    }

    /// Create output that stops execution
    pub fn stop() -> Self {
        Self {
            success: true,
            data: None,
            metrics: ExecutionMetrics::default(),
            next_node: None,
            continue_execution: false,
        }
    }

    /// Create output that routes to a specific node
    pub fn route_to(node_id: NodeId) -> Self {
        Self {
            success: true,
            data: None,
            metrics: ExecutionMetrics::default(),
            next_node: Some(node_id),
            continue_execution: true,
        }
    }
}

/// Execution metrics for a node
#[derive(Debug, Clone, Default)]
pub struct ExecutionMetrics {
    /// Execution time in milliseconds
    pub duration_ms: u64,
    /// Memory used in bytes
    pub memory_used_bytes: Option<u64>,
    /// Number of operations performed
    pub operations_count: Option<u64>,
    /// Custom metrics
    pub custom_metrics: std::collections::HashMap<String, f64>,
}

impl ExecutionMetrics {
    /// Create new metrics with duration
    pub fn with_duration(duration_ms: u64) -> Self {
        Self {
            duration_ms,
            ..Default::default()
        }
    }

    /// Add a custom metric
    pub fn add_metric(&mut self, name: String, value: f64) {
        self.custom_metrics.insert(name, value);
    }
}