//! Streaming execution events and protocols.

use crate::{NodeId};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Execution events for streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ExecutionEvent {
    /// Execution started
    ExecutionStarted {
        execution_id: String,
        timestamp: chrono::DateTime<chrono::Utc>,
        total_nodes: usize,
    },
    /// Execution completed
    ExecutionCompleted {
        execution_id: String,
        timestamp: chrono::DateTime<chrono::Utc>,
        success: bool,
        total_duration: Duration,
        completed_nodes: usize,
    },
    /// Execution failed
    ExecutionFailed {
        execution_id: String,
        timestamp: chrono::DateTime<chrono::Utc>,
        error: String,
        completed_nodes: usize,
    },
    /// Node execution started
    NodeStarted {
        execution_id: String,
        node_id: NodeId,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Node execution completed
    NodeCompleted {
        execution_id: String,
        node_id: NodeId,
        timestamp: chrono::DateTime<chrono::Utc>,
        success: bool,
        duration: Duration,
        output: Option<agent_graph_core::node::NodeOutput>,
    },
    /// Node execution failed
    NodeFailed {
        execution_id: String,
        node_id: NodeId,
        timestamp: chrono::DateTime<chrono::Utc>,
        error: String,
    },
    /// State updated
    StateUpdated {
        execution_id: String,
        timestamp: chrono::DateTime<chrono::Utc>,
        state: serde_json::Value,
    },
    /// Progress update
    ProgressUpdate {
        execution_id: String,
        timestamp: chrono::DateTime<chrono::Utc>,
        completed_nodes: usize,
        total_nodes: usize,
        current_node: Option<NodeId>,
        progress_percentage: f64,
    },
    /// Custom event
    Custom {
        execution_id: String,
        timestamp: chrono::DateTime<chrono::Utc>,
        event_type: String,
        data: serde_json::Value,
    },
}

impl ExecutionEvent {
    /// Get the execution ID for this event
    pub fn execution_id(&self) -> &str {
        match self {
            ExecutionEvent::ExecutionStarted { execution_id, .. } => execution_id,
            ExecutionEvent::ExecutionCompleted { execution_id, .. } => execution_id,
            ExecutionEvent::ExecutionFailed { execution_id, .. } => execution_id,
            ExecutionEvent::NodeStarted { execution_id, .. } => execution_id,
            ExecutionEvent::NodeCompleted { execution_id, .. } => execution_id,
            ExecutionEvent::NodeFailed { execution_id, .. } => execution_id,
            ExecutionEvent::StateUpdated { execution_id, .. } => execution_id,
            ExecutionEvent::ProgressUpdate { execution_id, .. } => execution_id,
            ExecutionEvent::Custom { execution_id, .. } => execution_id,
        }
    }

    /// Get the timestamp for this event
    pub fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        match self {
            ExecutionEvent::ExecutionStarted { timestamp, .. } => *timestamp,
            ExecutionEvent::ExecutionCompleted { timestamp, .. } => *timestamp,
            ExecutionEvent::ExecutionFailed { timestamp, .. } => *timestamp,
            ExecutionEvent::NodeStarted { timestamp, .. } => *timestamp,
            ExecutionEvent::NodeCompleted { timestamp, .. } => *timestamp,
            ExecutionEvent::NodeFailed { timestamp, .. } => *timestamp,
            ExecutionEvent::StateUpdated { timestamp, .. } => *timestamp,
            ExecutionEvent::ProgressUpdate { timestamp, .. } => *timestamp,
            ExecutionEvent::Custom { timestamp, .. } => *timestamp,
        }
    }

    /// Get the event type as a string
    pub fn event_type(&self) -> &'static str {
        match self {
            ExecutionEvent::ExecutionStarted { .. } => "execution_started",
            ExecutionEvent::ExecutionCompleted { .. } => "execution_completed",
            ExecutionEvent::ExecutionFailed { .. } => "execution_failed",
            ExecutionEvent::NodeStarted { .. } => "node_started",
            ExecutionEvent::NodeCompleted { .. } => "node_completed",
            ExecutionEvent::NodeFailed { .. } => "node_failed",
            ExecutionEvent::StateUpdated { .. } => "state_updated",
            ExecutionEvent::ProgressUpdate { .. } => "progress_update",
            ExecutionEvent::Custom { .. } => "custom",
        }
    }

    /// Check if this is an error event
    pub fn is_error(&self) -> bool {
        matches!(self, ExecutionEvent::ExecutionFailed { .. } | ExecutionEvent::NodeFailed { .. })
    }

    /// Check if this is a completion event
    pub fn is_completion(&self) -> bool {
        matches!(self, ExecutionEvent::ExecutionCompleted { .. } | ExecutionEvent::NodeCompleted { .. })
    }

    /// Create a custom event
    pub fn custom(
        execution_id: String,
        event_type: String,
        data: serde_json::Value,
    ) -> Self {
        Self::Custom {
            execution_id,
            timestamp: chrono::Utc::now(),
            event_type,
            data,
        }
    }
}

/// Event filter for streaming subscriptions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventFilter {
    /// Filter by execution ID
    pub execution_id: Option<String>,
    /// Filter by event types
    pub event_types: Option<Vec<String>>,
    /// Filter by node IDs
    pub node_ids: Option<Vec<NodeId>>,
    /// Include error events
    pub include_errors: bool,
    /// Include completion events
    pub include_completions: bool,
    /// Include progress events
    pub include_progress: bool,
    /// Include state updates
    pub include_state_updates: bool,
    /// Custom filter criteria
    pub custom_filters: std::collections::HashMap<String, serde_json::Value>,
}

impl Default for EventFilter {
    fn default() -> Self {
        Self {
            execution_id: None,
            event_types: None,
            node_ids: None,
            include_errors: true,
            include_completions: true,
            include_progress: true,
            include_state_updates: false,
            custom_filters: std::collections::HashMap::new(),
        }
    }
}

impl EventFilter {
    /// Create a filter for a specific execution
    pub fn for_execution(execution_id: String) -> Self {
        Self {
            execution_id: Some(execution_id),
            ..Default::default()
        }
    }

    /// Create a filter for specific event types
    pub fn for_event_types(event_types: Vec<String>) -> Self {
        Self {
            event_types: Some(event_types),
            ..Default::default()
        }
    }

    /// Create a filter for specific nodes
    pub fn for_nodes(node_ids: Vec<NodeId>) -> Self {
        Self {
            node_ids: Some(node_ids),
            ..Default::default()
        }
    }

    /// Check if an event matches this filter
    pub fn matches(&self, event: &ExecutionEvent) -> bool {
        // Check execution ID
        if let Some(ref filter_execution_id) = self.execution_id {
            if event.execution_id() != filter_execution_id {
                return false;
            }
        }

        // Check event types
        if let Some(ref filter_event_types) = self.event_types {
            if !filter_event_types.contains(&event.event_type().to_string()) {
                return false;
            }
        }

        // Check node IDs
        if let Some(ref filter_node_ids) = self.node_ids {
            let event_node_id = match event {
                ExecutionEvent::NodeStarted { node_id, .. } => Some(node_id),
                ExecutionEvent::NodeCompleted { node_id, .. } => Some(node_id),
                ExecutionEvent::NodeFailed { node_id, .. } => Some(node_id),
                _ => None,
            };

            if let Some(node_id) = event_node_id {
                if !filter_node_ids.contains(node_id) {
                    return false;
                }
            } else {
                // Event doesn't have a node ID, but filter requires specific nodes
                return false;
            }
        }

        // Check specific event type filters
        match event {
            ExecutionEvent::ExecutionFailed { .. } | ExecutionEvent::NodeFailed { .. } => {
                if !self.include_errors {
                    return false;
                }
            }
            ExecutionEvent::ExecutionCompleted { .. } | ExecutionEvent::NodeCompleted { .. } => {
                if !self.include_completions {
                    return false;
                }
            }
            ExecutionEvent::ProgressUpdate { .. } => {
                if !self.include_progress {
                    return false;
                }
            }
            ExecutionEvent::StateUpdated { .. } => {
                if !self.include_state_updates {
                    return false;
                }
            }
            _ => {}
        }

        true
    }

    /// Add a custom filter
    pub fn with_custom_filter(mut self, key: String, value: serde_json::Value) -> Self {
        self.custom_filters.insert(key, value);
        self
    }

    /// Enable state updates
    pub fn with_state_updates(mut self) -> Self {
        self.include_state_updates = true;
        self
    }

    /// Disable progress updates
    pub fn without_progress(mut self) -> Self {
        self.include_progress = false;
        self
    }
}

/// Event subscription configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionConfig {
    /// Event filter
    pub filter: EventFilter,
    /// Buffer size for the event channel
    pub buffer_size: usize,
    /// Maximum events per second (rate limiting)
    pub max_events_per_second: Option<u32>,
    /// Event batching configuration
    pub batching: Option<BatchingConfig>,
}

impl Default for SubscriptionConfig {
    fn default() -> Self {
        Self {
            filter: EventFilter::default(),
            buffer_size: 1000,
            max_events_per_second: None,
            batching: None,
        }
    }
}

/// Event batching configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchingConfig {
    /// Maximum batch size
    pub max_batch_size: usize,
    /// Maximum time to wait before sending a batch
    pub max_wait_time: Duration,
    /// Whether to flush on execution completion
    pub flush_on_completion: bool,
}

impl Default for BatchingConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 10,
            max_wait_time: Duration::from_millis(100),
            flush_on_completion: true,
        }
    }
}

/// Batch of execution events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBatch {
    /// Events in this batch
    pub events: Vec<ExecutionEvent>,
    /// Batch timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Batch sequence number
    pub sequence: u64,
}

impl EventBatch {
    /// Create a new event batch
    pub fn new(events: Vec<ExecutionEvent>, sequence: u64) -> Self {
        Self {
            events,
            timestamp: chrono::Utc::now(),
            sequence,
        }
    }

    /// Get the number of events in this batch
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if the batch is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Get events of a specific type
    pub fn events_of_type(&self, event_type: &str) -> Vec<&ExecutionEvent> {
        self.events
            .iter()
            .filter(|event| event.event_type() == event_type)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_event_creation() {
        let event = ExecutionEvent::ExecutionStarted {
            execution_id: "test-123".to_string(),
            timestamp: chrono::Utc::now(),
            total_nodes: 5,
        };

        assert_eq!(event.execution_id(), "test-123");
        assert_eq!(event.event_type(), "execution_started");
        assert!(!event.is_error());
        assert!(!event.is_completion());
    }

    #[test]
    fn test_event_filter_matching() {
        let filter = EventFilter::for_execution("test-123".to_string());
        
        let matching_event = ExecutionEvent::NodeStarted {
            execution_id: "test-123".to_string(),
            node_id: "node1".to_string(),
            timestamp: chrono::Utc::now(),
        };

        let non_matching_event = ExecutionEvent::NodeStarted {
            execution_id: "test-456".to_string(),
            node_id: "node1".to_string(),
            timestamp: chrono::Utc::now(),
        };

        assert!(filter.matches(&matching_event));
        assert!(!filter.matches(&non_matching_event));
    }

    #[test]
    fn test_event_batch() {
        let events = vec![
            ExecutionEvent::ExecutionStarted {
                execution_id: "test".to_string(),
                timestamp: chrono::Utc::now(),
                total_nodes: 2,
            },
            ExecutionEvent::NodeStarted {
                execution_id: "test".to_string(),
                node_id: "node1".to_string(),
                timestamp: chrono::Utc::now(),
            },
        ];

        let batch = EventBatch::new(events, 1);
        assert_eq!(batch.len(), 2);
        assert!(!batch.is_empty());
        assert_eq!(batch.sequence, 1);
    }
}