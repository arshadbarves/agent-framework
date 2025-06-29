//! Specialized node types for common workflow patterns.

use crate::{CoreError, CoreResult, State, Node, NodeId, NodeMetadata, NodeOutput};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Agent node that wraps an AI agent for use in graph workflows
#[derive(Debug)]
pub struct AgentNode<S>
where
    S: State,
{
    /// Agent ID to execute
    agent_id: String,
    /// Task template with placeholders for state values
    task_template: String,
    /// Input mapping from state to task variables
    input_mapping: HashMap<String, String>,
    /// Output mapping from agent response to state
    output_mapping: HashMap<String, String>,
    /// Whether this node supports command-based routing
    supports_routing: bool,
    /// Node metadata
    metadata: NodeMetadata,
    /// State type marker
    _phantom: std::marker::PhantomData<S>,
}

impl<S> AgentNode<S>
where
    S: State,
{
    /// Create a new agent node
    pub fn new(agent_id: String, task_template: String) -> Self {
        let metadata = NodeMetadata {
            name: "Agent Node".to_string(),
            description: Some("AI agent execution node".to_string()),
            version: "1.0.0".to_string(),
            parallel_safe: true,
            expected_duration_ms: Some(5000), // 5 seconds for agent execution
            tags: vec!["agent".to_string(), "ai".to_string()],
            custom_properties: HashMap::new(),
        };

        Self {
            agent_id,
            task_template,
            input_mapping: HashMap::new(),
            output_mapping: HashMap::new(),
            supports_routing: false,
            metadata,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Add input mapping from state field to task variable
    pub fn with_input_mapping(mut self, state_field: String, task_variable: String) -> Self {
        self.input_mapping.insert(state_field, task_variable);
        self
    }

    /// Add output mapping from agent response to state field
    pub fn with_output_mapping(mut self, response_field: String, state_field: String) -> Self {
        self.output_mapping.insert(response_field, state_field);
        self
    }

    /// Enable command-based routing
    pub fn with_routing(mut self) -> Self {
        self.supports_routing = true;
        self
    }

    /// Build task from template and state
    fn build_task(&self, state: &S) -> String {
        let mut task = self.task_template.clone();
        
        // Replace placeholders with state values
        for (state_field, task_variable) in &self.input_mapping {
            if let Some(value) = state.get_value(state_field) {
                let placeholder = format!("{{{}}}", task_variable);
                let value_str = match value {
                    serde_json::Value::String(s) => s,
                    other => other.to_string(),
                };
                task = task.replace(&placeholder, &value_str);
            }
        }
        
        task
    }

    /// Apply output mapping to state
    fn apply_output_mapping(&self, state: &mut S, response: &serde_json::Value) -> CoreResult<()> {
        for (response_field, state_field) in &self.output_mapping {
            if let Some(value) = response.get(response_field) {
                state.set_value(state_field.clone(), value.clone())?;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl<S> Node<S> for AgentNode<S>
where
    S: State + Send + Sync + 'static,
{
    async fn execute(&self, state: &mut S) -> CoreResult<NodeOutput> {
        let task = self.build_task(state);
        
        // TODO: In a real implementation, this would call the agent runtime
        // For now, simulate agent execution
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        let mock_response = serde_json::json!({
            "response": format!("Agent {} completed task: {}", self.agent_id, task),
            "confidence": 0.95,
            "next_action": if self.supports_routing { Some("continue") } else { None }
        });

        // Apply output mapping
        self.apply_output_mapping(state, &mock_response)?;

        Ok(NodeOutput::success_with_data(mock_response))
    }

    fn id(&self) -> &str {
        &self.agent_id
    }

    fn metadata(&self) -> &NodeMetadata {
        &self.metadata
    }
}

/// Tool node that executes a tool as part of a workflow
#[derive(Debug)]
pub struct ToolNode<S>
where
    S: State,
{
    /// Name of the tool to execute
    tool_name: String,
    /// Input mapping from state to tool parameters
    input_mapping: HashMap<String, String>,
    /// Output mapping from tool results to state
    output_mapping: HashMap<String, String>,
    /// Tool configuration
    tool_config: HashMap<String, serde_json::Value>,
    /// Node metadata
    metadata: NodeMetadata,
    /// State type marker
    _phantom: std::marker::PhantomData<S>,
}

impl<S> ToolNode<S>
where
    S: State,
{
    /// Create a new tool node
    pub fn new(tool_name: String) -> Self {
        let metadata = NodeMetadata {
            name: format!("Tool: {}", tool_name),
            description: Some(format!("Tool execution node for {}", tool_name)),
            version: "1.0.0".to_string(),
            parallel_safe: true,
            expected_duration_ms: Some(1000), // 1 second for tool execution
            tags: vec!["tool".to_string(), tool_name.clone()],
            custom_properties: HashMap::new(),
        };

        Self {
            tool_name,
            input_mapping: HashMap::new(),
            output_mapping: HashMap::new(),
            tool_config: HashMap::new(),
            metadata,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Add input mapping from state field to tool parameter
    pub fn with_input_mapping(mut self, state_field: String, tool_parameter: String) -> Self {
        self.input_mapping.insert(state_field, tool_parameter);
        self
    }

    /// Add output mapping from tool result to state field
    pub fn with_output_mapping(mut self, tool_field: String, state_field: String) -> Self {
        self.output_mapping.insert(tool_field, state_field);
        self
    }

    /// Add tool configuration
    pub fn with_config(mut self, key: String, value: serde_json::Value) -> Self {
        self.tool_config.insert(key, value);
        self
    }

    /// Build tool input from state
    fn build_tool_input(&self, state: &S) -> HashMap<String, serde_json::Value> {
        let mut tool_input = HashMap::new();
        
        // Add mapped inputs
        for (state_field, tool_parameter) in &self.input_mapping {
            if let Some(value) = state.get_value(state_field) {
                tool_input.insert(tool_parameter.clone(), value);
            }
        }
        
        // Add configuration
        for (key, value) in &self.tool_config {
            tool_input.insert(key.clone(), value.clone());
        }
        
        tool_input
    }

    /// Apply output mapping to state
    fn apply_output_mapping(&self, state: &mut S, tool_result: &serde_json::Value) -> CoreResult<()> {
        for (tool_field, state_field) in &self.output_mapping {
            if let Some(value) = tool_result.get(tool_field) {
                state.set_value(state_field.clone(), value.clone())?;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl<S> Node<S> for ToolNode<S>
where
    S: State + Send + Sync + 'static,
{
    async fn execute(&self, state: &mut S) -> CoreResult<NodeOutput> {
        let tool_input = self.build_tool_input(state);
        
        // TODO: In a real implementation, this would call the tool registry
        // For now, simulate tool execution
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        
        let mock_result = serde_json::json!({
            "result": format!("Tool {} executed successfully", self.tool_name),
            "input": tool_input,
            "execution_time_ms": 50,
            "success": true
        });

        // Apply output mapping
        self.apply_output_mapping(state, &mock_result)?;

        Ok(NodeOutput::success_with_data(mock_result))
    }

    fn id(&self) -> &str {
        &self.tool_name
    }

    fn metadata(&self) -> &NodeMetadata {
        &self.metadata
    }
}

/// Routing node that can dynamically route to different next nodes
#[derive(Debug)]
pub struct RoutingNode<S>
where
    S: State,
{
    /// Node ID
    node_id: String,
    /// Routing rules: condition -> target_node
    routing_rules: HashMap<String, String>,
    /// Condition evaluator function
    condition_evaluator: Arc<dyn Fn(&S) -> Option<String> + Send + Sync>,
    /// Fallback node if no routing rules match
    fallback_node: Option<String>,
    /// Node metadata
    metadata: NodeMetadata,
    /// State type marker
    _phantom: std::marker::PhantomData<S>,
}

impl<S> RoutingNode<S>
where
    S: State,
{
    /// Create a new routing node
    pub fn new<F>(node_id: String, condition_evaluator: F) -> Self
    where
        F: Fn(&S) -> Option<String> + Send + Sync + 'static,
    {
        let metadata = NodeMetadata {
            name: "Routing Node".to_string(),
            description: Some("Dynamic routing node".to_string()),
            version: "1.0.0".to_string(),
            parallel_safe: false, // Routing nodes should be sequential
            expected_duration_ms: Some(10), // Very fast routing decision
            tags: vec!["routing".to_string(), "control".to_string()],
            custom_properties: HashMap::new(),
        };

        Self {
            node_id,
            routing_rules: HashMap::new(),
            condition_evaluator: Arc::new(condition_evaluator),
            fallback_node: None,
            metadata,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Add a routing rule
    pub fn with_route(mut self, condition: String, target_node: String) -> Self {
        self.routing_rules.insert(condition, target_node);
        self
    }

    /// Set fallback node
    pub fn with_fallback(mut self, fallback_node: String) -> Self {
        self.fallback_node = Some(fallback_node);
        self
    }
}

#[async_trait]
impl<S> Node<S> for RoutingNode<S>
where
    S: State + Send + Sync + 'static,
{
    async fn execute(&self, state: &mut S) -> CoreResult<NodeOutput> {
        // Evaluate condition
        let condition_result = (self.condition_evaluator)(state);
        
        // Find matching route
        let next_node = if let Some(condition) = condition_result {
            self.routing_rules.get(&condition).cloned()
                .or_else(|| self.fallback_node.clone())
        } else {
            self.fallback_node.clone()
        };

        let result = serde_json::json!({
            "routing_decision": condition_result,
            "next_node": next_node,
            "available_routes": self.routing_rules.keys().collect::<Vec<_>>()
        });

        let mut output = NodeOutput::success_with_data(result);
        
        // Set next node if determined
        if let Some(next) = next_node {
            output.next_node = Some(next);
        }

        Ok(output)
    }

    fn id(&self) -> &str {
        &self.node_id
    }

    fn metadata(&self) -> &NodeMetadata {
        &self.metadata
    }
}

/// Conditional node that executes different logic based on state
#[derive(Debug)]
pub struct ConditionalNode<S>
where
    S: State,
{
    /// Node ID
    node_id: String,
    /// Condition evaluator
    condition: Arc<dyn Fn(&S) -> bool + Send + Sync>,
    /// Node to execute if condition is true
    true_node: Option<String>,
    /// Node to execute if condition is false
    false_node: Option<String>,
    /// Node metadata
    metadata: NodeMetadata,
    /// State type marker
    _phantom: std::marker::PhantomData<S>,
}

impl<S> ConditionalNode<S>
where
    S: State,
{
    /// Create a new conditional node
    pub fn new<F>(node_id: String, condition: F) -> Self
    where
        F: Fn(&S) -> bool + Send + Sync + 'static,
    {
        let metadata = NodeMetadata {
            name: "Conditional Node".to_string(),
            description: Some("Conditional execution node".to_string()),
            version: "1.0.0".to_string(),
            parallel_safe: false,
            expected_duration_ms: Some(5),
            tags: vec!["conditional".to_string(), "control".to_string()],
            custom_properties: HashMap::new(),
        };

        Self {
            node_id,
            condition: Arc::new(condition),
            true_node: None,
            false_node: None,
            metadata,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Set the node to execute if condition is true
    pub fn with_true_node(mut self, node_id: String) -> Self {
        self.true_node = Some(node_id);
        self
    }

    /// Set the node to execute if condition is false
    pub fn with_false_node(mut self, node_id: String) -> Self {
        self.false_node = Some(node_id);
        self
    }
}

#[async_trait]
impl<S> Node<S> for ConditionalNode<S>
where
    S: State + Send + Sync + 'static,
{
    async fn execute(&self, state: &mut S) -> CoreResult<NodeOutput> {
        let condition_result = (self.condition)(state);
        
        let next_node = if condition_result {
            self.true_node.clone()
        } else {
            self.false_node.clone()
        };

        let result = serde_json::json!({
            "condition_result": condition_result,
            "next_node": next_node
        });

        let mut output = NodeOutput::success_with_data(result);
        
        if let Some(next) = next_node {
            output.next_node = Some(next);
        }

        Ok(output)
    }

    fn id(&self) -> &str {
        &self.node_id
    }

    fn metadata(&self) -> &NodeMetadata {
        &self.metadata
    }
}