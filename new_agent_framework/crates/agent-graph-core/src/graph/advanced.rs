//! Advanced graph features including dynamic modification and optimization.

use crate::{CoreError, CoreResult, State, Node, NodeId, Graph};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

/// Advanced graph operations and algorithms
pub struct GraphAnalyzer;

impl GraphAnalyzer {
    /// Detect cycles in the graph using DFS
    pub fn detect_cycles<S: State>(graph: &Graph<S>) -> CoreResult<Vec<Vec<NodeId>>> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut cycles = Vec::new();
        let mut current_path = Vec::new();

        for node_id in graph.get_node_ids() {
            if !visited.contains(&node_id) {
                Self::dfs_cycle_detection(
                    graph,
                    &node_id,
                    &mut visited,
                    &mut rec_stack,
                    &mut current_path,
                    &mut cycles,
                )?;
            }
        }

        Ok(cycles)
    }

    /// DFS helper for cycle detection
    fn dfs_cycle_detection<S: State>(
        graph: &Graph<S>,
        node_id: &NodeId,
        visited: &mut HashSet<NodeId>,
        rec_stack: &mut HashSet<NodeId>,
        current_path: &mut Vec<NodeId>,
        cycles: &mut Vec<Vec<NodeId>>,
    ) -> CoreResult<()> {
        visited.insert(node_id.clone());
        rec_stack.insert(node_id.clone());
        current_path.push(node_id.clone());

        // Get outgoing edges for this node
        for neighbor in graph.get_outgoing_edges(node_id)? {
            if !visited.contains(&neighbor) {
                Self::dfs_cycle_detection(graph, &neighbor, visited, rec_stack, current_path, cycles)?;
            } else if rec_stack.contains(&neighbor) {
                // Found a cycle
                if let Some(cycle_start) = current_path.iter().position(|id| id == &neighbor) {
                    let cycle = current_path[cycle_start..].to_vec();
                    cycles.push(cycle);
                }
            }
        }

        rec_stack.remove(node_id);
        current_path.pop();
        Ok(())
    }

    /// Perform topological sort of the graph
    pub fn topological_sort<S: State>(graph: &Graph<S>) -> CoreResult<Vec<NodeId>> {
        let mut in_degree = HashMap::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        // Initialize in-degree count
        for node_id in graph.get_node_ids() {
            in_degree.insert(node_id.clone(), 0);
        }

        // Calculate in-degrees
        for node_id in graph.get_node_ids() {
            for neighbor in graph.get_outgoing_edges(&node_id)? {
                *in_degree.entry(neighbor).or_insert(0) += 1;
            }
        }

        // Add nodes with no incoming edges to queue
        for (node_id, degree) in &in_degree {
            if *degree == 0 {
                queue.push_back(node_id.clone());
            }
        }

        // Process nodes
        while let Some(node_id) = queue.pop_front() {
            result.push(node_id.clone());

            // Reduce in-degree of neighbors
            for neighbor in graph.get_outgoing_edges(&node_id)? {
                if let Some(degree) = in_degree.get_mut(&neighbor) {
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        // Check if all nodes were processed (no cycles)
        if result.len() != graph.get_node_ids().len() {
            return Err(CoreError::graph_structure("Graph contains cycles, cannot perform topological sort"));
        }

        Ok(result)
    }

    /// Find strongly connected components using Tarjan's algorithm
    pub fn find_strongly_connected_components<S: State>(graph: &Graph<S>) -> CoreResult<Vec<Vec<NodeId>>> {
        let mut index = 0;
        let mut stack = Vec::new();
        let mut indices = HashMap::new();
        let mut lowlinks = HashMap::new();
        let mut on_stack = HashSet::new();
        let mut components = Vec::new();

        for node_id in graph.get_node_ids() {
            if !indices.contains_key(&node_id) {
                Self::tarjan_scc(
                    graph,
                    &node_id,
                    &mut index,
                    &mut stack,
                    &mut indices,
                    &mut lowlinks,
                    &mut on_stack,
                    &mut components,
                )?;
            }
        }

        Ok(components)
    }

    /// Tarjan's algorithm helper for SCC
    fn tarjan_scc<S: State>(
        graph: &Graph<S>,
        node_id: &NodeId,
        index: &mut usize,
        stack: &mut Vec<NodeId>,
        indices: &mut HashMap<NodeId, usize>,
        lowlinks: &mut HashMap<NodeId, usize>,
        on_stack: &mut HashSet<NodeId>,
        components: &mut Vec<Vec<NodeId>>,
    ) -> CoreResult<()> {
        indices.insert(node_id.clone(), *index);
        lowlinks.insert(node_id.clone(), *index);
        *index += 1;
        stack.push(node_id.clone());
        on_stack.insert(node_id.clone());

        // Consider successors
        for neighbor in graph.get_outgoing_edges(node_id)? {
            if !indices.contains_key(&neighbor) {
                Self::tarjan_scc(graph, &neighbor, index, stack, indices, lowlinks, on_stack, components)?;
                let neighbor_lowlink = lowlinks[&neighbor];
                let current_lowlink = lowlinks.get_mut(node_id).unwrap();
                *current_lowlink = (*current_lowlink).min(neighbor_lowlink);
            } else if on_stack.contains(&neighbor) {
                let neighbor_index = indices[&neighbor];
                let current_lowlink = lowlinks.get_mut(node_id).unwrap();
                *current_lowlink = (*current_lowlink).min(neighbor_index);
            }
        }

        // If node_id is a root node, pop the stack and create an SCC
        if lowlinks[node_id] == indices[node_id] {
            let mut component = Vec::new();
            loop {
                let w = stack.pop().unwrap();
                on_stack.remove(&w);
                component.push(w.clone());
                if w == *node_id {
                    break;
                }
            }
            components.push(component);
        }

        Ok(())
    }

    /// Calculate graph metrics
    pub fn calculate_metrics<S: State>(graph: &Graph<S>) -> CoreResult<GraphMetrics> {
        let node_count = graph.get_node_ids().len();
        let mut edge_count = 0;
        let mut max_in_degree = 0;
        let mut max_out_degree = 0;
        let mut in_degrees = HashMap::new();

        // Count edges and degrees
        for node_id in graph.get_node_ids() {
            let out_edges = graph.get_outgoing_edges(&node_id)?;
            let out_degree = out_edges.len();
            max_out_degree = max_out_degree.max(out_degree);
            edge_count += out_degree;

            for neighbor in out_edges {
                *in_degrees.entry(neighbor).or_insert(0) += 1;
            }
        }

        max_in_degree = in_degrees.values().max().copied().unwrap_or(0);

        // Calculate average degrees
        let avg_in_degree = if node_count > 0 { edge_count as f64 / node_count as f64 } else { 0.0 };
        let avg_out_degree = avg_in_degree; // Same for directed graphs

        // Check if graph is connected (weakly connected for directed graphs)
        let is_connected = Self::is_weakly_connected(graph)?;

        // Calculate diameter (longest shortest path)
        let diameter = Self::calculate_diameter(graph)?;

        Ok(GraphMetrics {
            node_count,
            edge_count,
            max_in_degree,
            max_out_degree,
            avg_in_degree,
            avg_out_degree,
            is_connected,
            diameter,
        })
    }

    /// Check if graph is weakly connected
    fn is_weakly_connected<S: State>(graph: &Graph<S>) -> CoreResult<bool> {
        let node_ids = graph.get_node_ids();
        if node_ids.is_empty() {
            return Ok(true);
        }

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        
        // Start from first node
        let start_node = &node_ids[0];
        queue.push_back(start_node.clone());
        visited.insert(start_node.clone());

        // BFS considering both directions
        while let Some(node_id) = queue.pop_front() {
            // Outgoing edges
            for neighbor in graph.get_outgoing_edges(&node_id)? {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor.clone());
                    queue.push_back(neighbor);
                }
            }

            // Incoming edges (reverse direction)
            for other_node in &node_ids {
                if other_node != &node_id && !visited.contains(other_node) {
                    let outgoing = graph.get_outgoing_edges(other_node)?;
                    if outgoing.contains(&node_id) {
                        visited.insert(other_node.clone());
                        queue.push_back(other_node.clone());
                    }
                }
            }
        }

        Ok(visited.len() == node_ids.len())
    }

    /// Calculate graph diameter
    fn calculate_diameter<S: State>(graph: &Graph<S>) -> CoreResult<Option<usize>> {
        let node_ids = graph.get_node_ids();
        if node_ids.len() < 2 {
            return Ok(None);
        }

        let mut max_distance = 0;

        for start_node in &node_ids {
            let distances = Self::bfs_distances(graph, start_node)?;
            if let Some(max_dist) = distances.values().max() {
                max_distance = max_distance.max(*max_dist);
            }
        }

        Ok(Some(max_distance))
    }

    /// BFS to calculate distances from a source node
    fn bfs_distances<S: State>(graph: &Graph<S>, start: &NodeId) -> CoreResult<HashMap<NodeId, usize>> {
        let mut distances = HashMap::new();
        let mut queue = VecDeque::new();

        distances.insert(start.clone(), 0);
        queue.push_back(start.clone());

        while let Some(node_id) = queue.pop_front() {
            let current_distance = distances[&node_id];

            for neighbor in graph.get_outgoing_edges(&node_id)? {
                if !distances.contains_key(&neighbor) {
                    distances.insert(neighbor.clone(), current_distance + 1);
                    queue.push_back(neighbor);
                }
            }
        }

        Ok(distances)
    }
}

/// Graph metrics and statistics
#[derive(Debug, Clone)]
pub struct GraphMetrics {
    /// Total number of nodes
    pub node_count: usize,
    /// Total number of edges
    pub edge_count: usize,
    /// Maximum in-degree
    pub max_in_degree: usize,
    /// Maximum out-degree
    pub max_out_degree: usize,
    /// Average in-degree
    pub avg_in_degree: f64,
    /// Average out-degree
    pub avg_out_degree: f64,
    /// Whether the graph is connected
    pub is_connected: bool,
    /// Graph diameter (longest shortest path)
    pub diameter: Option<usize>,
}

/// Dynamic graph modifier for runtime changes
pub struct GraphModifier;

impl GraphModifier {
    /// Add a node to the graph at runtime
    pub fn add_node_runtime<S: State>(
        graph: &mut Graph<S>,
        node_id: NodeId,
        node: Box<dyn Node<S>>,
    ) -> CoreResult<()> {
        // Validate that node doesn't already exist
        if graph.has_node(&node_id) {
            return Err(CoreError::graph_structure(format!("Node {} already exists", node_id)));
        }

        graph.add_node(node_id, node)?;
        Ok(())
    }

    /// Remove a node from the graph at runtime
    pub fn remove_node_runtime<S: State>(
        graph: &mut Graph<S>,
        node_id: &NodeId,
    ) -> CoreResult<()> {
        // Check if node exists
        if !graph.has_node(node_id) {
            return Err(CoreError::graph_structure(format!("Node {} does not exist", node_id)));
        }

        // Remove all edges connected to this node
        Self::remove_node_edges(graph, node_id)?;

        // Remove the node itself
        graph.remove_node(node_id)?;
        Ok(())
    }

    /// Add an edge to the graph at runtime
    pub fn add_edge_runtime<S: State>(
        graph: &mut Graph<S>,
        from: NodeId,
        to: NodeId,
    ) -> CoreResult<()> {
        // Validate nodes exist
        if !graph.has_node(&from) {
            return Err(CoreError::graph_structure(format!("Source node {} does not exist", from)));
        }
        if !graph.has_node(&to) {
            return Err(CoreError::graph_structure(format!("Target node {} does not exist", to)));
        }

        graph.add_edge(from, to)?;
        Ok(())
    }

    /// Remove an edge from the graph at runtime
    pub fn remove_edge_runtime<S: State>(
        graph: &mut Graph<S>,
        from: &NodeId,
        to: &NodeId,
    ) -> CoreResult<()> {
        graph.remove_edge(from, to)?;
        Ok(())
    }

    /// Helper to remove all edges connected to a node
    fn remove_node_edges<S: State>(
        graph: &mut Graph<S>,
        node_id: &NodeId,
    ) -> CoreResult<()> {
        // Get all nodes to check for edges
        let all_nodes = graph.get_node_ids();

        // Remove outgoing edges
        let outgoing = graph.get_outgoing_edges(node_id)?;
        for target in outgoing {
            graph.remove_edge(node_id, &target)?;
        }

        // Remove incoming edges
        for source in &all_nodes {
            if source != node_id {
                let outgoing_from_source = graph.get_outgoing_edges(source)?;
                if outgoing_from_source.contains(node_id) {
                    graph.remove_edge(source, node_id)?;
                }
            }
        }

        Ok(())
    }

    /// Optimize graph structure for better performance
    pub fn optimize_graph<S: State>(graph: &mut Graph<S>) -> CoreResult<GraphOptimizationReport> {
        let mut report = GraphOptimizationReport::default();

        // Remove unreachable nodes
        let reachable = Self::find_reachable_nodes(graph)?;
        let all_nodes = graph.get_node_ids();
        let unreachable: Vec<_> = all_nodes.iter()
            .filter(|node| !reachable.contains(*node))
            .cloned()
            .collect();

        for node_id in &unreachable {
            Self::remove_node_runtime(graph, node_id)?;
            report.removed_unreachable_nodes += 1;
        }

        // Remove redundant edges (if any)
        report.removed_redundant_edges = Self::remove_redundant_edges(graph)?;

        // Detect and report cycles
        let cycles = GraphAnalyzer::detect_cycles(graph)?;
        report.cycles_detected = cycles.len();

        Ok(report)
    }

    /// Find all nodes reachable from entry points
    fn find_reachable_nodes<S: State>(graph: &Graph<S>) -> CoreResult<HashSet<NodeId>> {
        let mut reachable = HashSet::new();
        let mut queue = VecDeque::new();

        // Start from entry points
        for entry_point in graph.get_entry_points() {
            if !reachable.contains(&entry_point) {
                queue.push_back(entry_point.clone());
                reachable.insert(entry_point);
            }
        }

        // BFS to find all reachable nodes
        while let Some(node_id) = queue.pop_front() {
            for neighbor in graph.get_outgoing_edges(&node_id)? {
                if !reachable.contains(&neighbor) {
                    reachable.insert(neighbor.clone());
                    queue.push_back(neighbor);
                }
            }
        }

        Ok(reachable)
    }

    /// Remove redundant edges (placeholder - implement based on specific redundancy rules)
    fn remove_redundant_edges<S: State>(_graph: &mut Graph<S>) -> CoreResult<usize> {
        // Placeholder implementation
        // In a real implementation, this would identify and remove redundant edges
        // based on specific criteria (e.g., transitive reduction)
        Ok(0)
    }
}

/// Report from graph optimization
#[derive(Debug, Default)]
pub struct GraphOptimizationReport {
    /// Number of unreachable nodes removed
    pub removed_unreachable_nodes: usize,
    /// Number of redundant edges removed
    pub removed_redundant_edges: usize,
    /// Number of cycles detected
    pub cycles_detected: usize,
}

/// Graph validation utilities
pub struct GraphValidator;

impl GraphValidator {
    /// Comprehensive graph validation
    pub fn validate_graph<S: State>(graph: &Graph<S>) -> CoreResult<ValidationReport> {
        let mut report = ValidationReport::default();

        // Check for cycles
        let cycles = GraphAnalyzer::detect_cycles(graph)?;
        report.has_cycles = !cycles.is_empty();
        report.cycle_count = cycles.len();

        // Check connectivity
        report.is_connected = GraphAnalyzer::is_weakly_connected(graph)?;

        // Check for unreachable nodes
        let reachable = GraphModifier::find_reachable_nodes(graph)?;
        let all_nodes = graph.get_node_ids();
        report.unreachable_nodes = all_nodes.iter()
            .filter(|node| !reachable.contains(*node))
            .cloned()
            .collect();

        // Check for dead ends (nodes with no outgoing edges that aren't finish points)
        let finish_points = graph.get_finish_points();
        report.dead_end_nodes = all_nodes.iter()
            .filter(|node| {
                let outgoing = graph.get_outgoing_edges(node).unwrap_or_default();
                outgoing.is_empty() && !finish_points.contains(*node)
            })
            .cloned()
            .collect();

        // Check for isolated nodes
        report.isolated_nodes = all_nodes.iter()
            .filter(|node| {
                let outgoing = graph.get_outgoing_edges(node).unwrap_or_default();
                let has_incoming = all_nodes.iter().any(|other| {
                    if other == *node { return false; }
                    let other_outgoing = graph.get_outgoing_edges(other).unwrap_or_default();
                    other_outgoing.contains(*node)
                });
                outgoing.is_empty() && !has_incoming
            })
            .cloned()
            .collect();

        report.is_valid = report.unreachable_nodes.is_empty() 
            && report.dead_end_nodes.is_empty() 
            && report.isolated_nodes.is_empty();

        Ok(report)
    }
}

/// Graph validation report
#[derive(Debug, Default)]
pub struct ValidationReport {
    /// Whether the graph is valid
    pub is_valid: bool,
    /// Whether the graph has cycles
    pub has_cycles: bool,
    /// Number of cycles detected
    pub cycle_count: usize,
    /// Whether the graph is connected
    pub is_connected: bool,
    /// Unreachable nodes
    pub unreachable_nodes: Vec<NodeId>,
    /// Dead end nodes (no outgoing edges, not finish points)
    pub dead_end_nodes: Vec<NodeId>,
    /// Isolated nodes (no incoming or outgoing edges)
    pub isolated_nodes: Vec<NodeId>,
}