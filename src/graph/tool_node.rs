//! Tool nodes for integrating tools into graph workflows
//! This module provides LangGraph-style tool integration for AgentGraph

use crate::error::{GraphError, GraphResult};
use crate::node::{Node, NodeMetadata};
use crate::state::State;
use crate::tools::{ToolRegistry, ToolExecutor, ToolInput, ToolConfig, ToolExecutionContext};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Tool node that executes a tool as part of a workflow
#[derive(Debug)]
pub struct ToolNode {
    /// Name of the tool to execute
    tool_name: String,
    /// Tool executor for running tools
    tool_executor: Arc<ToolExecutor>,
    /// Tool registry for tool lookup
    tool_registry: Arc<ToolRegistry>,
    /// Input mapping from state to tool parameters
    input_mapping: HashMap<String, String>,
    /// Output mapping from tool results to state
    output_mapping: HashMap<String, String>,
    /// Tool configuration
    tool_config: ToolConfig,
    /// Node metadata
    metadata: NodeMetadata,
}

impl ToolNode {
    /// Create a new tool node
    pub fn new(
        tool_name: String,
        tool_executor: Arc<ToolExecutor>,
        tool_registry: Arc<ToolRegistry>,
    ) -> Self {
        let metadata = NodeMetadata::new("ToolNode")
            .with_description(&format!("Tool execution node for {}", tool_name))
            .with_tag("tool")
            .with_tag(&tool_name)
            .with_parallel_safe(true);

        Self {
            tool_name,
            tool_executor,
            tool_registry,
            input_mapping: HashMap::new(),
            output_mapping: HashMap::new(),
            tool_config: ToolConfig::default(),
            metadata,
        }
    }

    /// Set input mapping for state variables
    pub fn with_input_mapping(mut self, mapping: HashMap<String, String>) -> Self {
        self.input_mapping = mapping;
        self
    }

    /// Set output mapping for tool results
    pub fn with_output_mapping(mut self, mapping: HashMap<String, String>) -> Self {
        self.output_mapping = mapping;
        self
    }

    /// Add input mapping
    pub fn map_input<K: Into<String>, V: Into<String>>(mut self, state_key: K, tool_param: V) -> Self {
        self.input_mapping.insert(state_key.into(), tool_param.into());
        self
    }

    /// Add output mapping
    pub fn map_output<K: Into<String>, V: Into<String>>(mut self, result_key: K, state_key: V) -> Self {
        self.output_mapping.insert(result_key.into(), state_key.into());
        self
    }

    /// Set tool configuration
    pub fn with_config(mut self, config: ToolConfig) -> Self {
        self.tool_config = config;
        self
    }

    /// Build tool input from state
    fn build_tool_input<S: State>(&self, state: &S) -> GraphResult<ToolInput> {
        let mut tool_params = HashMap::new();

        // Map state values to tool parameters
        for (state_key, tool_param) in &self.input_mapping {
            if let Some(value) = state.get_value(state_key) {
                tool_params.insert(tool_param.clone(), value);
            }
        }

        // Handle common parameter patterns
        if tool_params.is_empty() {
            // If no explicit mapping, try common patterns
            if let Some(input) = state.get_value("input") {
                tool_params.insert("input".to_string(), input);
            }
            if let Some(query) = state.get_value("query") {
                tool_params.insert("query".to_string(), query);
            }
            if let Some(text) = state.get_value("text") {
                tool_params.insert("text".to_string(), text);
            }
        }

        Ok(ToolInput::new(serde_json::Value::Object(
            tool_params.into_iter()
                .map(|(k, v)| (k, v))
                .collect()
        )))
    }

    /// Update state with tool results
    fn update_state_with_result<S: State>(&self, state: &mut S, result: serde_json::Value) -> GraphResult<()> {
        // Default output mapping
        if self.output_mapping.is_empty() {
            state.set_value("tool_output", result)?;
            return Ok(());
        }

        // Custom output mapping
        for (result_key, state_key) in &self.output_mapping {
            let value = match result_key.as_str() {
                "result" | "output" => result.clone(),
                "success" => serde_json::Value::Bool(true), // Tool executed successfully
                "tool_name" => serde_json::Value::String(self.tool_name.clone()),
                _ => {
                    // Try to extract from result object
                    if let serde_json::Value::Object(ref obj) = result {
                        obj.get(result_key).cloned().unwrap_or(serde_json::Value::Null)
                    } else {
                        serde_json::Value::Null
                    }
                }
            };
            
            state.set_value(state_key, value)?;
        }

        Ok(())
    }

    /// Get tool information
    pub fn tool_info(&self) -> ToolInfo {
        ToolInfo {
            name: self.tool_name.clone(),
            input_mapping: self.input_mapping.clone(),
            output_mapping: self.output_mapping.clone(),
            config: self.tool_config.clone(),
        }
    }
}

#[async_trait]
impl<S> Node<S> for ToolNode
where
    S: State + Send + Sync,
{
    async fn invoke(&self, state: &mut S) -> GraphResult<()> {
        tracing::info!("Executing tool node: {}", self.tool_name);

        // Check if tool exists
        if !self.tool_registry.has_tool(&self.tool_name) {
            return Err(GraphError::node_error(
                "tool_node".to_string(),
                format!("Tool '{}' not found in registry", self.tool_name),
                None,
            ));
        }

        // Build tool input from state
        let tool_input = self.build_tool_input(state)?;
        
        tracing::debug!("Tool input: {:?}", tool_input);

        // Create execution context
        let context = ToolExecutionContext::new(uuid::Uuid::new_v4().to_string());

        // Execute tool
        let result = self.tool_executor.execute_tool(
            &self.tool_name,
            tool_input,
            &self.tool_config,
            context,
        ).await.map_err(|e| GraphError::node_error(
            "tool_node".to_string(),
            format!("Tool execution failed: {}", e),
            Some(Box::new(e)),
        ))?;

        tracing::info!("Tool execution completed successfully");

        // Update state with results
        self.update_state_with_result(state, result)?;

        Ok(())
    }

    fn metadata(&self) -> NodeMetadata {
        self.metadata.clone()
    }
}

/// Tool information for debugging and monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    /// Tool name
    pub name: String,
    /// Input parameter mapping
    pub input_mapping: HashMap<String, String>,
    /// Output result mapping
    pub output_mapping: HashMap<String, String>,
    /// Tool configuration
    pub config: ToolConfig,
}

/// Builder for creating tool nodes with fluent API
pub struct ToolNodeBuilder {
    tool_name: Option<String>,
    tool_executor: Option<Arc<ToolExecutor>>,
    tool_registry: Option<Arc<ToolRegistry>>,
    input_mapping: HashMap<String, String>,
    output_mapping: HashMap<String, String>,
    tool_config: ToolConfig,
}

impl ToolNodeBuilder {
    /// Create a new tool node builder
    pub fn new() -> Self {
        Self {
            tool_name: None,
            tool_executor: None,
            tool_registry: None,
            input_mapping: HashMap::new(),
            output_mapping: HashMap::new(),
            tool_config: ToolConfig::default(),
        }
    }

    /// Set the tool name
    pub fn with_tool<S: Into<String>>(mut self, tool_name: S) -> Self {
        self.tool_name = Some(tool_name.into());
        self
    }

    /// Set the tool executor
    pub fn with_executor(mut self, executor: Arc<ToolExecutor>) -> Self {
        self.tool_executor = Some(executor);
        self
    }

    /// Set the tool registry
    pub fn with_registry(mut self, registry: Arc<ToolRegistry>) -> Self {
        self.tool_registry = Some(registry);
        self
    }

    /// Add input mapping
    pub fn map_input<K: Into<String>, V: Into<String>>(mut self, state_key: K, tool_param: V) -> Self {
        self.input_mapping.insert(state_key.into(), tool_param.into());
        self
    }

    /// Add output mapping
    pub fn map_output<K: Into<String>, V: Into<String>>(mut self, result_key: K, state_key: V) -> Self {
        self.output_mapping.insert(result_key.into(), state_key.into());
        self
    }

    /// Set tool configuration
    pub fn with_config(mut self, config: ToolConfig) -> Self {
        self.tool_config = config;
        self
    }

    /// Build the tool node
    pub fn build(self) -> GraphResult<ToolNode> {
        let tool_name = self.tool_name.ok_or_else(|| {
            GraphError::validation_error("Tool name is required for ToolNode".to_string())
        })?;

        let tool_executor = self.tool_executor.ok_or_else(|| {
            GraphError::validation_error("Tool executor is required for ToolNode".to_string())
        })?;

        let tool_registry = self.tool_registry.ok_or_else(|| {
            GraphError::validation_error("Tool registry is required for ToolNode".to_string())
        })?;

        Ok(ToolNode {
            tool_name,
            tool_executor,
            tool_registry,
            input_mapping: self.input_mapping,
            output_mapping: self.output_mapping,
            tool_config: self.tool_config,
            metadata: NodeMetadata::new("ToolNode")
                .with_description("Tool execution node")
                .with_tag("tool"),
        })
    }
}

impl Default for ToolNodeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_node_builder() {
        let tool_registry = Arc::new(ToolRegistry::new());
        let tool_executor = Arc::new(ToolExecutor::new());
        
        let tool_node = ToolNodeBuilder::new()
            .with_tool("test_tool")
            .with_executor(tool_executor)
            .with_registry(tool_registry)
            .map_input("query", "search_query")
            .map_output("results", "search_results")
            .build()
            .unwrap();
        
        assert_eq!(tool_node.tool_name, "test_tool");
        assert_eq!(tool_node.input_mapping.get("query"), Some(&"search_query".to_string()));
        assert_eq!(tool_node.output_mapping.get("results"), Some(&"search_results".to_string()));
    }

    #[test]
    fn test_tool_info() {
        let tool_registry = Arc::new(ToolRegistry::new());
        let tool_executor = Arc::new(ToolExecutor::new());
        
        let tool_node = ToolNode::new("test_tool".to_string(), tool_executor, tool_registry)
            .map_input("input", "query")
            .map_output("result", "output");
        
        let info = tool_node.tool_info();
        assert_eq!(info.name, "test_tool");
        assert!(!info.input_mapping.is_empty());
        assert!(!info.output_mapping.is_empty());
    }
}
