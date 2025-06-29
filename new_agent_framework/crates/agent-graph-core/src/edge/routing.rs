//! Edge routing and path selection logic.

use crate::error::{CoreError, CoreResult};
use crate::edge::{Edge, EdgeCondition};
use crate::node::NodeId;
use crate::state::State;
use std::collections::HashMap;
use std::sync::Arc;

/// Router for determining which edges to traverse
#[derive(Debug)]
pub struct EdgeRouter<S>
where
    S: State,
{
    /// Registered edge conditions
    conditions: HashMap<String, Arc<dyn EdgeCondition<S>>>,
    /// Routing strategy
    strategy: RoutingStrategy,
}

impl<S> EdgeRouter<S>
where
    S: State,
{
    /// Create a new edge router
    pub fn new(strategy: RoutingStrategy) -> Self {
        Self {
            conditions: HashMap::new(),
            strategy,
        }
    }

    /// Register an edge condition
    pub fn register_condition(&mut self, name: String, condition: Arc<dyn EdgeCondition<S>>) {
        self.conditions.insert(name, condition);
    }

    /// Evaluate which edges should be traversed
    pub async fn evaluate_edges(
        &self,
        edges: &[Edge],
        state: &S,
    ) -> CoreResult<Vec<EdgeEvaluation>> {
        let mut evaluations = Vec::new();

        for edge in edges {
            let evaluation = self.evaluate_edge(edge, state).await?;
            evaluations.push(evaluation);
        }

        // Apply routing strategy
        self.apply_strategy(evaluations)
    }

    /// Evaluate a single edge
    async fn evaluate_edge(&self, edge: &Edge, state: &S) -> CoreResult<EdgeEvaluation> {
        // Check if edge should be traversed
        let should_traverse = edge.should_traverse(state).await?;
        
        if !should_traverse {
            return Ok(EdgeEvaluation::skip());
        }

        // Evaluate any registered conditions
        if let Some(condition_name) = &edge.condition {
            if let Some(condition) = self.conditions.get(condition_name) {
                let condition_result = condition.evaluate(state).await?;
                if !condition_result {
                    return Ok(EdgeEvaluation::skip());
                }
            }
        }

        // Calculate weight based on edge properties
        let computed_weight = self.calculate_weight(edge, state).await?;
        
        Ok(EdgeEvaluation::with_weight(computed_weight))
    }

    /// Calculate edge weight based on various factors
    async fn calculate_weight(&self, edge: &Edge, _state: &S) -> CoreResult<f64> {
        let mut weight = edge.weight;

        // Adjust weight based on edge frequency hint
        match edge.metadata.frequency_hint {
            crate::edge::EdgeFrequency::VeryFrequent => weight *= 1.5,
            crate::edge::EdgeFrequency::Frequent => weight *= 1.2,
            crate::edge::EdgeFrequency::Normal => {}, // No change
            crate::edge::EdgeFrequency::Rare => weight *= 0.8,
        }

        // Ensure weight is non-negative
        Ok(weight.max(0.0))
    }

    /// Apply routing strategy to edge evaluations
    fn apply_strategy(&self, mut evaluations: Vec<EdgeEvaluation>) -> CoreResult<Vec<EdgeEvaluation>> {
        match self.strategy {
            RoutingStrategy::All => {
                // Return all edges that should be traversed
                Ok(evaluations)
            }
            RoutingStrategy::First => {
                // Return only the first edge that should be traversed
                if let Some(first_idx) = evaluations.iter().position(|e| e.should_traverse) {
                    for (i, eval) in evaluations.iter_mut().enumerate() {
                        if i != first_idx {
                            eval.should_traverse = false;
                        }
                    }
                }
                Ok(evaluations)
            }
            RoutingStrategy::HighestWeight => {
                // Return only the edge with the highest weight
                if let Some(max_idx) = evaluations
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| e.should_traverse)
                    .max_by(|(_, a), (_, b)| a.computed_weight.partial_cmp(&b.computed_weight).unwrap())
                    .map(|(idx, _)| idx)
                {
                    for (i, eval) in evaluations.iter_mut().enumerate() {
                        if i != max_idx {
                            eval.should_traverse = false;
                        }
                    }
                }
                Ok(evaluations)
            }
            RoutingStrategy::WeightedRandom => {
                // Select edges based on weighted random selection
                self.weighted_random_selection(evaluations)
            }
            RoutingStrategy::RoundRobin => {
                // This would require state tracking, simplified for now
                Ok(evaluations)
            }
        }
    }

    /// Perform weighted random selection
    fn weighted_random_selection(&self, mut evaluations: Vec<EdgeEvaluation>) -> CoreResult<Vec<EdgeEvaluation>> {
        use rand::Rng;

        let traversable: Vec<_> = evaluations
            .iter()
            .enumerate()
            .filter(|(_, e)| e.should_traverse)
            .collect();

        if traversable.is_empty() {
            return Ok(evaluations);
        }

        let total_weight: f64 = traversable.iter().map(|(_, e)| e.computed_weight).sum();
        
        if total_weight <= 0.0 {
            return Ok(evaluations);
        }

        let mut rng = rand::thread_rng();
        let random_value = rng.gen::<f64>() * total_weight;
        
        let mut cumulative_weight = 0.0;
        let mut selected_idx = None;
        
        for (idx, eval) in &traversable {
            cumulative_weight += eval.computed_weight;
            if random_value <= cumulative_weight {
                selected_idx = Some(*idx);
                break;
            }
        }

        if let Some(selected) = selected_idx {
            for (i, eval) in evaluations.iter_mut().enumerate() {
                if i != selected {
                    eval.should_traverse = false;
                }
            }
        }

        Ok(evaluations)
    }

    /// Get the next nodes to execute based on edge evaluations
    pub fn get_next_nodes(&self, edges: &[Edge], evaluations: &[EdgeEvaluation]) -> Vec<NodeId> {
        edges
            .iter()
            .zip(evaluations.iter())
            .filter(|(_, eval)| eval.should_traverse)
            .map(|(edge, _)| edge.to.clone())
            .collect()
    }
}

/// Routing strategy for edge selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingStrategy {
    /// Traverse all applicable edges (parallel execution)
    All,
    /// Traverse only the first applicable edge
    First,
    /// Traverse the edge with the highest weight
    HighestWeight,
    /// Weighted random selection
    WeightedRandom,
    /// Round-robin selection
    RoundRobin,
}

impl Default for RoutingStrategy {
    fn default() -> Self {
        Self::All
    }
}

/// Result of edge evaluation
#[derive(Debug, Clone)]
pub struct EdgeEvaluation {
    /// Whether the edge should be traversed
    pub should_traverse: bool,
    /// Computed weight for this traversal
    pub computed_weight: f64,
    /// Any metadata from the evaluation
    pub metadata: HashMap<String, serde_json::Value>,
}

impl EdgeEvaluation {
    /// Create a positive evaluation
    pub fn traverse() -> Self {
        Self {
            should_traverse: true,
            computed_weight: 1.0,
            metadata: HashMap::new(),
        }
    }

    /// Create a negative evaluation
    pub fn skip() -> Self {
        Self {
            should_traverse: false,
            computed_weight: 0.0,
            metadata: HashMap::new(),
        }
    }

    /// Create evaluation with weight
    pub fn with_weight(weight: f64) -> Self {
        Self {
            should_traverse: weight > 0.0,
            computed_weight: weight,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the evaluation
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Path finder for discovering routes through the graph
#[derive(Debug)]
pub struct PathFinder {
    /// Maximum path length to search
    max_depth: usize,
    /// Whether to include cycles in paths
    allow_cycles: bool,
}

impl PathFinder {
    /// Create a new path finder
    pub fn new(max_depth: usize, allow_cycles: bool) -> Self {
        Self {
            max_depth,
            allow_cycles,
        }
    }

    /// Find all possible paths from start to end
    pub fn find_paths(
        &self,
        edges: &[Edge],
        start: &NodeId,
        end: &NodeId,
    ) -> CoreResult<Vec<Path>> {
        let mut paths = Vec::new();
        let mut current_path = Path::new(start.clone());
        let mut visited = std::collections::HashSet::new();
        
        if !self.allow_cycles {
            visited.insert(start.clone());
        }

        self.find_paths_recursive(
            edges,
            start,
            end,
            &mut current_path,
            &mut visited,
            &mut paths,
            0,
        )?;

        Ok(paths)
    }

    /// Recursive path finding implementation
    fn find_paths_recursive(
        &self,
        edges: &[Edge],
        current: &NodeId,
        target: &NodeId,
        current_path: &mut Path,
        visited: &mut std::collections::HashSet<NodeId>,
        paths: &mut Vec<Path>,
        depth: usize,
    ) -> CoreResult<()> {
        if depth >= self.max_depth {
            return Ok(());
        }

        if current == target {
            paths.push(current_path.clone());
            return Ok(());
        }

        // Find outgoing edges from current node
        let outgoing_edges: Vec<_> = edges.iter().filter(|e| &e.from == current).collect();

        for edge in outgoing_edges {
            if !self.allow_cycles && visited.contains(&edge.to) {
                continue;
            }

            // Add to path and visited set
            current_path.add_edge(edge.clone());
            if !self.allow_cycles {
                visited.insert(edge.to.clone());
            }

            // Recurse
            self.find_paths_recursive(
                edges,
                &edge.to,
                target,
                current_path,
                visited,
                paths,
                depth + 1,
            )?;

            // Backtrack
            current_path.remove_last_edge();
            if !self.allow_cycles {
                visited.remove(&edge.to);
            }
        }

        Ok(())
    }

    /// Find the shortest path between two nodes
    pub fn find_shortest_path(
        &self,
        edges: &[Edge],
        start: &NodeId,
        end: &NodeId,
    ) -> CoreResult<Option<Path>> {
        let paths = self.find_paths(edges, start, end)?;
        Ok(paths.into_iter().min_by_key(|p| p.length()))
    }
}

/// Represents a path through the graph
#[derive(Debug, Clone)]
pub struct Path {
    /// Starting node
    pub start: NodeId,
    /// Edges in the path
    pub edges: Vec<Edge>,
    /// Total weight of the path
    pub total_weight: f64,
}

impl Path {
    /// Create a new path starting at the given node
    pub fn new(start: NodeId) -> Self {
        Self {
            start,
            edges: Vec::new(),
            total_weight: 0.0,
        }
    }

    /// Add an edge to the path
    pub fn add_edge(&mut self, edge: Edge) {
        self.total_weight += edge.weight;
        self.edges.push(edge);
    }

    /// Remove the last edge from the path
    pub fn remove_last_edge(&mut self) {
        if let Some(edge) = self.edges.pop() {
            self.total_weight -= edge.weight;
        }
    }

    /// Get the length of the path (number of edges)
    pub fn length(&self) -> usize {
        self.edges.len()
    }

    /// Get the end node of the path
    pub fn end(&self) -> Option<&NodeId> {
        self.edges.last().map(|e| &e.to)
    }

    /// Get all nodes in the path
    pub fn nodes(&self) -> Vec<NodeId> {
        let mut nodes = vec![self.start.clone()];
        nodes.extend(self.edges.iter().map(|e| e.to.clone()));
        nodes
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

    fn create_test_edge(id: &str, from: &str, to: &str, weight: f64) -> Edge {
        Edge {
            id: id.to_string(),
            from: from.to_string(),
            to: to.to_string(),
            edge_type: EdgeType::Normal,
            condition: None,
            metadata: EdgeMetadata::default(),
            weight,
        }
    }

    #[tokio::test]
    async fn test_edge_router_all_strategy() {
        let router = EdgeRouter::<TestState>::new(RoutingStrategy::All);
        let state = TestState { value: 42 };
        
        let edges = vec![
            create_test_edge("e1", "n1", "n2", 1.0),
            create_test_edge("e2", "n1", "n3", 2.0),
        ];

        let evaluations = router.evaluate_edges(&edges, &state).await.unwrap();
        
        assert_eq!(evaluations.len(), 2);
        assert!(evaluations[0].should_traverse);
        assert!(evaluations[1].should_traverse);
    }

    #[tokio::test]
    async fn test_edge_router_highest_weight_strategy() {
        let router = EdgeRouter::<TestState>::new(RoutingStrategy::HighestWeight);
        let state = TestState { value: 42 };
        
        let edges = vec![
            create_test_edge("e1", "n1", "n2", 1.0),
            create_test_edge("e2", "n1", "n3", 2.0),
        ];

        let evaluations = router.evaluate_edges(&edges, &state).await.unwrap();
        
        assert_eq!(evaluations.len(), 2);
        assert!(!evaluations[0].should_traverse);
        assert!(evaluations[1].should_traverse);
    }

    #[test]
    fn test_path_finder() {
        let path_finder = PathFinder::new(10, false);
        
        let edges = vec![
            create_test_edge("e1", "n1", "n2", 1.0),
            create_test_edge("e2", "n2", "n3", 1.0),
            create_test_edge("e3", "n1", "n3", 2.0),
        ];

        let paths = path_finder
            .find_paths(&edges, &"n1".to_string(), &"n3".to_string())
            .unwrap();
        
        assert_eq!(paths.len(), 2);
        
        let shortest = path_finder
            .find_shortest_path(&edges, &"n1".to_string(), &"n3".to_string())
            .unwrap()
            .unwrap();
        
        assert_eq!(shortest.length(), 1); // Direct path n1 -> n3
    }
}