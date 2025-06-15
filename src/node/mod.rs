//! Node definitions and traits for the AgentGraph framework.

pub mod traits;

use crate::error::GraphResult;
use crate::state::State;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use uuid::Uuid;

/// Unique identifier for a node
pub type NodeId = String;

/// Metadata associated with a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetadata {
    /// Human-readable name of the node
    pub name: String,
    /// Description of what the node does
    pub description: Option<String>,
    /// Tags for categorizing nodes
    pub tags: Vec<String>,
    /// Custom metadata fields
    pub custom: HashMap<String, serde_json::Value>,
    /// Node version for compatibility tracking
    pub version: String,
    /// Whether this node can be executed in parallel with others
    pub parallel_safe: bool,
    /// Expected execution time in milliseconds (for scheduling)
    pub expected_duration_ms: Option<u64>,
    /// Resource requirements
    pub resource_requirements: ResourceRequirements,
}

/// Resource requirements for a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// Memory requirement in MB
    pub memory_mb: Option<u64>,
    /// CPU cores required
    pub cpu_cores: Option<f32>,
    /// Whether the node requires network access
    pub network_access: bool,
    /// Whether the node requires file system access
    pub filesystem_access: bool,
}

impl Default for NodeMetadata {
    fn default() -> Self {
        Self {
            name: "Unnamed Node".to_string(),
            description: None,
            tags: Vec::new(),
            custom: HashMap::new(),
            version: "1.0.0".to_string(),
            parallel_safe: true,
            expected_duration_ms: None,
            resource_requirements: ResourceRequirements::default(),
        }
    }
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            memory_mb: None,
            cpu_cores: None,
            network_access: false,
            filesystem_access: false,
        }
    }
}

impl NodeMetadata {
    /// Create new metadata with a name
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Set the description
    pub fn with_description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a tag
    pub fn with_tag<S: Into<String>>(mut self, tag: S) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Set parallel safety
    pub fn with_parallel_safe(mut self, parallel_safe: bool) -> Self {
        self.parallel_safe = parallel_safe;
        self
    }

    /// Set expected duration
    pub fn with_expected_duration(mut self, duration_ms: u64) -> Self {
        self.expected_duration_ms = Some(duration_ms);
        self
    }

    /// Set custom metadata
    pub fn with_custom<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Serialize,
    {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.custom.insert(key.into(), json_value);
        }
        self
    }
}

/// Core trait that all nodes must implement
#[async_trait]
pub trait Node<S>: Send + Sync + Debug
where
    S: State,
{
    /// Execute the node with the given state
    async fn invoke(&self, state: &mut S) -> GraphResult<()>;

    /// Get metadata about this node
    fn metadata(&self) -> NodeMetadata {
        NodeMetadata::default()
    }

    /// Validate that the node can execute with the given state
    async fn validate(&self, _state: &S) -> GraphResult<()> {
        Ok(())
    }

    /// Called before the node is executed (setup phase)
    async fn setup(&self) -> GraphResult<()> {
        Ok(())
    }

    /// Called after the node is executed (cleanup phase)
    async fn cleanup(&self) -> GraphResult<()> {
        Ok(())
    }

    /// Check if this node can run in parallel with another node
    fn can_run_parallel_with(&self, _other: &dyn Node<S>) -> bool {
        self.metadata().parallel_safe
    }

    /// Get the unique identifier for this node type
    fn node_type(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// A wrapper for boxed nodes to enable dynamic dispatch
pub type BoxedNode<S> = Box<dyn Node<S>>;

/// Node execution context with timing and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeExecutionContext {
    /// Unique execution ID
    pub execution_id: Uuid,
    /// Node ID being executed
    pub node_id: NodeId,
    /// Start time of execution
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// End time of execution (if completed)
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    /// Execution duration in milliseconds
    pub duration_ms: Option<u64>,
    /// Whether the execution was successful
    pub success: Option<bool>,
    /// Error message if execution failed
    pub error_message: Option<String>,
    /// Custom execution metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl NodeExecutionContext {
    /// Create a new execution context
    pub fn new(node_id: NodeId) -> Self {
        Self {
            execution_id: Uuid::new_v4(),
            node_id,
            start_time: chrono::Utc::now(),
            end_time: None,
            duration_ms: None,
            success: None,
            error_message: None,
            metadata: HashMap::new(),
        }
    }

    /// Mark the execution as completed successfully
    pub fn mark_success(&mut self) {
        let now = chrono::Utc::now();
        self.end_time = Some(now);
        self.duration_ms = Some((now - self.start_time).num_milliseconds() as u64);
        self.success = Some(true);
    }

    /// Mark the execution as failed
    pub fn mark_failure<S: Into<String>>(&mut self, error: S) {
        let now = chrono::Utc::now();
        self.end_time = Some(now);
        self.duration_ms = Some((now - self.start_time).num_milliseconds() as u64);
        self.success = Some(false);
        self.error_message = Some(error.into());
    }

    /// Add custom metadata
    pub fn add_metadata<K, V>(&mut self, key: K, value: V)
    where
        K: Into<String>,
        V: Serialize,
    {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.metadata.insert(key.into(), json_value);
        }
    }

    /// Check if the execution is complete
    pub fn is_complete(&self) -> bool {
        self.end_time.is_some()
    }

    /// Check if the execution was successful
    pub fn is_successful(&self) -> bool {
        self.success.unwrap_or(false)
    }
}

/// Registry for managing node types and instances
#[derive(Debug, Default)]
pub struct NodeRegistry<S>
where
    S: State,
{
    nodes: HashMap<NodeId, BoxedNode<S>>,
    metadata: HashMap<NodeId, NodeMetadata>,
}

impl<S> NodeRegistry<S>
where
    S: State,
{
    /// Create a new node registry
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Register a node with the given ID
    pub fn register<N>(&mut self, id: NodeId, node: N) -> GraphResult<()>
    where
        N: Node<S> + 'static,
    {
        if self.nodes.contains_key(&id) {
            return Err(crate::error::GraphError::graph_structure(format!(
                "Node with ID '{}' already exists",
                id
            )));
        }

        let metadata = node.metadata();
        self.nodes.insert(id.clone(), Box::new(node));
        self.metadata.insert(id, metadata);
        Ok(())
    }

    /// Get a node by ID
    pub fn get(&self, id: &NodeId) -> Option<&BoxedNode<S>> {
        self.nodes.get(id)
    }

    /// Get node metadata by ID
    pub fn get_metadata(&self, id: &NodeId) -> Option<&NodeMetadata> {
        self.metadata.get(id)
    }

    /// List all registered node IDs
    pub fn list_nodes(&self) -> Vec<&NodeId> {
        self.nodes.keys().collect()
    }

    /// Remove a node from the registry
    pub fn unregister(&mut self, id: &NodeId) -> Option<BoxedNode<S>> {
        self.metadata.remove(id);
        self.nodes.remove(id)
    }

    /// Check if a node is registered
    pub fn contains(&self, id: &NodeId) -> bool {
        self.nodes.contains_key(id)
    }

    /// Get the number of registered nodes
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[derive(Debug, Clone)]
    struct TestState {
        value: i32,
    }



    #[derive(Debug)]
    struct TestNode {
        increment: i32,
    }

    #[async_trait]
    impl Node<TestState> for TestNode {
        async fn invoke(&self, state: &mut TestState) -> GraphResult<()> {
            state.value += self.increment;
            Ok(())
        }

        fn metadata(&self) -> NodeMetadata {
            NodeMetadata::new("TestNode")
                .with_description("A simple test node")
                .with_tag("test")
        }
    }

    #[tokio::test]
    async fn test_node_execution() {
        let node = TestNode { increment: 5 };
        let mut state = TestState { value: 10 };

        node.invoke(&mut state).await.unwrap();
        assert_eq!(state.value, 15);
    }

    #[test]
    fn test_node_registry() {
        let mut registry = NodeRegistry::new();
        let node = TestNode { increment: 1 };

        registry.register("test_node".to_string(), node).unwrap();
        assert!(registry.contains(&"test_node".to_string()));
        assert_eq!(registry.len(), 1);

        let metadata = registry.get_metadata(&"test_node".to_string()).unwrap();
        assert_eq!(metadata.name, "TestNode");
    }

    #[test]
    fn test_execution_context() {
        let mut context = NodeExecutionContext::new("test".to_string());
        assert!(!context.is_complete());

        context.mark_success();
        assert!(context.is_complete());
        assert!(context.is_successful());
    }
}
