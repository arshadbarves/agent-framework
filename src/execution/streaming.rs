// Streaming execution for AgentGraph
// Provides real-time streaming of execution results and progress updates

#![allow(missing_docs)]

use super::{ExecutionConfig, ExecutionContext, NodeExecution, ExecutionStatus};
use crate::node::NodeId;
use crate::state::State;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio_stream::{Stream, StreamExt};
use thiserror::Error;

/// Streaming configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConfig {
    /// Enable streaming
    pub enabled: bool,
    /// Buffer size for streaming
    pub buffer_size: usize,
    /// Update interval
    pub update_interval: Duration,
    /// Include intermediate states
    pub include_intermediate_states: bool,
    /// Include node execution details
    pub include_node_details: bool,
    /// Maximum concurrent streams
    pub max_concurrent_streams: usize,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            buffer_size: 1000,
            update_interval: Duration::from_millis(100),
            include_intermediate_states: true,
            include_node_details: true,
            max_concurrent_streams: 100,
        }
    }
}

/// Stream event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum StreamEvent {
    /// Execution started
    ExecutionStarted {
        execution_id: String,
        timestamp: SystemTime,
        context: ExecutionContext,
    },
    
    /// Execution progress update
    ExecutionProgress {
        execution_id: String,
        timestamp: SystemTime,
        progress: ExecutionProgress,
    },
    
    /// Node execution started
    NodeStarted {
        execution_id: String,
        node_id: NodeId,
        timestamp: SystemTime,
        input_state: Option<State>,
    },
    
    /// Node execution completed
    NodeCompleted {
        execution_id: String,
        node_id: NodeId,
        timestamp: SystemTime,
        execution: NodeExecution,
        output_state: Option<State>,
    },
    
    /// Node execution failed
    NodeFailed {
        execution_id: String,
        node_id: NodeId,
        timestamp: SystemTime,
        error: String,
    },
    
    /// State update
    StateUpdate {
        execution_id: String,
        timestamp: SystemTime,
        state: State,
    },
    
    /// Execution completed
    ExecutionCompleted {
        execution_id: String,
        timestamp: SystemTime,
        final_state: State,
        statistics: ExecutionStatistics,
    },
    
    /// Execution failed
    ExecutionFailed {
        execution_id: String,
        timestamp: SystemTime,
        error: String,
        partial_state: Option<State>,
    },
    
    /// Custom event
    Custom {
        execution_id: String,
        timestamp: SystemTime,
        event_type: String,
        data: serde_json::Value,
    },
}

impl StreamEvent {
    /// Get execution ID
    pub fn execution_id(&self) -> &str {
        match self {
            StreamEvent::ExecutionStarted { execution_id, .. } => execution_id,
            StreamEvent::ExecutionProgress { execution_id, .. } => execution_id,
            StreamEvent::NodeStarted { execution_id, .. } => execution_id,
            StreamEvent::NodeCompleted { execution_id, .. } => execution_id,
            StreamEvent::NodeFailed { execution_id, .. } => execution_id,
            StreamEvent::StateUpdate { execution_id, .. } => execution_id,
            StreamEvent::ExecutionCompleted { execution_id, .. } => execution_id,
            StreamEvent::ExecutionFailed { execution_id, .. } => execution_id,
            StreamEvent::Custom { execution_id, .. } => execution_id,
        }
    }
    
    /// Get timestamp
    pub fn timestamp(&self) -> SystemTime {
        match self {
            StreamEvent::ExecutionStarted { timestamp, .. } => *timestamp,
            StreamEvent::ExecutionProgress { timestamp, .. } => *timestamp,
            StreamEvent::NodeStarted { timestamp, .. } => *timestamp,
            StreamEvent::NodeCompleted { timestamp, .. } => *timestamp,
            StreamEvent::NodeFailed { timestamp, .. } => *timestamp,
            StreamEvent::StateUpdate { timestamp, .. } => *timestamp,
            StreamEvent::ExecutionCompleted { timestamp, .. } => *timestamp,
            StreamEvent::ExecutionFailed { timestamp, .. } => *timestamp,
            StreamEvent::Custom { timestamp, .. } => *timestamp,
        }
    }
    
    /// Check if event is terminal (execution ending)
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            StreamEvent::ExecutionCompleted { .. } | StreamEvent::ExecutionFailed { .. }
        )
    }
}

/// Execution progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionProgress {
    /// Total nodes in execution
    pub total_nodes: usize,
    /// Completed nodes
    pub completed_nodes: usize,
    /// Failed nodes
    pub failed_nodes: usize,
    /// Currently running nodes
    pub running_nodes: usize,
    /// Progress percentage (0.0 - 1.0)
    pub progress_percentage: f64,
    /// Estimated time remaining
    pub estimated_time_remaining: Option<Duration>,
    /// Current execution status
    pub status: ExecutionStatus,
}

impl ExecutionProgress {
    /// Create new execution progress
    pub fn new(total_nodes: usize) -> Self {
        Self {
            total_nodes,
            completed_nodes: 0,
            failed_nodes: 0,
            running_nodes: 0,
            progress_percentage: 0.0,
            estimated_time_remaining: None,
            status: ExecutionStatus::Running,
        }
    }
    
    /// Update progress
    pub fn update(&mut self, completed: usize, failed: usize, running: usize) {
        self.completed_nodes = completed;
        self.failed_nodes = failed;
        self.running_nodes = running;
        
        if self.total_nodes > 0 {
            self.progress_percentage = (completed + failed) as f64 / self.total_nodes as f64;
        }
    }
    
    /// Check if execution is complete
    pub fn is_complete(&self) -> bool {
        self.completed_nodes + self.failed_nodes >= self.total_nodes
    }
}

/// Execution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStatistics {
    /// Total execution time
    pub total_time: Duration,
    /// Number of nodes executed
    pub nodes_executed: usize,
    /// Number of successful nodes
    pub successful_nodes: usize,
    /// Number of failed nodes
    pub failed_nodes: usize,
    /// Average node execution time
    pub average_node_time: Duration,
    /// Total tokens used (if applicable)
    pub total_tokens: u64,
    /// Total cost (if applicable)
    pub total_cost: f64,
}

/// Stream subscription
#[derive(Debug)]
pub struct StreamSubscription {
    /// Subscription ID
    pub id: String,
    /// Execution ID being streamed
    pub execution_id: String,
    /// Event receiver
    pub receiver: mpsc::UnboundedReceiver<StreamEvent>,
    /// Subscription filters
    pub filters: StreamFilters,
    /// Created timestamp
    pub created_at: SystemTime,
}

impl StreamSubscription {
    /// Create a new subscription
    pub fn new(
        execution_id: String,
        receiver: mpsc::UnboundedReceiver<StreamEvent>,
        filters: StreamFilters,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            execution_id,
            receiver,
            filters,
            created_at: SystemTime::now(),
        }
    }
}

/// Stream filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamFilters {
    /// Include execution events
    pub include_execution_events: bool,
    /// Include node events
    pub include_node_events: bool,
    /// Include state updates
    pub include_state_updates: bool,
    /// Include custom events
    pub include_custom_events: bool,
    /// Specific event types to include
    pub event_types: Option<Vec<String>>,
    /// Node IDs to filter by
    pub node_ids: Option<Vec<NodeId>>,
}

impl Default for StreamFilters {
    fn default() -> Self {
        Self {
            include_execution_events: true,
            include_node_events: true,
            include_state_updates: false,
            include_custom_events: false,
            event_types: None,
            node_ids: None,
        }
    }
}

impl StreamFilters {
    /// Check if event passes filters
    pub fn passes(&self, event: &StreamEvent) -> bool {
        match event {
            StreamEvent::ExecutionStarted { .. } | 
            StreamEvent::ExecutionProgress { .. } |
            StreamEvent::ExecutionCompleted { .. } |
            StreamEvent::ExecutionFailed { .. } => self.include_execution_events,
            
            StreamEvent::NodeStarted { node_id, .. } |
            StreamEvent::NodeCompleted { node_id, .. } |
            StreamEvent::NodeFailed { node_id, .. } => {
                self.include_node_events && 
                self.node_ids.as_ref().map_or(true, |ids| ids.contains(node_id))
            }
            
            StreamEvent::StateUpdate { .. } => self.include_state_updates,
            
            StreamEvent::Custom { event_type, .. } => {
                self.include_custom_events &&
                self.event_types.as_ref().map_or(true, |types| types.contains(event_type))
            }
        }
    }
}

/// Streaming manager
#[derive(Debug)]
pub struct StreamingManager {
    /// Configuration
    config: StreamingConfig,
    /// Active streams
    active_streams: Arc<RwLock<HashMap<String, broadcast::Sender<StreamEvent>>>>,
    /// Stream subscriptions
    subscriptions: Arc<RwLock<HashMap<String, StreamSubscription>>>,
    /// Execution progress tracking
    progress_tracking: Arc<RwLock<HashMap<String, ExecutionProgress>>>,
}

impl StreamingManager {
    /// Create a new streaming manager
    pub fn new(execution_config: ExecutionConfig) -> Self {
        let config = StreamingConfig {
            enabled: execution_config.streaming_enabled,
            ..Default::default()
        };
        
        Self {
            config,
            active_streams: Arc::new(RwLock::new(HashMap::new())),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            progress_tracking: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Start streaming for execution
    pub async fn start_stream(&self, execution_id: String, total_nodes: usize) -> Result<(), StreamingError> {
        if !self.config.enabled {
            return Err(StreamingError::StreamingDisabled);
        }
        
        let (sender, _) = broadcast::channel(self.config.buffer_size);
        
        // Add to active streams
        {
            let mut streams = self.active_streams.write().await;
            streams.insert(execution_id.clone(), sender);
        }
        
        // Initialize progress tracking
        {
            let mut progress = self.progress_tracking.write().await;
            progress.insert(execution_id.clone(), ExecutionProgress::new(total_nodes));
        }
        
        Ok(())
    }
    
    /// Subscribe to execution stream
    pub async fn subscribe(
        &self,
        execution_id: String,
        filters: StreamFilters,
    ) -> Result<StreamSubscription, StreamingError> {
        let streams = self.active_streams.read().await;
        let sender = streams.get(&execution_id)
            .ok_or_else(|| StreamingError::StreamNotFound {
                execution_id: execution_id.clone(),
            })?;
        
        let mut receiver = sender.subscribe();
        let (tx, rx) = mpsc::unbounded_channel();
        
        // Spawn task to filter and forward events
        let filters_clone = filters.clone();
        tokio::spawn(async move {
            while let Ok(event) = receiver.recv().await {
                if filters_clone.passes(&event) {
                    if tx.send(event).is_err() {
                        break; // Receiver dropped
                    }
                }
            }
        });
        
        let subscription = StreamSubscription::new(execution_id, rx, filters);
        let subscription_id = subscription.id.clone();
        
        // Store subscription
        {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.insert(subscription_id, subscription);
        }
        
        // Return the subscription we just stored
        let subscriptions = self.subscriptions.read().await;
        let subscription = subscriptions.get(&subscription_id).unwrap();
        
        // We need to create a new subscription with a new receiver
        // since we can't move out of the HashMap
        let (new_tx, new_rx) = mpsc::unbounded_channel();
        let new_subscription = StreamSubscription::new(
            subscription.execution_id.clone(),
            new_rx,
            subscription.filters.clone(),
        );
        
        Ok(new_subscription)
    }
    
    /// Emit event to stream
    pub async fn emit_event(&self, event: StreamEvent) -> Result<(), StreamingError> {
        let execution_id = event.execution_id().to_string();
        
        // Update progress tracking if applicable
        if let StreamEvent::NodeCompleted { .. } | StreamEvent::NodeFailed { .. } = &event {
            self.update_progress(&execution_id).await?;
        }
        
        // Send event to stream
        let streams = self.active_streams.read().await;
        if let Some(sender) = streams.get(&execution_id) {
            let _ = sender.send(event); // Ignore if no receivers
        }
        
        Ok(())
    }
    
    /// Update execution progress
    async fn update_progress(&self, execution_id: &str) -> Result<(), StreamingError> {
        let mut progress_map = self.progress_tracking.write().await;
        if let Some(progress) = progress_map.get_mut(execution_id) {
            // This is a simplified update - in practice, you'd track actual counts
            progress.completed_nodes += 1;
            progress.update(progress.completed_nodes, progress.failed_nodes, progress.running_nodes);
            
            // Emit progress event
            let progress_event = StreamEvent::ExecutionProgress {
                execution_id: execution_id.to_string(),
                timestamp: SystemTime::now(),
                progress: progress.clone(),
            };
            
            drop(progress_map); // Release lock before emitting
            self.emit_event(progress_event).await?;
        }
        
        Ok(())
    }
    
    /// End streaming for execution
    pub async fn end_stream(&self, execution_id: &str) -> Result<(), StreamingError> {
        // Remove from active streams
        {
            let mut streams = self.active_streams.write().await;
            streams.remove(execution_id);
        }
        
        // Remove progress tracking
        {
            let mut progress = self.progress_tracking.write().await;
            progress.remove(execution_id);
        }
        
        // Clean up subscriptions for this execution
        {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.retain(|_, sub| sub.execution_id != execution_id);
        }
        
        Ok(())
    }
    
    /// Get streaming statistics
    pub async fn get_stats(&self) -> StreamingStats {
        let streams = self.active_streams.read().await;
        let subscriptions = self.subscriptions.read().await;
        
        StreamingStats {
            active_streams: streams.len(),
            total_subscriptions: subscriptions.len(),
            buffer_size: self.config.buffer_size,
            streaming_enabled: self.config.enabled,
        }
    }
    
    /// Get configuration
    pub fn config(&self) -> &StreamingConfig {
        &self.config
    }
}

/// Streaming statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingStats {
    /// Number of active streams
    pub active_streams: usize,
    /// Total number of subscriptions
    pub total_subscriptions: usize,
    /// Buffer size
    pub buffer_size: usize,
    /// Streaming enabled
    pub streaming_enabled: bool,
}

/// Streaming errors
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum StreamingError {
    /// Streaming is disabled
    #[error("Streaming is disabled")]
    StreamingDisabled,
    
    /// Stream not found
    #[error("Stream not found for execution: {execution_id}")]
    StreamNotFound { execution_id: String },
    
    /// Subscription not found
    #[error("Subscription not found: {subscription_id}")]
    SubscriptionNotFound { subscription_id: String },
    
    /// Buffer overflow
    #[error("Stream buffer overflow for execution: {execution_id}")]
    BufferOverflow { execution_id: String },
    
    /// Too many streams
    #[error("Too many concurrent streams: {current}/{limit}")]
    TooManyStreams { current: usize, limit: usize },
    
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
    fn test_streaming_config_default() {
        let config = StreamingConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.buffer_size, 1000);
        assert!(config.include_intermediate_states);
    }

    #[test]
    fn test_execution_progress() {
        let mut progress = ExecutionProgress::new(10);
        assert_eq!(progress.total_nodes, 10);
        assert_eq!(progress.progress_percentage, 0.0);
        assert!(!progress.is_complete());
        
        progress.update(5, 0, 2);
        assert_eq!(progress.completed_nodes, 5);
        assert_eq!(progress.progress_percentage, 0.5);
        assert!(!progress.is_complete());
        
        progress.update(8, 2, 0);
        assert!(progress.is_complete());
    }

    #[test]
    fn test_stream_filters() {
        let filters = StreamFilters::default();
        
        let execution_event = StreamEvent::ExecutionStarted {
            execution_id: "test".to_string(),
            timestamp: SystemTime::now(),
            context: ExecutionContext::new(ExecutionConfig::default(), State::new()),
        };
        
        assert!(filters.passes(&execution_event));
        
        let state_event = StreamEvent::StateUpdate {
            execution_id: "test".to_string(),
            timestamp: SystemTime::now(),
            state: State::new(),
        };
        
        assert!(!filters.passes(&state_event)); // State updates disabled by default
    }

    #[test]
    fn test_stream_event_properties() {
        let event = StreamEvent::ExecutionStarted {
            execution_id: "test_execution".to_string(),
            timestamp: SystemTime::now(),
            context: ExecutionContext::new(ExecutionConfig::default(), State::new()),
        };
        
        assert_eq!(event.execution_id(), "test_execution");
        assert!(!event.is_terminal());
        
        let terminal_event = StreamEvent::ExecutionCompleted {
            execution_id: "test_execution".to_string(),
            timestamp: SystemTime::now(),
            final_state: State::new(),
            statistics: ExecutionStatistics {
                total_time: Duration::from_secs(10),
                nodes_executed: 5,
                successful_nodes: 4,
                failed_nodes: 1,
                average_node_time: Duration::from_secs(2),
                total_tokens: 1000,
                total_cost: 0.05,
            },
        };
        
        assert!(terminal_event.is_terminal());
    }

    #[tokio::test]
    async fn test_streaming_manager_creation() {
        let config = ExecutionConfig::default();
        let manager = StreamingManager::new(config);
        
        let stats = manager.get_stats().await;
        assert_eq!(stats.active_streams, 0);
        assert_eq!(stats.total_subscriptions, 0);
    }
}
