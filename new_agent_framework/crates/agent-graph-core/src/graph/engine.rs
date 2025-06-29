//! Core graph engine implementation.

use crate::error::{CoreError, CoreResult};
use crate::edge::EdgeRegistry;
use crate::node::{Node, NodeId, NodeRegistry};
use crate::runtime::ExecutionContext;
use crate::state::{State, StateManager};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Core graph structure for managing nodes and edges
#[derive(Debug)]
pub struct Graph<S>
where
    S: State,
{
    /// Graph metadata
    pub metadata: GraphMetadata,
    /// Node registry
    pub nodes: NodeRegistry<S>,
    /// Edge registry
    pub edges: EdgeRegistry<S>,
    /// Entry point nodes
    pub entry_points: Vec<NodeId>,
    /// Exit point nodes
    pub exit_points: Vec<NodeId>,
    /// State manager
    pub state_manager: Arc<StateManager<S>>,
}

impl<S> Graph<S>
where
    S: State,
{
    /// Create a new graph
    pub fn new(initial_state: S) -> Self {
        Self {
            metadata: GraphMetadata::default(),
            nodes: NodeRegistry::new(),
            edges: EdgeRegistry::new(),
            entry_points: Vec::new(),
            exit_points: Vec::new(),
            state_manager: Arc::new(StateManager::new(initial_state)),
        }
    }

    /// Create a new graph with metadata
    pub fn with_metadata(initial_state: S, metadata: GraphMetadata) -> Self {
        Self {
            metadata,
            nodes: NodeRegistry::new(),
            edges: EdgeRegistry::new(),
            entry_points: Vec::new(),
            exit_points: Vec::new(),
            state_manager: Arc::new(StateManager::new(initial_state)),
        }
    }

    /// Add a node to the graph
    pub fn add_node<N>(&mut self, id: NodeId, node: N) -> CoreResult<()>
    where
        N: Node<S> + 'static,
    {
        self.nodes.register(id, node)
    }

    /// Remove a node from the graph
    pub fn remove_node(&mut self, id: &NodeId) -> CoreResult<()> {
        // Remove all edges connected to this node
        let outgoing_edges: Vec<_> = self.edges.get_outgoing_edges(id)
            .iter()
            .map(|e| e.id.clone())
            .collect();
        
        let incoming_edges: Vec<_> = self.edges.get_incoming_edges(id)
            .iter()
            .map(|e| e.id.clone())
            .collect();

        for edge_id in outgoing_edges.iter().chain(incoming_edges.iter()) {
            self.edges.remove_edge(edge_id)?;
        }

        // Remove from entry/exit points
        self.entry_points.retain(|node_id| node_id != id);
        self.exit_points.retain(|node_id| node_id != id);

        // Remove the node
        self.nodes.unregister(id)
    }

    /// Add an edge to the graph
    pub fn add_edge(&mut self, edge: crate::edge::Edge) -> CoreResult<()> {
        // Validate that both nodes exist
        if !self.nodes.contains(&edge.from) {
            return Err(CoreError::validation_error(format!(
                "Source node '{}' does not exist",
                edge.from
            )));
        }

        if !self.nodes.contains(&edge.to) {
            return Err(CoreError::validation_error(format!(
                "Target node '{}' does not exist",
                edge.to
            )));
        }

        self.edges.add_edge(edge)
    }

    /// Set entry points for the graph
    pub fn set_entry_points(&mut self, entry_points: Vec<NodeId>) -> CoreResult<()> {
        // Validate that all entry points exist
        for node_id in &entry_points {
            if !self.nodes.contains(node_id) {
                return Err(CoreError::validation_error(format!(
                    "Entry point node '{}' does not exist",
                    node_id
                )));
            }
        }

        self.entry_points = entry_points;
        Ok(())
    }

    /// Set exit points for the graph
    pub fn set_exit_points(&mut self, exit_points: Vec<NodeId>) -> CoreResult<()> {
        // Validate that all exit points exist
        for node_id in &exit_points {
            if !self.nodes.contains(node_id) {
                return Err(CoreError::validation_error(format!(
                    "Exit point node '{}' does not exist",
                    node_id
                )));
            }
        }

        self.exit_points = exit_points;
        Ok(())
    }

    /// Get entry points
    pub fn get_entry_points(&self) -> &[NodeId] {
        &self.entry_points
    }

    /// Get exit points
    pub fn get_exit_points(&self) -> &[NodeId] {
        &self.exit_points
    }

    /// Validate the graph structure
    pub fn validate(&self) -> CoreResult<()> {
        // Validate nodes
        self.nodes.validate_all()?;

        // Validate edges
        self.edges.validate()?;

        // Check that we have at least one entry point
        if self.entry_points.is_empty() {
            return Err(CoreError::validation_error(
                "Graph must have at least one entry point".to_string()
            ));
        }

        Ok(())
    }

    /// Clone the graph structure (without state)
    pub fn clone_structure(&self) -> Graph<S>
    where
        S: Clone,
    {
        let initial_state = self.state_manager.read_state(|state| state.clone());
        
        Graph {
            metadata: self.metadata.clone(),
            nodes: self.nodes.clone(),
            edges: self.edges.clone(),
            entry_points: self.entry_points.clone(),
            exit_points: self.exit_points.clone(),
            state_manager: Arc::new(StateManager::new(initial_state)),
        }
    }
}

/// Graph metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    /// Graph ID
    pub id: Uuid,
    /// Graph name
    pub name: String,
    /// Graph description
    pub description: Option<String>,
    /// Graph version
    pub version: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last modified timestamp
    pub modified_at: DateTime<Utc>,
    /// Graph tags
    pub tags: Vec<String>,
    /// Custom metadata
    pub custom: HashMap<String, serde_json::Value>,
}

impl Default for GraphMetadata {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: "Unnamed Graph".to_string(),
            description: None,
            version: "1.0.0".to_string(),
            created_at: now,
            modified_at: now,
            tags: Vec::new(),
            custom: HashMap::new(),
        }
    }
}

impl GraphMetadata {
    /// Create new metadata with name
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }

    /// Update the modified timestamp
    pub fn touch(&mut self) {
        self.modified_at = Utc::now();
    }

    /// Add a tag
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// Set custom metadata
    pub fn set_custom(&mut self, key: String, value: serde_json::Value) {
        self.custom.insert(key, value);
    }
}