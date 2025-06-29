//! Execution context for graph runs.

use crate::error::{CoreError, CoreResult};
use crate::runtime::ExecutionConfig;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

/// Execution context for a graph run
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Unique execution ID
    pub execution_id: Uuid,
    /// Execution configuration
    pub config: ExecutionConfig,
    /// Start time
    pub start_time: DateTime<Utc>,
    /// Execution metadata
    pub metadata: Arc<parking_lot::RwLock<HashMap<String, serde_json::Value>>>,
    /// Execution metrics
    pub metrics: Arc<parking_lot::RwLock<ExecutionMetrics>>,
    /// Current execution depth
    pub depth: usize,
}

impl ExecutionContext {
    /// Create a new execution context
    pub fn new(config: ExecutionConfig) -> Self {
        Self {
            execution_id: Uuid::new_v4(),
            config,
            start_time: Utc::now(),
            metadata: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            metrics: Arc::new(parking_lot::RwLock::new(ExecutionMetrics::default())),
            depth: 0,
        }
    }

    /// Create a child context for nested execution
    pub fn create_child(&self, config: ExecutionConfig) -> CoreResult<Self> {
        if self.depth >= 100 { // Prevent excessive nesting
            return Err(CoreError::execution_error("Maximum execution depth exceeded"));
        }

        Ok(Self {
            execution_id: Uuid::new_v4(),
            config,
            start_time: Utc::now(),
            metadata: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            metrics: Arc::new(parking_lot::RwLock::new(ExecutionMetrics::default())),
            depth: self.depth + 1,
        })
    }

    /// Get execution duration
    pub fn duration(&self) -> Duration {
        let now = Utc::now();
        (now - self.start_time).to_std().unwrap_or_default()
    }

    /// Set metadata value
    pub fn set_metadata(&self, key: String, value: serde_json::Value) {
        let mut metadata = self.metadata.write();
        metadata.insert(key, value);
    }

    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<serde_json::Value> {
        let metadata = self.metadata.read();
        metadata.get(key).cloned()
    }

    /// Update metrics
    pub fn update_metrics<F>(&self, f: F)
    where
        F: FnOnce(&mut ExecutionMetrics),
    {
        let mut metrics = self.metrics.write();
        f(&mut *metrics);
    }

    /// Get current metrics snapshot
    pub fn get_metrics(&self) -> ExecutionMetrics {
        let metrics = self.metrics.read();
        metrics.clone()
    }

    /// Check if execution should timeout
    pub fn should_timeout(&self) -> bool {
        self.duration() > self.config.timeout
    }

    /// Check if parallel execution is enabled
    pub fn is_parallel_enabled(&self) -> bool {
        self.config.parallel_execution
    }

    /// Get maximum parallel nodes
    pub fn max_parallel_nodes(&self) -> usize {
        self.config.max_concurrency
    }

    /// Check if metrics collection is enabled
    pub fn is_metrics_enabled(&self) -> bool {
        self.config.collect_metrics
    }

    /// Check if state validation is enabled
    pub fn is_validation_enabled(&self) -> bool {
        self.config.validate_state
    }
}

/// Execution metrics collected during graph execution
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    /// Total number of nodes executed
    pub nodes_executed: usize,
    /// Number of nodes that succeeded
    pub nodes_succeeded: usize,
    /// Number of nodes that failed
    pub nodes_failed: usize,
    /// Total time spent executing nodes
    pub total_node_execution_time: Duration,
    /// Peak memory usage in MB
    pub memory_peak_mb: Option<f64>,
    /// Current memory usage in MB
    pub memory_current_mb: Option<f64>,
    /// Number of parallel executions
    pub parallel_executions: usize,
    /// Number of retries performed
    pub retries_performed: usize,
    /// Number of timeouts encountered
    pub timeouts_encountered: usize,
    /// Custom metrics
    pub custom_metrics: HashMap<String, f64>,
}

impl ExecutionMetrics {
    /// Record node execution
    pub fn record_node_execution(&mut self, duration: Duration, success: bool) {
        self.nodes_executed += 1;
        if success {
            self.nodes_succeeded += 1;
        } else {
            self.nodes_failed += 1;
        }
        self.total_node_execution_time += duration;
    }

    /// Record memory usage
    pub fn record_memory_usage(&mut self, memory_mb: f64) {
        self.memory_current_mb = Some(memory_mb);
        if let Some(peak) = self.memory_peak_mb {
            if memory_mb > peak {
                self.memory_peak_mb = Some(memory_mb);
            }
        } else {
            self.memory_peak_mb = Some(memory_mb);
        }
    }

    /// Record parallel execution
    pub fn record_parallel_execution(&mut self) {
        self.parallel_executions += 1;
    }

    /// Record retry
    pub fn record_retry(&mut self) {
        self.retries_performed += 1;
    }

    /// Record timeout
    pub fn record_timeout(&mut self) {
        self.timeouts_encountered += 1;
    }

    /// Add custom metric
    pub fn add_custom_metric(&mut self, name: String, value: f64) {
        self.custom_metrics.insert(name, value);
    }

    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        if self.nodes_executed == 0 {
            0.0
        } else {
            (self.nodes_succeeded as f64 / self.nodes_executed as f64) * 100.0
        }
    }

    /// Get average execution time per node
    pub fn average_execution_time(&self) -> Duration {
        if self.nodes_executed == 0 {
            Duration::default()
        } else {
            self.total_node_execution_time / self.nodes_executed as u32
        }
    }
}