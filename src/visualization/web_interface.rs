//! Web interface for AgentGraph Studio
//! Provides LangGraph Studio and LangSmith equivalent web dashboard

use crate::error::GraphResult;
use crate::visualization::{execution_tracer::ExecutionTracer, graph_visualizer::GraphVisualizer, metrics_collector::MetricsCollector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::{Filter, Reply};

/// Web server for AgentGraph Studio
#[derive(Debug)]
pub struct WebServer {
    /// Server port
    port: u16,
    /// Execution tracer
    tracer: Arc<ExecutionTracer>,
    /// Graph visualizer
    visualizer: Arc<GraphVisualizer>,
    /// Metrics collector
    metrics: Arc<MetricsCollector>,
    /// Server handle
    server_handle: Option<tokio::task::JoinHandle<()>>,
    /// Active workflows
    workflows: Arc<RwLock<HashMap<String, crate::visualization::VisualWorkflow>>>,
}

impl WebServer {
    /// Create a new web server
    pub async fn new(
        port: u16,
        tracer: Arc<ExecutionTracer>,
        visualizer: Arc<GraphVisualizer>,
        metrics: Arc<MetricsCollector>,
    ) -> GraphResult<Self> {
        Ok(Self {
            port,
            tracer,
            visualizer,
            metrics,
            server_handle: None,
            workflows: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Start the web server
    pub async fn start(&mut self) -> GraphResult<()> {
        let tracer = self.tracer.clone();
        let visualizer = self.visualizer.clone();
        let metrics = self.metrics.clone();
        let workflows = self.workflows.clone();
        let port = self.port;

        // Create routes
        let routes = self.create_routes(tracer, visualizer, metrics, workflows).await;

        // Start server
        let server = warp::serve(routes).run(([127, 0, 0, 1], port));

        let handle = tokio::spawn(server);
        self.server_handle = Some(handle);

        tracing::info!("🚀 AgentGraph API Server running on http://localhost:{}", port);
        tracing::info!("📊 Use the Next.js frontend at http://localhost:3000 for the visual interface");
        Ok(())
    }

    /// Stop the web server
    pub async fn stop(&self) -> GraphResult<()> {
        if let Some(handle) = &self.server_handle {
            handle.abort();
        }
        Ok(())
    }

    /// Create web routes - API only, no HTML dashboard
    async fn create_routes(
        tracer: Arc<ExecutionTracer>,
        visualizer: Arc<GraphVisualizer>,
        metrics: Arc<MetricsCollector>,
        workflows: Arc<RwLock<HashMap<String, crate::visualization::VisualWorkflow>>>,
    ) -> impl Filter<Extract = impl Reply> + Clone {
        // API routes only - frontend is served by Next.js
        let api = warp::path("api");

        // Get all traces
        let traces_route = api
            .and(warp::path("traces"))
            .and(warp::path::end())
            .and(warp::get())
            .and(with_tracer(tracer.clone()))
            .and_then(get_traces);

        // Get specific trace
        let trace_route = api
            .and(warp::path("traces"))
            .and(warp::path::param::<String>())
            .and(warp::path::end())
            .and(warp::get())
            .and(with_tracer(tracer.clone()))
            .and_then(get_trace);

        // Get workflows
        let workflows_route = api
            .and(warp::path("workflows"))
            .and(warp::path::end())
            .and(warp::get())
            .and(with_workflows(workflows.clone()))
            .and_then(get_workflows);

        // Get metrics
        let metrics_route = api
            .and(warp::path("metrics"))
            .and(warp::path::end())
            .and(warp::get())
            .and(with_metrics(metrics.clone()))
            .and_then(get_metrics);

        // WebSocket for real-time events
        let events_ws = api
            .and(warp::path("events"))
            .and(warp::path::end())
            .and(warp::ws())
            .and(with_tracer(tracer))
            .map(|ws: warp::ws::Ws, tracer: Arc<ExecutionTracer>| {
                ws.on_upgrade(move |socket| handle_websocket(socket, tracer))
            });

        // CORS
        let cors = warp::cors()
            .allow_any_origin()
            .allow_headers(vec!["content-type"])
            .allow_methods(vec!["GET", "POST", "PUT", "DELETE"]);

        // Only API routes - no static files or dashboard
        traces_route
            .or(trace_route)
            .or(workflows_route)
            .or(metrics_route)
            .or(events_ws)
            .with(cors)
    }


}

// Helper functions for warp filters
fn with_tracer(tracer: Arc<ExecutionTracer>) -> impl Filter<Extract = (Arc<ExecutionTracer>,)> + Clone {
    warp::any().map(move || tracer.clone())
}

fn with_workflows(workflows: Arc<RwLock<HashMap<String, crate::visualization::VisualWorkflow>>>) -> impl Filter<Extract = (Arc<RwLock<HashMap<String, crate::visualization::VisualWorkflow>>>,)> + Clone {
    warp::any().map(move || workflows.clone())
}

fn with_metrics(metrics: Arc<MetricsCollector>) -> impl Filter<Extract = (Arc<MetricsCollector>,)> + Clone {
    warp::any().map(move || metrics.clone())
}

// API handlers
async fn get_traces(tracer: Arc<ExecutionTracer>) -> Result<impl Reply, warp::Rejection> {
    let traces = tracer.get_all_traces().await;
    Ok(warp::reply::json(&traces))
}

async fn get_trace(trace_id: String, tracer: Arc<ExecutionTracer>) -> Result<impl Reply, warp::Rejection> {
    match tracer.get_trace(&trace_id).await {
        Some(trace) => Ok(warp::reply::json(&trace)),
        None => Ok(warp::reply::json(&serde_json::json!({"error": "Trace not found"}))),
    }
}

async fn get_workflows(workflows: Arc<RwLock<HashMap<String, crate::visualization::VisualWorkflow>>>) -> Result<impl Reply, warp::Rejection> {
    let workflows = workflows.read().await;
    let workflow_list: Vec<_> = workflows.values().collect();
    Ok(warp::reply::json(&workflow_list))
}

async fn get_metrics(metrics: Arc<MetricsCollector>) -> Result<impl Reply, warp::Rejection> {
    let metrics_data = metrics.get_current_metrics().await;
    Ok(warp::reply::json(&metrics_data))
}

// WebSocket handler for real-time events
async fn handle_websocket(ws: warp::ws::WebSocket, tracer: Arc<ExecutionTracer>) {
    let mut event_receiver = tracer.subscribe_events();
    let (ws_tx, mut ws_rx) = ws.split();
    
    // Forward events to WebSocket
    let forward_events = async {
        while let Ok(event) = event_receiver.recv().await {
            let message = serde_json::to_string(&event).unwrap_or_default();
            if ws_tx.send(warp::ws::Message::text(message)).await.is_err() {
                break;
            }
        }
    };

    // Handle incoming WebSocket messages (if needed)
    let handle_messages = async {
        while let Some(result) = ws_rx.next().await {
            if result.is_err() {
                break;
            }
        }
    };

    // Run both tasks concurrently
    tokio::select! {
        _ = forward_events => {},
        _ = handle_messages => {},
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visualization::execution_tracer::ExecutionTracer;
    use crate::visualization::graph_visualizer::GraphVisualizer;
    use crate::visualization::metrics_collector::MetricsCollector;

    #[tokio::test]
    async fn test_web_server_creation() {
        let tracer = Arc::new(ExecutionTracer::new(100, true));
        let visualizer = Arc::new(GraphVisualizer::new());
        let metrics = Arc::new(MetricsCollector::new(true, 5));
        
        let server = WebServer::new(8080, tracer, visualizer, metrics).await.unwrap();
        assert_eq!(server.port, 8080);
    }


}
