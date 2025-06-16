//! Visual debugging and monitoring interface for AgentGraph
//! Provides LangSmith and LangGraph Studio equivalent functionality

pub mod execution_tracer;
pub mod graph_visualizer;
pub mod metrics_collector;
pub mod web_interface;

use crate::error::GraphResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Main visualization engine for AgentGraph
#[derive(Debug)]
pub struct VisualizationEngine {
    /// Execution tracer for monitoring workflow runs
    tracer: Arc<execution_tracer::ExecutionTracer>,
    /// Graph visualizer for rendering workflow structure
    visualizer: Arc<graph_visualizer::GraphVisualizer>,
    /// Metrics collector for performance analytics
    metrics: Arc<metrics_collector::MetricsCollector>,
    /// Web interface server
    web_server: Option<Arc<web_interface::WebServer>>,
    /// Configuration
    config: VisualizationConfig,
}

/// Configuration for visualization engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    /// Enable real-time tracing
    pub enable_tracing: bool,
    /// Enable web interface
    pub enable_web_interface: bool,
    /// Web server port
    pub web_port: u16,
    /// Maximum trace history to keep
    pub max_trace_history: usize,
    /// Enable performance metrics
    pub enable_metrics: bool,
    /// Metrics collection interval (seconds)
    pub metrics_interval: u64,
    /// Enable debug mode
    pub debug_mode: bool,
}

impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            enable_tracing: true,
            enable_web_interface: true,
            web_port: 8080,
            max_trace_history: 1000,
            enable_metrics: true,
            metrics_interval: 5,
            debug_mode: false,
        }
    }
}

impl VisualizationEngine {
    /// Create a new visualization engine
    pub fn new(config: VisualizationConfig) -> GraphResult<Self> {
        let tracer = Arc::new(execution_tracer::ExecutionTracer::new(
            config.max_trace_history,
            config.enable_tracing,
        ));
        
        let visualizer = Arc::new(graph_visualizer::GraphVisualizer::new());
        
        let metrics = Arc::new(metrics_collector::MetricsCollector::new(
            config.enable_metrics,
            config.metrics_interval,
        ));

        Ok(Self {
            tracer,
            visualizer,
            metrics,
            web_server: None,
            config,
        })
    }

    /// Start the visualization engine
    pub async fn start(&mut self) -> GraphResult<()> {
        tracing::info!("Starting AgentGraph Visualization Engine");

        // Start metrics collection
        if self.config.enable_metrics {
            self.metrics.start().await?;
        }

        // Start web interface
        if self.config.enable_web_interface {
            let web_server = Arc::new(
                web_interface::WebServer::new(
                    self.config.web_port,
                    self.tracer.clone(),
                    self.visualizer.clone(),
                    self.metrics.clone(),
                ).await?
            );
            
            web_server.start().await?;
            self.web_server = Some(web_server);
            
            tracing::info!("AgentGraph Studio available at http://localhost:{}", self.config.web_port);
        }

        Ok(())
    }

    /// Stop the visualization engine
    pub async fn stop(&mut self) -> GraphResult<()> {
        tracing::info!("Stopping AgentGraph Visualization Engine");

        if let Some(web_server) = &self.web_server {
            web_server.stop().await?;
        }

        self.metrics.stop().await?;

        Ok(())
    }

    /// Get execution tracer
    pub fn tracer(&self) -> Arc<execution_tracer::ExecutionTracer> {
        self.tracer.clone()
    }

    /// Get graph visualizer
    pub fn visualizer(&self) -> Arc<graph_visualizer::GraphVisualizer> {
        self.visualizer.clone()
    }

    /// Get metrics collector
    pub fn metrics(&self) -> Arc<metrics_collector::MetricsCollector> {
        self.metrics.clone()
    }

    /// Get web interface URL
    pub fn web_url(&self) -> Option<String> {
        if self.config.enable_web_interface {
            Some(format!("http://localhost:{}", self.config.web_port))
        } else {
            None
        }
    }
}

/// Visual execution event for real-time monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualExecutionEvent {
    /// Event ID
    pub id: String,
    /// Execution ID
    pub execution_id: String,
    /// Event type
    pub event_type: VisualEventType,
    /// Node ID (if applicable)
    pub node_id: Option<String>,
    /// Event timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Event data
    pub data: serde_json::Value,
    /// Execution context
    pub context: HashMap<String, serde_json::Value>,
}

/// Types of visual events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VisualEventType {
    /// Execution started
    ExecutionStarted,
    /// Node execution started
    NodeStarted,
    /// Node execution completed
    NodeCompleted,
    /// Node execution failed
    NodeFailed,
    /// Agent response received
    AgentResponse,
    /// Tool execution
    ToolExecution,
    /// Command routing
    CommandRouting,
    /// State update
    StateUpdate,
    /// Execution completed
    ExecutionCompleted,
    /// Execution failed
    ExecutionFailed,
    /// Custom event
    Custom(String),
}

/// Visual node representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualNode {
    /// Node ID
    pub id: String,
    /// Node type
    pub node_type: String,
    /// Display name
    pub name: String,
    /// Node position (x, y)
    pub position: (f64, f64),
    /// Node status
    pub status: NodeStatus,
    /// Node metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Execution statistics
    pub stats: NodeStats,
}

/// Visual edge representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualEdge {
    /// Edge ID
    pub id: String,
    /// Source node ID
    pub source: String,
    /// Target node ID
    pub target: String,
    /// Edge type
    pub edge_type: String,
    /// Edge condition (if conditional)
    pub condition: Option<String>,
    /// Execution count
    pub execution_count: u64,
}

/// Node execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeStatus {
    /// Not executed yet
    Pending,
    /// Currently executing
    Running,
    /// Completed successfully
    Completed,
    /// Failed with error
    Failed,
    /// Skipped
    Skipped,
}

/// Node execution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStats {
    /// Total executions
    pub total_executions: u64,
    /// Successful executions
    pub successful_executions: u64,
    /// Failed executions
    pub failed_executions: u64,
    /// Average execution time (ms)
    pub avg_execution_time_ms: f64,
    /// Last execution time
    pub last_execution: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for NodeStats {
    fn default() -> Self {
        Self {
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            avg_execution_time_ms: 0.0,
            last_execution: None,
        }
    }
}

/// Visual workflow representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualWorkflow {
    /// Workflow ID
    pub id: String,
    /// Workflow name
    pub name: String,
    /// Nodes in the workflow
    pub nodes: Vec<VisualNode>,
    /// Edges in the workflow
    pub edges: Vec<VisualEdge>,
    /// Current execution state
    pub current_execution: Option<String>,
    /// Workflow metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Execution trace for debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrace {
    /// Trace ID
    pub id: String,
    /// Execution ID
    pub execution_id: String,
    /// Workflow ID
    pub workflow_id: String,
    /// Start time
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// End time
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    /// Execution events
    pub events: Vec<VisualExecutionEvent>,
    /// Final status
    pub status: ExecutionStatus,
    /// Error information (if failed)
    pub error: Option<String>,
}

/// Execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStatus {
    /// Currently running
    Running,
    /// Completed successfully
    Completed,
    /// Failed with error
    Failed,
    /// Cancelled
    Cancelled,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visualization_config() {
        let config = VisualizationConfig::default();
        assert!(config.enable_tracing);
        assert!(config.enable_web_interface);
        assert_eq!(config.web_port, 8080);
    }

    #[tokio::test]
    async fn test_visualization_engine_creation() {
        let config = VisualizationConfig::default();
        let engine = VisualizationEngine::new(config).unwrap();
        
        assert!(engine.tracer.is_enabled());
        assert_eq!(engine.config.web_port, 8080);
    }

    #[test]
    fn test_visual_node_creation() {
        let node = VisualNode {
            id: "test_node".to_string(),
            node_type: "agent".to_string(),
            name: "Test Agent".to_string(),
            position: (100.0, 200.0),
            status: NodeStatus::Pending,
            metadata: HashMap::new(),
            stats: NodeStats::default(),
        };
        
        assert_eq!(node.id, "test_node");
        assert_eq!(node.position.0, 100.0);
        assert!(matches!(node.status, NodeStatus::Pending));
    }
}
