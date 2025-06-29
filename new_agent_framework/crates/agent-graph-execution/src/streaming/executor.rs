//! Streaming execution engine for real-time updates.

use crate::{CoreError, CoreResult, State, NodeId};
use crate::streaming::{ExecutionEvent, EventFilter, SubscriptionConfig};
use agent_graph_core::{Graph, ExecutionContext};
use async_stream::stream;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio_stream::wrappers::BroadcastStream;
use tracing::{info, debug, error};

/// Streaming execution engine for real-time updates
#[derive(Debug)]
pub struct StreamingExecutor<S>
where
    S: State,
{
    config: StreamingConfig,
    event_sender: broadcast::Sender<ExecutionEvent>,
    active_streams: Arc<RwLock<HashMap<String, StreamHandle>>>,
}

impl<S> StreamingExecutor<S>
where
    S: State + Send + Sync + 'static,
{
    /// Create a new streaming executor
    pub fn new(config: StreamingConfig) -> Self {
        let (event_sender, _) = broadcast::channel(config.buffer_size);
        
        Self {
            config,
            event_sender,
            active_streams: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Execute a graph with streaming updates
    pub async fn execute_graph_streaming(
        &self,
        graph: &Graph<S>,
        context: ExecutionContext,
    ) -> CoreResult<ExecutionStream> {
        let execution_id = context.execution_id.to_string();
        info!("Starting streaming execution: {}", execution_id);

        // Create execution stream
        let stream = self.create_execution_stream(graph, context).await?;
        
        // Register stream
        let handle = StreamHandle {
            execution_id: execution_id.clone(),
            start_time: chrono::Utc::now(),
            status: StreamStatus::Running,
        };
        
        self.active_streams.write().await.insert(execution_id, handle);
        
        Ok(stream)
    }

    /// Create an execution stream
    async fn create_execution_stream(
        &self,
        graph: &Graph<S>,
        context: ExecutionContext,
    ) -> CoreResult<ExecutionStream> {
        let event_sender = self.event_sender.clone();
        let graph_clone = graph.clone_structure();
        let active_streams = self.active_streams.clone();
        let execution_id = context.execution_id.to_string();
        let config = self.config.clone();
        
        // Send start event
        let start_event = ExecutionEvent::ExecutionStarted {
            execution_id: execution_id.clone(),
            timestamp: chrono::Utc::now(),
            total_nodes: graph.nodes.count(),
        };
        
        let _ = event_sender.send(start_event);
        
        // Create the stream
        let stream = stream! {
            let mut current_nodes = graph_clone.entry_points.clone();
            let mut completed_nodes = std::collections::HashSet::new();
            let mut execution_queue = std::collections::VecDeque::new();
            
            // Add entry points to queue
            for node_id in current_nodes {
                execution_queue.push_back(node_id);
            }
            
            while let Some(node_id) = execution_queue.pop_front() {
                // Send node start event
                let node_start_event = ExecutionEvent::NodeStarted {
                    execution_id: execution_id.clone(),
                    node_id: node_id.clone(),
                    timestamp: chrono::Utc::now(),
                };
                
                yield node_start_event;
                
                // Execute the node
                let node_result = Self::execute_node_with_streaming(
                    &graph_clone,
                    &node_id,
                    &context,
                    &event_sender,
                    &config,
                ).await;
                
                match node_result {
                    Ok(output) => {
                        completed_nodes.insert(node_id.clone());
                        
                        // Send node completed event
                        let node_completed_event = ExecutionEvent::NodeCompleted {
                            execution_id: execution_id.clone(),
                            node_id: node_id.clone(),
                            timestamp: chrono::Utc::now(),
                            success: output.success,
                            duration: output.duration,
                            output: output.output,
                        };
                        
                        yield node_completed_event;
                        
                        // Send state update event if configured
                        if config.include_state_updates {
                            let state_json = graph_clone.state_manager.read_state(|state| {
                                state.to_json().ok()
                            });
                            
                            if let Some(state) = state_json {
                                let state_event = ExecutionEvent::StateUpdated {
                                    execution_id: execution_id.clone(),
                                    timestamp: chrono::Utc::now(),
                                    state,
                                };
                                
                                yield state_event;
                            }
                        }
                        
                        // Send progress update
                        let progress_percentage = (completed_nodes.len() as f64 / graph_clone.nodes.count() as f64) * 100.0;
                        let progress_event = ExecutionEvent::ProgressUpdate {
                            execution_id: execution_id.clone(),
                            timestamp: chrono::Utc::now(),
                            completed_nodes: completed_nodes.len(),
                            total_nodes: graph_clone.nodes.count(),
                            current_node: execution_queue.front().cloned(),
                            progress_percentage,
                        };
                        
                        yield progress_event;
                        
                        // Determine next nodes
                        if output.continue_execution {
                            let next_nodes = if let Some(next_node) = output.next_node {
                                vec![next_node]
                            } else {
                                // Use edge routing to find next nodes
                                let outgoing_edges = graph_clone.edges.get_outgoing_edges(&node_id);
                                outgoing_edges.iter().map(|edge| edge.to.clone()).collect()
                            };
                            
                            for next_node in next_nodes {
                                if !completed_nodes.contains(&next_node) && 
                                   !execution_queue.contains(&next_node) {
                                    execution_queue.push_back(next_node);
                                }
                            }
                        }
                    }
                    Err(error) => {
                        error!("Node '{}' failed: {}", node_id, error);
                        
                        // Send node failed event
                        let node_failed_event = ExecutionEvent::NodeFailed {
                            execution_id: execution_id.clone(),
                            node_id: node_id.clone(),
                            timestamp: chrono::Utc::now(),
                            error: error.to_string(),
                        };
                        
                        yield node_failed_event;
                        
                        // Send execution failed event and stop
                        let execution_failed_event = ExecutionEvent::ExecutionFailed {
                            execution_id: execution_id.clone(),
                            timestamp: chrono::Utc::now(),
                            error: error.to_string(),
                            completed_nodes: completed_nodes.len(),
                        };
                        
                        yield execution_failed_event;
                        break;
                    }
                }
            }
            
            // Send execution completed event
            let execution_completed_event = ExecutionEvent::ExecutionCompleted {
                execution_id: execution_id.clone(),
                timestamp: chrono::Utc::now(),
                success: true,
                total_duration: context.duration(),
                completed_nodes: completed_nodes.len(),
            };
            
            yield execution_completed_event;
            
            // Update stream status
            if let Some(mut handle) = active_streams.write().await.get_mut(&execution_id) {
                handle.status = StreamStatus::Completed;
            }
        };
        
        Ok(ExecutionStream::new(Box::pin(stream)))
    }

    /// Execute a single node with streaming events
    async fn execute_node_with_streaming(
        graph: &Graph<S>,
        node_id: &NodeId,
        context: &ExecutionContext,
        event_sender: &broadcast::Sender<ExecutionEvent>,
        config: &StreamingConfig,
    ) -> CoreResult<StreamingNodeResult> {
        debug!("Executing node '{}' with streaming", node_id);
        
        let node = graph.nodes.get(node_id)
            .ok_or_else(|| CoreError::execution_error(format!("Node '{}' not found", node_id)))?;
        
        let start_time = std::time::Instant::now();
        
        // Send detailed execution events if configured
        if config.detailed_events {
            let pre_execution_event = ExecutionEvent::Custom {
                execution_id: context.execution_id.to_string(),
                timestamp: chrono::Utc::now(),
                event_type: "node_pre_execution".to_string(),
                data: serde_json::json!({
                    "node_id": node_id,
                    "metadata": node.metadata()
                }),
            };
            
            let _ = event_sender.send(pre_execution_event);
        }
        
        // Execute the node
        let result = graph.state_manager.write_state(|state| {
            node.execute(state)
        }).await;
        
        let duration = start_time.elapsed();
        
        // Update context metrics
        context.update_metrics(|metrics| {
            metrics.record_node_execution(duration, result.is_ok());
        });
        
        match result {
            Ok(output) => {
                if config.detailed_events {
                    let post_execution_event = ExecutionEvent::Custom {
                        execution_id: context.execution_id.to_string(),
                        timestamp: chrono::Utc::now(),
                        event_type: "node_post_execution".to_string(),
                        data: serde_json::json!({
                            "node_id": node_id,
                            "success": output.success,
                            "metrics": output.metrics
                        }),
                    };
                    
                    let _ = event_sender.send(post_execution_event);
                }
                
                Ok(StreamingNodeResult {
                    success: output.success,
                    duration,
                    output: Some(output),
                    continue_execution: output.continue_execution,
                    next_node: output.next_node,
                })
            }
            Err(error) => Err(error),
        }
    }

    /// Subscribe to execution events
    pub fn subscribe(&self) -> BroadcastStream<ExecutionEvent> {
        BroadcastStream::new(self.event_sender.subscribe())
    }

    /// Subscribe with filter
    pub fn subscribe_filtered(&self, filter: EventFilter) -> FilteredEventStream {
        let stream = BroadcastStream::new(self.event_sender.subscribe());
        FilteredEventStream::new(stream, filter)
    }

    /// Get active streams
    pub async fn get_active_streams(&self) -> Vec<StreamHandle> {
        self.active_streams.read().await.values().cloned().collect()
    }

    /// Stop a specific stream
    pub async fn stop_stream(&self, execution_id: &str) -> CoreResult<()> {
        if let Some(mut handle) = self.active_streams.write().await.get_mut(execution_id) {
            handle.status = StreamStatus::Stopped;
            info!("Stopped streaming execution: {}", execution_id);
            Ok(())
        } else {
            Err(CoreError::validation_error(format!(
                "Stream '{}' not found", execution_id
            )))
        }
    }
}

/// Configuration for streaming execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConfig {
    /// Buffer size for event channel
    pub buffer_size: usize,
    /// Include state updates in stream
    pub include_state_updates: bool,
    /// Include detailed execution events
    pub detailed_events: bool,
    /// Maximum stream duration
    pub max_stream_duration: std::time::Duration,
    /// Event sampling rate (1.0 = all events, 0.5 = half events)
    pub sampling_rate: f64,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1000,
            include_state_updates: true,
            detailed_events: false,
            max_stream_duration: std::time::Duration::from_hours(1),
            sampling_rate: 1.0,
        }
    }
}

/// Execution stream wrapper
pub struct ExecutionStream {
    inner: Pin<Box<dyn Stream<Item = ExecutionEvent> + Send>>,
}

impl ExecutionStream {
    /// Create a new execution stream
    pub fn new(stream: Pin<Box<dyn Stream<Item = ExecutionEvent> + Send>>) -> Self {
        Self { inner: stream }
    }
}

impl Stream for ExecutionStream {
    type Item = ExecutionEvent;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.inner.as_mut().poll_next(cx)
    }
}

/// Filtered event stream
pub struct FilteredEventStream {
    inner: BroadcastStream<ExecutionEvent>,
    filter: EventFilter,
}

impl FilteredEventStream {
    /// Create a new filtered event stream
    pub fn new(stream: BroadcastStream<ExecutionEvent>, filter: EventFilter) -> Self {
        Self {
            inner: stream,
            filter,
        }
    }
}

impl Stream for FilteredEventStream {
    type Item = Result<ExecutionEvent, tokio_stream::wrappers::errors::BroadcastStreamRecvError>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        loop {
            match Pin::new(&mut self.inner).poll_next(cx) {
                std::task::Poll::Ready(Some(Ok(event))) => {
                    if self.filter.matches(&event) {
                        return std::task::Poll::Ready(Some(Ok(event)));
                    }
                    // Continue polling if event doesn't match filter
                }
                std::task::Poll::Ready(Some(Err(e))) => {
                    return std::task::Poll::Ready(Some(Err(e)));
                }
                std::task::Poll::Ready(None) => {
                    return std::task::Poll::Ready(None);
                }
                std::task::Poll::Pending => {
                    return std::task::Poll::Pending;
                }
            }
        }
    }
}

/// Handle for tracking active streams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamHandle {
    /// Execution ID
    pub execution_id: String,
    /// Stream start time
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// Current status
    pub status: StreamStatus,
}

/// Status of a streaming execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StreamStatus {
    /// Stream is running
    Running,
    /// Stream completed successfully
    Completed,
    /// Stream failed
    Failed,
    /// Stream was stopped
    Stopped,
}

/// Result of streaming node execution
#[derive(Debug, Clone)]
struct StreamingNodeResult {
    success: bool,
    duration: std::time::Duration,
    output: Option<agent_graph_core::node::NodeOutput>,
    continue_execution: bool,
    next_node: Option<NodeId>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;

    #[test]
    fn test_streaming_config_default() {
        let config = StreamingConfig::default();
        assert_eq!(config.buffer_size, 1000);
        assert!(config.include_state_updates);
        assert!(!config.detailed_events);
        assert_eq!(config.sampling_rate, 1.0);
    }

    #[test]
    fn test_stream_handle_creation() {
        let handle = StreamHandle {
            execution_id: "test-123".to_string(),
            start_time: chrono::Utc::now(),
            status: StreamStatus::Running,
        };
        
        assert_eq!(handle.execution_id, "test-123");
        assert_eq!(handle.status, StreamStatus::Running);
    }

    #[tokio::test]
    async fn test_streaming_executor_creation() {
        let config = StreamingConfig::default();
        let executor: StreamingExecutor<serde_json::Value> = StreamingExecutor::new(config);
        
        let active_streams = executor.get_active_streams().await;
        assert!(active_streams.is_empty());
    }
}