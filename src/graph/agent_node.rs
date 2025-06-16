//! Agent nodes for integrating AI agents into graph workflows
//! This module bridges the gap between the graph workflow system and the AI agent system

use crate::agents::Agent;
use crate::error::{GraphError, GraphResult};
use crate::graph::command::{Command, CommandParser, CommandContext};
use crate::node::{Node, NodeMetadata};
use crate::state::State;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Agent node that wraps an AI agent for use in graph workflows
#[derive(Debug)]
pub struct AgentNode {
    /// The AI agent to execute
    agent: Arc<Mutex<Agent>>,
    /// Task template with placeholders for state values
    task_template: String,
    /// Input mapping from state to task variables
    input_mapping: HashMap<String, String>,
    /// Output mapping from agent response to state
    output_mapping: HashMap<String, String>,
    /// Command parser for routing decisions
    command_parser: CommandParser,
    /// Whether this node supports command-based routing
    supports_routing: bool,
    /// Node metadata
    metadata: NodeMetadata,
}

impl AgentNode {
    /// Create a new agent node
    pub fn new(agent: Agent, task_template: String) -> Self {
        let metadata = NodeMetadata::new("AgentNode")
            .with_description("AI agent execution node")
            .with_tag("agent")
            .with_parallel_safe(true);

        Self {
            agent: Arc::new(Mutex::new(agent)),
            task_template,
            input_mapping: HashMap::new(),
            output_mapping: HashMap::new(),
            command_parser: CommandParser::new(),
            supports_routing: false,
            metadata,
        }
    }

    /// Create a new agent node with command-based routing support
    pub fn new_with_routing(agent: Agent, task_template: String) -> Self {
        let metadata = NodeMetadata::new("RoutingAgentNode")
            .with_description("AI agent execution node with command routing")
            .with_tag("agent")
            .with_tag("routing")
            .with_parallel_safe(false); // Routing nodes should be sequential

        Self {
            agent: Arc::new(Mutex::new(agent)),
            task_template,
            input_mapping: HashMap::new(),
            output_mapping: HashMap::new(),
            command_parser: CommandParser::new(),
            supports_routing: true,
            metadata,
        }
    }

    /// Create agent node with input/output mapping
    pub fn with_mapping(
        agent: Agent,
        task_template: String,
        input_mapping: HashMap<String, String>,
        output_mapping: HashMap<String, String>,
    ) -> Self {
        let metadata = NodeMetadata::new("AgentNode")
            .with_description("AI agent execution node with mapping")
            .with_tag("agent")
            .with_parallel_safe(true);

        Self {
            agent: Arc::new(Mutex::new(agent)),
            task_template,
            input_mapping,
            output_mapping,
            metadata,
        }
    }

    /// Set input mapping for state variables
    pub fn with_input_mapping(mut self, mapping: HashMap<String, String>) -> Self {
        self.input_mapping = mapping;
        self
    }

    /// Set output mapping for agent response
    pub fn with_output_mapping(mut self, mapping: HashMap<String, String>) -> Self {
        self.output_mapping = mapping;
        self
    }

    /// Add input mapping
    pub fn map_input(mut self, state_key: String, template_var: String) -> Self {
        self.input_mapping.insert(state_key, template_var);
        self
    }

    /// Add output mapping
    pub fn map_output(mut self, response_key: String, state_key: String) -> Self {
        self.output_mapping.insert(response_key, state_key);
        self
    }

    /// Build task from template and state
    fn build_task<S: State>(&self, state: &S) -> GraphResult<String> {
        let mut task = self.task_template.clone();
        
        // Replace placeholders with state values
        for (state_key, template_var) in &self.input_mapping {
            if let Some(value) = state.get_value(state_key) {
                let placeholder = format!("{{{}}}", template_var);
                let value_str = match value {
                    serde_json::Value::String(s) => s,
                    other => other.to_string(),
                };
                task = task.replace(&placeholder, &value_str);
            }
        }

        // Handle simple placeholder replacement for common patterns
        if task.contains("{input}") {
            if let Some(input) = state.get_value("input") {
                let input_str = match input {
                    serde_json::Value::String(s) => s,
                    other => other.to_string(),
                };
                task = task.replace("{input}", &input_str);
            }
        }

        if task.contains("{query}") {
            if let Some(query) = state.get_value("query") {
                let query_str = match query {
                    serde_json::Value::String(s) => s,
                    other => other.to_string(),
                };
                task = task.replace("{query}", &query_str);
            }
        }

        Ok(task)
    }

    /// Update state with agent response
    fn update_state<S: State>(&self, state: &mut S, response: &str) -> GraphResult<()> {
        // Default output mapping
        if self.output_mapping.is_empty() {
            state.set_value("output", serde_json::Value::String(response.to_string()))?;
            return Ok(());
        }

        // Custom output mapping
        for (response_key, state_key) in &self.output_mapping {
            match response_key.as_str() {
                "response" | "output" => {
                    state.set_value(state_key, serde_json::Value::String(response.to_string()))?;
                }
                "length" => {
                    state.set_value(state_key, serde_json::Value::Number(serde_json::Number::from(response.len())))?;
                }
                "word_count" => {
                    let word_count = response.split_whitespace().count();
                    state.set_value(state_key, serde_json::Value::Number(serde_json::Number::from(word_count)))?;
                }
                _ => {
                    // Try to parse as JSON for structured responses
                    if let Ok(json_response) = serde_json::from_str::<serde_json::Value>(response) {
                        if let Some(value) = json_response.get(response_key) {
                            state.set_value(state_key, value.clone())?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Execute agent and return command for routing
    pub async fn invoke_with_command<S: State>(&self, state: &mut S, context: &CommandContext) -> GraphResult<Command> {
        tracing::info!("Executing agent node with command routing: {}", self.task_template);

        // Build task from template and state
        let task = self.build_task(state)?;

        tracing::debug!("Built task: {}", task);

        // Execute agent task
        let mut agent = self.agent.lock().await;
        let response = agent.execute_task(task).await
            .map_err(|e| GraphError::node_error(
                "agent_node".to_string(),
                format!("Agent execution failed: {}", e),
                Some(Box::new(e)),
            ))?;

        tracing::info!("Agent response received: {} characters", response.len());

        // Parse command from response if routing is supported
        let command = if self.supports_routing {
            let cmd = self.command_parser.parse_command(&response)?;
            context.validate_command(&cmd)?;
            cmd
        } else {
            Command::continue_()
        };

        // Update state with response (unless it's an END command)
        if !command.is_end() {
            self.update_state(state, &response)?;
        }

        // Apply any state updates from the command
        for (key, value) in command.state_updates() {
            state.set_value(key, value.clone())?;
        }

        Ok(command)
    }

    /// Check if this node supports command-based routing
    pub fn supports_routing(&self) -> bool {
        self.supports_routing
    }

    /// Enable or disable routing support
    pub fn set_routing_support(mut self, enabled: bool) -> Self {
        self.supports_routing = enabled;
        if enabled {
            self.metadata = self.metadata.with_tag("routing");
        }
        self
    }

    /// Get agent statistics
    pub async fn get_agent_stats(&self) -> AgentStats {
        let agent = self.agent.lock().await;
        let state = agent.state();

        AgentStats {
            total_tokens_used: state.total_tokens_used,
            tasks_completed: 0, // TODO: Add task counter to AgentState
            average_response_time_ms: 0.0, // TODO: Add timing to AgentState
            memory_entries: agent.memory().get_stats().total_entries,
        }
    }
}

/// Statistics for agent node performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStats {
    pub total_tokens_used: u64,
    pub tasks_completed: u64,
    pub average_response_time_ms: f64,
    pub memory_entries: usize,
}

#[async_trait]
impl<S> Node<S> for AgentNode
where
    S: State + Send + Sync,
{
    async fn invoke(&self, state: &mut S) -> GraphResult<()> {
        tracing::info!("Executing agent node with task template: {}", self.task_template);

        // Build task from template and state
        let task = self.build_task(state)?;
        
        tracing::debug!("Built task: {}", task);

        // Execute agent task
        let mut agent = self.agent.lock().await;
        let response = agent.execute_task(task).await
            .map_err(|e| GraphError::node_error(
                "agent_node".to_string(),
                format!("Agent execution failed: {}", e),
                Some(Box::new(e)),
            ))?;

        tracing::info!("Agent response received: {} characters", response.len());

        // Update state with response
        self.update_state(state, &response)?;

        Ok(())
    }

    fn metadata(&self) -> NodeMetadata {
        self.metadata.clone()
    }
}

/// Builder for creating agent nodes with fluent API
pub struct AgentNodeBuilder {
    agent: Option<Agent>,
    task_template: Option<String>,
    input_mapping: HashMap<String, String>,
    output_mapping: HashMap<String, String>,
}

impl AgentNodeBuilder {
    /// Create a new agent node builder
    pub fn new() -> Self {
        Self {
            agent: None,
            task_template: None,
            input_mapping: HashMap::new(),
            output_mapping: HashMap::new(),
        }
    }

    /// Set the agent
    pub fn with_agent(mut self, agent: Agent) -> Self {
        self.agent = Some(agent);
        self
    }

    /// Set the task template
    pub fn with_task_template<S: Into<String>>(mut self, template: S) -> Self {
        self.task_template = Some(template.into());
        self
    }

    /// Add input mapping
    pub fn map_input<K: Into<String>, V: Into<String>>(mut self, state_key: K, template_var: V) -> Self {
        self.input_mapping.insert(state_key.into(), template_var.into());
        self
    }

    /// Add output mapping
    pub fn map_output<K: Into<String>, V: Into<String>>(mut self, response_key: K, state_key: V) -> Self {
        self.output_mapping.insert(response_key.into(), state_key.into());
        self
    }

    /// Build the agent node
    pub fn build(self) -> GraphResult<AgentNode> {
        let agent = self.agent.ok_or_else(|| {
            GraphError::validation_error("Agent is required for AgentNode".to_string())
        })?;

        let task_template = self.task_template.ok_or_else(|| {
            GraphError::validation_error("Task template is required for AgentNode".to_string())
        })?;

        Ok(AgentNode::with_mapping(
            agent,
            task_template,
            self.input_mapping,
            self.output_mapping,
        ))
    }
}

impl Default for AgentNodeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::roles::RoleTemplates;
    use crate::llm::{LLMManager, LLMConfig, providers::MockProvider};
    use crate::tools::{ToolRegistry, ToolExecutor};
    use serde_json::json;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestState {
        input: String,
        output: String,
        metadata: HashMap<String, serde_json::Value>,
    }

    impl State for TestState {
        fn get_value(&self, key: &str) -> Option<serde_json::Value> {
            match key {
                "input" => Some(json!(self.input)),
                "output" => Some(json!(self.output)),
                _ => self.metadata.get(key).cloned(),
            }
        }

        fn set_value(&mut self, key: &str, value: serde_json::Value) -> GraphResult<()> {
            match key {
                "input" => {
                    if let serde_json::Value::String(s) = value {
                        self.input = s;
                    }
                }
                "output" => {
                    if let serde_json::Value::String(s) = value {
                        self.output = s;
                    }
                }
                _ => {
                    self.metadata.insert(key.to_string(), value);
                }
            }
            Ok(())
        }
    }

    async fn create_test_agent() -> Agent {
        let llm_config = LLMConfig::default();
        let mut llm_manager = LLMManager::new(llm_config);
        let mock_provider = MockProvider::new();
        llm_manager.register_provider("mock".to_string(), Arc::new(mock_provider));
        
        let tool_registry = Arc::new(ToolRegistry::new());
        let tool_executor = Arc::new(ToolExecutor::new());
        
        let template = RoleTemplates::software_developer();
        let config = template.to_agent_config("TestAgent".to_string(), "mock".to_string());
        
        Agent::new(config, Arc::new(llm_manager), tool_registry, tool_executor).unwrap()
    }

    #[tokio::test]
    async fn test_agent_node_creation() {
        let agent = create_test_agent().await;
        let agent_node = AgentNode::new(agent, "Process this input: {input}".to_string());
        
        assert_eq!(agent_node.task_template, "Process this input: {input}");
        assert!(agent_node.input_mapping.is_empty());
        assert!(agent_node.output_mapping.is_empty());
    }

    #[tokio::test]
    async fn test_agent_node_execution() {
        let agent = create_test_agent().await;
        let agent_node = AgentNode::new(agent, "Process this input: {input}".to_string());
        
        let mut state = TestState {
            input: "Hello, world!".to_string(),
            output: String::new(),
            metadata: HashMap::new(),
        };
        
        agent_node.invoke(&mut state).await.unwrap();
        
        assert!(!state.output.is_empty());
    }

    #[tokio::test]
    async fn test_agent_node_builder() {
        let agent = create_test_agent().await;
        
        let agent_node = AgentNodeBuilder::new()
            .with_agent(agent)
            .with_task_template("Analyze: {query}")
            .map_input("user_query", "query")
            .map_output("response", "analysis_result")
            .build()
            .unwrap();
        
        assert_eq!(agent_node.task_template, "Analyze: {query}");
        assert_eq!(agent_node.input_mapping.get("user_query"), Some(&"query".to_string()));
        assert_eq!(agent_node.output_mapping.get("response"), Some(&"analysis_result".to_string()));
    }
}
