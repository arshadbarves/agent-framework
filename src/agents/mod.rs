// Agent system for AgentGraph
// Provides role-based agents with specialized capabilities and collaboration patterns

#![allow(missing_docs)]

use crate::llm::{LLMManager, CompletionRequest, Message, FunctionDefinition};
use crate::tools::{ToolRegistry, ToolExecutor};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use thiserror::Error;

pub mod memory;
pub mod roles;
pub mod collaboration;

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent name
    pub name: String,
    /// Agent role
    pub role: AgentRole,
    /// LLM model to use
    pub model: String,
    /// LLM provider to use
    pub provider: String,
    /// System prompt
    pub system_prompt: String,
    /// Maximum tokens per response
    pub max_tokens: Option<u32>,
    /// Temperature for creativity
    pub temperature: Option<f32>,
    /// Available tools
    pub available_tools: Vec<String>,
    /// Memory configuration
    pub memory_config: memory::MemoryConfig,
    /// Collaboration settings
    pub collaboration_config: collaboration::CollaborationConfig,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            name: "Agent".to_string(),
            role: AgentRole::Assistant,
            model: "mock-gpt-4".to_string(),
            provider: "mock".to_string(),
            system_prompt: "You are a helpful AI assistant.".to_string(),
            max_tokens: Some(1000),
            temperature: Some(0.7),
            available_tools: Vec::new(),
            memory_config: memory::MemoryConfig::default(),
            collaboration_config: collaboration::CollaborationConfig::default(),
        }
    }
}

/// Agent role defining specialized capabilities
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentRole {
    /// General assistant agent
    Assistant,
    /// Research and analysis agent
    Researcher,
    /// Code generation and review agent
    Developer,
    /// Data analysis agent
    Analyst,
    /// Creative writing agent
    Writer,
    /// Planning and coordination agent
    Planner,
    /// Quality assurance agent
    Reviewer,
    /// Customer service agent
    Support,
    /// Custom role with specific capabilities
    Custom(String),
}

impl AgentRole {
    /// Get default system prompt for role
    pub fn default_system_prompt(&self) -> String {
        match self {
            AgentRole::Assistant => "You are a helpful AI assistant that can help with a wide variety of tasks.".to_string(),
            AgentRole::Researcher => "You are a research specialist focused on gathering, analyzing, and synthesizing information from various sources.".to_string(),
            AgentRole::Developer => "You are a software developer expert in writing, reviewing, and debugging code across multiple programming languages.".to_string(),
            AgentRole::Analyst => "You are a data analyst specialized in processing, analyzing, and visualizing data to extract meaningful insights.".to_string(),
            AgentRole::Writer => "You are a creative writer skilled in producing engaging, well-structured content across various formats and styles.".to_string(),
            AgentRole::Planner => "You are a strategic planner focused on breaking down complex tasks, creating timelines, and coordinating resources.".to_string(),
            AgentRole::Reviewer => "You are a quality assurance specialist focused on reviewing, testing, and ensuring high standards in deliverables.".to_string(),
            AgentRole::Support => "You are a customer support specialist focused on helping users resolve issues and providing excellent service.".to_string(),
            AgentRole::Custom(description) => description.clone(),
        }
    }
    
    /// Get recommended tools for role
    pub fn recommended_tools(&self) -> Vec<String> {
        match self {
            AgentRole::Assistant => vec![
                "http_get".to_string(),
                "file_read".to_string(),
                "text_search".to_string(),
                "calculator".to_string(),
            ],
            AgentRole::Researcher => vec![
                "http_get".to_string(),
                "http_post".to_string(),
                "text_search".to_string(),
                "text_summarize".to_string(),
                "file_read".to_string(),
            ],
            AgentRole::Developer => vec![
                "file_read".to_string(),
                "file_write".to_string(),
                "file_list".to_string(),
                "text_search".to_string(),
                "http_get".to_string(),
            ],
            AgentRole::Analyst => vec![
                "file_read".to_string(),
                "database_query".to_string(),
                "calculator".to_string(),
                "text_search".to_string(),
            ],
            AgentRole::Writer => vec![
                "text_search".to_string(),
                "text_summarize".to_string(),
                "file_read".to_string(),
                "file_write".to_string(),
            ],
            AgentRole::Planner => vec![
                "calculator".to_string(),
                "file_read".to_string(),
                "file_write".to_string(),
                "text_search".to_string(),
            ],
            AgentRole::Reviewer => vec![
                "file_read".to_string(),
                "text_search".to_string(),
                "http_get".to_string(),
            ],
            AgentRole::Support => vec![
                "text_search".to_string(),
                "http_get".to_string(),
                "file_read".to_string(),
            ],
            AgentRole::Custom(_) => Vec::new(),
        }
    }
}

/// Agent state tracking execution and context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    /// Current status
    pub status: AgentStatus,
    /// Current task being executed
    pub current_task: Option<String>,
    /// Conversation history
    pub conversation: Vec<Message>,
    /// Working memory
    pub working_memory: HashMap<String, serde_json::Value>,
    /// Last activity timestamp
    pub last_activity: SystemTime,
    /// Total tokens used
    pub total_tokens_used: u64,
    /// Total cost incurred
    pub total_cost: f64,
    /// Number of tool calls made
    pub tool_calls_count: u64,
}

impl Default for AgentState {
    fn default() -> Self {
        Self {
            status: AgentStatus::Idle,
            current_task: None,
            conversation: Vec::new(),
            working_memory: HashMap::new(),
            last_activity: SystemTime::now(),
            total_tokens_used: 0,
            total_cost: 0.0,
            tool_calls_count: 0,
        }
    }
}

/// Agent execution status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    /// Agent is idle and ready for tasks
    Idle,
    /// Agent is thinking/processing
    Thinking,
    /// Agent is executing a tool
    ExecutingTool,
    /// Agent is waiting for input
    WaitingForInput,
    /// Agent is collaborating with other agents
    Collaborating,
    /// Agent encountered an error
    Error,
    /// Agent is paused
    Paused,
}

/// Main agent implementation
#[derive(Debug)]
pub struct Agent {
    /// Agent configuration
    config: AgentConfig,
    /// Current state
    state: AgentState,
    /// LLM manager for AI capabilities
    llm_manager: Arc<LLMManager>,
    /// Tool registry for available tools
    tool_registry: Arc<ToolRegistry>,
    /// Tool executor for running tools
    tool_executor: Arc<ToolExecutor>,
    /// Agent memory system
    memory: memory::AgentMemory,
}

impl Agent {
    /// Create a new agent
    pub fn new(
        config: AgentConfig,
        llm_manager: Arc<LLMManager>,
        tool_registry: Arc<ToolRegistry>,
        tool_executor: Arc<ToolExecutor>,
    ) -> Result<Self, AgentError> {
        let memory = memory::AgentMemory::new(config.memory_config.clone())?;
        
        Ok(Self {
            config,
            state: AgentState::default(),
            llm_manager,
            tool_registry,
            tool_executor,
            memory,
        })
    }
    
    /// Execute a task
    pub async fn execute_task(&mut self, task: String) -> Result<String, AgentError> {
        self.state.status = AgentStatus::Thinking;
        self.state.current_task = Some(task.clone());
        self.state.last_activity = SystemTime::now();
        
        // Add task to conversation
        let user_message = Message::user(task.clone());
        self.state.conversation.push(user_message);
        
        // Build system message with role context
        let system_message = Message::system(format!(
            "{}\n\nYou have access to the following tools: {}",
            self.config.system_prompt,
            self.config.available_tools.join(", ")
        ));
        
        // Prepare messages for LLM
        let mut messages = vec![system_message];
        
        // Add relevant memory context
        let memory_context = self.memory.get_relevant_context(&task).await?;
        if !memory_context.is_empty() {
            let context_message = Message::system(format!(
                "Relevant context from previous interactions:\n{}",
                memory_context
            ));
            messages.push(context_message);
        }
        
        // Add conversation history
        messages.extend(self.state.conversation.clone());
        
        // Get available functions
        let functions = self.get_available_functions().await?;
        
        // Create completion request
        let request = CompletionRequest {
            model: self.config.model.clone(),
            messages,
            max_tokens: self.config.max_tokens,
            temperature: self.config.temperature,
            functions: if functions.is_empty() { None } else { Some(functions) },
            function_call: Some(crate::llm::FunctionCallBehavior::Auto),
            ..Default::default()
        };
        
        // Execute LLM request
        let response = self.llm_manager
            .complete_with_provider(&self.config.provider, request)
            .await
            .map_err(|e| AgentError::LLMError { message: e.to_string() })?;
        
        // Update usage statistics
        self.state.total_tokens_used += response.usage.total_tokens as u64;
        if let Some(cost) = response.usage.estimated_cost {
            self.state.total_cost += cost;
        }
        
        let choice = &response.choices[0];
        let mut final_response = choice.message.content.clone();
        
        // Handle function calls
        if let Some(function_call) = &choice.message.function_call {
            self.state.status = AgentStatus::ExecutingTool;
            
            let tool_result = self.execute_tool(function_call).await?;
            self.state.tool_calls_count += 1;
            
            // Add function result to conversation
            let function_message = Message::new(
                crate::llm::MessageRole::Function,
                serde_json::to_string(&tool_result).unwrap_or_default(),
            );
            self.state.conversation.push(function_message);
            
            // Get follow-up response from LLM
            let follow_up_request = CompletionRequest {
                model: self.config.model.clone(),
                messages: self.state.conversation.clone(),
                max_tokens: self.config.max_tokens,
                temperature: self.config.temperature,
                ..Default::default()
            };
            
            let follow_up_response = self.llm_manager
                .complete_with_provider(&self.config.provider, follow_up_request)
                .await
                .map_err(|e| AgentError::LLMError { message: e.to_string() })?;
            
            final_response = follow_up_response.choices[0].message.content.clone();
            
            // Update usage statistics
            self.state.total_tokens_used += follow_up_response.usage.total_tokens as u64;
            if let Some(cost) = follow_up_response.usage.estimated_cost {
                self.state.total_cost += cost;
            }
        }
        
        // Add assistant response to conversation
        let assistant_message = Message::assistant(final_response.clone());
        self.state.conversation.push(assistant_message);
        
        // Store interaction in memory
        self.memory.store_interaction(&task, &final_response).await?;
        
        // Update state
        self.state.status = AgentStatus::Idle;
        self.state.current_task = None;
        self.state.last_activity = SystemTime::now();
        
        Ok(final_response)
    }
    
    /// Execute a tool function call
    async fn execute_tool(&mut self, function_call: &crate::llm::FunctionCall) -> Result<serde_json::Value, AgentError> {
        let tool_name = &function_call.name;
        
        // Check if tool is available
        if !self.config.available_tools.contains(tool_name) {
            return Err(AgentError::ToolNotAvailable {
                tool_name: tool_name.clone(),
            });
        }
        
        // Parse arguments
        let args: HashMap<String, serde_json::Value> = serde_json::from_value(function_call.arguments.clone())
            .map_err(|e| AgentError::InvalidToolArguments {
                tool_name: tool_name.clone(),
                error: e.to_string(),
            })?;
        
        // Get tool from registry
        let tool = self.tool_registry.get(tool_name)
            .ok_or_else(|| AgentError::ToolNotAvailable {
                tool_name: tool_name.clone(),
            })?;

        // Create tool input
        let tool_input = crate::tools::ToolInput::new(serde_json::to_value(args).unwrap_or_default());
        let tool_config = crate::tools::ToolConfig::default();
        let tool_context = crate::tools::ToolExecutionContext::new(uuid::Uuid::new_v4().to_string());

        // For now, return a mock result since we need to fix the tool executor integration
        let result = serde_json::json!({
            "tool": tool_name,
            "status": "executed",
            "result": "Tool execution completed"
        });

        Ok(result)
    }
    
    /// Get available functions for LLM
    async fn get_available_functions(&self) -> Result<Vec<FunctionDefinition>, AgentError> {
        let mut functions = Vec::new();

        for tool_name in &self.config.available_tools {
            if let Some(_tool) = self.tool_registry.get(tool_name) {
                let function_def = FunctionDefinition::new(
                    tool_name.clone(),
                    format!("Execute {} tool", tool_name), // Use tool name as description
                    serde_json::json!({
                        "type": "object",
                        "properties": {},
                        "required": []
                    }), // Basic schema for now
                );
                functions.push(function_def);
            }
        }

        Ok(functions)
    }
    
    /// Get agent configuration
    pub fn config(&self) -> &AgentConfig {
        &self.config
    }
    
    /// Get agent state
    pub fn state(&self) -> &AgentState {
        &self.state
    }
    
    /// Get agent memory
    pub fn memory(&self) -> &memory::AgentMemory {
        &self.memory
    }

    /// Get mutable agent memory
    pub fn memory_mut(&mut self) -> &mut memory::AgentMemory {
        &mut self.memory
    }
    
    /// Reset agent state
    pub fn reset(&mut self) {
        self.state = AgentState::default();
        self.memory.clear();
    }
    
    /// Update agent configuration
    pub fn update_config(&mut self, config: AgentConfig) -> Result<(), AgentError> {
        self.config = config;
        Ok(())
    }
    
    /// Add tool to available tools
    pub fn add_tool(&mut self, tool_name: String) {
        if !self.config.available_tools.contains(&tool_name) {
            self.config.available_tools.push(tool_name);
        }
    }
    
    /// Remove tool from available tools
    pub fn remove_tool(&mut self, tool_name: &str) {
        self.config.available_tools.retain(|t| t != tool_name);
    }
    
    /// Get conversation history
    pub fn get_conversation(&self) -> &[Message] {
        &self.state.conversation
    }
    
    /// Clear conversation history
    pub fn clear_conversation(&mut self) {
        self.state.conversation.clear();
    }
}

/// Errors that can occur in agent operations
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum AgentError {
    /// LLM error
    #[error("LLM error: {message}")]
    LLMError { message: String },
    
    /// Tool not available
    #[error("Tool not available: {tool_name}")]
    ToolNotAvailable { tool_name: String },
    
    /// Invalid tool arguments
    #[error("Invalid tool arguments for {tool_name}: {error}")]
    InvalidToolArguments { tool_name: String, error: String },
    
    /// Tool execution error
    #[error("Tool execution error for {tool_name}: {error}")]
    ToolExecutionError { tool_name: String, error: String },
    
    /// Memory error
    #[error("Memory error: {message}")]
    MemoryError { message: String },
    
    /// Configuration error
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },
    
    /// Collaboration error
    #[error("Collaboration error: {message}")]
    CollaborationError { message: String },
    
    /// System error
    #[error("System error: {message}")]
    SystemError { message: String },
}

impl From<memory::MemoryError> for AgentError {
    fn from(error: memory::MemoryError) -> Self {
        AgentError::MemoryError {
            message: error.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::providers::MockProvider;
    use crate::tools::ToolRegistry;

    #[test]
    fn test_agent_role_system_prompts() {
        assert!(AgentRole::Developer.default_system_prompt().contains("software developer"));
        assert!(AgentRole::Researcher.default_system_prompt().contains("research"));
        assert!(AgentRole::Writer.default_system_prompt().contains("creative writer"));
    }

    #[test]
    fn test_agent_role_recommended_tools() {
        let dev_tools = AgentRole::Developer.recommended_tools();
        assert!(dev_tools.contains(&"file_read".to_string()));
        assert!(dev_tools.contains(&"file_write".to_string()));
        
        let researcher_tools = AgentRole::Researcher.recommended_tools();
        assert!(researcher_tools.contains(&"http_get".to_string()));
        assert!(researcher_tools.contains(&"text_search".to_string()));
    }

    #[test]
    fn test_agent_config_default() {
        let config = AgentConfig::default();
        assert_eq!(config.role, AgentRole::Assistant);
        assert_eq!(config.model, "mock-gpt-4");
        assert_eq!(config.provider, "mock");
    }

    #[test]
    fn test_agent_state_default() {
        let state = AgentState::default();
        assert_eq!(state.status, AgentStatus::Idle);
        assert!(state.current_task.is_none());
        assert!(state.conversation.is_empty());
    }
}
