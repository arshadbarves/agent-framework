//! Runtime configuration for the AgentGraph core.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Core runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Maximum number of concurrent node executions
    pub max_concurrency: usize,
    /// Default timeout for node execution
    pub default_timeout: Duration,
    /// Enable debug mode
    pub debug_mode: bool,
    /// Enable metrics collection
    pub metrics_enabled: bool,
    /// Memory limits
    pub memory_limits: MemoryLimits,
    /// Execution limits
    pub execution_limits: ExecutionLimits,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_concurrency: 10,
            default_timeout: Duration::from_secs(300), // 5 minutes
            debug_mode: false,
            metrics_enabled: true,
            memory_limits: MemoryLimits::default(),
            execution_limits: ExecutionLimits::default(),
        }
    }
}

/// Memory-related limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryLimits {
    /// Maximum memory per node execution (MB)
    pub max_memory_per_node_mb: Option<u64>,
    /// Maximum total memory usage (MB)
    pub max_total_memory_mb: Option<u64>,
    /// Enable memory monitoring
    pub monitor_memory: bool,
}

impl Default for MemoryLimits {
    fn default() -> Self {
        Self {
            max_memory_per_node_mb: Some(1024), // 1GB per node
            max_total_memory_mb: Some(8192),    // 8GB total
            monitor_memory: true,
        }
    }
}

/// Execution-related limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLimits {
    /// Maximum execution depth (prevents infinite loops)
    pub max_execution_depth: usize,
    /// Maximum number of nodes that can be executed
    pub max_nodes_executed: Option<usize>,
    /// Maximum total execution time
    pub max_total_execution_time: Option<Duration>,
}

impl Default for ExecutionLimits {
    fn default() -> Self {
        Self {
            max_execution_depth: 1000,
            max_nodes_executed: Some(10000),
            max_total_execution_time: Some(Duration::from_secs(3600)), // 1 hour
        }
    }
}

/// Execution configuration for a specific graph run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    /// Enable parallel execution
    pub parallel_execution: bool,
    /// Maximum concurrent nodes
    pub max_concurrency: usize,
    /// Execution timeout
    pub timeout: Duration,
    /// Collect execution metrics
    pub collect_metrics: bool,
    /// Validate state after each node
    pub validate_state: bool,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            parallel_execution: true,
            max_concurrency: 10,
            timeout: Duration::from_secs(300),
            collect_metrics: true,
            validate_state: false,
        }
    }
}

impl ExecutionConfig {
    /// Create a new execution config
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable parallel execution with max nodes
    pub fn with_parallel(mut self, max_nodes: usize) -> Self {
        self.parallel_execution = true;
        self.max_concurrency = max_nodes;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Enable metrics collection
    pub fn with_metrics(mut self) -> Self {
        self.collect_metrics = true;
        self
    }

    /// Enable state validation
    pub fn with_validation(mut self) -> Self {
        self.validate_state = true;
        self
    }
}