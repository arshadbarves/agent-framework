// Tools module for AgentGraph
// Provides a framework for integrating external tools and capabilities

/// Core traits and types for tools
pub mod traits;
/// Tool registry for managing and discovering tools
pub mod registry;
/// Tool execution engine with retry, timeout, and caching
pub mod execution;
/// Common tools for various tasks
pub mod common;

pub use traits::{Tool, ToolMetadata, ToolInput, ToolOutput, ToolError, ToolResult};
pub use registry::{ToolRegistry, ToolRegistryBuilder};
pub use execution::{ToolExecutor, ToolExecutionContext};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Tool configuration for execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    /// Maximum execution time for the tool
    pub timeout: Option<Duration>,
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Delay between retry attempts
    pub retry_delay: Duration,
    /// Whether to cache tool results
    pub cache_results: bool,
    /// Custom configuration parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

impl Default for ToolConfig {
    fn default() -> Self {
        Self {
            timeout: Some(Duration::from_secs(30)),
            max_retries: 3,
            retry_delay: Duration::from_millis(500),
            cache_results: true,
            parameters: HashMap::new(),
        }
    }
}

/// Tool execution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolStats {
    /// Number of times the tool has been executed
    pub execution_count: u64,
    /// Number of successful executions
    pub success_count: u64,
    /// Number of failed executions
    pub failure_count: u64,
    /// Average execution time in milliseconds
    pub avg_execution_time_ms: f64,
    /// Total execution time in milliseconds
    pub total_execution_time_ms: u64,
    /// Last execution timestamp
    pub last_execution: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for ToolStats {
    fn default() -> Self {
        Self {
            execution_count: 0,
            success_count: 0,
            failure_count: 0,
            avg_execution_time_ms: 0.0,
            total_execution_time_ms: 0,
            last_execution: None,
        }
    }
}

impl ToolStats {
    /// Update statistics after a tool execution
    pub fn update(&mut self, execution_time_ms: u64, success: bool) {
        self.execution_count += 1;
        self.total_execution_time_ms += execution_time_ms;
        self.avg_execution_time_ms = self.total_execution_time_ms as f64 / self.execution_count as f64;
        self.last_execution = Some(chrono::Utc::now());
        
        if success {
            self.success_count += 1;
        } else {
            self.failure_count += 1;
        }
    }
    
    /// Get success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        if self.execution_count == 0 {
            0.0
        } else {
            (self.success_count as f64 / self.execution_count as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_config_default() {
        let config = ToolConfig::default();
        assert_eq!(config.timeout, Some(Duration::from_secs(30)));
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.retry_delay, Duration::from_millis(500));
        assert!(config.cache_results);
        assert!(config.parameters.is_empty());
    }

    #[test]
    fn test_tool_stats_update() {
        let mut stats = ToolStats::default();
        
        // Test successful execution
        stats.update(100, true);
        assert_eq!(stats.execution_count, 1);
        assert_eq!(stats.success_count, 1);
        assert_eq!(stats.failure_count, 0);
        assert_eq!(stats.avg_execution_time_ms, 100.0);
        assert_eq!(stats.success_rate(), 100.0);
        
        // Test failed execution
        stats.update(200, false);
        assert_eq!(stats.execution_count, 2);
        assert_eq!(stats.success_count, 1);
        assert_eq!(stats.failure_count, 1);
        assert_eq!(stats.avg_execution_time_ms, 150.0);
        assert_eq!(stats.success_rate(), 50.0);
    }

    #[test]
    fn test_tool_stats_success_rate() {
        let mut stats = ToolStats::default();
        assert_eq!(stats.success_rate(), 0.0);
        
        stats.update(100, true);
        stats.update(100, true);
        stats.update(100, false);
        
        assert!((stats.success_rate() - 66.66666666666667).abs() < 0.0001);
    }
}
