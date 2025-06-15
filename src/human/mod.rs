// Human-in-the-Loop module for AgentGraph
// Provides capabilities for human interaction during graph execution

/// Core traits and types for human interaction
pub mod traits;
/// Interrupt and resume system
pub mod interrupt;
/// Human input collection
pub mod input;
/// Approval workflows
pub mod approval;

pub use traits::{HumanInteraction, HumanInput, HumanResponse, InteractionType, InteractionError};
pub use interrupt::{InterruptManager, InterruptPoint, InterruptState, ResumeToken};
pub use input::{InputCollector, InputRequest, InputValidator};
pub use approval::{ApprovalManager, ApprovalRequest, ApprovalResponse, ApprovalPolicy};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Configuration for human interaction timeouts and behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanConfig {
    /// Maximum time to wait for human response
    pub timeout: Option<Duration>,
    /// Whether to allow skipping human interaction in automated mode
    pub allow_skip: bool,
    /// Default response when timeout occurs
    pub default_response: Option<serde_json::Value>,
    /// Retry attempts for failed interactions
    pub max_retries: u32,
    /// Custom configuration parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

impl Default for HumanConfig {
    fn default() -> Self {
        Self {
            timeout: Some(Duration::from_secs(300)), // 5 minutes default
            allow_skip: false,
            default_response: None,
            max_retries: 3,
            parameters: HashMap::new(),
        }
    }
}

/// Statistics for human interaction tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanStats {
    /// Total number of human interactions
    pub total_interactions: u64,
    /// Number of successful interactions
    pub successful_interactions: u64,
    /// Number of timed out interactions
    pub timeout_interactions: u64,
    /// Number of skipped interactions
    pub skipped_interactions: u64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Total response time in milliseconds
    pub total_response_time_ms: u64,
    /// Last interaction timestamp
    pub last_interaction: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for HumanStats {
    fn default() -> Self {
        Self {
            total_interactions: 0,
            successful_interactions: 0,
            timeout_interactions: 0,
            skipped_interactions: 0,
            avg_response_time_ms: 0.0,
            total_response_time_ms: 0,
            last_interaction: None,
        }
    }
}

impl HumanStats {
    /// Update statistics after a human interaction
    pub fn update(&mut self, response_time_ms: u64, outcome: InteractionOutcome) {
        self.total_interactions += 1;
        self.total_response_time_ms += response_time_ms;
        self.avg_response_time_ms = self.total_response_time_ms as f64 / self.total_interactions as f64;
        self.last_interaction = Some(chrono::Utc::now());
        
        match outcome {
            InteractionOutcome::Success => self.successful_interactions += 1,
            InteractionOutcome::Timeout => self.timeout_interactions += 1,
            InteractionOutcome::Skipped => self.skipped_interactions += 1,
        }
    }
    
    /// Get success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_interactions == 0 {
            0.0
        } else {
            (self.successful_interactions as f64 / self.total_interactions as f64) * 100.0
        }
    }
    
    /// Get timeout rate as a percentage
    pub fn timeout_rate(&self) -> f64 {
        if self.total_interactions == 0 {
            0.0
        } else {
            (self.timeout_interactions as f64 / self.total_interactions as f64) * 100.0
        }
    }
}

/// Outcome of a human interaction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InteractionOutcome {
    /// Human responded successfully
    Success,
    /// Interaction timed out
    Timeout,
    /// Interaction was skipped
    Skipped,
}

/// Context for human interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanContext {
    /// Unique identifier for this interaction
    pub interaction_id: String,
    /// User ID for the human participant
    pub user_id: Option<String>,
    /// Session ID for grouping interactions
    pub session_id: Option<String>,
    /// Graph execution context
    pub graph_context: HashMap<String, String>,
    /// Node context where interaction occurs
    pub node_context: HashMap<String, serde_json::Value>,
    /// Timestamp when interaction was initiated
    pub initiated_at: SystemTime,
}

impl HumanContext {
    /// Create a new human interaction context
    pub fn new(interaction_id: String) -> Self {
        Self {
            interaction_id,
            user_id: None,
            session_id: None,
            graph_context: HashMap::new(),
            node_context: HashMap::new(),
            initiated_at: SystemTime::now(),
        }
    }
    
    /// Set user ID
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }
    
    /// Set session ID
    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }
    
    /// Add graph context
    pub fn with_graph_context(mut self, key: String, value: String) -> Self {
        self.graph_context.insert(key, value);
        self
    }
    
    /// Add node context
    pub fn with_node_context(mut self, key: String, value: serde_json::Value) -> Self {
        self.node_context.insert(key, value);
        self
    }
    
    /// Get elapsed time since interaction was initiated
    pub fn elapsed_time(&self) -> Duration {
        self.initiated_at.elapsed().unwrap_or(Duration::ZERO)
    }
}

/// Human interaction modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InteractionMode {
    /// Synchronous interaction - block execution until response
    Synchronous,
    /// Asynchronous interaction - continue execution, handle response later
    Asynchronous,
    /// Optional interaction - continue if no response within timeout
    Optional,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human_config_default() {
        let config = HumanConfig::default();
        assert_eq!(config.timeout, Some(Duration::from_secs(300)));
        assert!(!config.allow_skip);
        assert_eq!(config.default_response, None);
        assert_eq!(config.max_retries, 3);
        assert!(config.parameters.is_empty());
    }

    #[test]
    fn test_human_stats_update() {
        let mut stats = HumanStats::default();
        
        // Test successful interaction
        stats.update(1000, InteractionOutcome::Success);
        assert_eq!(stats.total_interactions, 1);
        assert_eq!(stats.successful_interactions, 1);
        assert_eq!(stats.timeout_interactions, 0);
        assert_eq!(stats.avg_response_time_ms, 1000.0);
        assert_eq!(stats.success_rate(), 100.0);
        assert_eq!(stats.timeout_rate(), 0.0);
        
        // Test timeout interaction
        stats.update(5000, InteractionOutcome::Timeout);
        assert_eq!(stats.total_interactions, 2);
        assert_eq!(stats.successful_interactions, 1);
        assert_eq!(stats.timeout_interactions, 1);
        assert_eq!(stats.avg_response_time_ms, 3000.0);
        assert_eq!(stats.success_rate(), 50.0);
        assert_eq!(stats.timeout_rate(), 50.0);
    }

    #[test]
    fn test_human_context_creation() {
        let context = HumanContext::new("interaction_1".to_string())
            .with_user_id("user_123".to_string())
            .with_session_id("session_456".to_string())
            .with_graph_context("graph_id".to_string(), "graph_123".to_string())
            .with_node_context("node_id".to_string(), serde_json::json!("node_456"));
        
        assert_eq!(context.interaction_id, "interaction_1");
        assert_eq!(context.user_id, Some("user_123".to_string()));
        assert_eq!(context.session_id, Some("session_456".to_string()));
        assert_eq!(context.graph_context.get("graph_id"), Some(&"graph_123".to_string()));
        assert_eq!(context.node_context.get("node_id"), Some(&serde_json::json!("node_456")));
    }

    #[test]
    fn test_interaction_outcome_serialization() {
        let outcome = InteractionOutcome::Success;
        let serialized = serde_json::to_string(&outcome).unwrap();
        let deserialized: InteractionOutcome = serde_json::from_str(&serialized).unwrap();
        assert_eq!(outcome, deserialized);
    }
}
