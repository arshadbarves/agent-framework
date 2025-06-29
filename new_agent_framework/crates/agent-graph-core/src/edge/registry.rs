//! Edge registry for managing graph edges.

use crate::error::{CoreError, CoreResult};
use crate::edge::{Edge, EdgeCondition};
use crate::node::NodeId;
use crate::state::State;
use std::collections::HashMap;
use std::sync::Arc;

/// Registry for managing edges and their conditions
#[derive(Debug)]
pub struct EdgeRegistry<S>
where
    S: State,
{
    /// All edges in the registry
    edges: HashMap<String, Edge>,
    /// Edge conditions
    conditions: HashMap<String, Arc<dyn EdgeCondition<S>>>,
    /// Index of outgoing edges by source node
    outgoing_index: HashMap<NodeId, Vec<String>>,
    /// Index of incoming edges by target node
    incoming_index: HashMap<NodeId, Vec<String>>,
}

impl<S> EdgeRegistry<S>
where
    S: State,
{
    /// Create a new edge registry
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
            conditions: HashMap::new(),
            outgoing_index: HashMap::new(),
            incoming_index: HashMap::new(),
        }
    }

    /// Add an edge to the registry
    pub fn add_edge(&mut self, edge: Edge) -> CoreResult<()> {
        let edge_id = edge.id.clone();
        let from_node = edge.from.clone();
        let to_node = edge.to.clone();

        // Check for duplicate edge ID
        if self.edges.contains_key(&edge_id) {
            return Err(CoreError::validation_error(format!(
                "Edge with ID '{}' already exists",
                edge_id
            )));
        }

        // Add to main storage
        self.edges.insert(edge_id.clone(), edge);

        // Update indices
        self.outgoing_index
            .entry(from_node)
            .or_insert_with(Vec::new)
            .push(edge_id.clone());

        self.incoming_index
            .entry(to_node)
            .or_insert_with(Vec::new)
            .push(edge_id);

        Ok(())
    }

    /// Remove an edge from the registry
    pub fn remove_edge(&mut self, edge_id: &str) -> CoreResult<Edge> {
        let edge = self.edges.remove(edge_id).ok_or_else(|| {
            CoreError::validation_error(format!("Edge '{}' not found", edge_id))
        })?;

        // Update indices
        if let Some(outgoing) = self.outgoing_index.get_mut(&edge.from) {
            outgoing.retain(|id| id != edge_id);
            if outgoing.is_empty() {
                self.outgoing_index.remove(&edge.from);
            }
        }

        if let Some(incoming) = self.incoming_index.get_mut(&edge.to) {
            incoming.retain(|id| id != edge_id);
            if incoming.is_empty() {
                self.incoming_index.remove(&edge.to);
            }
        }

        Ok(edge)
    }

    /// Get an edge by ID
    pub fn get_edge(&self, edge_id: &str) -> Option<&Edge> {
        self.edges.get(edge_id)
    }

    /// Get all edges
    pub fn get_all_edges(&self) -> Vec<&Edge> {
        self.edges.values().collect()
    }

    /// Get outgoing edges from a node
    pub fn get_outgoing_edges(&self, node_id: &NodeId) -> Vec<&Edge> {
        self.outgoing_index
            .get(node_id)
            .map(|edge_ids| {
                edge_ids
                    .iter()
                    .filter_map(|id| self.edges.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get incoming edges to a node
    pub fn get_incoming_edges(&self, node_id: &NodeId) -> Vec<&Edge> {
        self.incoming_index
            .get(node_id)
            .map(|edge_ids| {
                edge_ids
                    .iter()
                    .filter_map(|id| self.edges.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Register an edge condition
    pub fn register_condition(&mut self, name: String, condition: Arc<dyn EdgeCondition<S>>) {
        self.conditions.insert(name, condition);
    }

    /// Get an edge condition
    pub fn get_condition(&self, name: &str) -> Option<Arc<dyn EdgeCondition<S>>> {
        self.conditions.get(name).cloned()
    }

    /// Check if a node has outgoing edges
    pub fn has_outgoing_edges(&self, node_id: &NodeId) -> bool {
        self.outgoing_index.contains_key(node_id)
    }

    /// Check if a node has incoming edges
    pub fn has_incoming_edges(&self, node_id: &NodeId) -> bool {
        self.incoming_index.contains_key(node_id)
    }

    /// Get all nodes that have outgoing edges
    pub fn get_source_nodes(&self) -> Vec<NodeId> {
        self.outgoing_index.keys().cloned().collect()
    }

    /// Get all nodes that have incoming edges
    pub fn get_target_nodes(&self) -> Vec<NodeId> {
        self.incoming_index.keys().cloned().collect()
    }

    /// Get all unique nodes referenced in edges
    pub fn get_all_nodes(&self) -> Vec<NodeId> {
        let mut nodes = std::collections::HashSet::new();
        
        for edge in self.edges.values() {
            nodes.insert(edge.from.clone());
            nodes.insert(edge.to.clone());
        }
        
        nodes.into_iter().collect()
    }

    /// Find edges between two specific nodes
    pub fn find_edges_between(&self, from: &NodeId, to: &NodeId) -> Vec<&Edge> {
        self.get_outgoing_edges(from)
            .into_iter()
            .filter(|edge| &edge.to == to)
            .collect()
    }

    /// Get edge count
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Get condition count
    pub fn condition_count(&self) -> usize {
        self.conditions.len()
    }

    /// Validate all edges
    pub fn validate(&self) -> CoreResult<()> {
        for edge in self.edges.values() {
            // Check if referenced condition exists
            if let Some(condition_name) = &edge.condition {
                if !self.conditions.contains_key(condition_name) {
                    return Err(CoreError::validation_error(format!(
                        "Edge '{}' references unknown condition '{}'",
                        edge.id, condition_name
                    )));
                }
            }
        }
        Ok(())
    }

    /// Clear all edges and conditions
    pub fn clear(&mut self) {
        self.edges.clear();
        self.conditions.clear();
        self.outgoing_index.clear();
        self.incoming_index.clear();
    }

    /// Get registry statistics
    pub fn statistics(&self) -> RegistryStatistics {
        let mut stats = RegistryStatistics::default();
        stats.total_edges = self.edges.len();
        stats.total_conditions = self.conditions.len();
        stats.nodes_with_outgoing = self.outgoing_index.len();
        stats.nodes_with_incoming = self.incoming_index.len();

        // Count edge types
        for edge in self.edges.values() {
            match edge.edge_type {
                crate::edge::EdgeType::Normal => stats.normal_edges += 1,
                crate::edge::EdgeType::Conditional => stats.conditional_edges += 1,
                crate::edge::EdgeType::ErrorHandler => stats.error_handler_edges += 1,
                crate::edge::EdgeType::Parallel => stats.parallel_edges += 1,
                crate::edge::EdgeType::Loop => stats.loop_edges += 1,
                crate::edge::EdgeType::Interrupt => stats.interrupt_edges += 1,
                crate::edge::EdgeType::Custom(_) => stats.custom_edges += 1,
            }

            if edge.condition.is_some() {
                stats.edges_with_conditions += 1;
            }
        }

        stats
    }

    /// Export edges for serialization
    pub fn export_edges(&self) -> Vec<Edge> {
        self.edges.values().cloned().collect()
    }

    /// Import edges from serialization
    pub fn import_edges(&mut self, edges: Vec<Edge>) -> CoreResult<()> {
        self.clear();
        for edge in edges {
            self.add_edge(edge)?;
        }
        Ok(())
    }
}

impl<S> Default for EdgeRegistry<S>
where
    S: State,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Clone for EdgeRegistry<S>
where
    S: State,
{
    fn clone(&self) -> Self {
        Self {
            edges: self.edges.clone(),
            conditions: self.conditions.clone(),
            outgoing_index: self.outgoing_index.clone(),
            incoming_index: self.incoming_index.clone(),
        }
    }
}

/// Statistics about the edge registry
#[derive(Debug, Default, Clone)]
pub struct RegistryStatistics {
    /// Total number of edges
    pub total_edges: usize,
    /// Total number of conditions
    pub total_conditions: usize,
    /// Number of nodes with outgoing edges
    pub nodes_with_outgoing: usize,
    /// Number of nodes with incoming edges
    pub nodes_with_incoming: usize,
    /// Number of normal edges
    pub normal_edges: usize,
    /// Number of conditional edges
    pub conditional_edges: usize,
    /// Number of error handler edges
    pub error_handler_edges: usize,
    /// Number of parallel edges
    pub parallel_edges: usize,
    /// Number of loop edges
    pub loop_edges: usize,
    /// Number of interrupt edges
    pub interrupt_edges: usize,
    /// Number of custom edges
    pub custom_edges: usize,
    /// Number of edges with conditions
    pub edges_with_conditions: usize,
}

impl RegistryStatistics {
    /// Get percentage of edges with conditions
    pub fn condition_percentage(&self) -> f64 {
        if self.total_edges == 0 {
            0.0
        } else {
            (self.edges_with_conditions as f64 / self.total_edges as f64) * 100.0
        }
    }

    /// Get average edges per node
    pub fn average_edges_per_node(&self) -> f64 {
        let total_nodes = (self.nodes_with_outgoing + self.nodes_with_incoming) as f64;
        if total_nodes == 0.0 {
            0.0
        } else {
            self.total_edges as f64 / total_nodes
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edge::{EdgeMetadata, EdgeType};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestState {
        value: i32,
    }

    fn create_test_edge(id: &str, from: &str, to: &str) -> Edge {
        Edge {
            id: id.to_string(),
            from: from.to_string(),
            to: to.to_string(),
            edge_type: EdgeType::Normal,
            condition: None,
            metadata: EdgeMetadata::default(),
            weight: 1.0,
        }
    }

    #[test]
    fn test_edge_registry_creation() {
        let registry: EdgeRegistry<TestState> = EdgeRegistry::new();
        assert_eq!(registry.edge_count(), 0);
        assert_eq!(registry.condition_count(), 0);
    }

    #[test]
    fn test_add_and_get_edge() {
        let mut registry = EdgeRegistry::new();
        let edge = create_test_edge("e1", "n1", "n2");
        
        assert!(registry.add_edge(edge.clone()).is_ok());
        assert_eq!(registry.edge_count(), 1);
        
        let retrieved = registry.get_edge("e1");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "e1");
    }

    #[test]
    fn test_outgoing_and_incoming_edges() {
        let mut registry = EdgeRegistry::new();
        
        let edge1 = create_test_edge("e1", "n1", "n2");
        let edge2 = create_test_edge("e2", "n1", "n3");
        let edge3 = create_test_edge("e3", "n2", "n3");
        
        registry.add_edge(edge1).unwrap();
        registry.add_edge(edge2).unwrap();
        registry.add_edge(edge3).unwrap();
        
        let outgoing_n1 = registry.get_outgoing_edges(&"n1".to_string());
        assert_eq!(outgoing_n1.len(), 2);
        
        let incoming_n3 = registry.get_incoming_edges(&"n3".to_string());
        assert_eq!(incoming_n3.len(), 2);
        
        let incoming_n1 = registry.get_incoming_edges(&"n1".to_string());
        assert_eq!(incoming_n1.len(), 0);
    }
}