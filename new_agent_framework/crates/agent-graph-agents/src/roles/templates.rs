//! Agent role templates and definitions.

use serde::{Deserialize, Serialize};

/// Agent role definitions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentRole {
    /// Research and information gathering agent
    Researcher,
    /// Data analysis and insights agent
    Analyst,
    /// Coordination and orchestration agent
    Coordinator,
    /// Task execution agent
    Executor,
    /// Monitoring and observability agent
    Monitor,
    /// General purpose assistant agent
    Assistant,
    /// Custom role with specific capabilities
    Custom(String),
}

impl Default for AgentRole {
    fn default() -> Self {
        Self::Assistant
    }
}

impl AgentRole {
    /// Get the default capabilities for this role
    pub fn default_capabilities(&self) -> Vec<String> {
        match self {
            AgentRole::Researcher => vec![
                "research".to_string(),
                "web_search".to_string(),
                "data_collection".to_string(),
                "analysis".to_string(),
            ],
            AgentRole::Analyst => vec![
                "data_analysis".to_string(),
                "pattern_recognition".to_string(),
                "reporting".to_string(),
                "visualization".to_string(),
            ],
            AgentRole::Coordinator => vec![
                "task_coordination".to_string(),
                "workflow_management".to_string(),
                "resource_allocation".to_string(),
                "communication".to_string(),
            ],
            AgentRole::Executor => vec![
                "task_execution".to_string(),
                "tool_usage".to_string(),
                "action_taking".to_string(),
                "result_validation".to_string(),
            ],
            AgentRole::Monitor => vec![
                "monitoring".to_string(),
                "alerting".to_string(),
                "health_checking".to_string(),
                "performance_tracking".to_string(),
            ],
            AgentRole::Assistant => vec![
                "general_assistance".to_string(),
                "question_answering".to_string(),
                "task_support".to_string(),
                "information_retrieval".to_string(),
            ],
            AgentRole::Custom(_) => vec![],
        }
    }

    /// Get the default system prompt for this role
    pub fn default_system_prompt(&self) -> String {
        match self {
            AgentRole::Researcher => {
                "You are a research agent specialized in gathering, analyzing, and synthesizing information. \
                Your primary goal is to conduct thorough research on given topics, collect relevant data, \
                and provide comprehensive insights. You excel at finding reliable sources, fact-checking, \
                and presenting information in a clear, organized manner.".to_string()
            }
            AgentRole::Analyst => {
                "You are a data analysis agent specialized in examining data, identifying patterns, \
                and generating insights. Your primary goal is to analyze information, create reports, \
                and provide actionable recommendations based on data-driven findings. You excel at \
                statistical analysis, trend identification, and clear communication of complex findings.".to_string()
            }
            AgentRole::Coordinator => {
                "You are a coordination agent specialized in managing workflows, allocating resources, \
                and facilitating communication between different agents and systems. Your primary goal \
                is to ensure smooth operation of multi-agent systems, optimize resource usage, and \
                maintain effective coordination across all components.".to_string()
            }
            AgentRole::Executor => {
                "You are an execution agent specialized in taking concrete actions and implementing tasks. \
                Your primary goal is to execute assigned tasks efficiently, use available tools effectively, \
                and ensure successful completion of objectives. You excel at following instructions, \
                adapting to changing requirements, and delivering results.".to_string()
            }
            AgentRole::Monitor => {
                "You are a monitoring agent specialized in observing system performance, detecting issues, \
                and maintaining system health. Your primary goal is to continuously monitor various metrics, \
                alert on anomalies, and provide insights into system performance and reliability.".to_string()
            }
            AgentRole::Assistant => {
                "You are a helpful assistant agent designed to provide general support and assistance. \
                Your primary goal is to help users with various tasks, answer questions, and provide \
                useful information. You are adaptable, friendly, and focused on being as helpful as possible.".to_string()
            }
            AgentRole::Custom(role_name) => {
                format!("You are a {} agent with specialized capabilities for your specific role.", role_name)
            }
        }
    }

    /// Get the role name as a string
    pub fn name(&self) -> String {
        match self {
            AgentRole::Researcher => "Researcher".to_string(),
            AgentRole::Analyst => "Analyst".to_string(),
            AgentRole::Coordinator => "Coordinator".to_string(),
            AgentRole::Executor => "Executor".to_string(),
            AgentRole::Monitor => "Monitor".to_string(),
            AgentRole::Assistant => "Assistant".to_string(),
            AgentRole::Custom(name) => name.clone(),
        }
    }

    /// Check if this role has a specific capability
    pub fn has_capability(&self, capability: &str) -> bool {
        self.default_capabilities().contains(&capability.to_string())
    }
}

/// Role template for creating agents with predefined configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleTemplate {
    /// Role type
    pub role: AgentRole,
    /// Template name
    pub name: String,
    /// Template description
    pub description: String,
    /// Default capabilities
    pub capabilities: Vec<String>,
    /// Default system prompt
    pub system_prompt: String,
    /// Default memory configuration
    pub memory_config: crate::memory::MemoryConfig,
    /// Template metadata
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl RoleTemplate {
    /// Create a new role template
    pub fn new(role: AgentRole, name: String, description: String) -> Self {
        let capabilities = role.default_capabilities();
        let system_prompt = role.default_system_prompt();
        
        Self {
            role,
            name,
            description,
            capabilities,
            system_prompt,
            memory_config: crate::memory::MemoryConfig::default(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Create a template from an agent role
    pub fn from_role(role: AgentRole) -> Self {
        let name = role.name();
        let description = format!("Default template for {} role", name);
        Self::new(role, name, description)
    }

    /// Add a capability to the template
    pub fn with_capability(mut self, capability: String) -> Self {
        if !self.capabilities.contains(&capability) {
            self.capabilities.push(capability);
        }
        self
    }

    /// Set custom system prompt
    pub fn with_system_prompt(mut self, prompt: String) -> Self {
        self.system_prompt = prompt;
        self
    }

    /// Set memory configuration
    pub fn with_memory_config(mut self, config: crate::memory::MemoryConfig) -> Self {
        self.memory_config = config;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}