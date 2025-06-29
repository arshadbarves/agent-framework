//! Agent builder for fluent configuration.

use crate::{CoreError, CoreResult, State};
use crate::roles::AgentRole;
use crate::memory::MemoryConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent builder for fluent configuration
#[derive(Debug)]
pub struct AgentBuilder<S>
where
    S: State,
{
    config: AgentConfig,
    _phantom: std::marker::PhantomData<S>,
}

impl<S> AgentBuilder<S>
where
    S: State + Send + Sync + 'static,
{
    /// Create a new agent builder
    pub fn new(name: String) -> Self {
        let mut config = AgentConfig::default();
        config.name = name;
        
        Self {
            config,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Set agent role
    pub fn with_role(mut self, role: AgentRole) -> Self {
        self.config.role = role;
        self
    }

    /// Set agent description
    pub fn with_description(mut self, description: String) -> Self {
        self.config.description = Some(description);
        self
    }

    /// Add capability
    pub fn with_capability(mut self, capability: String) -> Self {
        self.config.capabilities.push(capability);
        self
    }

    /// Set memory configuration
    pub fn with_memory_config(mut self, memory_config: MemoryConfig) -> Self {
        self.config.memory_config = memory_config;
        self
    }

    /// Add parameter
    pub fn with_parameter(mut self, key: String, value: serde_json::Value) -> Self {
        self.config.parameters.insert(key, value);
        self
    }

    /// Build the agent
    pub fn build(self) -> CoreResult<super::Agent<S>> {
        super::Agent::new(self.config)
    }
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Unique agent identifier
    pub id: String,
    /// Agent name
    pub name: String,
    /// Agent description
    pub description: Option<String>,
    /// Agent role
    pub role: AgentRole,
    /// Memory configuration
    pub memory_config: MemoryConfig,
    /// Agent capabilities
    pub capabilities: Vec<String>,
    /// Custom parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Agent".to_string(),
            description: None,
            role: AgentRole::Assistant,
            memory_config: MemoryConfig::default(),
            capabilities: Vec::new(),
            parameters: HashMap::new(),
        }
    }
}