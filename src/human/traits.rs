// Core traits and types for human interaction

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Result type for human interaction operations
pub type HumanResult<T> = Result<T, InteractionError>;

/// Errors that can occur during human interaction
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum InteractionError {
    /// Human interaction timed out
    #[error("Human interaction timed out after {timeout_ms}ms")]
    TimeoutError { 
        /// Timeout duration in milliseconds
        timeout_ms: u64 
    },
    
    /// Human interaction was cancelled
    #[error("Human interaction was cancelled: {reason}")]
    CancelledError { 
        /// Cancellation reason
        reason: String 
    },
    
    /// Invalid input provided by human
    #[error("Invalid human input: {message}")]
    ValidationError { 
        /// Validation error message
        message: String 
    },
    
    /// Human interaction not available
    #[error("Human interaction not available: {message}")]
    UnavailableError { 
        /// Unavailability reason
        message: String 
    },
    
    /// Configuration error
    #[error("Human interaction configuration error: {message}")]
    ConfigurationError { 
        /// Configuration error message
        message: String 
    },
    
    /// Network or communication error
    #[error("Human interaction communication error: {message}")]
    CommunicationError { 
        /// Communication error message
        message: String 
    },
    
    /// Permission denied
    #[error("Human interaction permission denied: {message}")]
    PermissionError { 
        /// Permission error message
        message: String 
    },
    
    /// Internal system error
    #[error("Human interaction system error: {message}")]
    SystemError { 
        /// System error message
        message: String 
    },
}

/// Types of human interactions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InteractionType {
    /// Simple approval (yes/no)
    Approval,
    /// Text input collection
    TextInput,
    /// Multiple choice selection
    MultipleChoice,
    /// File upload
    FileUpload,
    /// Custom interaction type
    Custom(String),
}

/// Input data for human interaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanInput {
    /// Type of interaction
    pub interaction_type: InteractionType,
    /// Primary prompt or question
    pub prompt: String,
    /// Additional context or instructions
    pub context: Option<String>,
    /// Options for multiple choice interactions
    pub options: Option<Vec<String>>,
    /// Default value if any
    pub default_value: Option<serde_json::Value>,
    /// Validation rules
    pub validation_rules: HashMap<String, serde_json::Value>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl HumanInput {
    /// Create a new human input request
    pub fn new(interaction_type: InteractionType, prompt: String) -> Self {
        Self {
            interaction_type,
            prompt,
            context: None,
            options: None,
            default_value: None,
            validation_rules: HashMap::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Create an approval request
    pub fn approval(prompt: String) -> Self {
        Self::new(InteractionType::Approval, prompt)
    }
    
    /// Create a text input request
    pub fn text_input(prompt: String) -> Self {
        Self::new(InteractionType::TextInput, prompt)
    }
    
    /// Create a multiple choice request
    pub fn multiple_choice(prompt: String, options: Vec<String>) -> Self {
        let mut input = Self::new(InteractionType::MultipleChoice, prompt);
        input.options = Some(options);
        input
    }
    
    /// Add context to the input
    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }
    
    /// Add default value
    pub fn with_default<T: Serialize>(mut self, default: T) -> Self {
        self.default_value = Some(serde_json::to_value(default).unwrap_or(serde_json::Value::Null));
        self
    }
    
    /// Add validation rule
    pub fn with_validation_rule<T: Serialize>(mut self, rule: &str, value: T) -> Self {
        self.validation_rules.insert(
            rule.to_string(),
            serde_json::to_value(value).unwrap_or(serde_json::Value::Null),
        );
        self
    }
    
    /// Add metadata
    pub fn with_metadata<T: Serialize>(mut self, key: &str, value: T) -> Self {
        self.metadata.insert(
            key.to_string(),
            serde_json::to_value(value).unwrap_or(serde_json::Value::Null),
        );
        self
    }
}

/// Response from human interaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanResponse {
    /// The response value
    pub value: serde_json::Value,
    /// Response timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Whether the response was provided by human or system default
    pub is_human_provided: bool,
    /// Additional metadata about the response
    pub metadata: HashMap<String, serde_json::Value>,
}

impl HumanResponse {
    /// Create a new human response
    pub fn new(value: serde_json::Value, response_time_ms: u64, is_human_provided: bool) -> Self {
        Self {
            value,
            timestamp: chrono::Utc::now(),
            response_time_ms,
            is_human_provided,
            metadata: HashMap::new(),
        }
    }
    
    /// Create a human-provided response
    pub fn human(value: serde_json::Value, response_time_ms: u64) -> Self {
        Self::new(value, response_time_ms, true)
    }
    
    /// Create a system default response
    pub fn default(value: serde_json::Value, response_time_ms: u64) -> Self {
        Self::new(value, response_time_ms, false)
    }
    
    /// Add metadata to the response
    pub fn with_metadata<T: Serialize>(mut self, key: &str, value: T) -> Self {
        self.metadata.insert(
            key.to_string(),
            serde_json::to_value(value).unwrap_or(serde_json::Value::Null),
        );
        self
    }
    
    /// Get the response as a specific type
    pub fn as_value<T: for<'de> Deserialize<'de>>(&self) -> Option<T> {
        serde_json::from_value(self.value.clone()).ok()
    }
    
    /// Get the response as a boolean (for approval interactions)
    pub fn as_bool(&self) -> Option<bool> {
        self.value.as_bool()
    }
    
    /// Get the response as a string
    pub fn as_string(&self) -> Option<String> {
        self.value.as_str().map(|s| s.to_string())
    }
    
    /// Get the response as an integer
    pub fn as_i64(&self) -> Option<i64> {
        self.value.as_i64()
    }
    
    /// Get the response as a float
    pub fn as_f64(&self) -> Option<f64> {
        self.value.as_f64()
    }
}

/// Core trait for human interaction providers
#[async_trait]
pub trait HumanInteraction: Send + Sync + std::fmt::Debug {
    /// Request input from a human
    async fn request_input(
        &self,
        input: HumanInput,
        context: &super::HumanContext,
        config: &super::HumanConfig,
    ) -> HumanResult<HumanResponse>;
    
    /// Check if human interaction is available
    async fn is_available(&self) -> bool;
    
    /// Cancel an ongoing interaction
    async fn cancel_interaction(&self, interaction_id: &str) -> HumanResult<()>;
    
    /// Get the name/type of this interaction provider
    fn provider_name(&self) -> &str;
    
    /// Validate input before requesting human interaction
    async fn validate_input(&self, input: &HumanInput) -> HumanResult<()> {
        // Default implementation does basic validation
        if input.prompt.is_empty() {
            return Err(InteractionError::ValidationError {
                message: "Prompt cannot be empty".to_string(),
            });
        }
        
        if input.interaction_type == InteractionType::MultipleChoice && input.options.is_none() {
            return Err(InteractionError::ValidationError {
                message: "Multiple choice interactions must have options".to_string(),
            });
        }
        
        Ok(())
    }
}

/// Trait for human interaction providers that support real-time updates
#[async_trait]
pub trait StreamingHumanInteraction: HumanInteraction {
    /// Stream updates about the interaction status
    async fn stream_updates(
        &self,
        interaction_id: &str,
    ) -> HumanResult<tokio::sync::mpsc::Receiver<InteractionUpdate>>;
}

/// Updates about interaction status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionUpdate {
    /// Interaction ID
    pub interaction_id: String,
    /// Update type
    pub update_type: UpdateType,
    /// Update timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Additional data
    pub data: Option<serde_json::Value>,
}

/// Types of interaction updates
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateType {
    /// Interaction was initiated
    Initiated,
    /// Human has seen the interaction
    Viewed,
    /// Human is typing/responding
    Responding,
    /// Interaction completed
    Completed,
    /// Interaction was cancelled
    Cancelled,
    /// Interaction timed out
    TimedOut,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_human_input_creation() {
        let input = HumanInput::approval("Do you approve this action?".to_string())
            .with_context("This will delete all data".to_string())
            .with_default(false)
            .with_validation_rule("required", true)
            .with_metadata("priority", "high");
        
        assert_eq!(input.interaction_type, InteractionType::Approval);
        assert_eq!(input.prompt, "Do you approve this action?");
        assert_eq!(input.context, Some("This will delete all data".to_string()));
        assert_eq!(input.default_value, Some(json!(false)));
        assert_eq!(input.validation_rules.get("required"), Some(&json!(true)));
        assert_eq!(input.metadata.get("priority"), Some(&json!("high")));
    }

    #[test]
    fn test_multiple_choice_input() {
        let options = vec!["Option A".to_string(), "Option B".to_string(), "Option C".to_string()];
        let input = HumanInput::multiple_choice(
            "Choose an option:".to_string(),
            options.clone()
        );
        
        assert_eq!(input.interaction_type, InteractionType::MultipleChoice);
        assert_eq!(input.options, Some(options));
    }

    #[test]
    fn test_human_response_creation() {
        let response = HumanResponse::human(json!("test response"), 1500)
            .with_metadata("source", "web_ui");
        
        assert_eq!(response.value, json!("test response"));
        assert_eq!(response.response_time_ms, 1500);
        assert!(response.is_human_provided);
        assert_eq!(response.metadata.get("source"), Some(&json!("web_ui")));
    }

    #[test]
    fn test_human_response_type_conversion() {
        let bool_response = HumanResponse::human(json!(true), 1000);
        assert_eq!(bool_response.as_bool(), Some(true));
        
        let string_response = HumanResponse::human(json!("hello"), 1000);
        assert_eq!(string_response.as_string(), Some("hello".to_string()));
        
        let number_response = HumanResponse::human(json!(42), 1000);
        assert_eq!(number_response.as_i64(), Some(42));
    }

    #[test]
    fn test_interaction_error_serialization() {
        let error = InteractionError::TimeoutError { timeout_ms: 5000 };
        let serialized = serde_json::to_string(&error).unwrap();
        let deserialized: InteractionError = serde_json::from_str(&serialized).unwrap();
        
        match deserialized {
            InteractionError::TimeoutError { timeout_ms } => assert_eq!(timeout_ms, 5000),
            _ => panic!("Wrong error type"),
        }
    }
}
