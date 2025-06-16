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
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualWorkflow {
    pub id: String,
    pub name: String,
    pub status: String,
    pub nodes: Vec<VisualNode>,
    pub edges: Vec<VisualEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualNode {
    pub id: String,
    pub name: String,
    pub node_type: String,
    pub position: (f64, f64),
    pub status: String,
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
    pub timestamp: String,
    pub data: serde_json::Value,
}

pub struct SimpleBackend {
    metrics: Arc<RwLock<SystemMetrics>>,
    workflows: Arc<RwLock<Vec<VisualWorkflow>>>,
    traces: Arc<RwLock<Vec<ExecutionTrace>>>,
}

impl SimpleBackend {
    pub fn new() -> Self {
        let metrics = SystemMetrics {
            total_executions: 1247,
            active_executions: 3,
            completed_executions: 1244,
            failed_executions: 3,
            avg_execution_time_ms: 850.0,
            success_rate: 95.2,
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
                    },
                    VisualNode {
                        id: "research".to_string(),
                        name: "Research Agent".to_string(),
                        node_type: "agent".to_string(),
                        position: (200.0, 100.0),
                        status: "completed".to_string(),
                    },
                    VisualNode {
                        id: "writing".to_string(),
                        name: "Writing Agent".to_string(),
                        node_type: "agent".to_string(),
                        position: (350.0, 100.0),
                        status: "running".to_string(),
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
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        data: json!({}),
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

        println!("🚀 AgentGraph Simple Backend running on http://localhost:{}", port);
        println!("📊 Frontend available at http://localhost:3001");

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
    let backend = SimpleBackend::new();
    backend.start(8080).await
}
