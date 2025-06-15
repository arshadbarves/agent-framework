//! Edge definitions and routing logic for the AgentGraph framework.

pub mod routing;

use crate::error::GraphResult;
use crate::node::NodeId;
use crate::state::State;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;

/// Represents different types of edges in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeType {
    /// Simple edge that always routes to the target
    Simple {
        /// Target node ID
        target: NodeId,
    },
    /// Conditional edge that routes based on state
    Conditional {
        /// Condition function identifier
        condition_id: String,
        /// Target node if condition is true
        true_target: NodeId,
        /// Target node if condition is false
        false_target: NodeId,
    },
    /// Dynamic edge that can route to multiple targets
    Dynamic {
        /// Router function identifier
        router_id: String,
        /// Possible target nodes
        possible_targets: Vec<NodeId>,
    },
    /// Parallel edge that routes to multiple targets simultaneously
    Parallel {
        /// Target nodes to execute in parallel
        targets: Vec<NodeId>,
    },
    /// Weighted edge for probabilistic routing
    Weighted {
        /// Weighted targets (node_id, weight)
        targets: Vec<(NodeId, f64)>,
    },
}

/// Edge condition trait for conditional routing
#[async_trait]
pub trait EdgeCondition<S>: Send + Sync + Debug
where
    S: State,
{
    /// Evaluate the condition with the given state
    async fn evaluate(&self, state: &S) -> GraphResult<bool>;

    /// Get a unique identifier for this condition
    fn condition_id(&self) -> String;

    /// Get a human-readable description of the condition
    fn description(&self) -> String {
        format!("Condition: {}", self.condition_id())
    }
}

/// Dynamic router trait for dynamic routing
#[async_trait]
pub trait DynamicRouter<S>: Send + Sync + Debug
where
    S: State,
{
    /// Route to the next node based on state
    async fn route(&self, state: &S, possible_targets: &[NodeId]) -> GraphResult<NodeId>;

    /// Get a unique identifier for this router
    fn router_id(&self) -> String;

    /// Get a human-readable description of the router
    fn description(&self) -> String {
        format!("Router: {}", self.router_id())
    }
}

/// Edge definition with metadata
#[derive(Debug, Clone)]
pub struct Edge {
    /// Source node ID
    pub from: NodeId,
    /// Edge type and routing logic
    pub edge_type: EdgeType,
    /// Edge metadata
    pub metadata: EdgeMetadata,
}

/// Metadata for edges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeMetadata {
    /// Human-readable name
    pub name: Option<String>,
    /// Description of the edge
    pub description: Option<String>,
    /// Tags for categorizing edges
    pub tags: Vec<String>,
    /// Custom metadata
    pub custom: HashMap<String, serde_json::Value>,
    /// Whether this edge can be traversed in parallel
    pub parallel_safe: bool,
    /// Priority for edge selection (higher = more priority)
    pub priority: i32,
}

impl Default for EdgeMetadata {
    fn default() -> Self {
        Self {
            name: None,
            description: None,
            tags: Vec::new(),
            custom: HashMap::new(),
            parallel_safe: true,
            priority: 0,
        }
    }
}

impl Edge {
    /// Create a simple edge
    pub fn simple<F, T>(from: F, to: T) -> Self
    where
        F: Into<NodeId>,
        T: Into<NodeId>,
    {
        Self {
            from: from.into(),
            edge_type: EdgeType::Simple {
                target: to.into(),
            },
            metadata: EdgeMetadata::default(),
        }
    }

    /// Create a conditional edge
    pub fn conditional<F, T, E>(
        from: F,
        condition_id: String,
        true_target: T,
        false_target: E,
    ) -> Self
    where
        F: Into<NodeId>,
        T: Into<NodeId>,
        E: Into<NodeId>,
    {
        Self {
            from: from.into(),
            edge_type: EdgeType::Conditional {
                condition_id,
                true_target: true_target.into(),
                false_target: false_target.into(),
            },
            metadata: EdgeMetadata::default(),
        }
    }

    /// Create a dynamic edge
    pub fn dynamic<F>(from: F, router_id: String, possible_targets: Vec<NodeId>) -> Self
    where
        F: Into<NodeId>,
    {
        Self {
            from: from.into(),
            edge_type: EdgeType::Dynamic {
                router_id,
                possible_targets,
            },
            metadata: EdgeMetadata::default(),
        }
    }

    /// Create a parallel edge
    pub fn parallel<F>(from: F, targets: Vec<NodeId>) -> Self
    where
        F: Into<NodeId>,
    {
        Self {
            from: from.into(),
            edge_type: EdgeType::Parallel { targets },
            metadata: EdgeMetadata::default(),
        }
    }

    /// Create a weighted edge
    pub fn weighted<F>(from: F, targets: Vec<(NodeId, f64)>) -> Self
    where
        F: Into<NodeId>,
    {
        Self {
            from: from.into(),
            edge_type: EdgeType::Weighted { targets },
            metadata: EdgeMetadata::default(),
        }
    }

    /// Set edge metadata
    pub fn with_metadata(mut self, metadata: EdgeMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Set edge name
    pub fn with_name<S: Into<String>>(mut self, name: S) -> Self {
        self.metadata.name = Some(name.into());
        self
    }

    /// Set edge description
    pub fn with_description<S: Into<String>>(mut self, description: S) -> Self {
        self.metadata.description = Some(description.into());
        self
    }

    /// Add a tag
    pub fn with_tag<S: Into<String>>(mut self, tag: S) -> Self {
        self.metadata.tags.push(tag.into());
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.metadata.priority = priority;
        self
    }

    /// Get all possible target nodes for this edge
    pub fn possible_targets(&self) -> Vec<&NodeId> {
        match &self.edge_type {
            EdgeType::Simple { target } => vec![target],
            EdgeType::Conditional {
                true_target,
                false_target,
                ..
            } => vec![true_target, false_target],
            EdgeType::Dynamic {
                possible_targets, ..
            } => possible_targets.iter().collect(),
            EdgeType::Parallel { targets } => targets.iter().collect(),
            EdgeType::Weighted { targets } => targets.iter().map(|(id, _)| id).collect(),
        }
    }

    /// Check if this edge can execute in parallel
    pub fn is_parallel_safe(&self) -> bool {
        self.metadata.parallel_safe
            && matches!(
                self.edge_type,
                EdgeType::Simple { .. }
                    | EdgeType::Conditional { .. }
                    | EdgeType::Dynamic { .. }
                    | EdgeType::Weighted { .. }
            )
    }
}

/// Registry for managing edge conditions and routers
#[derive(Debug, Default)]
pub struct EdgeRegistry<S>
where
    S: State,
{
    conditions: HashMap<String, Box<dyn EdgeCondition<S>>>,
    routers: HashMap<String, Box<dyn DynamicRouter<S>>>,
}

impl<S> EdgeRegistry<S>
where
    S: State,
{
    /// Create a new edge registry
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

    /// Get a condition by ID
    pub fn get_condition(&self, id: &str) -> Option<&Box<dyn EdgeCondition<S>>> {
        self.conditions.get(id)
    }

    /// Get a router by ID
    pub fn get_router(&self, id: &str) -> Option<&Box<dyn DynamicRouter<S>>> {
        self.routers.get(id)
    }

    /// List all condition IDs
    pub fn list_conditions(&self) -> Vec<&String> {
        self.conditions.keys().collect()
    }

    /// List all router IDs
    pub fn list_routers(&self) -> Vec<&String> {
        self.routers.keys().collect()
    }
}

/// Simple condition implementations
pub mod conditions {
    use super::*;

    /// Always true condition
    #[derive(Debug)]
    pub struct AlwaysTrue;

    #[async_trait]
    impl<S: State> EdgeCondition<S> for AlwaysTrue {
        async fn evaluate(&self, _state: &S) -> GraphResult<bool> {
            Ok(true)
        }

        fn condition_id(&self) -> String {
            "always_true".to_string()
        }
    }

    /// Always false condition
    #[derive(Debug)]
    pub struct AlwaysFalse;

    #[async_trait]
    impl<S: State> EdgeCondition<S> for AlwaysFalse {
        async fn evaluate(&self, _state: &S) -> GraphResult<bool> {
            Ok(false)
        }

        fn condition_id(&self) -> String {
            "always_false".to_string()
        }
    }

    /// Function-based condition
    #[derive(Debug)]
    pub struct FunctionCondition<F> {
        func: F,
        id: String,
    }

    impl<F> FunctionCondition<F> {
        /// Create a new function-based condition
        pub fn new<S: Into<String>>(id: S, func: F) -> Self {
            Self {
                func,
                id: id.into(),
            }
        }
    }

    #[async_trait]
    impl<S, F> EdgeCondition<S> for FunctionCondition<F>
    where
        S: State,
        F: Fn(&S) -> bool + Send + Sync + Debug,
    {
        async fn evaluate(&self, state: &S) -> GraphResult<bool> {
            Ok((self.func)(state))
        }

        fn condition_id(&self) -> String {
            self.id.clone()
        }
    }
}

/// Simple router implementations
pub mod routers {
    use super::*;

    /// Round-robin router
    #[derive(Debug)]
    pub struct RoundRobinRouter {
        counter: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    }

    impl RoundRobinRouter {
        /// Create a new round-robin router
        pub fn new() -> Self {
            Self {
                counter: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            }
        }
    }

    #[async_trait]
    impl<S: State> DynamicRouter<S> for RoundRobinRouter {
        async fn route(&self, _state: &S, possible_targets: &[NodeId]) -> GraphResult<NodeId> {
            if possible_targets.is_empty() {
                return Err(crate::error::GraphError::graph_structure(
                    "No possible targets for routing".to_string(),
                ));
            }

            let index = self
                .counter
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                % possible_targets.len();
            Ok(possible_targets[index].clone())
        }

        fn router_id(&self) -> String {
            "round_robin".to_string()
        }
    }

    /// Random router
    #[derive(Debug)]
    pub struct RandomRouter;

    #[async_trait]
    impl<S: State> DynamicRouter<S> for RandomRouter {
        async fn route(&self, _state: &S, possible_targets: &[NodeId]) -> GraphResult<NodeId> {
            if possible_targets.is_empty() {
                return Err(crate::error::GraphError::graph_structure(
                    "No possible targets for routing".to_string(),
                ));
            }

            use rand::seq::SliceRandom;
            let mut rng = rand::thread_rng();
            Ok(possible_targets
                .choose(&mut rng)
                .unwrap()
                .clone())
        }

        fn router_id(&self) -> String {
            "random".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestState {
        value: i32,
    }



    #[test]
    fn test_edge_creation() {
        let edge = Edge::simple("node1", "node2")
            .with_name("test_edge")
            .with_description("A test edge")
            .with_tag("test");

        assert_eq!(edge.from, "node1");
        assert!(matches!(edge.edge_type, EdgeType::Simple { .. }));
        assert_eq!(edge.metadata.name, Some("test_edge".to_string()));
    }

    #[test]
    fn test_edge_targets() {
        let edge = Edge::conditional(
            "node1",
            "test_condition".to_string(),
            "node2",
            "node3",
        );

        let targets = edge.possible_targets();
        assert_eq!(targets.len(), 2);
        assert!(targets.contains(&&"node2".to_string()));
        assert!(targets.contains(&&"node3".to_string()));
    }

    #[tokio::test]
    async fn test_condition() {
        use conditions::*;

        let condition = AlwaysTrue;
        let state = TestState { value: 42 };

        let result = condition.evaluate(&state).await.unwrap();
        assert!(result);
    }
}
