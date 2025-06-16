//! Dynamic agent handoff and routing system
//! This module implements LangGraph-style dynamic agent routing for AgentGraph

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

/// Routing agent node that can dynamically hand off to other agents
#[derive(Debug)]
pub struct RoutingAgentNode {
    /// The primary AI agent
    agent: Arc<Mutex<Agent>>,
    /// Task template for the agent
    task_template: String,
    /// Routing rules: condition -> target_node
    routing_rules: HashMap<String, String>,
    /// Command parser for extracting routing decisions
    command_parser: CommandParser,
    /// Fallback node if no routing rules match
    fallback_node: Option<String>,
    /// Node metadata
    metadata: NodeMetadata,
}

impl RoutingAgentNode {
    /// Create a new routing agent node
    pub fn new(agent: Agent, task_template: String) -> Self {
        let metadata = NodeMetadata::new("RoutingAgentNode")
            .with_description("AI agent with dynamic routing capabilities")
            .with_tag("agent")
            .with_tag("routing")
            .with_parallel_safe(false); // Routing nodes should be sequential

        Self {
            agent: Arc::new(Mutex::new(agent)),
            task_template,
            routing_rules: HashMap::new(),
            command_parser: CommandParser::new(),
            fallback_node: None,
            metadata,
        }
    }

    /// Add a routing rule
    pub fn add_routing_rule<K: Into<String>, V: Into<String>>(mut self, condition: K, target_node: V) -> Self {
        self.routing_rules.insert(condition.into(), target_node.into());
        self
    }

    /// Set multiple routing rules
    pub fn with_routing_rules(mut self, rules: HashMap<String, String>) -> Self {
        self.routing_rules = rules;
        self
    }

    /// Set fallback node for when no rules match
    pub fn with_fallback<S: Into<String>>(mut self, fallback_node: S) -> Self {
        self.fallback_node = Some(fallback_node.into());
        self
    }

    /// Execute agent and determine routing
    pub async fn execute_with_routing<S: State>(
        &self, 
        state: &mut S, 
        context: &CommandContext
    ) -> GraphResult<Command> {
        tracing::info!("Executing routing agent node");

        // Build task from template
        let task = self.build_task(state)?;
        
        // Execute agent
        let mut agent = self.agent.lock().await;
        let response = agent.execute_task(task).await
            .map_err(|e| GraphError::node_error(
                "routing_agent_node".to_string(),
                format!("Agent execution failed: {}", e),
                Some(Box::new(e)),
            ))?;

        tracing::info!("Agent response: {} characters", response.len());

        // Analyze response for routing decisions
        let command = self.analyze_routing_decision(&response, context)?;

        // Update state with response if not ending
        if !command.is_end() {
            self.update_state(state, &response)?;
        }

        Ok(command)
    }

    /// Build task from template and state
    fn build_task<S: State>(&self, state: &S) -> GraphResult<String> {
        let mut task = self.task_template.clone();
        
        // Replace common placeholders
        if let Some(input) = state.get_value("input") {
            let input_str = match input {
                serde_json::Value::String(s) => s,
                other => other.to_string(),
            };
            task = task.replace("{input}", &input_str);
        }

        if let Some(context) = state.get_value("context") {
            let context_str = match context {
                serde_json::Value::String(s) => s,
                other => other.to_string(),
            };
            task = task.replace("{context}", &context_str);
        }

        Ok(task)
    }

    /// Update state with agent response
    fn update_state<S: State>(&self, state: &mut S, response: &str) -> GraphResult<()> {
        state.set_value("agent_response", serde_json::Value::String(response.to_string()))?;
        state.set_value("last_agent", serde_json::Value::String("routing_agent".to_string()))?;
        Ok(())
    }

    /// Analyze agent response for routing decisions
    fn analyze_routing_decision(&self, response: &str, context: &CommandContext) -> GraphResult<Command> {
        // First, try to parse explicit commands
        let explicit_command = self.command_parser.parse_command(response)?;
        if !explicit_command.is_continue() {
            context.validate_command(&explicit_command)?;
            return Ok(explicit_command);
        }

        // Check routing rules based on response content
        for (condition, target_node) in &self.routing_rules {
            if self.matches_condition(response, condition) {
                tracing::info!("Routing condition '{}' matched, going to '{}'", condition, target_node);
                return Ok(Command::goto(target_node.clone()));
            }
        }

        // Use fallback node if available
        if let Some(fallback) = &self.fallback_node {
            tracing::info!("No routing rules matched, using fallback node: {}", fallback);
            return Ok(Command::goto(fallback.clone()));
        }

        // Default to continue
        Ok(Command::continue_())
    }

    /// Check if response matches a routing condition
    fn matches_condition(&self, response: &str, condition: &str) -> bool {
        let response_lower = response.to_lowercase();
        let condition_lower = condition.to_lowercase();

        // Simple keyword matching
        if response_lower.contains(&condition_lower) {
            return true;
        }

        // Pattern matching for common conditions
        match condition_lower.as_str() {
            "needs_review" => {
                response_lower.contains("review") || 
                response_lower.contains("check") || 
                response_lower.contains("verify")
            }
            "needs_approval" => {
                response_lower.contains("approve") || 
                response_lower.contains("authorization") || 
                response_lower.contains("permission")
            }
            "error" | "failed" => {
                response_lower.contains("error") || 
                response_lower.contains("failed") || 
                response_lower.contains("problem")
            }
            "complete" | "done" => {
                response_lower.contains("complete") || 
                response_lower.contains("done") || 
                response_lower.contains("finished")
            }
            "escalate" => {
                response_lower.contains("escalate") || 
                response_lower.contains("supervisor") || 
                response_lower.contains("manager")
            }
            _ => false,
        }
    }

    /// Get routing information
    pub fn routing_info(&self) -> RoutingInfo {
        RoutingInfo {
            rules: self.routing_rules.clone(),
            fallback_node: self.fallback_node.clone(),
            supports_commands: true,
        }
    }
}

#[async_trait]
impl<S> Node<S> for RoutingAgentNode
where
    S: State + Send + Sync,
{
    async fn invoke(&self, state: &mut S) -> GraphResult<()> {
        // For standard Node interface, just execute without routing
        let task = self.build_task(state)?;
        
        let mut agent = self.agent.lock().await;
        let response = agent.execute_task(task).await
            .map_err(|e| GraphError::node_error(
                "routing_agent_node".to_string(),
                format!("Agent execution failed: {}", e),
                Some(Box::new(e)),
            ))?;

        self.update_state(state, &response)?;
        Ok(())
    }

    fn metadata(&self) -> NodeMetadata {
        self.metadata.clone()
    }
}

/// Routing information for debugging and monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingInfo {
    /// Routing rules
    pub rules: HashMap<String, String>,
    /// Fallback node
    pub fallback_node: Option<String>,
    /// Whether the node supports command parsing
    pub supports_commands: bool,
}

/// Multi-agent coordinator that manages handoffs between multiple agents
#[derive(Debug)]
pub struct MultiAgentCoordinator {
    /// Available agents by role
    agents: HashMap<String, Arc<Mutex<Agent>>>,
    /// Current active agent
    current_agent: Option<String>,
    /// Handoff rules: from_agent -> conditions -> to_agent
    handoff_rules: HashMap<String, HashMap<String, String>>,
    /// Execution history
    execution_history: Vec<AgentExecution>,
}

/// Record of agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExecution {
    /// Agent role/name
    pub agent: String,
    /// Task executed
    pub task: String,
    /// Response generated
    pub response: String,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Next agent (if handed off)
    pub next_agent: Option<String>,
}

impl MultiAgentCoordinator {
    /// Create a new multi-agent coordinator
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            current_agent: None,
            handoff_rules: HashMap::new(),
            execution_history: Vec::new(),
        }
    }

    /// Add an agent to the coordinator
    pub fn add_agent<S: Into<String>>(mut self, role: S, agent: Agent) -> Self {
        self.agents.insert(role.into(), Arc::new(Mutex::new(agent)));
        self
    }

    /// Set the initial agent
    pub fn with_initial_agent<S: Into<String>>(mut self, agent_role: S) -> Self {
        self.current_agent = Some(agent_role.into());
        self
    }

    /// Add handoff rule
    pub fn add_handoff_rule<S: Into<String>>(
        mut self, 
        from_agent: S, 
        condition: S, 
        to_agent: S
    ) -> Self {
        let from = from_agent.into();
        let rules = self.handoff_rules.entry(from).or_insert_with(HashMap::new);
        rules.insert(condition.into(), to_agent.into());
        self
    }

    /// Execute task with current agent and handle handoffs
    pub async fn execute_task<S: State>(&mut self, task: String, state: &mut S) -> GraphResult<String> {
        let agent_role = self.current_agent.clone().ok_or_else(|| {
            GraphError::validation_error("No current agent set".to_string())
        })?;

        let agent = self.agents.get(&agent_role).ok_or_else(|| {
            GraphError::validation_error(format!("Agent '{}' not found", agent_role))
        })?.clone();

        // Execute task
        let mut agent_guard = agent.lock().await;
        let response = agent_guard.execute_task(task.clone()).await
            .map_err(|e| GraphError::node_error(
                agent_role.clone(),
                format!("Agent execution failed: {}", e),
                Some(Box::new(e)),
            ))?;
        drop(agent_guard);

        // Check for handoff
        let next_agent = self.check_handoff(&agent_role, &response);

        // Record execution
        let execution = AgentExecution {
            agent: agent_role.clone(),
            task,
            response: response.clone(),
            timestamp: chrono::Utc::now(),
            next_agent: next_agent.clone(),
        };
        self.execution_history.push(execution);

        // Update current agent if handoff occurred
        if let Some(next) = next_agent {
            self.current_agent = Some(next);
        }

        Ok(response)
    }

    /// Check if handoff should occur based on response
    fn check_handoff(&self, current_agent: &str, response: &str) -> Option<String> {
        if let Some(rules) = self.handoff_rules.get(current_agent) {
            for (condition, target_agent) in rules {
                if response.to_lowercase().contains(&condition.to_lowercase()) {
                    return Some(target_agent.clone());
                }
            }
        }
        None
    }

    /// Get execution history
    pub fn execution_history(&self) -> &[AgentExecution] {
        &self.execution_history
    }

    /// Get current agent
    pub fn current_agent(&self) -> Option<&str> {
        self.current_agent.as_deref()
    }
}

impl Default for MultiAgentCoordinator {
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

    async fn create_test_agent(name: &str) -> Agent {
        let llm_config = LLMConfig::default();
        let mut llm_manager = LLMManager::new(llm_config);
        let mock_provider = MockProvider::new();
        llm_manager.register_provider("mock".to_string(), Arc::new(mock_provider));
        
        let tool_registry = Arc::new(ToolRegistry::new());
        let tool_executor = Arc::new(ToolExecutor::new());
        
        let template = RoleTemplates::software_developer();
        let config = template.to_agent_config(name.to_string(), "mock".to_string());
        
        Agent::new(config, Arc::new(llm_manager), tool_registry, tool_executor).unwrap()
    }

    #[tokio::test]
    async fn test_routing_agent_node() {
        let agent = create_test_agent("TestAgent").await;
        let routing_node = RoutingAgentNode::new(agent, "Process: {input}".to_string())
            .add_routing_rule("review", "review_node")
            .add_routing_rule("approve", "approval_node")
            .with_fallback("default_node");

        let info = routing_node.routing_info();
        assert_eq!(info.rules.len(), 2);
        assert_eq!(info.fallback_node, Some("default_node".to_string()));
        assert!(info.supports_commands);
    }

    #[tokio::test]
    async fn test_multi_agent_coordinator() {
        let agent1 = create_test_agent("Agent1").await;
        let agent2 = create_test_agent("Agent2").await;

        let mut coordinator = MultiAgentCoordinator::new()
            .add_agent("developer", agent1)
            .add_agent("reviewer", agent2)
            .with_initial_agent("developer")
            .add_handoff_rule("developer", "review", "reviewer");

        assert_eq!(coordinator.current_agent(), Some("developer"));
        assert_eq!(coordinator.execution_history().len(), 0);
    }
}
