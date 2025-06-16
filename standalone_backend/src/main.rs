use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::{Filter, Reply};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub total_executions: u64,
    pub active_executions: u64,
    pub completed_executions: u64,
    pub failed_executions: u64,
    pub avg_execution_time_ms: f64,
    pub success_rate: f64,
    pub node_metrics: HashMap<String, serde_json::Value>,
    pub agent_metrics: HashMap<String, serde_json::Value>,
    pub tool_metrics: HashMap<String, serde_json::Value>,
    pub resource_metrics: ResourceMetrics,
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    pub cpu_usage: f64,
    pub memory_usage_mb: u64,
    pub memory_usage_percent: f64,
    pub active_threads: u32,
    pub network_rps: u64,
    pub disk_iops: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualWorkflow {
    pub id: String,
    pub name: String,
    pub status: String,
    pub nodes: Vec<VisualNode>,
    pub edges: Vec<VisualEdge>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualNode {
    pub id: String,
    pub name: String,
    pub node_type: String,
    pub position: (f64, f64),
    pub status: String,
    pub metadata: HashMap<String, serde_json::Value>,
    pub stats: NodeStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStats {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub avg_execution_time_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualEdge {
    pub id: String,
    pub source: String,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrace {
    pub id: String,
    pub execution_id: String,
    pub workflow_id: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub status: String,
    pub events: Vec<TraceEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEvent {
    pub id: String,
    pub execution_id: String,
    pub event_type: String,
    pub node_id: Option<String>,
    pub timestamp: String,
    pub data: serde_json::Value,
    pub context: HashMap<String, serde_json::Value>,
}

pub struct AgentGraphBackend {
    metrics: Arc<RwLock<SystemMetrics>>,
    workflows: Arc<RwLock<Vec<VisualWorkflow>>>,
    traces: Arc<RwLock<Vec<ExecutionTrace>>>,
}

impl AgentGraphBackend {
    pub fn new() -> Self {
        let metrics = SystemMetrics {
            total_executions: 1247,
            active_executions: 3,
            completed_executions: 1244,
            failed_executions: 3,
            avg_execution_time_ms: 850.0,
            success_rate: 95.2,
            node_metrics: HashMap::new(),
            agent_metrics: HashMap::new(),
            tool_metrics: HashMap::new(),
            resource_metrics: ResourceMetrics {
                cpu_usage: 45.0,
                memory_usage_mb: 2048,
                memory_usage_percent: 68.0,
                active_threads: 12,
                network_rps: 156,
                disk_iops: 89,
            },
            last_updated: chrono::Utc::now().to_rfc3339(),
        };

        let workflows = vec![
            VisualWorkflow {
                id: "content-creation".to_string(),
                name: "Content Creation Pipeline".to_string(),
                status: "running".to_string(),
                nodes: vec![
                    VisualNode {
                        id: "start".to_string(),
                        name: "Start".to_string(),
                        node_type: "start".to_string(),
                        position: (50.0, 100.0),
                        status: "completed".to_string(),
                        metadata: HashMap::new(),
                        stats: NodeStats {
                            total_executions: 45,
                            successful_executions: 44,
                            failed_executions: 1,
                            avg_execution_time_ms: 120.0,
                        },
                    },
                    VisualNode {
                        id: "research".to_string(),
                        name: "Research Agent".to_string(),
                        node_type: "agent".to_string(),
                        position: (200.0, 100.0),
                        status: "completed".to_string(),
                        metadata: HashMap::new(),
                        stats: NodeStats {
                            total_executions: 45,
                            successful_executions: 44,
                            failed_executions: 1,
                            avg_execution_time_ms: 2300.0,
                        },
                    },
                    VisualNode {
                        id: "writing".to_string(),
                        name: "Writing Agent".to_string(),
                        node_type: "agent".to_string(),
                        position: (350.0, 100.0),
                        status: "running".to_string(),
                        metadata: HashMap::new(),
                        stats: NodeStats {
                            total_executions: 44,
                            successful_executions: 43,
                            failed_executions: 1,
                            avg_execution_time_ms: 3100.0,
                        },
                    },
                ],
                edges: vec![
                    VisualEdge {
                        id: "start_research".to_string(),
                        source: "start".to_string(),
                        target: "research".to_string(),
                    },
                    VisualEdge {
                        id: "research_writing".to_string(),
                        source: "research".to_string(),
                        target: "writing".to_string(),
                    },
                ],
                metadata: HashMap::new(),
            },
            VisualWorkflow {
                id: "data-analysis".to_string(),
                name: "Data Analysis Workflow".to_string(),
                status: "completed".to_string(),
                nodes: vec![
                    VisualNode {
                        id: "start".to_string(),
                        name: "Start".to_string(),
                        node_type: "start".to_string(),
                        position: (50.0, 100.0),
                        status: "completed".to_string(),
                        metadata: HashMap::new(),
                        stats: NodeStats {
                            total_executions: 23,
                            successful_executions: 23,
                            failed_executions: 0,
                            avg_execution_time_ms: 80.0,
                        },
                    },
                    VisualNode {
                        id: "extract".to_string(),
                        name: "Data Extraction".to_string(),
                        node_type: "tool".to_string(),
                        position: (200.0, 100.0),
                        status: "completed".to_string(),
                        metadata: HashMap::new(),
                        stats: NodeStats {
                            total_executions: 23,
                            successful_executions: 23,
                            failed_executions: 0,
                            avg_execution_time_ms: 1200.0,
                        },
                    },
                    VisualNode {
                        id: "analyze".to_string(),
                        name: "Analysis Agent".to_string(),
                        node_type: "agent".to_string(),
                        position: (350.0, 100.0),
                        status: "completed".to_string(),
                        metadata: HashMap::new(),
                        stats: NodeStats {
                            total_executions: 23,
                            successful_executions: 22,
                            failed_executions: 1,
                            avg_execution_time_ms: 4500.0,
                        },
                    },
                ],
                edges: vec![
                    VisualEdge {
                        id: "start_extract".to_string(),
                        source: "start".to_string(),
                        target: "extract".to_string(),
                    },
                    VisualEdge {
                        id: "extract_analyze".to_string(),
                        source: "extract".to_string(),
                        target: "analyze".to_string(),
                    },
                ],
                metadata: HashMap::new(),
            },
        ];

        let traces = vec![
            ExecutionTrace {
                id: "trace-1".to_string(),
                execution_id: "exec-abc123".to_string(),
                workflow_id: "content-creation".to_string(),
                start_time: chrono::Utc::now().to_rfc3339(),
                end_time: None,
                status: "running".to_string(),
                events: vec![
                    TraceEvent {
                        id: "evt-1".to_string(),
                        execution_id: "exec-abc123".to_string(),
                        event_type: "execution_started".to_string(),
                        node_id: None,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        data: json!({}),
                        context: HashMap::new(),
                    },
                    TraceEvent {
                        id: "evt-2".to_string(),
                        execution_id: "exec-abc123".to_string(),
                        event_type: "node_started".to_string(),
                        node_id: Some("research".to_string()),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        data: json!({}),
                        context: HashMap::new(),
                    },
                ],
            },
        ];

        Self {
            metrics: Arc::new(RwLock::new(metrics)),
            workflows: Arc::new(RwLock::new(workflows)),
            traces: Arc::new(RwLock::new(traces)),
        }
    }

    pub async fn start(&self, port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let metrics = self.metrics.clone();
        let workflows = self.workflows.clone();
        let traces = self.traces.clone();

        // CORS
        let cors = warp::cors()
            .allow_any_origin()
            .allow_headers(vec!["content-type"])
            .allow_methods(vec!["GET", "POST", "PUT", "DELETE"]);

        // Metrics endpoint
        let metrics_route = warp::path!("api" / "agentgraph" / "metrics")
            .and(warp::get())
            .and(with_metrics(metrics))
            .and_then(get_metrics);

        // Workflows endpoint
        let workflows_route = warp::path!("api" / "agentgraph" / "workflows")
            .and(warp::get())
            .and(with_workflows(workflows))
            .and_then(get_workflows);

        // Traces endpoint
        let traces_route = warp::path!("api" / "agentgraph" / "traces")
            .and(warp::get())
            .and(with_traces(traces))
            .and_then(get_traces);

        let routes = metrics_route
            .or(workflows_route)
            .or(traces_route)
            .with(cors);

        println!("🚀 AgentGraph Backend running on http://localhost:{}", port);
        println!("📊 Frontend available at http://localhost:3001");
        println!("🔗 API endpoints:");
        println!("   • GET /api/agentgraph/metrics");
        println!("   • GET /api/agentgraph/workflows");
        println!("   • GET /api/agentgraph/traces");

        warp::serve(routes)
            .run(([127, 0, 0, 1], port))
            .await;

        Ok(())
    }
}

fn with_metrics(
    metrics: Arc<RwLock<SystemMetrics>>,
) -> impl Filter<Extract = (Arc<RwLock<SystemMetrics>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || metrics.clone())
}

fn with_workflows(
    workflows: Arc<RwLock<Vec<VisualWorkflow>>>,
) -> impl Filter<Extract = (Arc<RwLock<Vec<VisualWorkflow>>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || workflows.clone())
}

fn with_traces(
    traces: Arc<RwLock<Vec<ExecutionTrace>>>,
) -> impl Filter<Extract = (Arc<RwLock<Vec<ExecutionTrace>>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || traces.clone())
}

async fn get_metrics(
    metrics: Arc<RwLock<SystemMetrics>>,
) -> Result<impl Reply, warp::Rejection> {
    let metrics = metrics.read().await;
    Ok(warp::reply::json(&*metrics))
}

async fn get_workflows(
    workflows: Arc<RwLock<Vec<VisualWorkflow>>>,
) -> Result<impl Reply, warp::Rejection> {
    let workflows = workflows.read().await;
    Ok(warp::reply::json(&*workflows))
}

async fn get_traces(
    traces: Arc<RwLock<Vec<ExecutionTrace>>>,
) -> Result<impl Reply, warp::Rejection> {
    let traces = traces.read().await;
    Ok(warp::reply::json(&*traces))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let backend = AgentGraphBackend::new();
    backend.start(8081).await
}
