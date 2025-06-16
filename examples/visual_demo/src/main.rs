// AgentGraph Studio Visual Demo
// This demonstrates the complete visual debugging and monitoring interface
// Similar to LangSmith and LangGraph Studio

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, broadcast};
use uuid::Uuid;

/// Mock visualization engine for demonstration
#[derive(Debug)]
pub struct MockVisualizationEngine {
    /// Execution tracer
    tracer: Arc<MockExecutionTracer>,
    /// Metrics collector
    metrics: Arc<MockMetricsCollector>,
    /// Active workflows
    workflows: Arc<RwLock<HashMap<String, VisualWorkflow>>>,
    /// Web server port
    port: u16,
}

/// Mock execution tracer
#[derive(Debug)]
pub struct MockExecutionTracer {
    /// Active traces
    traces: Arc<RwLock<HashMap<String, ExecutionTrace>>>,
    /// Event broadcaster
    event_broadcaster: broadcast::Sender<VisualExecutionEvent>,
}

/// Mock metrics collector
#[derive(Debug)]
pub struct MockMetricsCollector {
    /// Current metrics
    metrics: Arc<RwLock<SystemMetrics>>,
}

/// Visual workflow representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualWorkflow {
    pub id: String,
    pub name: String,
    pub nodes: Vec<VisualNode>,
    pub edges: Vec<VisualEdge>,
    pub status: WorkflowStatus,
}

/// Visual node representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualNode {
    pub id: String,
    pub name: String,
    pub node_type: String,
    pub position: (f64, f64),
    pub status: NodeStatus,
    pub execution_time_ms: Option<u64>,
}

/// Visual edge representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub edge_type: String,
}

/// Workflow status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

/// Node status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

/// Execution trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrace {
    pub id: String,
    pub execution_id: String,
    pub workflow_id: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub events: Vec<VisualExecutionEvent>,
    pub status: ExecutionStatus,
}

/// Execution event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualExecutionEvent {
    pub id: String,
    pub execution_id: String,
    pub event_type: String,
    pub node_id: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub data: serde_json::Value,
}

/// Execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Running,
    Completed,
    Failed,
}

/// System metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub total_executions: u64,
    pub active_executions: u64,
    pub completed_executions: u64,
    pub failed_executions: u64,
    pub avg_execution_time_ms: f64,
    pub success_rate: f64,
}

impl MockVisualizationEngine {
    /// Create a new visualization engine
    pub fn new(port: u16) -> Self {
        let (event_broadcaster, _) = broadcast::channel(1000);
        
        Self {
            tracer: Arc::new(MockExecutionTracer {
                traces: Arc::new(RwLock::new(HashMap::new())),
                event_broadcaster,
            }),
            metrics: Arc::new(MockMetricsCollector {
                metrics: Arc::new(RwLock::new(SystemMetrics {
                    total_executions: 0,
                    active_executions: 0,
                    completed_executions: 0,
                    failed_executions: 0,
                    avg_execution_time_ms: 0.0,
                    success_rate: 0.0,
                })),
            }),
            workflows: Arc::new(RwLock::new(HashMap::new())),
            port,
        }
    }

    /// Start the visualization engine
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("🚀 Starting AgentGraph Studio on http://localhost:{}", self.port);
        
        // Create sample workflow
        self.create_sample_workflow().await;
        
        // Start web server
        self.start_web_server().await?;
        
        Ok(())
    }

    /// Create a sample workflow for demonstration
    async fn create_sample_workflow(&self) {
        let workflow = VisualWorkflow {
            id: "sample_workflow".to_string(),
            name: "Content Creation Pipeline".to_string(),
            nodes: vec![
                VisualNode {
                    id: "start".to_string(),
                    name: "Start".to_string(),
                    node_type: "start".to_string(),
                    position: (100.0, 100.0),
                    status: NodeStatus::Completed,
                    execution_time_ms: Some(5),
                },
                VisualNode {
                    id: "researcher".to_string(),
                    name: "Research Agent".to_string(),
                    node_type: "agent".to_string(),
                    position: (300.0, 100.0),
                    status: NodeStatus::Completed,
                    execution_time_ms: Some(1200),
                },
                VisualNode {
                    id: "web_search".to_string(),
                    name: "Web Search Tool".to_string(),
                    node_type: "tool".to_string(),
                    position: (500.0, 50.0),
                    status: NodeStatus::Completed,
                    execution_time_ms: Some(800),
                },
                VisualNode {
                    id: "writer".to_string(),
                    name: "Writing Agent".to_string(),
                    node_type: "agent".to_string(),
                    position: (500.0, 150.0),
                    status: NodeStatus::Running,
                    execution_time_ms: None,
                },
                VisualNode {
                    id: "quality_gate".to_string(),
                    name: "Quality Gate".to_string(),
                    node_type: "quality_gate".to_string(),
                    position: (700.0, 100.0),
                    status: NodeStatus::Pending,
                    execution_time_ms: None,
                },
                VisualNode {
                    id: "end".to_string(),
                    name: "End".to_string(),
                    node_type: "end".to_string(),
                    position: (900.0, 100.0),
                    status: NodeStatus::Pending,
                    execution_time_ms: None,
                },
            ],
            edges: vec![
                VisualEdge {
                    id: "start_researcher".to_string(),
                    source: "start".to_string(),
                    target: "researcher".to_string(),
                    edge_type: "default".to_string(),
                },
                VisualEdge {
                    id: "researcher_web_search".to_string(),
                    source: "researcher".to_string(),
                    target: "web_search".to_string(),
                    edge_type: "tool_call".to_string(),
                },
                VisualEdge {
                    id: "researcher_writer".to_string(),
                    source: "researcher".to_string(),
                    target: "writer".to_string(),
                    edge_type: "default".to_string(),
                },
                VisualEdge {
                    id: "writer_quality_gate".to_string(),
                    source: "writer".to_string(),
                    target: "quality_gate".to_string(),
                    edge_type: "default".to_string(),
                },
                VisualEdge {
                    id: "quality_gate_end".to_string(),
                    source: "quality_gate".to_string(),
                    target: "end".to_string(),
                    edge_type: "conditional".to_string(),
                },
            ],
            status: WorkflowStatus::Running,
        };

        let mut workflows = self.workflows.write().await;
        workflows.insert(workflow.id.clone(), workflow);
    }

    /// Start the web server
    async fn start_web_server(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use warp::Filter;

        // Clone for move into async block
        let workflows = self.workflows.clone();
        let tracer = self.tracer.clone();
        let metrics = self.metrics.clone();

        // Dashboard route
        let dashboard = warp::path::end()
            .map(|| warp::reply::html(Self::dashboard_html()));

        // API routes
        let api = warp::path("api");

        // Get workflows
        let workflows_route = api
            .and(warp::path("workflows"))
            .and(warp::path::end())
            .and(warp::get())
            .and(warp::any().map(move || workflows.clone()))
            .and_then(|workflows: Arc<RwLock<HashMap<String, VisualWorkflow>>>| async move {
                let workflows = workflows.read().await;
                let workflow_list: Vec<_> = workflows.values().collect();
                Ok::<_, warp::Rejection>(warp::reply::json(&workflow_list))
            });

        // Get metrics
        let metrics_route = api
            .and(warp::path("metrics"))
            .and(warp::path::end())
            .and(warp::get())
            .and(warp::any().map(move || metrics.clone()))
            .and_then(|metrics: Arc<MockMetricsCollector>| async move {
                let metrics_data = metrics.metrics.read().await;
                Ok::<_, warp::Rejection>(warp::reply::json(&*metrics_data))
            });

        // Get traces
        let traces_route = api
            .and(warp::path("traces"))
            .and(warp::path::end())
            .and(warp::get())
            .and(warp::any().map(move || tracer.clone()))
            .and_then(|tracer: Arc<MockExecutionTracer>| async move {
                let traces = tracer.traces.read().await;
                let trace_list: Vec<_> = traces.values().collect();
                Ok::<_, warp::Rejection>(warp::reply::json(&trace_list))
            });

        let routes = dashboard
            .or(workflows_route)
            .or(metrics_route)
            .or(traces_route)
            .with(warp::cors().allow_any_origin());

        println!("🌐 AgentGraph Studio available at http://localhost:{}", self.port);
        println!("📊 Features available:");
        println!("  • Real-time workflow visualization");
        println!("  • Execution tracing and debugging");
        println!("  • Performance metrics and analytics");
        println!("  • Agent and tool monitoring");

        warp::serve(routes)
            .run(([127, 0, 0, 1], self.port))
            .await;

        Ok(())
    }

    /// Generate dashboard HTML
    fn dashboard_html() -> String {
        r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>AgentGraph Studio - Visual Debugging Interface</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f8f9fa; }
        
        .header { background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 1.5rem 2rem; box-shadow: 0 4px 6px rgba(0,0,0,0.1); }
        .header h1 { font-size: 2rem; font-weight: 700; margin-bottom: 0.5rem; }
        .header .subtitle { opacity: 0.9; font-size: 1.1rem; }
        .header .features { margin-top: 1rem; display: flex; gap: 2rem; font-size: 0.9rem; }
        .feature { display: flex; align-items: center; gap: 0.5rem; }
        
        .container { max-width: 1600px; margin: 0 auto; padding: 2rem; }
        .grid { display: grid; grid-template-columns: 1fr 1fr; gap: 2rem; margin-bottom: 2rem; }
        .full-width { grid-column: 1 / -1; }
        
        .card { background: white; border-radius: 12px; padding: 2rem; box-shadow: 0 4px 6px rgba(0,0,0,0.05); border: 1px solid #e9ecef; }
        .card h2 { color: #495057; margin-bottom: 1.5rem; font-size: 1.3rem; display: flex; align-items: center; gap: 0.5rem; }
        
        .metrics { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 1.5rem; }
        .metric { text-align: center; padding: 1.5rem; background: linear-gradient(135deg, #f8f9fa 0%, #e9ecef 100%); border-radius: 8px; border: 1px solid #dee2e6; }
        .metric-value { font-size: 2.5rem; font-weight: bold; color: #28a745; margin-bottom: 0.5rem; }
        .metric-label { color: #6c757d; font-size: 0.95rem; font-weight: 500; }
        
        .workflow-canvas { width: 100%; height: 500px; border: 2px solid #dee2e6; border-radius: 8px; background: white; position: relative; overflow: hidden; }
        .node { position: absolute; padding: 12px 16px; border-radius: 8px; color: white; font-weight: 500; font-size: 0.9rem; text-align: center; min-width: 120px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); cursor: pointer; transition: transform 0.2s; }
        .node:hover { transform: scale(1.05); }
        .node.agent { background: linear-gradient(135deg, #28a745, #20c997); }
        .node.tool { background: linear-gradient(135deg, #007bff, #6610f2); }
        .node.start { background: linear-gradient(135deg, #28a745, #34ce57); }
        .node.end { background: linear-gradient(135deg, #dc3545, #fd7e14); }
        .node.quality_gate { background: linear-gradient(135deg, #6f42c1, #e83e8c); }
        .node.running { animation: pulse 2s infinite; }
        .node.completed { opacity: 0.8; }
        .node.pending { opacity: 0.6; }
        
        @keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.7; } }
        
        .edge { position: absolute; height: 2px; background: #6c757d; transform-origin: left center; }
        .edge.tool_call { background: #007bff; }
        .edge.conditional { background: #ffc107; }
        
        .trace-list { max-height: 400px; overflow-y: auto; }
        .trace-item { padding: 1rem; border-bottom: 1px solid #e9ecef; cursor: pointer; transition: background 0.2s; }
        .trace-item:hover { background: #f8f9fa; }
        .trace-status { display: inline-block; width: 10px; height: 10px; border-radius: 50%; margin-right: 0.75rem; }
        .status-running { background: #ffc107; animation: pulse 1s infinite; }
        .status-completed { background: #28a745; }
        .status-failed { background: #dc3545; }
        
        .loading { text-align: center; padding: 3rem; color: #6c757d; }
        .status-indicator { display: inline-block; width: 8px; height: 8px; border-radius: 50%; margin-left: 0.5rem; }
        .status-indicator.running { background: #ffc107; animation: pulse 1s infinite; }
        .status-indicator.completed { background: #28a745; }
    </style>
</head>
<body>
    <div class="header">
        <h1>🚀 AgentGraph Studio</h1>
        <div class="subtitle">Visual debugging and monitoring interface for agent workflows</div>
        <div class="features">
            <div class="feature">📊 Real-time Metrics</div>
            <div class="feature">🔍 Execution Tracing</div>
            <div class="feature">🎯 Workflow Visualization</div>
            <div class="feature">⚡ Performance Analytics</div>
        </div>
    </div>
    
    <div class="container">
        <div class="grid">
            <div class="card">
                <h2>📊 System Metrics</h2>
                <div class="metrics" id="metrics">
                    <div class="metric">
                        <div class="metric-value">5</div>
                        <div class="metric-label">Total Executions</div>
                    </div>
                    <div class="metric">
                        <div class="metric-value">1</div>
                        <div class="metric-label">Active Executions</div>
                    </div>
                    <div class="metric">
                        <div class="metric-value">850ms</div>
                        <div class="metric-label">Avg Execution Time</div>
                    </div>
                    <div class="metric">
                        <div class="metric-value">95%</div>
                        <div class="metric-label">Success Rate</div>
                    </div>
                </div>
            </div>
            
            <div class="card">
                <h2>🔄 Recent Executions</h2>
                <div class="trace-list" id="traces">
                    <div class="trace-item">
                        <span class="trace-status status-running"></span>
                        <strong>content_creation_001</strong>
                        <span class="status-indicator running"></span>
                        <div style="font-size: 0.85rem; color: #6c757d; margin-top: 0.5rem;">
                            Started 2 minutes ago • 4 events • Research → Writing
                        </div>
                    </div>
                    <div class="trace-item">
                        <span class="trace-status status-completed"></span>
                        <strong>data_analysis_002</strong>
                        <span class="status-indicator completed"></span>
                        <div style="font-size: 0.85rem; color: #6c757d; margin-top: 0.5rem;">
                            Completed 5 minutes ago • 8 events • 1.2s duration
                        </div>
                    </div>
                    <div class="trace-item">
                        <span class="trace-status status-completed"></span>
                        <strong>workflow_test_003</strong>
                        <span class="status-indicator completed"></span>
                        <div style="font-size: 0.85rem; color: #6c757d; margin-top: 0.5rem;">
                            Completed 8 minutes ago • 6 events • 0.8s duration
                        </div>
                    </div>
                </div>
            </div>
        </div>
        
        <div class="card full-width">
            <h2>🎯 Workflow Visualization - Content Creation Pipeline</h2>
            <div class="workflow-canvas" id="workflow-canvas">
                <!-- Nodes will be rendered here -->
                <div class="node start" style="left: 100px; top: 100px;">Start</div>
                <div class="node agent completed" style="left: 300px; top: 100px;">Research Agent<br><small>1.2s</small></div>
                <div class="node tool completed" style="left: 500px; top: 50px;">Web Search<br><small>0.8s</small></div>
                <div class="node agent running" style="left: 500px; top: 150px;">Writing Agent<br><small>Running...</small></div>
                <div class="node quality_gate pending" style="left: 700px; top: 100px;">Quality Gate</div>
                <div class="node end pending" style="left: 900px; top: 100px;">End</div>
                
                <!-- Edges will be rendered here -->
                <div class="edge" style="left: 220px; top: 115px; width: 80px;"></div>
                <div class="edge tool_call" style="left: 420px; top: 85px; width: 80px; transform: rotate(-15deg);"></div>
                <div class="edge" style="left: 420px; top: 115px; width: 80px;"></div>
                <div class="edge" style="left: 620px; top: 165px; width: 80px; transform: rotate(-15deg);"></div>
                <div class="edge conditional" style="left: 820px; top: 115px; width: 80px;"></div>
            </div>
        </div>
    </div>

    <script>
        console.log('🚀 AgentGraph Studio initialized');
        console.log('📊 This is a demonstration of LangSmith/LangGraph Studio equivalent functionality');
        console.log('🎯 Features: Real-time visualization, execution tracing, performance monitoring');
        
        // Simulate real-time updates
        setInterval(() => {
            console.log('📡 Real-time update simulation');
        }, 5000);
    </script>
</body>
</html>
        "#.to_string()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🎨 AgentGraph Studio - Visual Debugging Demo");
    println!("=============================================");
    println!("This demonstrates LangSmith and LangGraph Studio equivalent functionality!");
    println!();

    // Create and start visualization engine
    let engine = MockVisualizationEngine::new(8080);
    engine.start().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visualization_engine_creation() {
        let engine = MockVisualizationEngine::new(8080);
        assert_eq!(engine.port, 8080);
    }

    #[tokio::test]
    async fn test_sample_workflow_creation() {
        let engine = MockVisualizationEngine::new(8080);
        engine.create_sample_workflow().await;
        
        let workflows = engine.workflows.read().await;
        assert!(!workflows.is_empty());
        assert!(workflows.contains_key("sample_workflow"));
    }
}
