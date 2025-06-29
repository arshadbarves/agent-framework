//! Agent runtime and execution logic.

use crate::{CoreError, CoreResult, State, Node, NodeId};
use crate::agent::AgentConfig;
use crate::roles::AgentRole;
use crate::memory::{MemorySystem, MemoryType, MemoryEntry};
use agent_graph_core::node::{NodeOutput, NodeMetadata, ExecutionMetrics};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, debug, warn, error};

/// Core agent implementation
#[derive(Debug)]
pub struct Agent<S>
where
    S: State,
{
    /// Agent configuration
    config: AgentConfig,
    /// Agent metadata
    metadata: AgentMetadata,
    /// Memory system
    memory: Arc<MemorySystem>,
    /// Agent role
    role: AgentRole,
    /// Agent state
    agent_state: AgentState,
}

impl<S> Agent<S>
where
    S: State + Send + Sync + 'static,
{
    /// Create a new agent with configuration
    pub fn new(config: AgentConfig) -> CoreResult<Self> {
        let metadata = AgentMetadata::from_config(&config);
        let memory = Arc::new(MemorySystem::new(config.memory_config.clone())?);
        let role = config.role.clone();
        
        Ok(Self {
            config,
            metadata,
            memory,
            role,
            agent_state: AgentState::Created,
        })
    }

    /// Get agent configuration
    pub fn config(&self) -> &AgentConfig {
        &self.config
    }

    /// Get agent metadata
    pub fn metadata(&self) -> &AgentMetadata {
        &self.metadata
    }

    /// Get agent role
    pub fn role(&self) -> &AgentRole {
        &self.role
    }

    /// Get memory system
    pub fn memory(&self) -> Arc<MemorySystem> {
        Arc::clone(&self.memory)
    }

    /// Get agent state
    pub fn state(&self) -> &AgentState {
        &self.agent_state
    }

    /// Initialize the agent
    pub async fn initialize(&mut self) -> CoreResult<()> {
        info!("Initializing agent '{}'", self.config.name);
        
        self.agent_state = AgentState::Initializing;
        
        // Initialize memory system
        self.memory.initialize().await?;
        
        // Store initial agent information in memory
        let agent_info = serde_json::json!({
            "agent_id": self.config.id,
            "agent_name": self.config.name,
            "role": self.role.name(),
            "capabilities": self.config.capabilities,
            "initialized_at": chrono::Utc::now()
        });
        
        self.memory.store(
            MemoryType::LongTerm,
            "agent_info".to_string(),
            agent_info,
            None, // No expiration for agent info
        ).await?;
        
        self.agent_state = AgentState::Ready;
        
        info!("Agent '{}' initialized successfully", self.config.name);
        Ok(())
    }

    /// Execute agent reasoning and action
    async fn execute_agent_logic(&self, state: &mut S) -> CoreResult<NodeOutput> {
        debug!("Executing agent logic for '{}'", self.config.name);
        
        if self.agent_state != AgentState::Ready {
            return Err(CoreError::execution_error(format!(
                "Agent '{}' is not ready for execution (state: {:?})",
                self.config.name, self.agent_state
            )));
        }
        
        let start_time = std::time::Instant::now();
        
        // Step 1: Perceive current state
        let perception = self.perceive_state(state).await?;
        
        // Step 2: Retrieve relevant memories
        let relevant_memories = self.memory.retrieve_relevant(&perception).await?;
        
        // Step 3: Reason about action
        let reasoning_result = self.reason_about_action(state, &perception, &relevant_memories).await?;
        
        // Step 4: Execute action
        let action_result = self.execute_action(state, &reasoning_result).await?;
        
        // Step 5: Store experience in memory
        self.store_experience(&perception, &reasoning_result, &action_result).await?;
        
        let duration = start_time.elapsed();
        
        // Create execution metrics
        let mut metrics = ExecutionMetrics::with_duration(duration.as_millis() as u64);
        metrics.add_metric("perception_complexity".to_string(), perception.complexity);
        metrics.add_metric("memory_retrievals".to_string(), relevant_memories.len() as f64);
        metrics.add_metric("reasoning_confidence".to_string(), reasoning_result.confidence);
        
        Ok(NodeOutput {
            success: action_result.success,
            data: action_result.data,
            metrics,
            next_node: action_result.next_node,
            continue_execution: action_result.continue_execution,
        })
    }

    /// Perceive the current state
    async fn perceive_state(&self, state: &S) -> CoreResult<Perception> {
        debug!("Agent '{}' perceiving state", self.config.name);
        
        // Convert state to JSON and analyze
        let state_json = state.to_json()?;
        let complexity = self.calculate_state_complexity(&state_json);
        
        // Extract key information based on agent role
        let key_observations = self.extract_role_specific_observations(&state_json);
        
        Ok(Perception {
            state_summary: state_json,
            complexity,
            key_observations,
            timestamp: chrono::Utc::now(),
            agent_id: self.config.id.clone(),
        })
    }

    /// Extract observations specific to the agent's role
    fn extract_role_specific_observations(&self, state_json: &serde_json::Value) -> Vec<String> {
        let mut observations = Vec::new();
        
        match self.role {
            AgentRole::Researcher => {
                // Look for data that needs research or analysis
                if let Some(obj) = state_json.as_object() {
                    for (key, value) in obj {
                        if key.contains("research") || key.contains("data") || key.contains("information") {
                            observations.push(format!("Found research-related data: {} = {}", key, value));
                        }
                    }
                }
            }
            AgentRole::Analyst => {
                // Look for patterns and data to analyze
                if let Some(obj) = state_json.as_object() {
                    for (key, value) in obj {
                        if key.contains("metric") || key.contains("count") || key.contains("value") {
                            observations.push(format!("Found analytical data: {} = {}", key, value));
                        }
                    }
                }
            }
            AgentRole::Monitor => {
                // Look for system health and performance indicators
                if let Some(obj) = state_json.as_object() {
                    for (key, value) in obj {
                        if key.contains("status") || key.contains("health") || key.contains("error") {
                            observations.push(format!("Found monitoring data: {} = {}", key, value));
                        }
                    }
                }
            }
            _ => {
                // General observations for other roles
                observations.push(format!("State contains {} top-level fields", 
                    state_json.as_object().map(|o| o.len()).unwrap_or(0)));
            }
        }
        
        observations
    }

    /// Reason about what action to take
    async fn reason_about_action(
        &self,
        _state: &S,
        perception: &Perception,
        memories: &[MemoryEntry],
    ) -> CoreResult<ReasoningResult> {
        debug!("Agent '{}' reasoning about action", self.config.name);
        
        // Use role-specific reasoning
        let (action_type, confidence, reasoning) = match self.role {
            AgentRole::Researcher => {
                let needs_research = perception.key_observations.iter()
                    .any(|obs| obs.contains("research") || obs.contains("data"));
                
                if needs_research {
                    ("research_and_analyze".to_string(), 0.9, "Detected data that requires research and analysis".to_string())
                } else {
                    ("gather_information".to_string(), 0.7, "No specific research targets found, gathering general information".to_string())
                }
            }
            AgentRole::Analyst => {
                let has_data = perception.key_observations.iter()
                    .any(|obs| obs.contains("data") || obs.contains("metric"));
                
                if has_data {
                    ("analyze_data".to_string(), 0.9, "Found data that requires analysis".to_string())
                } else {
                    ("request_data".to_string(), 0.6, "No data found for analysis, requesting data".to_string())
                }
            }
            AgentRole::Coordinator => {
                ("coordinate_workflow".to_string(), 0.8, "Coordinating workflow based on current state".to_string())
            }
            AgentRole::Executor => {
                ("execute_task".to_string(), 0.8, "Executing assigned task".to_string())
            }
            AgentRole::Monitor => {
                let has_issues = perception.key_observations.iter()
                    .any(|obs| obs.contains("error") || obs.contains("fail"));
                
                if has_issues {
                    ("alert_and_investigate".to_string(), 0.9, "Detected potential issues requiring investigation".to_string())
                } else {
                    ("monitor_status".to_string(), 0.7, "Monitoring system status".to_string())
                }
            }
            AgentRole::Assistant => {
                ("provide_assistance".to_string(), 0.8, "Providing general assistance".to_string())
            }
            AgentRole::Custom(ref role_name) => {
                (format!("custom_{}_action", role_name.to_lowercase()), 0.7, format!("Executing custom action for {} role", role_name))
            }
        };
        
        // Consider relevant memories
        let memory_influence = memories.len() as f64 * 0.1;
        let adjusted_confidence = (confidence + memory_influence).min(1.0);
        
        let mut parameters = HashMap::new();
        parameters.insert("memory_count".to_string(), serde_json::json!(memories.len()));
        parameters.insert("perception_complexity".to_string(), serde_json::json!(perception.complexity));
        
        Ok(ReasoningResult {
            action_type,
            confidence: adjusted_confidence,
            reasoning,
            parameters,
        })
    }

    /// Execute the determined action
    async fn execute_action(
        &self,
        state: &mut S,
        reasoning: &ReasoningResult,
    ) -> CoreResult<ActionResult> {
        debug!("Agent '{}' executing action: {}", self.config.name, reasoning.action_type);
        
        // Create action data based on the reasoning
        let action_data = serde_json::json!({
            "agent_id": self.config.id,
            "agent_name": self.config.name,
            "agent_role": self.role.name(),
            "action_type": reasoning.action_type,
            "timestamp": chrono::Utc::now(),
            "confidence": reasoning.confidence,
            "reasoning": reasoning.reasoning,
            "parameters": reasoning.parameters
        });
        
        // Update state with agent's action (this is a simple implementation)
        // In a real implementation, this would perform actual work based on the action type
        if let Ok(mut state_json) = state.to_json() {
            if let Some(obj) = state_json.as_object_mut() {
                obj.insert("last_agent_action".to_string(), action_data.clone());
                obj.insert("last_action_timestamp".to_string(), serde_json::json!(chrono::Utc::now()));
            }
        }
        
        Ok(ActionResult {
            success: true,
            data: Some(action_data),
            next_node: None,
            continue_execution: true,
        })
    }

    /// Store experience in memory
    async fn store_experience(
        &self,
        perception: &Perception,
        reasoning: &ReasoningResult,
        action: &ActionResult,
    ) -> CoreResult<()> {
        debug!("Agent '{}' storing experience", self.config.name);
        
        let experience = serde_json::json!({
            "perception": perception,
            "reasoning": reasoning,
            "action": action,
            "timestamp": chrono::Utc::now(),
            "agent_role": self.role.name()
        });
        
        // Store as episodic memory with 24-hour expiration
        self.memory.store(
            MemoryType::Episodic,
            "experience".to_string(),
            experience,
            Some(chrono::Duration::hours(24)),
        ).await?;
        
        // Also store key insights as long-term memory
        if reasoning.confidence > 0.8 {
            let insight = serde_json::json!({
                "action_type": reasoning.action_type,
                "confidence": reasoning.confidence,
                "reasoning": reasoning.reasoning,
                "success": action.success,
                "timestamp": chrono::Utc::now()
            });
            
            self.memory.store(
                MemoryType::LongTerm,
                "high_confidence_insight".to_string(),
                insight,
                Some(chrono::Duration::days(30)),
            ).await?;
        }
        
        Ok(())
    }

    /// Calculate complexity of state
    fn calculate_state_complexity(&self, state_json: &serde_json::Value) -> f64 {
        match state_json {
            serde_json::Value::Object(obj) => {
                let field_count = obj.len() as f64;
                let nested_complexity: f64 = obj.values()
                    .map(|v| match v {
                        serde_json::Value::Object(_) => 0.5,
                        serde_json::Value::Array(_) => 0.3,
                        _ => 0.1,
                    })
                    .sum();
                field_count * 0.1 + nested_complexity
            }
            serde_json::Value::Array(arr) => arr.len() as f64 * 0.05,
            _ => 0.1,
        }
    }

    /// Shutdown the agent
    pub async fn shutdown(&mut self) -> CoreResult<()> {
        info!("Shutting down agent '{}'", self.config.name);
        
        self.agent_state = AgentState::ShuttingDown;
        
        // Save important memories
        self.memory.flush().await?;
        
        self.agent_state = AgentState::Shutdown;
        
        info!("Agent '{}' shutdown complete", self.config.name);
        Ok(())
    }
}

#[async_trait]
impl<S> Node<S> for Agent<S>
where
    S: State + Send + Sync + 'static,
{
    async fn execute(&self, state: &mut S) -> CoreResult<NodeOutput> {
        self.execute_agent_logic(state).await
    }

    fn id(&self) -> &str {
        &self.config.id
    }

    fn metadata(&self) -> &NodeMetadata {
        &self.metadata.node_metadata
    }
}

/// Agent metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    /// Node metadata for graph integration
    pub node_metadata: NodeMetadata,
    /// Agent creation time
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Agent version
    pub version: String,
    /// Agent tags
    pub tags: Vec<String>,
}

impl AgentMetadata {
    /// Create metadata from agent configuration
    pub fn from_config(config: &AgentConfig) -> Self {
        let node_metadata = NodeMetadata::new(config.name.clone())
            .with_description(config.description.clone().unwrap_or_else(|| {
                format!("Agent with role: {:?}", config.role)
            }))
            .with_category(agent_graph_core::node::NodeCategory::Processing);
        
        Self {
            node_metadata,
            created_at: chrono::Utc::now(),
            version: "1.0.0".to_string(),
            tags: vec!["agent".to_string(), format!("role:{:?}", config.role).to_lowercase()],
        }
    }
}

/// Agent state tracking
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentState {
    /// Agent created but not initialized
    Created,
    /// Agent is initializing
    Initializing,
    /// Agent is ready for execution
    Ready,
    /// Agent is currently executing
    Executing,
    /// Agent is paused
    Paused,
    /// Agent is shutting down
    ShuttingDown,
    /// Agent is shutdown
    Shutdown,
    /// Agent encountered an error
    Error(String),
}

/// Agent perception of current state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Perception {
    /// Summary of perceived state
    pub state_summary: serde_json::Value,
    /// Complexity score of the state
    pub complexity: f64,
    /// Key observations specific to agent role
    pub key_observations: Vec<String>,
    /// Timestamp of perception
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Agent that made the perception
    pub agent_id: String,
}

/// Result of agent reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningResult {
    /// Type of action to take
    pub action_type: String,
    /// Confidence in the decision (0.0 to 1.0)
    pub confidence: f64,
    /// Reasoning explanation
    pub reasoning: String,
    /// Action parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Result of agent action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    /// Whether action was successful
    pub success: bool,
    /// Action output data
    pub data: Option<serde_json::Value>,
    /// Next node to execute
    pub next_node: Option<NodeId>,
    /// Whether to continue execution
    pub continue_execution: bool,
}

/// Memory system placeholder (will be implemented in memory module)
pub struct MemorySystem {
    config: crate::memory::MemoryConfig,
}

impl MemorySystem {
    pub fn new(config: crate::memory::MemoryConfig) -> CoreResult<Self> {
        Ok(Self { config })
    }

    pub async fn initialize(&self) -> CoreResult<()> {
        // TODO: Implement memory system initialization
        Ok(())
    }

    pub async fn store(
        &self,
        _memory_type: MemoryType,
        _key: String,
        _data: serde_json::Value,
        _expiration: Option<chrono::Duration>,
    ) -> CoreResult<()> {
        // TODO: Implement memory storage
        Ok(())
    }

    pub async fn retrieve_relevant(&self, _perception: &Perception) -> CoreResult<Vec<MemoryEntry>> {
        // TODO: Implement memory retrieval
        Ok(Vec::new())
    }

    pub async fn flush(&self) -> CoreResult<()> {
        // TODO: Implement memory flush
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestState {
        value: i32,
        message: String,
    }

    #[tokio::test]
    async fn test_agent_creation() {
        let config = AgentConfig {
            name: "TestAgent".to_string(),
            role: AgentRole::Assistant,
            ..Default::default()
        };

        let agent: Agent<TestState> = Agent::new(config).unwrap();
        assert_eq!(agent.config().name, "TestAgent");
        assert_eq!(agent.role(), &AgentRole::Assistant);
        assert_eq!(agent.state(), &AgentState::Created);
    }

    #[tokio::test]
    async fn test_agent_initialization() {
        let config = AgentConfig {
            name: "TestAgent".to_string(),
            role: AgentRole::Researcher,
            ..Default::default()
        };

        let mut agent: Agent<TestState> = Agent::new(config).unwrap();
        assert_eq!(agent.state(), &AgentState::Created);

        agent.initialize().await.unwrap();
        assert_eq!(agent.state(), &AgentState::Ready);
    }

    #[test]
    fn test_agent_metadata_creation() {
        let config = AgentConfig {
            name: "MetadataAgent".to_string(),
            description: Some("Test description".to_string()),
            role: AgentRole::Analyst,
            ..Default::default()
        };

        let metadata = AgentMetadata::from_config(&config);
        assert_eq!(metadata.node_metadata.name, "MetadataAgent");
        assert!(metadata.tags.contains(&"agent".to_string()));
        assert!(metadata.tags.contains(&"role:analyst".to_string()));
    }
}