//! Node registry for managing and discovering nodes.

use crate::error::{CoreError, CoreResult};
use crate::node::{Node, NodeId, NodeMetadata, NodeCategory, NodePriority};
use crate::state::State;
use std::collections::HashMap;
use std::sync::Arc;

/// Registry for managing nodes
#[derive(Debug)]
pub struct NodeRegistry<S>
where
    S: State,
{
    nodes: HashMap<NodeId, Arc<dyn Node<S>>>,
    metadata_cache: HashMap<NodeId, NodeMetadata>,
}

impl<S> NodeRegistry<S>
where
    S: State,
{
    /// Create a new node registry
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            metadata_cache: HashMap::new(),
        }
    }

    /// Register a node
    pub fn register<N>(&mut self, id: NodeId, node: N) -> CoreResult<()>
    where
        N: Node<S> + 'static,
    {
        if self.nodes.contains_key(&id) {
            return Err(CoreError::validation_error(format!(
                "Node with id '{}' already exists",
                id
            )));
        }

        let metadata = node.metadata().clone();
        let node_arc = Arc::new(node);

        self.nodes.insert(id.clone(), node_arc);
        self.metadata_cache.insert(id, metadata);

        Ok(())
    }

    /// Unregister a node
    pub fn unregister(&mut self, id: &NodeId) -> CoreResult<()> {
        if !self.nodes.contains_key(id) {
            return Err(CoreError::validation_error(format!(
                "Node with id '{}' not found",
                id
            )));
        }

        self.nodes.remove(id);
        self.metadata_cache.remove(id);

        Ok(())
    }

    /// Get a node by ID
    pub fn get(&self, id: &NodeId) -> Option<Arc<dyn Node<S>>> {
        self.nodes.get(id).cloned()
    }

    /// Check if a node exists
    pub fn contains(&self, id: &NodeId) -> bool {
        self.nodes.contains_key(id)
    }

    /// Get all node IDs
    pub fn node_ids(&self) -> Vec<NodeId> {
        self.nodes.keys().cloned().collect()
    }

    /// Get node count
    pub fn count(&self) -> usize {
        self.nodes.len()
    }

    /// Get node metadata
    pub fn get_metadata(&self, id: &NodeId) -> Option<&NodeMetadata> {
        self.metadata_cache.get(id)
    }

    /// Find nodes by category
    pub fn find_by_category(&self, category: &NodeCategory) -> Vec<NodeId> {
        self.metadata_cache
            .iter()
            .filter(|(_, metadata)| &metadata.category == category)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Find nodes by tag
    pub fn find_by_tag(&self, tag: &str) -> Vec<NodeId> {
        self.metadata_cache
            .iter()
            .filter(|(_, metadata)| metadata.has_tag(tag))
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Validate all registered nodes
    pub fn validate_all(&self) -> CoreResult<()> {
        for (id, node) in &self.nodes {
            node.validate().map_err(|e| {
                CoreError::validation_error(format!("Node '{}' validation failed: {}", id, e))
            })?;
        }
        Ok(())
    }
}

impl<S> Default for NodeRegistry<S>
where
    S: State,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Clone for NodeRegistry<S>
where
    S: State,
{
    fn clone(&self) -> Self {
        Self {
            nodes: self.nodes.clone(),
            metadata_cache: self.metadata_cache.clone(),
        }
    }
}