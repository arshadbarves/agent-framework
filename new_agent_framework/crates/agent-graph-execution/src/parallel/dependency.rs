//! Dependency graph management for parallel execution.

use crate::{CoreError, CoreResult, NodeId};
use agent_graph_core::Graph;
use crate::state::State;
use std::collections::{HashMap, HashSet};

/// Dependency graph for managing node execution order
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// Node dependencies (node -> dependencies)
    dependencies: HashMap<NodeId, HashSet<NodeId>>,
    /// Reverse dependencies (node -> dependents)
    dependents: HashMap<NodeId, HashSet<NodeId>>,
    /// Completed nodes
    completed: HashSet<NodeId>,
}

impl DependencyGraph {
    /// Create a new dependency graph
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
            dependents: HashMap::new(),
            completed: HashSet::new(),
        }
    }

    /// Create dependency graph from a graph structure
    pub fn from_graph<S>(graph: &Graph<S>) -> CoreResult<Self>
    where
        S: State,
    {
        let mut dep_graph = Self::new();
        
        // Add all nodes
        for node_id in graph.nodes.node_ids() {
            dep_graph.add_node(node_id);
        }
        
        // Add dependencies based on edges
        for edge in graph.edges.get_all_edges() {
            dep_graph.add_dependency(edge.to.clone(), edge.from.clone());
        }
        
        // Validate for cycles
        if dep_graph.has_cycles() {
            return Err(CoreError::validation_error("Dependency graph contains cycles"));
        }
        
        Ok(dep_graph)
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, node_id: NodeId) {
        self.dependencies.entry(node_id.clone()).or_insert_with(HashSet::new);
        self.dependents.entry(node_id).or_insert_with(HashSet::new);
    }

    /// Add a dependency (dependent depends on dependency)
    pub fn add_dependency(&mut self, dependent: NodeId, dependency: NodeId) {
        self.dependencies.entry(dependent.clone())
            .or_insert_with(HashSet::new)
            .insert(dependency.clone());
        
        self.dependents.entry(dependency)
            .or_insert_with(HashSet::new)
            .insert(dependent);
    }

    /// Get nodes that are ready for execution (no pending dependencies)
    pub fn get_ready_nodes(&self) -> Vec<NodeId> {
        self.dependencies
            .iter()
            .filter(|(node_id, deps)| {
                !self.completed.contains(*node_id) && 
                deps.iter().all(|dep| self.completed.contains(dep))
            })
            .map(|(node_id, _)| node_id.clone())
            .collect()
    }

    /// Mark a node as completed and return newly ready nodes
    pub fn mark_completed(&mut self, node_id: &NodeId) -> Vec<NodeId> {
        self.completed.insert(node_id.clone());
        
        // Find dependents that might now be ready
        if let Some(dependents) = self.dependents.get(node_id) {
            dependents
                .iter()
                .filter(|dependent| {
                    !self.completed.contains(*dependent) &&
                    self.dependencies.get(*dependent)
                        .map(|deps| deps.iter().all(|dep| self.completed.contains(dep)))
                        .unwrap_or(false)
                })
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Check if the graph has cycles using DFS
    pub fn has_cycles(&self) -> bool {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        
        for node in self.dependencies.keys() {
            if !visited.contains(node) {
                if self.has_cycle_util(node, &mut visited, &mut rec_stack) {
                    return true;
                }
            }
        }
        
        false
    }

    /// Utility function for cycle detection
    fn has_cycle_util(
        &self,
        node: &NodeId,
        visited: &mut HashSet<NodeId>,
        rec_stack: &mut HashSet<NodeId>,
    ) -> bool {
        visited.insert(node.clone());
        rec_stack.insert(node.clone());
        
        if let Some(dependencies) = self.dependencies.get(node) {
            for dep in dependencies {
                if !visited.contains(dep) {
                    if self.has_cycle_util(dep, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(dep) {
                    return true;
                }
            }
        }
        
        rec_stack.remove(node);
        false
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}