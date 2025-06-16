//! Metrics collection for AgentGraph performance monitoring
//! Provides LangSmith-style analytics and performance tracking

use crate::error::GraphResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::interval;

/// Metrics collector for performance analytics
#[derive(Debug)]
pub struct MetricsCollector {
    /// Current metrics data
    metrics: Arc<RwLock<SystemMetrics>>,
    /// Historical metrics
    history: Arc<RwLock<Vec<MetricsSnapshot>>>,
    /// Collection interval
    collection_interval: Duration,
    /// Whether collection is enabled
    enabled: bool,
    /// Collection task handle
    collection_task: Option<tokio::task::JoinHandle<()>>,
}

/// System-wide metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// Total number of executions
    pub total_executions: u64,
    /// Currently active executions
    pub active_executions: u64,
    /// Completed executions
    pub completed_executions: u64,
    /// Failed executions
    pub failed_executions: u64,
    /// Average execution time (milliseconds)
    pub avg_execution_time_ms: f64,
    /// Success rate (percentage)
    pub success_rate: f64,
    /// Node execution metrics
    pub node_metrics: HashMap<String, NodeMetrics>,
    /// Agent performance metrics
    pub agent_metrics: HashMap<String, AgentMetrics>,
    /// Tool usage metrics
    pub tool_metrics: HashMap<String, ToolMetrics>,
    /// System resource metrics
    pub resource_metrics: ResourceMetrics,
    /// Last updated timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Node-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    /// Node ID
    pub node_id: String,
    /// Node type
    pub node_type: String,
    /// Total executions
    pub total_executions: u64,
    /// Successful executions
    pub successful_executions: u64,
    /// Failed executions
    pub failed_executions: u64,
    /// Average execution time (ms)
    pub avg_execution_time_ms: f64,
    /// Min execution time (ms)
    pub min_execution_time_ms: f64,
    /// Max execution time (ms)
    pub max_execution_time_ms: f64,
    /// Success rate
    pub success_rate: f64,
    /// Last execution time
    pub last_execution: Option<chrono::DateTime<chrono::Utc>>,
}

/// Agent-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    /// Agent name
    pub agent_name: String,
    /// Total tasks executed
    pub total_tasks: u64,
    /// Total tokens used
    pub total_tokens: u64,
    /// Average tokens per task
    pub avg_tokens_per_task: f64,
    /// Average response time (ms)
    pub avg_response_time_ms: f64,
    /// Success rate
    pub success_rate: f64,
    /// Cost metrics (if available)
    pub total_cost: f64,
    /// Last activity
    pub last_activity: Option<chrono::DateTime<chrono::Utc>>,
}

/// Tool-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetrics {
    /// Tool name
    pub tool_name: String,
    /// Total executions
    pub total_executions: u64,
    /// Successful executions
    pub successful_executions: u64,
    /// Failed executions
    pub failed_executions: u64,
    /// Average execution time (ms)
    pub avg_execution_time_ms: f64,
    /// Success rate
    pub success_rate: f64,
    /// Last used
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
}

/// System resource metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Memory usage in MB
    pub memory_usage_mb: f64,
    /// Memory usage percentage
    pub memory_usage_percent: f64,
    /// Active threads
    pub active_threads: u64,
    /// Network requests per second
    pub network_rps: f64,
    /// Disk I/O operations per second
    pub disk_iops: f64,
}

/// Metrics snapshot for historical tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    /// Snapshot timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Metrics at this point in time
    pub metrics: SystemMetrics,
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            total_executions: 0,
            active_executions: 0,
            completed_executions: 0,
            failed_executions: 0,
            avg_execution_time_ms: 0.0,
            success_rate: 0.0,
            node_metrics: HashMap::new(),
            agent_metrics: HashMap::new(),
            tool_metrics: HashMap::new(),
            resource_metrics: ResourceMetrics::default(),
            last_updated: chrono::Utc::now(),
        }
    }
}

impl Default for ResourceMetrics {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage_mb: 0.0,
            memory_usage_percent: 0.0,
            active_threads: 0,
            network_rps: 0.0,
            disk_iops: 0.0,
        }
    }
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(enabled: bool, collection_interval_seconds: u64) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(SystemMetrics::default())),
            history: Arc::new(RwLock::new(Vec::new())),
            collection_interval: Duration::from_secs(collection_interval_seconds),
            enabled,
            collection_task: None,
        }
    }

    /// Start metrics collection
    pub async fn start(&mut self) -> GraphResult<()> {
        if !self.enabled {
            return Ok(());
        }

        let metrics = self.metrics.clone();
        let history = self.history.clone();
        let interval_duration = self.collection_interval;

        let task = tokio::spawn(async move {
            let mut interval = interval(interval_duration);
            
            loop {
                interval.tick().await;
                
                // Collect current metrics
                let current_metrics = Self::collect_system_metrics().await;
                
                // Update metrics
                {
                    let mut metrics_guard = metrics.write().await;
                    *metrics_guard = current_metrics.clone();
                }
                
                // Add to history
                {
                    let mut history_guard = history.write().await;
                    history_guard.push(MetricsSnapshot {
                        timestamp: chrono::Utc::now(),
                        metrics: current_metrics,
                    });
                    
                    // Keep only last 1000 snapshots
                    if history_guard.len() > 1000 {
                        history_guard.remove(0);
                    }
                }
            }
        });

        self.collection_task = Some(task);
        tracing::info!("Metrics collection started");
        Ok(())
    }

    /// Stop metrics collection
    pub async fn stop(&mut self) -> GraphResult<()> {
        if let Some(task) = &self.collection_task {
            task.abort();
            self.collection_task = None;
        }
        tracing::info!("Metrics collection stopped");
        Ok(())
    }

    /// Get current metrics
    pub async fn get_current_metrics(&self) -> SystemMetrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    /// Get metrics history
    pub async fn get_metrics_history(&self, limit: Option<usize>) -> Vec<MetricsSnapshot> {
        let history = self.history.read().await;
        let limit = limit.unwrap_or(100);
        
        if history.len() <= limit {
            history.clone()
        } else {
            history[history.len() - limit..].to_vec()
        }
    }

    /// Record execution start
    pub async fn record_execution_start(&self, execution_id: &str) {
        if !self.enabled {
            return;
        }

        let mut metrics = self.metrics.write().await;
        metrics.active_executions += 1;
        metrics.total_executions += 1;
        metrics.last_updated = chrono::Utc::now();
    }

    /// Record execution completion
    pub async fn record_execution_complete(&self, execution_id: &str, duration_ms: u64, success: bool) {
        if !self.enabled {
            return;
        }

        let mut metrics = self.metrics.write().await;
        metrics.active_executions = metrics.active_executions.saturating_sub(1);
        
        if success {
            metrics.completed_executions += 1;
        } else {
            metrics.failed_executions += 1;
        }

        // Update average execution time
        let total_completed = metrics.completed_executions + metrics.failed_executions;
        if total_completed > 0 {
            metrics.avg_execution_time_ms = 
                (metrics.avg_execution_time_ms * (total_completed - 1) as f64 + duration_ms as f64) / total_completed as f64;
        }

        // Update success rate
        if metrics.total_executions > 0 {
            metrics.success_rate = (metrics.completed_executions as f64 / metrics.total_executions as f64) * 100.0;
        }

        metrics.last_updated = chrono::Utc::now();
    }

    /// Record node execution
    pub async fn record_node_execution(&self, node_id: &str, node_type: &str, duration_ms: u64, success: bool) {
        if !self.enabled {
            return;
        }

        let mut metrics = self.metrics.write().await;
        let node_metrics = metrics.node_metrics.entry(node_id.to_string()).or_insert_with(|| NodeMetrics {
            node_id: node_id.to_string(),
            node_type: node_type.to_string(),
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            avg_execution_time_ms: 0.0,
            min_execution_time_ms: f64::MAX,
            max_execution_time_ms: 0.0,
            success_rate: 0.0,
            last_execution: None,
        });

        node_metrics.total_executions += 1;
        if success {
            node_metrics.successful_executions += 1;
        } else {
            node_metrics.failed_executions += 1;
        }

        // Update timing metrics
        let duration_f64 = duration_ms as f64;
        node_metrics.avg_execution_time_ms = 
            (node_metrics.avg_execution_time_ms * (node_metrics.total_executions - 1) as f64 + duration_f64) / node_metrics.total_executions as f64;
        node_metrics.min_execution_time_ms = node_metrics.min_execution_time_ms.min(duration_f64);
        node_metrics.max_execution_time_ms = node_metrics.max_execution_time_ms.max(duration_f64);

        // Update success rate
        node_metrics.success_rate = (node_metrics.successful_executions as f64 / node_metrics.total_executions as f64) * 100.0;
        node_metrics.last_execution = Some(chrono::Utc::now());
    }

    /// Record agent execution
    pub async fn record_agent_execution(&self, agent_name: &str, tokens_used: u32, duration_ms: u64, success: bool, cost: Option<f64>) {
        if !self.enabled {
            return;
        }

        let mut metrics = self.metrics.write().await;
        let agent_metrics = metrics.agent_metrics.entry(agent_name.to_string()).or_insert_with(|| AgentMetrics {
            agent_name: agent_name.to_string(),
            total_tasks: 0,
            total_tokens: 0,
            avg_tokens_per_task: 0.0,
            avg_response_time_ms: 0.0,
            success_rate: 0.0,
            total_cost: 0.0,
            last_activity: None,
        });

        agent_metrics.total_tasks += 1;
        agent_metrics.total_tokens += tokens_used as u64;
        agent_metrics.avg_tokens_per_task = agent_metrics.total_tokens as f64 / agent_metrics.total_tasks as f64;
        agent_metrics.avg_response_time_ms = 
            (agent_metrics.avg_response_time_ms * (agent_metrics.total_tasks - 1) as f64 + duration_ms as f64) / agent_metrics.total_tasks as f64;

        if let Some(cost) = cost {
            agent_metrics.total_cost += cost;
        }

        agent_metrics.last_activity = Some(chrono::Utc::now());
    }

    /// Record tool execution
    pub async fn record_tool_execution(&self, tool_name: &str, duration_ms: u64, success: bool) {
        if !self.enabled {
            return;
        }

        let mut metrics = self.metrics.write().await;
        let tool_metrics = metrics.tool_metrics.entry(tool_name.to_string()).or_insert_with(|| ToolMetrics {
            tool_name: tool_name.to_string(),
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            avg_execution_time_ms: 0.0,
            success_rate: 0.0,
            last_used: None,
        });

        tool_metrics.total_executions += 1;
        if success {
            tool_metrics.successful_executions += 1;
        } else {
            tool_metrics.failed_executions += 1;
        }

        tool_metrics.avg_execution_time_ms = 
            (tool_metrics.avg_execution_time_ms * (tool_metrics.total_executions - 1) as f64 + duration_ms as f64) / tool_metrics.total_executions as f64;
        tool_metrics.success_rate = (tool_metrics.successful_executions as f64 / tool_metrics.total_executions as f64) * 100.0;
        tool_metrics.last_used = Some(chrono::Utc::now());
    }

    /// Collect system metrics (simplified implementation)
    async fn collect_system_metrics() -> SystemMetrics {
        // In a real implementation, this would collect actual system metrics
        // For now, we'll return the current metrics with updated resource info
        let mut metrics = SystemMetrics::default();
        
        // Simulate resource metrics collection
        metrics.resource_metrics = ResourceMetrics {
            cpu_usage: 15.5, // Would use actual CPU monitoring
            memory_usage_mb: 256.0, // Would use actual memory monitoring
            memory_usage_percent: 25.6,
            active_threads: 12,
            network_rps: 10.5,
            disk_iops: 50.0,
        };
        
        metrics.last_updated = chrono::Utc::now();
        metrics
    }

    /// Get performance summary
    pub async fn get_performance_summary(&self) -> PerformanceSummary {
        let metrics = self.metrics.read().await;
        
        PerformanceSummary {
            total_executions: metrics.total_executions,
            success_rate: metrics.success_rate,
            avg_execution_time_ms: metrics.avg_execution_time_ms,
            active_executions: metrics.active_executions,
            top_performing_nodes: self.get_top_nodes(&metrics, 5),
            resource_usage: metrics.resource_metrics.clone(),
        }
    }

    /// Get top performing nodes
    fn get_top_nodes(&self, metrics: &SystemMetrics, limit: usize) -> Vec<NodeMetrics> {
        let mut nodes: Vec<_> = metrics.node_metrics.values().cloned().collect();
        nodes.sort_by(|a, b| b.success_rate.partial_cmp(&a.success_rate).unwrap_or(std::cmp::Ordering::Equal));
        nodes.truncate(limit);
        nodes
    }
}

/// Performance summary for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub total_executions: u64,
    pub success_rate: f64,
    pub avg_execution_time_ms: f64,
    pub active_executions: u64,
    pub top_performing_nodes: Vec<NodeMetrics>,
    pub resource_usage: ResourceMetrics,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_collector() {
        let mut collector = MetricsCollector::new(true, 1);
        
        // Record some metrics
        collector.record_execution_start("test_execution").await;
        collector.record_node_execution("test_node", "agent", 100, true).await;
        collector.record_execution_complete("test_execution", 150, true).await;
        
        let metrics = collector.get_current_metrics().await;
        assert_eq!(metrics.total_executions, 1);
        assert_eq!(metrics.completed_executions, 1);
        assert!(metrics.success_rate > 0.0);
    }

    #[tokio::test]
    async fn test_performance_summary() {
        let collector = MetricsCollector::new(true, 1);
        
        collector.record_node_execution("node1", "agent", 100, true).await;
        collector.record_node_execution("node2", "tool", 200, false).await;
        
        let summary = collector.get_performance_summary().await;
        assert!(!summary.top_performing_nodes.is_empty());
    }
}
