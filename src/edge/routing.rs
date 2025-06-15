//! Advanced routing algorithms and utilities.

use crate::edge::{DynamicRouter, Edge, EdgeCondition, EdgeType};
use crate::error::{GraphError, GraphResult};
use crate::node::NodeId;
use crate::state::State;

use std::collections::HashMap;

/// Route resolution result
#[derive(Debug, Clone)]
pub enum RouteResolution {
    /// Single target node
    Single(NodeId),
    /// Multiple target nodes (for parallel execution)
    Multiple(Vec<NodeId>),
    /// No route found (terminal node)
    None,
}

/// Edge resolver for determining next nodes
#[derive(Debug)]
pub struct EdgeResolver<S>
where
    S: State,
{
    conditions: HashMap<String, Box<dyn EdgeCondition<S>>>,
    routers: HashMap<String, Box<dyn DynamicRouter<S>>>,
}

impl<S> EdgeResolver<S>
where
    S: State,
{
    /// Create a new edge resolver
    pub fn new() -> Self {
        Self {
            conditions: HashMap::new(),
            routers: HashMap::new(),
        }
    }

    /// Register an edge condition
    pub fn register_condition<C>(&mut self, condition: C)
    where
        C: EdgeCondition<S> + 'static,
    {
        let id = condition.condition_id();
        self.conditions.insert(id, Box::new(condition));
    }

    /// Register a dynamic router
    pub fn register_router<R>(&mut self, router: R)
    where
        R: DynamicRouter<S> + 'static,
    {
        let id = router.router_id();
        self.routers.insert(id, Box::new(router));
    }

    /// Resolve the next nodes for a given edge and state
    pub async fn resolve_edge(&self, edge: &Edge, state: &S) -> GraphResult<RouteResolution> {
        match &edge.edge_type {
            EdgeType::Simple { target } => Ok(RouteResolution::Single(target.clone())),

            EdgeType::Conditional {
                condition_id,
                true_target,
                false_target,
            } => {
                let condition = self.conditions.get(condition_id).ok_or_else(|| {
                    GraphError::graph_structure(format!(
                        "Condition '{}' not found in resolver",
                        condition_id
                    ))
                })?;

                let result = condition.evaluate(state).await?;
                let target = if result { true_target } else { false_target };
                Ok(RouteResolution::Single(target.clone()))
            }

            EdgeType::Dynamic {
                router_id,
                possible_targets,
            } => {
                let router = self.routers.get(router_id).ok_or_else(|| {
                    GraphError::graph_structure(format!(
                        "Router '{}' not found in resolver",
                        router_id
                    ))
                })?;

                let target = router.route(state, possible_targets).await?;
                Ok(RouteResolution::Single(target))
            }

            EdgeType::Parallel { targets } => {
                if targets.is_empty() {
                    Ok(RouteResolution::None)
                } else {
                    Ok(RouteResolution::Multiple(targets.clone()))
                }
            }

            EdgeType::Weighted { targets } => {
                if targets.is_empty() {
                    return Ok(RouteResolution::None);
                }

                // Weighted random selection
                let total_weight: f64 = targets.iter().map(|(_, weight)| weight).sum();
                if total_weight <= 0.0 {
                    return Err(GraphError::graph_structure(
                        "Total weight must be positive for weighted routing".to_string(),
                    ));
                }

                use rand::Rng;
                let mut rng = rand::thread_rng();
                let random_value = rng.gen::<f64>() * total_weight;

                let mut cumulative_weight = 0.0;
                for (node_id, weight) in targets {
                    cumulative_weight += weight;
                    if random_value <= cumulative_weight {
                        return Ok(RouteResolution::Single(node_id.clone()));
                    }
                }

                // Fallback to last target (shouldn't happen with proper weights)
                Ok(RouteResolution::Single(targets.last().unwrap().0.clone()))
            }
        }
    }
}

impl<S> Default for EdgeResolver<S>
where
    S: State,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Path finding utilities for graph analysis
pub struct PathFinder {
    /// Graph adjacency list
    adjacency: HashMap<NodeId, Vec<NodeId>>,
}

impl PathFinder {
    /// Create a new path finder
    pub fn new() -> Self {
        Self {
            adjacency: HashMap::new(),
        }
    }

    /// Add an edge to the graph
    pub fn add_edge(&mut self, from: NodeId, to: NodeId) {
        self.adjacency.entry(from).or_default().push(to);
    }

    /// Add edges from an Edge definition
    pub fn add_edges_from_edge(&mut self, edge: &Edge) {
        for target in edge.possible_targets() {
            self.add_edge(edge.from.clone(), target.clone());
        }
    }

    /// Find all paths from start to end node
    pub fn find_all_paths(&self, start: &NodeId, end: &NodeId) -> Vec<Vec<NodeId>> {
        let mut paths = Vec::new();
        let mut current_path = Vec::new();
        let mut visited = std::collections::HashSet::new();

        self.dfs_paths(start, end, &mut current_path, &mut visited, &mut paths);
        paths
    }

    /// Find the shortest path from start to end node
    pub fn find_shortest_path(&self, start: &NodeId, end: &NodeId) -> Option<Vec<NodeId>> {
        use std::collections::VecDeque;

        let mut queue = VecDeque::new();
        let mut visited = std::collections::HashSet::new();
        let mut parent: HashMap<NodeId, NodeId> = HashMap::new();

        queue.push_back(start.clone());
        visited.insert(start.clone());

        while let Some(current) = queue.pop_front() {
            if current == *end {
                // Reconstruct path
                let mut path = Vec::new();
                let mut node = end.clone();
                path.push(node.clone());

                while let Some(p) = parent.get(&node) {
                    path.push(p.clone());
                    node = p.clone();
                }

                path.reverse();
                return Some(path);
            }

            if let Some(neighbors) = self.adjacency.get(&current) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        visited.insert(neighbor.clone());
                        parent.insert(neighbor.clone(), current.clone());
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }

        None
    }

    /// Check if there's a path from start to end
    pub fn has_path(&self, start: &NodeId, end: &NodeId) -> bool {
        self.find_shortest_path(start, end).is_some()
    }

    /// Get all reachable nodes from a starting node
    pub fn get_reachable_nodes(&self, start: &NodeId) -> std::collections::HashSet<NodeId> {
        let mut reachable = std::collections::HashSet::new();
        let mut stack = vec![start.clone()];

        while let Some(current) = stack.pop() {
            if reachable.insert(current.clone()) {
                if let Some(neighbors) = self.adjacency.get(&current) {
                    for neighbor in neighbors {
                        if !reachable.contains(neighbor) {
                            stack.push(neighbor.clone());
                        }
                    }
                }
            }
        }

        reachable
    }

    /// Detect cycles in the graph
    pub fn has_cycles(&self) -> bool {
        let mut visited = std::collections::HashSet::new();
        let mut rec_stack = std::collections::HashSet::new();

        for node in self.adjacency.keys() {
            if !visited.contains(node) {
                if self.has_cycle_dfs(node, &mut visited, &mut rec_stack) {
                    return true;
                }
            }
        }

        false
    }

    /// Get strongly connected components
    pub fn get_strongly_connected_components(&self) -> Vec<Vec<NodeId>> {
        // Simplified implementation - in practice, you'd use Tarjan's or Kosaraju's algorithm
        let mut components = Vec::new();
        let mut visited = std::collections::HashSet::new();

        for node in self.adjacency.keys() {
            if !visited.contains(node) {
                let mut component = Vec::new();
                self.dfs_component(node, &mut visited, &mut component);
                if !component.is_empty() {
                    components.push(component);
                }
            }
        }

        components
    }

    // Helper methods
    fn dfs_paths(
        &self,
        current: &NodeId,
        end: &NodeId,
        path: &mut Vec<NodeId>,
        visited: &mut std::collections::HashSet<NodeId>,
        all_paths: &mut Vec<Vec<NodeId>>,
    ) {
        path.push(current.clone());
        visited.insert(current.clone());

        if current == end {
            all_paths.push(path.clone());
        } else if let Some(neighbors) = self.adjacency.get(current) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    self.dfs_paths(neighbor, end, path, visited, all_paths);
                }
            }
        }

        path.pop();
        visited.remove(current);
    }

    fn has_cycle_dfs(
        &self,
        node: &NodeId,
        visited: &mut std::collections::HashSet<NodeId>,
        rec_stack: &mut std::collections::HashSet<NodeId>,
    ) -> bool {
        visited.insert(node.clone());
        rec_stack.insert(node.clone());

        if let Some(neighbors) = self.adjacency.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if self.has_cycle_dfs(neighbor, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(neighbor) {
                    return true;
                }
            }
        }

        rec_stack.remove(node);
        false
    }

    fn dfs_component(
        &self,
        node: &NodeId,
        visited: &mut std::collections::HashSet<NodeId>,
        component: &mut Vec<NodeId>,
    ) {
        visited.insert(node.clone());
        component.push(node.clone());

        if let Some(neighbors) = self.adjacency.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    self.dfs_component(neighbor, visited, component);
                }
            }
        }
    }
}

impl Default for PathFinder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edge::conditions::AlwaysTrue;

    #[derive(Debug, Clone)]
    struct TestState {
        value: i32,
    }



    #[tokio::test]
    async fn test_edge_resolver() {
        let mut resolver = EdgeResolver::new();
        resolver.register_condition(AlwaysTrue);

        let edge = Edge::conditional(
            "node1",
            "always_true".to_string(),
            "node2",
            "node3",
        );

        let state = TestState { value: 42 };
        let result = resolver.resolve_edge(&edge, &state).await.unwrap();

        match result {
            RouteResolution::Single(target) => assert_eq!(target, "node2"),
            _ => panic!("Expected single route resolution"),
        }
    }

    #[test]
    fn test_path_finder() {
        let mut finder = PathFinder::new();
        finder.add_edge("A".to_string(), "B".to_string());
        finder.add_edge("B".to_string(), "C".to_string());
        finder.add_edge("A".to_string(), "C".to_string());

        let shortest = finder.find_shortest_path(&"A".to_string(), &"C".to_string());
        assert!(shortest.is_some());

        let all_paths = finder.find_all_paths(&"A".to_string(), &"C".to_string());
        assert_eq!(all_paths.len(), 2); // A->C and A->B->C

        assert!(finder.has_path(&"A".to_string(), &"C".to_string()));
        assert!(!finder.has_path(&"C".to_string(), &"A".to_string()));
    }

    #[test]
    fn test_cycle_detection() {
        let mut finder = PathFinder::new();
        finder.add_edge("A".to_string(), "B".to_string());
        finder.add_edge("B".to_string(), "C".to_string());
        assert!(!finder.has_cycles());

        finder.add_edge("C".to_string(), "A".to_string());
        assert!(finder.has_cycles());
    }
}
