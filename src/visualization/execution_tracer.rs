//! Real-time execution tracing for AgentGraph workflows
//! Provides LangSmith-style execution monitoring and debugging

use crate::error::GraphResult;
use crate::visualization::{VisualExecutionEvent, VisualEventType, ExecutionTrace, ExecutionStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use uuid::Uuid;

/// Real-time execution tracer
#[derive(Debug)]
pub struct ExecutionTracer {
    /// Active execution traces
    traces: Arc<RwLock<HashMap<String, ExecutionTrace>>>,
    /// Event broadcaster for real-time updates
    event_broadcaster: broadcast::Sender<VisualExecutionEvent>,
    /// Maximum number of traces to keep
    max_traces: usize,
    /// Whether tracing is enabled
    enabled: bool,
}

impl ExecutionTracer {
    /// Create a new execution tracer
    pub fn new(max_traces: usize, enabled: bool) -> Self {
        let (event_broadcaster, _) = broadcast::channel(1000);
        
        Self {
            traces: Arc::new(RwLock::new(HashMap::new())),
            event_broadcaster,
            max_traces,
            enabled,
        }
    }

    /// Start tracing a new execution
    pub async fn start_execution(&self, execution_id: String, workflow_id: String) -> GraphResult<()> {
        if !self.enabled {
            return Ok(());
        }

        let trace = ExecutionTrace {
            id: Uuid::new_v4().to_string(),
            execution_id: execution_id.clone(),
            workflow_id,
            start_time: chrono::Utc::now(),
            end_time: None,
            events: Vec::new(),
            status: ExecutionStatus::Running,
            error: None,
        };

        // Add to traces
        let mut traces = self.traces.write().await;
        traces.insert(execution_id.clone(), trace);

        // Cleanup old traces if needed
        if traces.len() > self.max_traces {
            let oldest_key = traces.keys().next().cloned();
            if let Some(key) = oldest_key {
                traces.remove(&key);
            }
        }

        // Broadcast execution started event
        let event = VisualExecutionEvent {
            id: Uuid::new_v4().to_string(),
            execution_id,
            event_type: VisualEventType::ExecutionStarted,
            node_id: None,
            timestamp: chrono::Utc::now(),
            data: serde_json::json!({}),
            context: HashMap::new(),
        };

        let _ = self.event_broadcaster.send(event);
        Ok(())
    }

    /// End execution tracing
    pub async fn end_execution(&self, execution_id: &str, status: ExecutionStatus, error: Option<String>) -> GraphResult<()> {
        if !self.enabled {
            return Ok(());
        }

        let mut traces = self.traces.write().await;
        if let Some(trace) = traces.get_mut(execution_id) {
            trace.end_time = Some(chrono::Utc::now());
            trace.status = status.clone();
            trace.error = error.clone();

            // Broadcast execution completed event
            let event_type = match status {
                ExecutionStatus::Completed => VisualEventType::ExecutionCompleted,
                ExecutionStatus::Failed => VisualEventType::ExecutionFailed,
                ExecutionStatus::Cancelled => VisualEventType::Custom("ExecutionCancelled".to_string()),
                _ => VisualEventType::ExecutionCompleted,
            };

            let event = VisualExecutionEvent {
                id: Uuid::new_v4().to_string(),
                execution_id: execution_id.to_string(),
                event_type,
                node_id: None,
                timestamp: chrono::Utc::now(),
                data: serde_json::json!({
                    "status": status,
                    "error": error
                }),
                context: HashMap::new(),
            };

            let _ = self.event_broadcaster.send(event);
        }

        Ok(())
    }

    /// Trace node execution start
    pub async fn trace_node_start(&self, execution_id: &str, node_id: &str, node_type: &str) -> GraphResult<()> {
        if !self.enabled {
            return Ok(());
        }

        let event = VisualExecutionEvent {
            id: Uuid::new_v4().to_string(),
            execution_id: execution_id.to_string(),
            event_type: VisualEventType::NodeStarted,
            node_id: Some(node_id.to_string()),
            timestamp: chrono::Utc::now(),
            data: serde_json::json!({
                "node_type": node_type
            }),
            context: HashMap::new(),
        };

        self.add_event(execution_id, event.clone()).await?;
        let _ = self.event_broadcaster.send(event);
        Ok(())
    }

    /// Trace node execution completion
    pub async fn trace_node_complete(&self, execution_id: &str, node_id: &str, duration_ms: u64, output: Option<serde_json::Value>) -> GraphResult<()> {
        if !self.enabled {
            return Ok(());
        }

        let event = VisualExecutionEvent {
            id: Uuid::new_v4().to_string(),
            execution_id: execution_id.to_string(),
            event_type: VisualEventType::NodeCompleted,
            node_id: Some(node_id.to_string()),
            timestamp: chrono::Utc::now(),
            data: serde_json::json!({
                "duration_ms": duration_ms,
                "output": output
            }),
            context: HashMap::new(),
        };

        self.add_event(execution_id, event.clone()).await?;
        let _ = self.event_broadcaster.send(event);
        Ok(())
    }

    /// Trace node execution failure
    pub async fn trace_node_failure(&self, execution_id: &str, node_id: &str, error: &str) -> GraphResult<()> {
        if !self.enabled {
            return Ok(());
        }

        let event = VisualExecutionEvent {
            id: Uuid::new_v4().to_string(),
            execution_id: execution_id.to_string(),
            event_type: VisualEventType::NodeFailed,
            node_id: Some(node_id.to_string()),
            timestamp: chrono::Utc::now(),
            data: serde_json::json!({
                "error": error
            }),
            context: HashMap::new(),
        };

        self.add_event(execution_id, event.clone()).await?;
        let _ = self.event_broadcaster.send(event);
        Ok(())
    }

    /// Trace agent response
    pub async fn trace_agent_response(&self, execution_id: &str, node_id: &str, agent_name: &str, response: &str, tokens_used: u32) -> GraphResult<()> {
        if !self.enabled {
            return Ok(());
        }

        let event = VisualExecutionEvent {
            id: Uuid::new_v4().to_string(),
            execution_id: execution_id.to_string(),
            event_type: VisualEventType::AgentResponse,
            node_id: Some(node_id.to_string()),
            timestamp: chrono::Utc::now(),
            data: serde_json::json!({
                "agent_name": agent_name,
                "response": response,
                "tokens_used": tokens_used,
                "response_length": response.len()
            }),
            context: HashMap::new(),
        };

        self.add_event(execution_id, event.clone()).await?;
        let _ = self.event_broadcaster.send(event);
        Ok(())
    }

    /// Trace tool execution
    pub async fn trace_tool_execution(&self, execution_id: &str, node_id: &str, tool_name: &str, input: &serde_json::Value, output: &serde_json::Value) -> GraphResult<()> {
        if !self.enabled {
            return Ok(());
        }

        let event = VisualExecutionEvent {
            id: Uuid::new_v4().to_string(),
            execution_id: execution_id.to_string(),
            event_type: VisualEventType::ToolExecution,
            node_id: Some(node_id.to_string()),
            timestamp: chrono::Utc::now(),
            data: serde_json::json!({
                "tool_name": tool_name,
                "input": input,
                "output": output
            }),
            context: HashMap::new(),
        };

        self.add_event(execution_id, event.clone()).await?;
        let _ = self.event_broadcaster.send(event);
        Ok(())
    }

    /// Trace command routing
    pub async fn trace_command_routing(&self, execution_id: &str, node_id: &str, command: &str, target_node: Option<&str>) -> GraphResult<()> {
        if !self.enabled {
            return Ok(());
        }

        let event = VisualExecutionEvent {
            id: Uuid::new_v4().to_string(),
            execution_id: execution_id.to_string(),
            event_type: VisualEventType::CommandRouting,
            node_id: Some(node_id.to_string()),
            timestamp: chrono::Utc::now(),
            data: serde_json::json!({
                "command": command,
                "target_node": target_node
            }),
            context: HashMap::new(),
        };

        self.add_event(execution_id, event.clone()).await?;
        let _ = self.event_broadcaster.send(event);
        Ok(())
    }

    /// Trace state update
    pub async fn trace_state_update(&self, execution_id: &str, node_id: Option<&str>, key: &str, value: &serde_json::Value) -> GraphResult<()> {
        if !self.enabled {
            return Ok(());
        }

        let event = VisualExecutionEvent {
            id: Uuid::new_v4().to_string(),
            execution_id: execution_id.to_string(),
            event_type: VisualEventType::StateUpdate,
            node_id: node_id.map(|s| s.to_string()),
            timestamp: chrono::Utc::now(),
            data: serde_json::json!({
                "key": key,
                "value": value
            }),
            context: HashMap::new(),
        };

        self.add_event(execution_id, event.clone()).await?;
        let _ = self.event_broadcaster.send(event);
        Ok(())
    }

    /// Add event to trace
    async fn add_event(&self, execution_id: &str, event: VisualExecutionEvent) -> GraphResult<()> {
        let mut traces = self.traces.write().await;
        if let Some(trace) = traces.get_mut(execution_id) {
            trace.events.push(event);
        }
        Ok(())
    }

    /// Get execution trace
    pub async fn get_trace(&self, execution_id: &str) -> Option<ExecutionTrace> {
        let traces = self.traces.read().await;
        traces.get(execution_id).cloned()
    }

    /// Get all traces
    pub async fn get_all_traces(&self) -> Vec<ExecutionTrace> {
        let traces = self.traces.read().await;
        traces.values().cloned().collect()
    }

    /// Subscribe to real-time events
    pub fn subscribe_events(&self) -> broadcast::Receiver<VisualExecutionEvent> {
        self.event_broadcaster.subscribe()
    }

    /// Get trace statistics
    pub async fn get_trace_stats(&self) -> TraceStats {
        let traces = self.traces.read().await;
        let total_traces = traces.len();
        let running_traces = traces.values().filter(|t| matches!(t.status, ExecutionStatus::Running)).count();
        let completed_traces = traces.values().filter(|t| matches!(t.status, ExecutionStatus::Completed)).count();
        let failed_traces = traces.values().filter(|t| matches!(t.status, ExecutionStatus::Failed)).count();

        TraceStats {
            total_traces,
            running_traces,
            completed_traces,
            failed_traces,
        }
    }

    /// Check if tracing is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enable/disable tracing
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

/// Trace statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceStats {
    /// Total number of traces
    pub total_traces: usize,
    /// Number of running traces
    pub running_traces: usize,
    /// Number of completed traces
    pub completed_traces: usize,
    /// Number of failed traces
    pub failed_traces: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execution_tracer() {
        let tracer = ExecutionTracer::new(100, true);
        
        let execution_id = "test_execution".to_string();
        let workflow_id = "test_workflow".to_string();
        
        // Start execution
        tracer.start_execution(execution_id.clone(), workflow_id).await.unwrap();
        
        // Trace node execution
        tracer.trace_node_start(&execution_id, "node1", "agent").await.unwrap();
        tracer.trace_node_complete(&execution_id, "node1", 100, None).await.unwrap();
        
        // End execution
        tracer.end_execution(&execution_id, ExecutionStatus::Completed, None).await.unwrap();
        
        // Check trace
        let trace = tracer.get_trace(&execution_id).await.unwrap();
        assert_eq!(trace.execution_id, execution_id);
        assert!(matches!(trace.status, ExecutionStatus::Completed));
        assert!(!trace.events.is_empty());
    }

    #[tokio::test]
    async fn test_event_subscription() {
        let tracer = ExecutionTracer::new(100, true);
        let mut receiver = tracer.subscribe_events();
        
        // Start execution in background
        let execution_id = "test_execution".to_string();
        let workflow_id = "test_workflow".to_string();
        
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            tracer.start_execution(execution_id, workflow_id).await.unwrap();
        });
        
        // Receive event
        let event = receiver.recv().await.unwrap();
        assert!(matches!(event.event_type, VisualEventType::ExecutionStarted));
    }
}
