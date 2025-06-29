//! Graph builder for fluent graph construction.

use crate::error::{CoreError, CoreResult};
use crate::graph::{Graph, GraphMetadata};
use crate::node::{Node, NodeId};
use crate::edge::Edge;
use crate::state::State;
use std::collections::HashMap;

/// Builder for constructing graphs with a fluent API
#[derive(Debug)]
pub struct GraphBuilder<S>
where
    S: State,
{
    metadata: GraphMetadata,
    nodes: Vec<(NodeId, Box<dyn Node<S>>)>,
    edges: Vec<Edge>,
    entry_points: Vec<NodeId>,
    exit_points: Vec<NodeId>,
    initial_state: Option<S>,
}

impl<S> GraphBuilder<S>
where
    S: State,
{
    /// Create a new graph builder
    pub fn new() -> Self {
        Self {
            metadata: GraphMetadata::default(),
            nodes: Vec::new(),
            edges: Vec::new(),
            entry_points: Vec::new(),
            exit_points: Vec::new(),
            initial_state: None,
        }
    }

    /// Create a graph builder with a name
    pub fn with_name(name: String) -> Self {
        Self {
            metadata: GraphMetadata::new(name),
            nodes: Vec::new(),
            edges: Vec::new(),
            entry_points: Vec::new(),
            exit_points: Vec::new(),
            initial_state: None,
        }
    }

    /// Set the graph metadata
    pub fn with_metadata(mut self, metadata: GraphMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Set the initial state
    pub fn with_initial_state(mut self, state: S) -> Self {
        self.initial_state = Some(state);
        self
    }

    /// Add a node to the graph
    pub fn add_node<N>(mut self, id: NodeId, node: N) -> Self
    where
        N: Node<S> + 'static,
    {
        self.nodes.push((id, Box::new(node)));
        self
    }

    /// Add an edge to the graph
    pub fn add_edge(mut self, edge: Edge) -> Self {
        self.edges.push(edge);
        self
    }

    /// Add a simple edge between two nodes
    pub fn connect(mut self, from: NodeId, to: NodeId) -> Self {
        let edge_id = format!("{}_{}", from, to);
        let edge = Edge::new(edge_id, from, to);
        self.edges.push(edge);
        self
    }

    /// Add a conditional edge
    pub fn connect_conditional(mut self, from: NodeId, to: NodeId, condition: String) -> Self {
        let edge_id = format!("{}_{}_conditional", from, to);
        let edge = Edge::conditional(edge_id, from, to, condition);
        self.edges.push(edge);
        self
    }

    /// Set entry points
    pub fn with_entry_points(mut self, entry_points: Vec<NodeId>) -> Self {
        self.entry_points = entry_points;
        self
    }

    /// Add a single entry point
    pub fn add_entry_point(mut self, node_id: NodeId) -> Self {
        if !self.entry_points.contains(&node_id) {
            self.entry_points.push(node_id);
        }
        self
    }

    /// Set exit points
    pub fn with_exit_points(mut self, exit_points: Vec<NodeId>) -> Self {
        self.exit_points = exit_points;
        self
    }

    /// Add a single exit point
    pub fn add_exit_point(mut self, node_id: NodeId) -> Self {
        if !self.exit_points.contains(&node_id) {
            self.exit_points.push(node_id);
        }
        self
    }

    /// Add a tag to the graph metadata
    pub fn add_tag(mut self, tag: String) -> Self {
        self.metadata.add_tag(tag);
        self
    }

    /// Set custom metadata
    pub fn set_custom_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.set_custom(key, value);
        self
    }

    /// Build the graph
    pub fn build(self) -> CoreResult<Graph<S>> {
        // Ensure we have an initial state
        let initial_state = self.initial_state.ok_or_else(|| {
            CoreError::validation_error("Initial state is required to build a graph")
        })?;

        // Create the graph
        let mut graph = Graph::with_metadata(initial_state, self.metadata);

        // Add all nodes
        for (id, node) in self.nodes {
            graph.add_node(id, node)?;
        }

        // Add all edges
        for edge in self.edges {
            graph.add_edge(edge)?;
        }

        // Set entry and exit points
        if !self.entry_points.is_empty() {
            graph.set_entry_points(self.entry_points)?;
        }

        if !self.exit_points.is_empty() {
            graph.set_exit_points(self.exit_points)?;
        }

        // Validate the graph
        graph.validate()?;

        Ok(graph)
    }

    /// Build the graph without validation (for testing)
    pub fn build_unchecked(self) -> CoreResult<Graph<S>> {
        let initial_state = self.initial_state.ok_or_else(|| {
            CoreError::validation_error("Initial state is required to build a graph")
        })?;

        let mut graph = Graph::with_metadata(initial_state, self.metadata);

        for (id, node) in self.nodes {
            graph.add_node(id, node)?;
        }

        for edge in self.edges {
            graph.add_edge(edge)?;
        }

        if !self.entry_points.is_empty() {
            graph.set_entry_points(self.entry_points)?;
        }

        if !self.exit_points.is_empty() {
            graph.set_exit_points(self.exit_points)?;
        }

        Ok(graph)
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

/// Convenience macro for building graphs
#[macro_export]
macro_rules! graph {
    ($state:expr) => {
        $crate::graph::GraphBuilder::new().with_initial_state($state)
    };
    ($name:expr, $state:expr) => {
        $crate::graph::GraphBuilder::with_name($name.to_string()).with_initial_state($state)
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::{NodeOutput, NodeMetadata};
    use async_trait::async_trait;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestState {
        value: i32,
    }

    struct TestNode {
        id: String,
        metadata: NodeMetadata,
    }

    #[async_trait]
    impl Node<TestState> for TestNode {
        async fn execute(&self, state: &mut TestState) -> CoreResult<NodeOutput> {
            state.value += 1;
            Ok(NodeOutput::success())
        }

        fn id(&self) -> &str {
            &self.id
        }

        fn metadata(&self) -> &NodeMetadata {
            &self.metadata
        }
    }

    #[test]
    fn test_graph_builder() {
        let initial_state = TestState { value: 0 };
        
        let node1 = TestNode {
            id: "node1".to_string(),
            metadata: NodeMetadata::new("Test Node 1".to_string()),
        };

        let node2 = TestNode {
            id: "node2".to_string(),
            metadata: NodeMetadata::new("Test Node 2".to_string()),
        };

        let graph = GraphBuilder::new()
            .with_initial_state(initial_state)
            .add_node("node1".to_string(), node1)
            .add_node("node2".to_string(), node2)
            .connect("node1".to_string(), "node2".to_string())
            .add_entry_point("node1".to_string())
            .add_exit_point("node2".to_string())
            .build()
            .unwrap();

        assert_eq!(graph.nodes.count(), 2);
        assert_eq!(graph.edges.edge_count(), 1);
        assert_eq!(graph.get_entry_points().len(), 1);
        assert_eq!(graph.get_exit_points().len(), 1);
    }

    #[test]
    fn test_graph_macro() {
        let initial_state = TestState { value: 0 };
        let builder = graph!(initial_state);
        
        // Should compile and create a builder
        assert!(builder.initial_state.is_some());
    }
}