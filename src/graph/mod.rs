//! Core graph engine and execution logic.

pub mod agent_node;
pub mod command;
pub mod engine;
pub mod executor;
pub mod routing_node;
pub mod tool_node;

use crate::edge::{Edge, EdgeRegistry};
use crate::error::{GraphError, GraphResult};
use crate::node::{Node, NodeId, NodeRegistry};
use crate::state::State;
use std::collections::HashMap;
use uuid::Uuid;

#[cfg(feature = "streaming")]
use crate::streaming::EventEmitter;

#[cfg(feature = "checkpointing")]
use crate::state::checkpointing::Checkpointer;

/// Core graph structure for managing nodes and edges
pub struct Graph<S>
where
    S: State,
{
    /// Node registry
    nodes: NodeRegistry<S>,
    /// Edge registry for conditions and routers
    edge_registry: EdgeRegistry<S>,
    /// Graph edges
    edges: Vec<Edge>,
    /// Entry point node
    entry_point: Option<NodeId>,
    /// Finish point nodes
    finish_points: Vec<NodeId>,
    /// Graph metadata
    metadata: GraphMetadata,
    /// Execution configuration
    config: ExecutionConfig,

    #[cfg(feature = "streaming")]
    /// Event emitter for streaming
    event_emitter: Option<EventEmitter>,

    #[cfg(feature = "checkpointing")]
    /// Checkpointer for state persistence
    checkpointer: Option<Box<dyn Checkpointer<S>>>,
}

/// Graph metadata
#[derive(Debug, Clone)]
pub struct GraphMetadata {
    /// Graph name
    pub name: String,
    /// Graph description
    pub description: Option<String>,
    /// Graph version
    pub version: String,
    /// Graph tags
    pub tags: Vec<String>,
    /// Custom metadata
    pub custom: HashMap<String, serde_json::Value>,
}

impl Default for GraphMetadata {
    fn default() -> Self {
        Self {
            name: "Unnamed Graph".to_string(),
            description: None,
            version: "1.0.0".to_string(),
            tags: Vec::new(),
            custom: HashMap::new(),
        }
    }
}

/// Execution configuration
#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    /// Maximum execution time in seconds
    pub max_execution_time_seconds: Option<u64>,
    /// Maximum number of steps
    pub max_steps: Option<u64>,
    /// Whether to enable parallel execution
    pub enable_parallel: bool,
    /// Whether to enable checkpointing
    pub enable_checkpointing: bool,
    /// Whether to enable streaming
    pub enable_streaming: bool,
    /// Checkpoint interval (number of steps)
    pub checkpoint_interval: Option<u64>,
    /// Maximum number of retries for failed nodes
    pub max_retries: u32,
    /// Whether to stop on first error
    pub stop_on_error: bool,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            max_execution_time_seconds: Some(300), // 5 minutes
            max_steps: Some(1000),
            enable_parallel: true,
            enable_checkpointing: false,
            enable_streaming: false,
            checkpoint_interval: Some(10),
            max_retries: 3,
            stop_on_error: true,
        }
    }
}

/// Execution context for graph runs
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Unique execution ID
    pub execution_id: Uuid,
    /// Start time
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// Current step number
    pub current_step: u64,
    /// Current node being executed
    pub current_node: Option<NodeId>,
    /// Execution path taken
    pub execution_path: Vec<NodeId>,
    /// Custom context data
    pub custom_data: HashMap<String, serde_json::Value>,
}

impl ExecutionContext {
    /// Create a new execution context
    pub fn new() -> Self {
        Self {
            execution_id: Uuid::new_v4(),
            start_time: chrono::Utc::now(),
            current_step: 0,
            current_node: None,
            execution_path: Vec::new(),
            custom_data: HashMap::new(),
        }
    }

    /// Add a node to the execution path
    pub fn add_to_path(&mut self, node_id: NodeId) {
        self.execution_path.push(node_id);
    }

    /// Increment the step counter
    pub fn increment_step(&mut self) {
        self.current_step += 1;
    }

    /// Set custom data
    pub fn set_custom_data<K, V>(&mut self, key: K, value: V)
    where
        K: Into<String>,
        V: serde::Serialize,
    {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.custom_data.insert(key.into(), json_value);
        }
    }

    /// Get custom data
    pub fn get_custom_data<T>(&self, key: &str) -> Option<T>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        self.custom_data
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Get execution duration
    pub fn duration(&self) -> chrono::Duration {
        chrono::Utc::now() - self.start_time
    }

    /// Get execution duration in milliseconds
    pub fn duration_ms(&self) -> u64 {
        self.duration().num_milliseconds() as u64
    }
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Graph<S>
where
    S: State,
{
    /// Create a new graph
    pub fn new() -> Self {
        Self {
            nodes: NodeRegistry::new(),
            edge_registry: EdgeRegistry::new(),
            edges: Vec::new(),
            entry_point: None,
            finish_points: Vec::new(),
            metadata: GraphMetadata::default(),
            config: ExecutionConfig::default(),

            #[cfg(feature = "streaming")]
            event_emitter: None,

            #[cfg(feature = "checkpointing")]
            checkpointer: None,
        }
    }

    /// Create a new graph with metadata
    pub fn with_metadata(metadata: GraphMetadata) -> Self {
        Self {
            metadata,
            ..Self::new()
        }
    }

    /// Create a new graph with configuration
    pub fn with_config(config: ExecutionConfig) -> Self {
        Self {
            config,
            ..Self::new()
        }
    }

    /// Add a node to the graph
    pub fn add_node<N>(&mut self, id: NodeId, node: N) -> GraphResult<()>
    where
        N: Node<S> + 'static,
    {
        self.nodes.register(id, node)
    }

    /// Add an edge to the graph
    pub fn add_edge(&mut self, edge: Edge) -> GraphResult<()> {
        // Validate that source node exists
        if !self.nodes.contains(&edge.from) {
            return Err(GraphError::graph_structure(format!(
                "Source node '{}' does not exist",
                edge.from
            )));
        }

        // Validate that target nodes exist
        for target in edge.possible_targets() {
            if !self.nodes.contains(target) {
                return Err(GraphError::graph_structure(format!(
                    "Target node '{}' does not exist",
                    target
                )));
            }
        }

        self.edges.push(edge);
        Ok(())
    }

    /// Set the entry point for graph execution
    pub fn set_entry_point(&mut self, node_id: NodeId) -> GraphResult<()> {
        if !self.nodes.contains(&node_id) {
            return Err(GraphError::graph_structure(format!(
                "Entry point node '{}' does not exist",
                node_id
            )));
        }
        self.entry_point = Some(node_id);
        Ok(())
    }

    /// Add a finish point
    pub fn add_finish_point(&mut self, node_id: NodeId) -> GraphResult<()> {
        if !self.nodes.contains(&node_id) {
            return Err(GraphError::graph_structure(format!(
                "Finish point node '{}' does not exist",
                node_id
            )));
        }
        self.finish_points.push(node_id);
        Ok(())
    }

    /// Set a single finish point (convenience method)
    pub fn set_finish_point(&mut self, node_id: NodeId) -> GraphResult<()> {
        self.finish_points.clear();
        self.add_finish_point(node_id)
    }

    /// Get graph metadata
    pub fn metadata(&self) -> &GraphMetadata {
        &self.metadata
    }

    /// Get execution configuration
    pub fn config(&self) -> &ExecutionConfig {
        &self.config
    }

    /// Set execution configuration
    pub fn set_config(&mut self, config: ExecutionConfig) {
        self.config = config;
    }

    /// Get the entry point
    pub fn entry_point(&self) -> Option<&NodeId> {
        self.entry_point.as_ref()
    }

    /// Get finish points
    pub fn finish_points(&self) -> &[NodeId] {
        &self.finish_points
    }

    /// Get all node IDs
    pub fn node_ids(&self) -> Vec<&NodeId> {
        self.nodes.list_nodes()
    }

    /// Get all edges
    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    /// Check if the graph is valid for execution
    pub fn validate(&self) -> GraphResult<()> {
        // Check if entry point is set
        if self.entry_point.is_none() {
            return Err(GraphError::graph_structure(
                "No entry point set for graph".to_string(),
            ));
        }

        // Check if there are any nodes
        if self.nodes.is_empty() {
            return Err(GraphError::graph_structure(
                "Graph has no nodes".to_string(),
            ));
        }

        // Check if finish points are set
        if self.finish_points.is_empty() {
            return Err(GraphError::graph_structure(
                "No finish points set for graph".to_string(),
            ));
        }

        // Validate that all edges reference existing nodes
        for edge in &self.edges {
            if !self.nodes.contains(&edge.from) {
                return Err(GraphError::graph_structure(format!(
                    "Edge references non-existent source node: {}",
                    edge.from
                )));
            }

            for target in edge.possible_targets() {
                if !self.nodes.contains(target) {
                    return Err(GraphError::graph_structure(format!(
                        "Edge references non-existent target node: {}",
                        target
                    )));
                }
            }
        }

        Ok(())
    }

    /// Get node registry (for advanced usage)
    pub fn node_registry(&self) -> &NodeRegistry<S> {
        &self.nodes
    }

    /// Get edge registry (for advanced usage)
    pub fn edge_registry(&self) -> &EdgeRegistry<S> {
        &self.edge_registry
    }

    /// Get mutable edge registry (for advanced usage)
    pub fn edge_registry_mut(&mut self) -> &mut EdgeRegistry<S> {
        &mut self.edge_registry
    }

    #[cfg(feature = "streaming")]
    /// Set event emitter for streaming
    pub fn set_event_emitter(&mut self, emitter: EventEmitter) {
        self.event_emitter = Some(emitter);
    }

    #[cfg(feature = "checkpointing")]
    /// Set checkpointer for state persistence
    pub fn set_checkpointer<C>(&mut self, checkpointer: C)
    where
        S: serde::Serialize + for<'de> serde::Deserialize<'de>,
        C: Checkpointer<S> + 'static,
    {
        self.checkpointer = Some(Box::new(checkpointer));
    }
}

impl<S> Default for Graph<S>
where
    S: State,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<S> std::fmt::Debug for Graph<S>
where
    S: State,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Graph")
            .field("nodes", &self.nodes)
            .field("edge_registry", &self.edge_registry)
            .field("edges", &self.edges)
            .field("entry_point", &self.entry_point)
            .field("finish_points", &self.finish_points)
            .field("metadata", &self.metadata)
            .field("config", &self.config)
            .finish()
    }
}

/// Builder pattern for constructing graphs
#[derive(Debug)]
pub struct GraphBuilder<S>
where
    S: State,
{
    graph: Graph<S>,
}

impl<S> GraphBuilder<S>
where
    S: State,
{
    /// Create a new graph builder
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
        }
    }

    /// Set graph metadata
    pub fn with_metadata(mut self, metadata: GraphMetadata) -> Self {
        self.graph.metadata = metadata;
        self
    }

    /// Set execution configuration
    pub fn with_config(mut self, config: ExecutionConfig) -> Self {
        self.graph.config = config;
        self
    }

    /// Add a node
    pub fn add_node<N>(mut self, id: NodeId, node: N) -> GraphResult<Self>
    where
        N: Node<S> + 'static,
    {
        self.graph.add_node(id, node)?;
        Ok(self)
    }

    /// Add an edge
    pub fn add_edge(mut self, edge: Edge) -> GraphResult<Self> {
        self.graph.add_edge(edge)?;
        Ok(self)
    }

    /// Set entry point
    pub fn with_entry_point(mut self, node_id: NodeId) -> GraphResult<Self> {
        self.graph.set_entry_point(node_id)?;
        Ok(self)
    }

    /// Add finish point
    pub fn add_finish_point(mut self, node_id: NodeId) -> GraphResult<Self> {
        self.graph.add_finish_point(node_id)?;
        Ok(self)
    }

    /// Build the graph
    pub fn build(self) -> GraphResult<Graph<S>> {
        self.graph.validate()?;
        Ok(self.graph)
    }
}

impl<S> Default for GraphBuilder<S>
where
    S: State,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::Node;
    use async_trait::async_trait;

    #[derive(Debug, Clone)]
    struct TestState {
        value: i32,
    }



    #[derive(Debug)]
    struct TestNode {
        increment: i32,
    }

    #[async_trait]
    impl Node<TestState> for TestNode {
        async fn invoke(&self, state: &mut TestState) -> GraphResult<()> {
            state.value += self.increment;
            Ok(())
        }
    }

    #[test]
    fn test_graph_creation() {
        let mut graph = Graph::new();
        let node = TestNode { increment: 1 };

        graph.add_node("test".to_string(), node).unwrap();
        graph.set_entry_point("test".to_string()).unwrap();
        graph.set_finish_point("test".to_string()).unwrap();

        assert!(graph.validate().is_ok());
    }

    #[test]
    fn test_graph_builder() {
        let node = TestNode { increment: 1 };
        let graph = GraphBuilder::new()
            .add_node("test".to_string(), node)
            .unwrap()
            .with_entry_point("test".to_string())
            .unwrap()
            .add_finish_point("test".to_string())
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(graph.node_ids().len(), 1);
        assert_eq!(graph.entry_point(), Some(&"test".to_string()));
    }

    #[test]
    fn test_execution_context() {
        let mut context = ExecutionContext::new();
        context.add_to_path("node1".to_string());
        context.increment_step();
        context.set_custom_data("key", "value");

        assert_eq!(context.current_step, 1);
        assert_eq!(context.execution_path.len(), 1);
        assert_eq!(
            context.get_custom_data::<String>("key"),
            Some("value".to_string())
        );
    }
}
