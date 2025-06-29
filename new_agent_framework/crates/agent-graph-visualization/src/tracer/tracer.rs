//! Execution tracing for debugging and monitoring.

use crate::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Execution tracer for monitoring graph execution
#[derive(Debug)]
pub struct ExecutionTracer {
    /// Trace configuration
    config: TracerConfig,
    /// Active traces
    traces: Arc<RwLock<HashMap<String, ExecutionTrace>>>,
    /// Trace storage
    storage: Arc<dyn TraceStorage>,
}

/// Configuration for execution tracing
#[derive(Debug, Clone)]
pub struct TracerConfig {
    /// Enable detailed node tracing
    pub detailed_tracing: bool,
    /// Enable state snapshots
    pub capture_state_snapshots: bool,
    /// Enable performance metrics
    pub capture_metrics: bool,
    /// Maximum trace history to keep in memory
    pub max_traces_in_memory: usize,
    /// Trace retention duration
    pub trace_retention: Duration,
}

impl Default for TracerConfig {
    fn default() -> Self {
        Self {
            detailed_tracing: true,
            capture_state_snapshots: false,
            capture_metrics: true,
            max_traces_in_memory: 100,
            trace_retention: Duration::from_secs(24 * 60 * 60), // 24 hours
        }
    }
}

/// Complete execution trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrace {
    /// Unique trace ID
    pub id: String,
    /// Execution start time
    pub start_time: SystemTime,
    /// Execution end time
    pub end_time: Option<SystemTime>,
    /// Total execution duration
    pub duration: Option<Duration>,
    /// Execution status
    pub status: ExecutionStatus,
    /// Node execution traces
    pub node_traces: Vec<NodeTrace>,
    /// State snapshots
    pub state_snapshots: Vec<StateSnapshot>,
    /// Performance metrics
    pub metrics: ExecutionMetrics,
    /// Error information if failed
    pub error: Option<ExecutionError>,
    /// Custom metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Status of an execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionStatus {
    /// Execution is starting
    Starting,
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

/// Trace for a single node execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeTrace {
    /// Node ID
    pub node_id: String,
    /// Node name
    pub node_name: String,
    /// Execution start time
    pub start_time: SystemTime,
    /// Execution end time
    pub end_time: Option<SystemTime>,
    /// Execution duration
    pub duration: Option<Duration>,
    /// Node execution status
    pub status: NodeExecutionStatus,
    /// Input data (if captured)
    pub input: Option<serde_json::Value>,
    /// Output data (if captured)
    pub output: Option<serde_json::Value>,
    /// Error information if failed
    pub error: Option<String>,
    /// Performance metrics
    pub metrics: NodeMetrics,
}

/// Status of a node execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeExecutionStatus {
    /// Node is starting
    Starting,
    /// Node is running
    Running,
    /// Node completed successfully
    Completed,
    /// Node failed
    Failed,
    /// Node was skipped
    Skipped,
    /// Node timed out
    TimedOut,
}

/// State snapshot at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    /// Snapshot timestamp
    pub timestamp: SystemTime,
    /// Node ID where snapshot was taken
    pub node_id: Option<String>,
    /// State data
    pub state: serde_json::Value,
    /// Snapshot type
    pub snapshot_type: SnapshotType,
}

/// Type of state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SnapshotType {
    /// Before execution starts
    Initial,
    /// Before node execution
    BeforeNode,
    /// After node execution
    AfterNode,
    /// Final state after execution
    Final,
    /// Error state
    Error,
}

/// Execution performance metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecutionMetrics {
    /// Total nodes executed
    pub total_nodes: usize,
    /// Successful node executions
    pub successful_nodes: usize,
    /// Failed node executions
    pub failed_nodes: usize,
    /// Total execution time
    pub total_duration: Option<Duration>,
    /// Average node execution time
    pub avg_node_duration: Option<Duration>,
    /// Memory usage statistics
    pub memory_stats: MemoryStats,
    /// Custom metrics
    pub custom_metrics: HashMap<String, f64>,
}

/// Node performance metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NodeMetrics {
    /// Execution duration
    pub duration: Option<Duration>,
    /// Memory used
    pub memory_used: Option<u64>,
    /// CPU time used
    pub cpu_time: Option<Duration>,
    /// Number of operations performed
    pub operations_count: Option<u64>,
    /// Custom metrics
    pub custom_metrics: HashMap<String, f64>,
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryStats {
    /// Peak memory usage in bytes
    pub peak_memory: u64,
    /// Average memory usage in bytes
    pub avg_memory: u64,
    /// Memory allocations
    pub allocations: u64,
    /// Memory deallocations
    pub deallocations: u64,
}

/// Execution error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionError {
    /// Error message
    pub message: String,
    /// Error code
    pub code: Option<String>,
    /// Node where error occurred
    pub node_id: Option<String>,
    /// Error timestamp
    pub timestamp: SystemTime,
    /// Stack trace if available
    pub stack_trace: Option<String>,
}

impl ExecutionTracer {
    /// Create a new execution tracer
    pub fn new(config: TracerConfig, storage: Arc<dyn TraceStorage>) -> Self {
        Self {
            config,
            traces: Arc::new(RwLock::new(HashMap::new())),
            storage,
        }
    }

    /// Start tracing an execution
    pub async fn start_trace(&self, execution_id: String) -> CoreResult<()> {
        let trace = ExecutionTrace {
            id: execution_id.clone(),
            start_time: SystemTime::now(),
            end_time: None,
            duration: None,
            status: ExecutionStatus::Starting,
            node_traces: Vec::new(),
            state_snapshots: Vec::new(),
            metrics: ExecutionMetrics::default(),
            error: None,
            metadata: HashMap::new(),
        };

        let mut traces = self.traces.write().await;
        traces.insert(execution_id, trace);

        Ok(())
    }

    /// Record node execution start
    pub async fn trace_node_start(
        &self,
        execution_id: &str,
        node_id: String,
        node_name: String,
    ) -> CoreResult<()> {
        let mut traces = self.traces.write().await;
        if let Some(trace) = traces.get_mut(execution_id) {
            let node_trace = NodeTrace {
                node_id,
                node_name,
                start_time: SystemTime::now(),
                end_time: None,
                duration: None,
                status: NodeExecutionStatus::Starting,
                input: None,
                output: None,
                error: None,
                metrics: NodeMetrics::default(),
            };
            trace.node_traces.push(node_trace);
        }
        Ok(())
    }

    /// Record node execution completion
    pub async fn trace_node_complete(
        &self,
        execution_id: &str,
        node_id: &str,
        output: Option<serde_json::Value>,
        metrics: NodeMetrics,
    ) -> CoreResult<()> {
        let mut traces = self.traces.write().await;
        if let Some(trace) = traces.get_mut(execution_id) {
            if let Some(node_trace) = trace.node_traces.iter_mut()
                .find(|nt| nt.node_id == node_id && nt.end_time.is_none()) {
                
                node_trace.end_time = Some(SystemTime::now());
                node_trace.duration = node_trace.end_time
                    .and_then(|end| end.duration_since(node_trace.start_time).ok());
                node_trace.status = NodeExecutionStatus::Completed;
                node_trace.output = output;
                node_trace.metrics = metrics;
            }
        }
        Ok(())
    }

    /// Record node execution failure
    pub async fn trace_node_error(
        &self,
        execution_id: &str,
        node_id: &str,
        error: String,
    ) -> CoreResult<()> {
        let mut traces = self.traces.write().await;
        if let Some(trace) = traces.get_mut(execution_id) {
            if let Some(node_trace) = trace.node_traces.iter_mut()
                .find(|nt| nt.node_id == node_id && nt.end_time.is_none()) {
                
                node_trace.end_time = Some(SystemTime::now());
                node_trace.duration = node_trace.end_time
                    .and_then(|end| end.duration_since(node_trace.start_time).ok());
                node_trace.status = NodeExecutionStatus::Failed;
                node_trace.error = Some(error);
            }
        }
        Ok(())
    }

    /// Capture state snapshot
    pub async fn capture_state_snapshot(
        &self,
        execution_id: &str,
        state: serde_json::Value,
        snapshot_type: SnapshotType,
        node_id: Option<String>,
    ) -> CoreResult<()> {
        if !self.config.capture_state_snapshots {
            return Ok(());
        }

        let mut traces = self.traces.write().await;
        if let Some(trace) = traces.get_mut(execution_id) {
            let snapshot = StateSnapshot {
                timestamp: SystemTime::now(),
                node_id,
                state,
                snapshot_type,
            };
            trace.state_snapshots.push(snapshot);
        }
        Ok(())
    }

    /// Complete execution trace
    pub async fn complete_trace(
        &self,
        execution_id: &str,
        status: ExecutionStatus,
        error: Option<ExecutionError>,
    ) -> CoreResult<()> {
        let mut traces = self.traces.write().await;
        if let Some(trace) = traces.get_mut(execution_id) {
            trace.end_time = Some(SystemTime::now());
            trace.duration = trace.end_time
                .and_then(|end| end.duration_since(trace.start_time).ok());
            trace.status = status;
            trace.error = error;

            // Calculate metrics
            trace.metrics.total_nodes = trace.node_traces.len();
            trace.metrics.successful_nodes = trace.node_traces.iter()
                .filter(|nt| nt.status == NodeExecutionStatus::Completed)
                .count();
            trace.metrics.failed_nodes = trace.node_traces.iter()
                .filter(|nt| nt.status == NodeExecutionStatus::Failed)
                .count();
            trace.metrics.total_duration = trace.duration;

            if !trace.node_traces.is_empty() {
                let total_node_duration: Duration = trace.node_traces.iter()
                    .filter_map(|nt| nt.duration)
                    .sum();
                trace.metrics.avg_node_duration = Some(total_node_duration / trace.node_traces.len() as u32);
            }

            // Store trace
            self.storage.store_trace(trace.clone()).await?;
        }
        Ok(())
    }

    /// Get trace by ID
    pub async fn get_trace(&self, execution_id: &str) -> Option<ExecutionTrace> {
        let traces = self.traces.read().await;
        traces.get(execution_id).cloned()
    }

    /// List all traces
    pub async fn list_traces(&self) -> Vec<String> {
        let traces = self.traces.read().await;
        traces.keys().cloned().collect()
    }

    /// Get traces by status
    pub async fn get_traces_by_status(&self, status: ExecutionStatus) -> Vec<ExecutionTrace> {
        let traces = self.traces.read().await;
        traces.values()
            .filter(|trace| trace.status == status)
            .cloned()
            .collect()
    }

    /// Clean up old traces
    pub async fn cleanup_old_traces(&self) -> CoreResult<()> {
        let cutoff = SystemTime::now() - self.config.trace_retention;
        let mut traces = self.traces.write().await;
        
        traces.retain(|_, trace| {
            trace.start_time > cutoff
        });

        Ok(())
    }
}

/// Trait for trace storage backends
#[async_trait::async_trait]
pub trait TraceStorage: Send + Sync + std::fmt::Debug {
    /// Store a trace
    async fn store_trace(&self, trace: ExecutionTrace) -> CoreResult<()>;
    
    /// Retrieve a trace by ID
    async fn get_trace(&self, trace_id: &str) -> CoreResult<Option<ExecutionTrace>>;
    
    /// List traces with optional filters
    async fn list_traces(&self, filter: Option<TraceFilter>) -> CoreResult<Vec<ExecutionTrace>>;
    
    /// Delete a trace
    async fn delete_trace(&self, trace_id: &str) -> CoreResult<()>;
}

/// Filter for trace queries
#[derive(Debug, Clone)]
pub struct TraceFilter {
    /// Filter by status
    pub status: Option<ExecutionStatus>,
    /// Filter by time range
    pub time_range: Option<(SystemTime, SystemTime)>,
    /// Filter by node ID
    pub node_id: Option<String>,
    /// Limit number of results
    pub limit: Option<usize>,
}

/// In-memory trace storage (for testing/development)
#[derive(Debug, Default)]
pub struct InMemoryTraceStorage {
    traces: Arc<RwLock<HashMap<String, ExecutionTrace>>>,
}

impl InMemoryTraceStorage {
    /// Create a new in-memory storage
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait::async_trait]
impl TraceStorage for InMemoryTraceStorage {
    async fn store_trace(&self, trace: ExecutionTrace) -> CoreResult<()> {
        let mut traces = self.traces.write().await;
        traces.insert(trace.id.clone(), trace);
        Ok(())
    }

    async fn get_trace(&self, trace_id: &str) -> CoreResult<Option<ExecutionTrace>> {
        let traces = self.traces.read().await;
        Ok(traces.get(trace_id).cloned())
    }

    async fn list_traces(&self, filter: Option<TraceFilter>) -> CoreResult<Vec<ExecutionTrace>> {
        let traces = self.traces.read().await;
        let mut result: Vec<ExecutionTrace> = traces.values().cloned().collect();

        if let Some(filter) = filter {
            if let Some(status) = filter.status {
                result.retain(|trace| trace.status == status);
            }

            if let Some((start, end)) = filter.time_range {
                result.retain(|trace| trace.start_time >= start && trace.start_time <= end);
            }

            if let Some(node_id) = filter.node_id {
                result.retain(|trace| {
                    trace.node_traces.iter().any(|nt| nt.node_id == node_id)
                });
            }

            if let Some(limit) = filter.limit {
                result.truncate(limit);
            }
        }

        Ok(result)
    }

    async fn delete_trace(&self, trace_id: &str) -> CoreResult<()> {
        let mut traces = self.traces.write().await;
        traces.remove(trace_id);
        Ok(())
    }
}