//! Streaming execution and real-time event handling.

use crate::error::GraphResult;
use crate::node::{NodeExecutionContext, NodeId};

use async_stream::stream;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use tokio::sync::mpsc;
use uuid::Uuid;

/// Events that can be emitted during graph execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionEvent {
    /// Graph execution started
    GraphStarted {
        /// Unique execution ID
        execution_id: Uuid,
        /// Timestamp
        timestamp: chrono::DateTime<chrono::Utc>,
        /// Entry point node
        entry_point: NodeId,
    },

    /// Graph execution completed
    GraphCompleted {
        /// Execution ID
        execution_id: Uuid,
        /// Timestamp
        timestamp: chrono::DateTime<chrono::Utc>,
        /// Final node
        final_node: Option<NodeId>,
        /// Total execution time in milliseconds
        duration_ms: u64,
        /// Whether execution was successful
        success: bool,
    },

    /// Node execution started
    NodeStarted {
        /// Execution ID
        execution_id: Uuid,
        /// Node ID
        node_id: NodeId,
        /// Timestamp
        timestamp: chrono::DateTime<chrono::Utc>,
        /// Node execution context
        context: NodeExecutionContext,
    },

    /// Node execution completed
    NodeCompleted {
        /// Execution ID
        execution_id: Uuid,
        /// Node ID
        node_id: NodeId,
        /// Timestamp
        timestamp: chrono::DateTime<chrono::Utc>,
        /// Execution duration in milliseconds
        duration_ms: u64,
        /// Whether execution was successful
        success: bool,
        /// Error message if failed
        error: Option<String>,
    },

    /// State updated
    StateUpdated {
        /// Execution ID
        execution_id: Uuid,
        /// Node that updated the state
        node_id: NodeId,
        /// Timestamp
        timestamp: chrono::DateTime<chrono::Utc>,
        /// State snapshot ID (if checkpointing is enabled)
        snapshot_id: Option<Uuid>,
    },

    /// Edge traversed
    EdgeTraversed {
        /// Execution ID
        execution_id: Uuid,
        /// Source node
        from_node: NodeId,
        /// Target node
        to_node: NodeId,
        /// Timestamp
        timestamp: chrono::DateTime<chrono::Utc>,
        /// Edge metadata
        edge_metadata: Option<serde_json::Value>,
    },

    /// Parallel execution started
    ParallelStarted {
        /// Execution ID
        execution_id: Uuid,
        /// Nodes being executed in parallel
        node_ids: Vec<NodeId>,
        /// Timestamp
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Parallel execution completed
    ParallelCompleted {
        /// Execution ID
        execution_id: Uuid,
        /// Results for each node
        results: Vec<(NodeId, bool)>, // (node_id, success)
        /// Timestamp
        timestamp: chrono::DateTime<chrono::Utc>,
        /// Total duration in milliseconds
        duration_ms: u64,
    },

    /// Error occurred
    Error {
        /// Execution ID
        execution_id: Uuid,
        /// Node where error occurred (if applicable)
        node_id: Option<NodeId>,
        /// Timestamp
        timestamp: chrono::DateTime<chrono::Utc>,
        /// Error message
        error: String,
        /// Error category
        category: String,
    },

    /// Custom event
    Custom {
        /// Execution ID
        execution_id: Uuid,
        /// Event type
        event_type: String,
        /// Event data
        data: serde_json::Value,
        /// Timestamp
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

impl ExecutionEvent {
    /// Get the execution ID for this event
    pub fn execution_id(&self) -> Uuid {
        match self {
            ExecutionEvent::GraphStarted { execution_id, .. }
            | ExecutionEvent::GraphCompleted { execution_id, .. }
            | ExecutionEvent::NodeStarted { execution_id, .. }
            | ExecutionEvent::NodeCompleted { execution_id, .. }
            | ExecutionEvent::StateUpdated { execution_id, .. }
            | ExecutionEvent::EdgeTraversed { execution_id, .. }
            | ExecutionEvent::ParallelStarted { execution_id, .. }
            | ExecutionEvent::ParallelCompleted { execution_id, .. }
            | ExecutionEvent::Error { execution_id, .. }
            | ExecutionEvent::Custom { execution_id, .. } => *execution_id,
        }
    }

    /// Get the timestamp for this event
    pub fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        match self {
            ExecutionEvent::GraphStarted { timestamp, .. }
            | ExecutionEvent::GraphCompleted { timestamp, .. }
            | ExecutionEvent::NodeStarted { timestamp, .. }
            | ExecutionEvent::NodeCompleted { timestamp, .. }
            | ExecutionEvent::StateUpdated { timestamp, .. }
            | ExecutionEvent::EdgeTraversed { timestamp, .. }
            | ExecutionEvent::ParallelStarted { timestamp, .. }
            | ExecutionEvent::ParallelCompleted { timestamp, .. }
            | ExecutionEvent::Error { timestamp, .. }
            | ExecutionEvent::Custom { timestamp, .. } => *timestamp,
        }
    }

    /// Get the event type as a string
    pub fn event_type(&self) -> &'static str {
        match self {
            ExecutionEvent::GraphStarted { .. } => "graph_started",
            ExecutionEvent::GraphCompleted { .. } => "graph_completed",
            ExecutionEvent::NodeStarted { .. } => "node_started",
            ExecutionEvent::NodeCompleted { .. } => "node_completed",
            ExecutionEvent::StateUpdated { .. } => "state_updated",
            ExecutionEvent::EdgeTraversed { .. } => "edge_traversed",
            ExecutionEvent::ParallelStarted { .. } => "parallel_started",
            ExecutionEvent::ParallelCompleted { .. } => "parallel_completed",
            ExecutionEvent::Error { .. } => "error",
            ExecutionEvent::Custom { .. } => "custom",
        }
    }

    /// Check if this is an error event
    pub fn is_error(&self) -> bool {
        matches!(self, ExecutionEvent::Error { .. })
    }

    /// Check if this is a completion event
    pub fn is_completion(&self) -> bool {
        matches!(
            self,
            ExecutionEvent::GraphCompleted { .. }
                | ExecutionEvent::NodeCompleted { .. }
                | ExecutionEvent::ParallelCompleted { .. }
        )
    }
}

/// Type alias for execution event stream
pub type ExecutionStream = Pin<Box<dyn Stream<Item = ExecutionEvent> + Send>>;

/// Event emitter for streaming execution events
#[derive(Debug)]
pub struct EventEmitter {
    sender: mpsc::UnboundedSender<ExecutionEvent>,
}

impl EventEmitter {
    /// Create a new event emitter
    pub fn new() -> (Self, mpsc::UnboundedReceiver<ExecutionEvent>) {
        let (sender, receiver) = mpsc::unbounded_channel();
        (Self { sender }, receiver)
    }

    /// Emit an event
    pub fn emit(&self, event: ExecutionEvent) -> GraphResult<()> {
        self.sender.send(event).map_err(|_| {
            crate::error::GraphError::Internal("Failed to emit event".to_string())
        })?;
        Ok(())
    }

    /// Emit a graph started event
    pub fn emit_graph_started(&self, execution_id: Uuid, entry_point: NodeId) -> GraphResult<()> {
        self.emit(ExecutionEvent::GraphStarted {
            execution_id,
            timestamp: chrono::Utc::now(),
            entry_point,
        })
    }

    /// Emit a graph completed event
    pub fn emit_graph_completed(
        &self,
        execution_id: Uuid,
        final_node: Option<NodeId>,
        duration_ms: u64,
        success: bool,
    ) -> GraphResult<()> {
        self.emit(ExecutionEvent::GraphCompleted {
            execution_id,
            timestamp: chrono::Utc::now(),
            final_node,
            duration_ms,
            success,
        })
    }

    /// Emit a node started event
    pub fn emit_node_started(
        &self,
        execution_id: Uuid,
        node_id: NodeId,
        context: NodeExecutionContext,
    ) -> GraphResult<()> {
        self.emit(ExecutionEvent::NodeStarted {
            execution_id,
            node_id,
            timestamp: chrono::Utc::now(),
            context,
        })
    }

    /// Emit a node completed event
    pub fn emit_node_completed(
        &self,
        execution_id: Uuid,
        node_id: NodeId,
        duration_ms: u64,
        success: bool,
        error: Option<String>,
    ) -> GraphResult<()> {
        self.emit(ExecutionEvent::NodeCompleted {
            execution_id,
            node_id,
            timestamp: chrono::Utc::now(),
            duration_ms,
            success,
            error,
        })
    }

    /// Emit a state updated event
    pub fn emit_state_updated(
        &self,
        execution_id: Uuid,
        node_id: NodeId,
        snapshot_id: Option<Uuid>,
    ) -> GraphResult<()> {
        self.emit(ExecutionEvent::StateUpdated {
            execution_id,
            node_id,
            timestamp: chrono::Utc::now(),
            snapshot_id,
        })
    }

    /// Emit an error event
    pub fn emit_error(
        &self,
        execution_id: Uuid,
        node_id: Option<NodeId>,
        error: String,
        category: String,
    ) -> GraphResult<()> {
        self.emit(ExecutionEvent::Error {
            execution_id,
            node_id,
            timestamp: chrono::Utc::now(),
            error,
            category,
        })
    }

    /// Emit a custom event
    pub fn emit_custom(
        &self,
        execution_id: Uuid,
        event_type: String,
        data: serde_json::Value,
    ) -> GraphResult<()> {
        self.emit(ExecutionEvent::Custom {
            execution_id,
            event_type,
            data,
            timestamp: chrono::Utc::now(),
        })
    }
}

impl Default for EventEmitter {
    fn default() -> Self {
        let (emitter, _) = Self::new();
        emitter
    }
}

/// Stream adapter for converting receiver to stream
pub fn create_execution_stream(
    mut receiver: mpsc::UnboundedReceiver<ExecutionEvent>,
) -> ExecutionStream {
    Box::pin(stream! {
        while let Some(event) = receiver.recv().await {
            yield event;
        }
    })
}

/// Event filter for filtering execution events
#[derive(Debug, Clone)]
pub struct EventFilter {
    /// Filter by event types
    pub event_types: Option<Vec<String>>,
    /// Filter by execution ID
    pub execution_id: Option<Uuid>,
    /// Filter by node ID
    pub node_id: Option<NodeId>,
    /// Filter by error events only
    pub errors_only: bool,
    /// Filter by completion events only
    pub completions_only: bool,
}

impl EventFilter {
    /// Create a new event filter
    pub fn new() -> Self {
        Self {
            event_types: None,
            execution_id: None,
            node_id: None,
            errors_only: false,
            completions_only: false,
        }
    }

    /// Filter by event types
    pub fn with_event_types(mut self, types: Vec<String>) -> Self {
        self.event_types = Some(types);
        self
    }

    /// Filter by execution ID
    pub fn with_execution_id(mut self, id: Uuid) -> Self {
        self.execution_id = Some(id);
        self
    }

    /// Filter by node ID
    pub fn with_node_id(mut self, id: NodeId) -> Self {
        self.node_id = Some(id);
        self
    }

    /// Filter errors only
    pub fn errors_only(mut self) -> Self {
        self.errors_only = true;
        self
    }

    /// Filter completions only
    pub fn completions_only(mut self) -> Self {
        self.completions_only = true;
        self
    }

    /// Check if an event passes the filter
    pub fn matches(&self, event: &ExecutionEvent) -> bool {
        // Check execution ID filter
        if let Some(exec_id) = self.execution_id {
            if event.execution_id() != exec_id {
                return false;
            }
        }

        // Check event type filter
        if let Some(ref types) = self.event_types {
            if !types.contains(&event.event_type().to_string()) {
                return false;
            }
        }

        // Check node ID filter
        if let Some(ref node_id) = self.node_id {
            match event {
                ExecutionEvent::NodeStarted { node_id: nid, .. }
                | ExecutionEvent::NodeCompleted { node_id: nid, .. }
                | ExecutionEvent::StateUpdated { node_id: nid, .. } => {
                    if nid != node_id {
                        return false;
                    }
                }
                ExecutionEvent::EdgeTraversed {
                    from_node, to_node, ..
                } => {
                    if from_node != node_id && to_node != node_id {
                        return false;
                    }
                }
                ExecutionEvent::Error {
                    node_id: Some(nid), ..
                } => {
                    if nid != node_id {
                        return false;
                    }
                }
                _ => {}
            }
        }

        // Check error filter
        if self.errors_only && !event.is_error() {
            return false;
        }

        // Check completion filter
        if self.completions_only && !event.is_completion() {
            return false;
        }

        true
    }
}

impl Default for EventFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Apply a filter to an execution stream
pub fn filter_stream(
    stream: ExecutionStream,
    filter: EventFilter,
) -> ExecutionStream {
    Box::pin(stream! {
        futures::pin_mut!(stream);
        while let Some(event) = futures::StreamExt::next(&mut stream).await {
            if filter.matches(&event) {
                yield event;
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let execution_id = Uuid::new_v4();
        let event = ExecutionEvent::GraphStarted {
            execution_id,
            timestamp: chrono::Utc::now(),
            entry_point: "start".to_string(),
        };

        assert_eq!(event.execution_id(), execution_id);
        assert_eq!(event.event_type(), "graph_started");
        assert!(!event.is_error());
        assert!(!event.is_completion());
    }

    #[test]
    fn test_event_filter() {
        let execution_id = Uuid::new_v4();
        let filter = EventFilter::new()
            .with_execution_id(execution_id)
            .errors_only();

        let error_event = ExecutionEvent::Error {
            execution_id,
            node_id: None,
            timestamp: chrono::Utc::now(),
            error: "test error".to_string(),
            category: "test".to_string(),
        };

        let start_event = ExecutionEvent::GraphStarted {
            execution_id,
            timestamp: chrono::Utc::now(),
            entry_point: "start".to_string(),
        };

        assert!(filter.matches(&error_event));
        assert!(!filter.matches(&start_event));
    }

    #[tokio::test]
    async fn test_event_emitter() {
        let (emitter, mut receiver) = EventEmitter::new();
        let execution_id = Uuid::new_v4();

        emitter
            .emit_graph_started(execution_id, "start".to_string())
            .unwrap();

        let event = receiver.recv().await.unwrap();
        assert_eq!(event.execution_id(), execution_id);
        assert_eq!(event.event_type(), "graph_started");
    }
}
