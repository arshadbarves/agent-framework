// Core tool traits and types for AgentGraph

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use thiserror::Error;

/// Result type for tool operations
pub type ToolResult<T> = Result<T, ToolError>;

/// Errors that can occur during tool execution
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum ToolError {
    /// Tool execution failed
    #[error("Tool execution failed: {message}")]
    ExecutionError {
        /// Error message
        message: String
    },

    /// Tool validation failed
    #[error("Tool validation failed: {message}")]
    ValidationError {
        /// Error message
        message: String
    },

    /// Tool execution timed out
    #[error("Tool timeout after {timeout_ms}ms")]
    TimeoutError {
        /// Timeout duration in milliseconds
        timeout_ms: u64
    },

    /// Tool configuration error
    #[error("Tool configuration error: {message}")]
    ConfigurationError {
        /// Error message
        message: String
    },

    /// Tool not found
    #[error("Tool not found: {tool_id}")]
    NotFoundError {
        /// Tool identifier
        tool_id: String
    },

    /// Tool input/output error
    #[error("Tool input/output error: {message}")]
    IoError {
        /// Error message
        message: String
    },

    /// Tool network error
    #[error("Tool network error: {message}")]
    NetworkError {
        /// Error message
        message: String
    },

    /// Tool authentication error
    #[error("Tool authentication error: {message}")]
    AuthenticationError {
        /// Error message
        message: String
    },
}

/// Input data for tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInput {
    /// Primary input data
    pub data: serde_json::Value,
    /// Additional parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Execution context
    pub context: HashMap<String, String>,
}

impl ToolInput {
    /// Create a new tool input with data
    pub fn new(data: serde_json::Value) -> Self {
        Self {
            data,
            parameters: HashMap::new(),
            context: HashMap::new(),
        }
    }
    
    /// Add a parameter to the input
    pub fn with_parameter<T: Serialize>(mut self, key: &str, value: T) -> Self {
        self.parameters.insert(
            key.to_string(),
            serde_json::to_value(value).unwrap_or(serde_json::Value::Null),
        );
        self
    }
    
    /// Add context information
    pub fn with_context(mut self, key: &str, value: &str) -> Self {
        self.context.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Get a parameter value
    pub fn get_parameter<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.parameters
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }
    
    /// Get context value
    pub fn get_context(&self, key: &str) -> Option<&String> {
        self.context.get(key)
    }
}

/// Output data from tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutput {
    /// Primary output data
    pub data: serde_json::Value,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Execution metrics
    pub metrics: HashMap<String, f64>,
}

impl ToolOutput {
    /// Create a new tool output with data
    pub fn new(data: serde_json::Value) -> Self {
        Self {
            data,
            metadata: HashMap::new(),
            metrics: HashMap::new(),
        }
    }
    
    /// Add metadata to the output
    pub fn with_metadata<T: Serialize>(mut self, key: &str, value: T) -> Self {
        self.metadata.insert(
            key.to_string(),
            serde_json::to_value(value).unwrap_or(serde_json::Value::Null),
        );
        self
    }
    
    /// Add a metric to the output
    pub fn with_metric(mut self, key: &str, value: f64) -> Self {
        self.metrics.insert(key.to_string(), value);
        self
    }
    
    /// Get metadata value
    pub fn get_metadata<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.metadata
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }
    
    /// Get metric value
    pub fn get_metric(&self, key: &str) -> Option<f64> {
        self.metrics.get(key).copied()
    }
}

/// Metadata describing a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    /// Unique identifier for the tool
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description of what the tool does
    pub description: String,
    /// Version of the tool
    pub version: String,
    /// Author or organization
    pub author: Option<String>,
    /// Tool category/tags
    pub tags: Vec<String>,
    /// Input schema (JSON Schema)
    pub input_schema: Option<serde_json::Value>,
    /// Output schema (JSON Schema)
    pub output_schema: Option<serde_json::Value>,
    /// Whether the tool is deterministic
    pub deterministic: bool,
    /// Whether the tool has side effects
    pub has_side_effects: bool,
    /// Estimated execution time in milliseconds
    pub estimated_duration_ms: Option<u64>,
}

impl ToolMetadata {
    /// Create new tool metadata
    pub fn new(id: &str, name: &str, description: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            version: "1.0.0".to_string(),
            author: None,
            tags: Vec::new(),
            input_schema: None,
            output_schema: None,
            deterministic: true,
            has_side_effects: false,
            estimated_duration_ms: None,
        }
    }
    
    /// Set the version
    pub fn with_version(mut self, version: &str) -> Self {
        self.version = version.to_string();
        self
    }
    
    /// Set the author
    pub fn with_author(mut self, author: &str) -> Self {
        self.author = Some(author.to_string());
        self
    }
    
    /// Add a tag
    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }
    
    /// Set deterministic flag
    pub fn with_deterministic(mut self, deterministic: bool) -> Self {
        self.deterministic = deterministic;
        self
    }
    
    /// Set side effects flag
    pub fn with_side_effects(mut self, has_side_effects: bool) -> Self {
        self.has_side_effects = has_side_effects;
        self
    }
    
    /// Set estimated duration
    pub fn with_estimated_duration_ms(mut self, duration_ms: u64) -> Self {
        self.estimated_duration_ms = Some(duration_ms);
        self
    }
}

/// Core trait that all tools must implement
#[async_trait]
pub trait Tool: Send + Sync + fmt::Debug {
    /// Get tool metadata
    fn metadata(&self) -> &ToolMetadata;
    
    /// Execute the tool with given input
    async fn execute(&self, input: ToolInput) -> ToolResult<ToolOutput>;
    
    /// Validate input before execution (optional)
    async fn validate_input(&self, _input: &ToolInput) -> ToolResult<()> {
        // Default implementation does no validation
        Ok(())
    }
    
    /// Cleanup after execution (optional)
    async fn cleanup(&self) -> ToolResult<()> {
        // Default implementation does nothing
        Ok(())
    }
    
    /// Check if the tool is available/healthy
    async fn health_check(&self) -> ToolResult<()> {
        // Default implementation always returns OK
        Ok(())
    }
}

/// Trait for tools that support configuration
pub trait ConfigurableTool: Tool {
    /// Configure the tool with parameters
    fn configure(&mut self, config: HashMap<String, serde_json::Value>) -> ToolResult<()>;
    
    /// Get current configuration
    fn get_configuration(&self) -> HashMap<String, serde_json::Value>;
}

/// Trait for tools that support caching
pub trait CacheableTool: Tool {
    /// Generate a cache key for the given input
    fn cache_key(&self, input: &ToolInput) -> String;
    
    /// Whether this input should be cached
    fn should_cache(&self, _input: &ToolInput) -> bool {
        true // Default: cache everything
    }
    
    /// Cache TTL in seconds
    fn cache_ttl(&self) -> Option<u64> {
        Some(3600) // Default: 1 hour
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_tool_input_creation() {
        let input = ToolInput::new(json!({"test": "data"}))
            .with_parameter("param1", "value1")
            .with_context("context1", "ctx_value1");
        
        assert_eq!(input.data, json!({"test": "data"}));
        assert_eq!(input.get_parameter::<String>("param1"), Some("value1".to_string()));
        assert_eq!(input.get_context("context1"), Some(&"ctx_value1".to_string()));
    }

    #[test]
    fn test_tool_output_creation() {
        let output = ToolOutput::new(json!({"result": "success"}))
            .with_metadata("timestamp", "2024-01-01T00:00:00Z")
            .with_metric("execution_time", 123.45);
        
        assert_eq!(output.data, json!({"result": "success"}));
        assert_eq!(output.get_metadata::<String>("timestamp"), Some("2024-01-01T00:00:00Z".to_string()));
        assert_eq!(output.get_metric("execution_time"), Some(123.45));
    }

    #[test]
    fn test_tool_metadata_creation() {
        let metadata = ToolMetadata::new("test_tool", "Test Tool", "A tool for testing")
            .with_version("2.0.0")
            .with_author("AgentGraph Team")
            .with_tag("testing")
            .with_tag("utility")
            .with_deterministic(false)
            .with_side_effects(true)
            .with_estimated_duration_ms(1000);
        
        assert_eq!(metadata.id, "test_tool");
        assert_eq!(metadata.name, "Test Tool");
        assert_eq!(metadata.version, "2.0.0");
        assert_eq!(metadata.author, Some("AgentGraph Team".to_string()));
        assert_eq!(metadata.tags, vec!["testing", "utility"]);
        assert!(!metadata.deterministic);
        assert!(metadata.has_side_effects);
        assert_eq!(metadata.estimated_duration_ms, Some(1000));
    }
}
