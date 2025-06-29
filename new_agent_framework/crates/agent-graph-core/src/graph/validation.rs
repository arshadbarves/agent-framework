//! Graph validation and analysis.

use crate::error::{CoreError, CoreResult};
use crate::graph::Graph;
use crate::node::NodeId;
use crate::state::State;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

/// Graph validator for comprehensive graph analysis
#[derive(Debug)]
pub struct GraphValidator;

impl GraphValidator {
    /// Validate a graph comprehensively
    pub fn validate<S>(graph: &Graph<S>) -> CoreResult<ValidationReport>
    where
        S: State,
    {
        let mut report = ValidationReport::new();

        // Basic structural validation
        Self::validate_structure(graph, &mut report)?;

        // Connectivity validation
        Self::validate_connectivity(graph, &mut report)?;

        // Reachability validation
        Self::validate_reachability(graph, &mut report)?;

        // Cycle detection
        Self::detect_cycles(graph, &mut report)?;

        // Resource validation
        Self::validate_resources(graph, &mut report)?;

        // Determine overall validity
        report.is_valid = report.errors.is_empty();

        Ok(report)
    }

    /// Validate basic graph structure
    fn validate_structure<S>(graph: &Graph<S>, report: &mut ValidationReport) -> CoreResult<()>
    where
        S: State,
    {
        // Check if graph has nodes
        if graph.nodes.count() == 0 {
            report.add_error("Graph has no nodes".to_string());
        }

        // Check if graph has entry points
        if graph.entry_points.is_empty() {
            report.add_error("Graph has no entry points".to_string());
        }

        // Validate that entry points exist
        for entry_point in &graph.entry_points {
            if !graph.nodes.contains(entry_point) {
                report.add_error(format!("Entry point '{}' does not exist", entry_point));
            }
        }

        // Validate that exit points exist
        for exit_point in &graph.exit_points {
            if !graph.nodes.contains(exit_point) {
                report.add_error(format!("Exit point '{}' does not exist", exit_point));
            }
        }

        // Validate edges reference existing nodes
        for edge in graph.edges.get_all_edges() {
            if !graph.nodes.contains(&edge.from) {
                report.add_error(format!("Edge '{}' references non-existent source node '{}'", edge.id, edge.from));
            }
            if !graph.nodes.contains(&edge.to) {
                report.add_error(format!("Edge '{}' references non-existent target node '{}'", edge.id, edge.to));
            }
        }

        Ok(())
    }

    /// Validate graph connectivity
    fn validate_connectivity<S>(graph: &Graph<S>, report: &mut ValidationReport) -> CoreResult<()>
    where
        S: State,
    {
        // Find isolated nodes (nodes with no incoming or outgoing edges)
        let mut isolated_nodes = Vec::new();
        
        for node_id in graph.nodes.node_ids() {
            let has_incoming = graph.edges.has_incoming_edges(&node_id);
            let has_outgoing = graph.edges.has_outgoing_edges(&node_id);
            let is_entry_point = graph.entry_points.contains(&node_id);
            let is_exit_point = graph.exit_points.contains(&node_id);

            if !has_incoming && !has_outgoing && !is_entry_point && !is_exit_point {
                isolated_nodes.push(node_id);
            }
        }

        if !isolated_nodes.is_empty() {
            report.add_warning(format!("Found {} isolated nodes: {:?}", isolated_nodes.len(), isolated_nodes));
        }

        // Check for nodes with no outgoing edges (potential dead ends)
        let mut dead_end_nodes = Vec::new();
        for node_id in graph.nodes.node_ids() {
            if !graph.edges.has_outgoing_edges(&node_id) && !graph.exit_points.contains(&node_id) {
                dead_end_nodes.push(node_id);
            }
        }

        if !dead_end_nodes.is_empty() {
            report.add_warning(format!("Found {} potential dead-end nodes: {:?}", dead_end_nodes.len(), dead_end_nodes));
        }

        Ok(())
    }

    /// Validate reachability from entry points
    fn validate_reachability<S>(graph: &Graph<S>, report: &mut ValidationReport) -> CoreResult<()>
    where
        S: State,
    {
        let reachable_nodes = Self::find_reachable_nodes(graph);
        let all_nodes: HashSet<_> = graph.nodes.node_ids().into_iter().collect();
        let unreachable_nodes: Vec<_> = all_nodes.difference(&reachable_nodes).cloned().collect();

        if !unreachable_nodes.is_empty() {
            report.add_warning(format!("Found {} unreachable nodes: {:?}", unreachable_nodes.len(), unreachable_nodes));
        }

        // Check if all exit points are reachable
        for exit_point in &graph.exit_points {
            if !reachable_nodes.contains(exit_point) {
                report.add_error(format!("Exit point '{}' is not reachable from entry points", exit_point));
            }
        }

        report.reachable_nodes = reachable_nodes.len();
        report.unreachable_nodes = unreachable_nodes.len();

        Ok(())
    }

    /// Detect cycles in the graph
    fn detect_cycles<S>(graph: &Graph<S>, report: &mut ValidationReport) -> CoreResult<()>
    where
        S: State,
    {
        let cycles = Self::find_cycles(graph);
        
        if !cycles.is_empty() {
            report.add_warning(format!("Found {} cycles in the graph", cycles.len()));
            for (i, cycle) in cycles.iter().enumerate() {
                report.add_info(format!("Cycle {}: {:?}", i + 1, cycle));
            }
        }

        report.cycle_count = cycles.len();

        Ok(())
    }

    /// Validate resource requirements
    fn validate_resources<S>(graph: &Graph<S>, report: &mut ValidationReport) -> CoreResult<()>
    where
        S: State,
    {
        let mut total_memory_mb = 0u64;
        let mut total_cpu_cores = 0.0f32;
        let mut nodes_requiring_network = 0;
        let mut nodes_requiring_filesystem = 0;

        for node_id in graph.nodes.node_ids() {
            if let Some(metadata) = graph.nodes.get_metadata(&node_id) {
                let req = &metadata.resource_requirements;
                
                if let Some(memory) = req.memory_mb {
                    total_memory_mb += memory;
                }
                
                if let Some(cpu) = req.cpu_cores {
                    total_cpu_cores += cpu;
                }
                
                if req.network_access {
                    nodes_requiring_network += 1;
                }
                
                if req.filesystem_access {
                    nodes_requiring_filesystem += 1;
                }
            }
        }

        report.add_info(format!("Total estimated memory requirement: {} MB", total_memory_mb));
        report.add_info(format!("Total estimated CPU requirement: {:.1} cores", total_cpu_cores));
        
        if nodes_requiring_network > 0 {
            report.add_info(format!("{} nodes require network access", nodes_requiring_network));
        }
        
        if nodes_requiring_filesystem > 0 {
            report.add_info(format!("{} nodes require filesystem access", nodes_requiring_filesystem));
        }

        // Warn about high resource usage
        if total_memory_mb > 8192 {
            report.add_warning(format!("High memory usage detected: {} MB", total_memory_mb));
        }
        
        if total_cpu_cores > 16.0 {
            report.add_warning(format!("High CPU usage detected: {:.1} cores", total_cpu_cores));
        }

        Ok(())
    }

    /// Find all nodes reachable from entry points
    fn find_reachable_nodes<S>(graph: &Graph<S>) -> HashSet<NodeId>
    where
        S: State,
    {
        let mut reachable = HashSet::new();
        let mut to_visit = VecDeque::new();

        // Start with entry points
        for entry_point in &graph.entry_points {
            to_visit.push_back(entry_point.clone());
            reachable.insert(entry_point.clone());
        }

        // BFS to find all reachable nodes
        while let Some(current) = to_visit.pop_front() {
            let outgoing_edges = graph.edges.get_outgoing_edges(&current);
            
            for edge in outgoing_edges {
                if !reachable.contains(&edge.to) {
                    reachable.insert(edge.to.clone());
                    to_visit.push_back(edge.to.clone());
                }
            }
        }

        reachable
    }

    /// Find cycles in the graph using DFS
    fn find_cycles<S>(graph: &Graph<S>) -> Vec<Vec<NodeId>>
    where
        S: State,
    {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for node_id in graph.nodes.node_ids() {
            if !visited.contains(&node_id) {
                Self::dfs_cycles(graph, &node_id, &mut visited, &mut rec_stack, &mut path, &mut cycles);
            }
        }

        cycles
    }

    /// DFS helper for cycle detection
    fn dfs_cycles<S>(
        graph: &Graph<S>,
        node_id: &NodeId,
        visited: &mut HashSet<NodeId>,
        rec_stack: &mut HashSet<NodeId>,
        path: &mut Vec<NodeId>,
        cycles: &mut Vec<Vec<NodeId>>,
    ) where
        S: State,
    {
        visited.insert(node_id.clone());
        rec_stack.insert(node_id.clone());
        path.push(node_id.clone());

        let outgoing_edges = graph.edges.get_outgoing_edges(node_id);
        for edge in outgoing_edges {
            if !visited.contains(&edge.to) {
                Self::dfs_cycles(graph, &edge.to, visited, rec_stack, path, cycles);
            } else if rec_stack.contains(&edge.to) {
                // Found a cycle
                if let Some(cycle_start) = path.iter().position(|n| n == &edge.to) {
                    let cycle = path[cycle_start..].to_vec();
                    cycles.push(cycle);
                }
            }
        }

        rec_stack.remove(node_id);
        path.pop();
    }
}

/// Validation report containing analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Whether the graph is valid
    pub is_valid: bool,
    /// Validation errors (critical issues)
    pub errors: Vec<String>,
    /// Validation warnings (potential issues)
    pub warnings: Vec<String>,
    /// Informational messages
    pub info: Vec<String>,
    /// Number of reachable nodes
    pub reachable_nodes: usize,
    /// Number of unreachable nodes
    pub unreachable_nodes: usize,
    /// Number of cycles detected
    pub cycle_count: usize,
    /// Validation timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ValidationReport {
    /// Create a new validation report
    pub fn new() -> Self {
        Self {
            is_valid: false,
            errors: Vec::new(),
            warnings: Vec::new(),
            info: Vec::new(),
            reachable_nodes: 0,
            unreachable_nodes: 0,
            cycle_count: 0,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Add an error to the report
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    /// Add a warning to the report
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    /// Add an info message to the report
    pub fn add_info(&mut self, info: String) {
        self.info.push(info);
    }

    /// Check if the report has any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Check if the report has any warnings
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Get a summary of the validation
    pub fn summary(&self) -> String {
        format!(
            "Validation: {} | Errors: {} | Warnings: {} | Reachable: {}/{} nodes | Cycles: {}",
            if self.is_valid { "VALID" } else { "INVALID" },
            self.errors.len(),
            self.warnings.len(),
            self.reachable_nodes,
            self.reachable_nodes + self.unreachable_nodes,
            self.cycle_count
        )
    }
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::GraphBuilder;
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
    impl crate::node::Node<TestState> for TestNode {
        async fn execute(&self, _state: &mut TestState) -> CoreResult<crate::node::NodeOutput> {
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
    fn test_valid_graph_validation() {
        let initial_state = TestState { value: 0 };
        
        let node1 = TestNode {
            id: "node1".to_string(),
            metadata: NodeMetadata::new("Node 1".to_string()),
        };

        let node2 = TestNode {
            id: "node2".to_string(),
            metadata: NodeMetadata::new("Node 2".to_string()),
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

        let report = GraphValidator::validate(&graph).unwrap();
        
        assert!(report.is_valid);
        assert!(report.errors.is_empty());
        assert_eq!(report.reachable_nodes, 2);
        assert_eq!(report.unreachable_nodes, 0);
    }

    #[test]
    fn test_invalid_graph_validation() {
        let initial_state = TestState { value: 0 };
        
        let graph = GraphBuilder::new()
            .with_initial_state(initial_state)
            .build_unchecked()
            .unwrap();

        let report = GraphValidator::validate(&graph).unwrap();
        
        assert!(!report.is_valid);
        assert!(!report.errors.is_empty());
        assert!(report.errors.iter().any(|e| e.contains("no nodes")));
        assert!(report.errors.iter().any(|e| e.contains("no entry points")));
    }
}