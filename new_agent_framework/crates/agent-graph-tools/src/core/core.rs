//! Core tool system traits and types.

use crate::{CoreError, CoreResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Core trait that all tools must implement
#[async_trait]
pub trait Tool: Send + Sync + std::fmt::Debug {
    /// Execute the tool with the given input
    async fn execute(&self, input: ToolInput) -> CoreResult<ToolOutput>;

    /// Get the tool's metadata
    fn metadata(&self) -> &ToolMetadata;

    /// Validate the tool's configuration
    fn validate(&self) -> CoreResult<()> {
        Ok(())
    }

    /// Get the tool's schema for function calling
    fn schema(&self) -> ToolSchema {
        ToolSchema::from_metadata(self.metadata())
    }

    /// Check if the tool is available
    async fn is_available(&self) -> bool {
        true
    }

    /// Get estimated execution time
    fn estimated_duration(&self) -> Option<Duration> {
        self.metadata().estimated_duration
    }
}

/// Tool metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    /// Unique tool identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Tool version
    pub version: String,
    /// Tool category
    pub category: ToolCategory,
    /// Input parameters schema
    pub input_schema: serde_json::Value,
    /// Output schema
    pub output_schema: serde_json::Value,
    /// Whether the tool is safe to run in parallel
    pub parallel_safe: bool,
    /// Estimated execution duration
    pub estimated_duration: Option<Duration>,
    /// Required permissions
    pub required_permissions: Vec<String>,
    /// Tool tags for discovery
    pub tags: Vec<String>,
    /// Custom properties
    pub properties: HashMap<String, serde_json::Value>,
}

/// Tool category enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ToolCategory {
    /// File system operations
    FileSystem,
    /// Network/HTTP operations
    Network,
    /// Database operations
    Database,
    /// Text processing
    TextProcessing,
    /// Mathematical operations
    Math,
    /// Data analysis
    DataAnalysis,
    /// System operations
    System,
    /// External API integration
    ExternalAPI,
    /// Custom tool category
    Custom(String),
}

/// Input for tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInput {
    /// Input parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Execution context
    pub context: ToolContext,
    /// Tool-specific configuration
    pub config: HashMap<String, serde_json::Value>,
}

/// Tool execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolContext {
    /// Execution ID for tracing
    pub execution_id: String,
    /// User ID (if applicable)
    pub user_id: Option<String>,
    /// Session ID
    pub session_id: Option<String>,
    /// Timeout for execution
    pub timeout: Option<Duration>,
    /// Security context
    pub security_context: SecurityContext,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Security context for tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    /// Allowed permissions
    pub permissions: Vec<String>,
    /// Sandbox mode
    pub sandboxed: bool,
    /// Resource limits
    pub resource_limits: ResourceLimits,
}

/// Resource limits for tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage in bytes
    pub max_memory_bytes: Option<u64>,
    /// Maximum execution time
    pub max_execution_time: Option<Duration>,
    /// Maximum file size for operations
    pub max_file_size_bytes: Option<u64>,
    /// Maximum network requests
    pub max_network_requests: Option<u32>,
}

/// Output from tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutput {
    /// Whether execution was successful
    pub success: bool,
    /// Output data
    pub data: Option<serde_json::Value>,
    /// Error information (if failed)
    pub error: Option<ToolError>,
    /// Execution metrics
    pub metrics: ToolMetrics,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Tool execution error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolError {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Error details
    pub details: Option<serde_json::Value>,
    /// Whether the error is retryable
    pub retryable: bool,
}

/// Tool execution metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolMetrics {
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Memory used in bytes
    pub memory_used_bytes: Option<u64>,
    /// Network requests made
    pub network_requests: Option<u32>,
    /// Files accessed
    pub files_accessed: Option<u32>,
    /// Custom metrics
    pub custom_metrics: HashMap<String, f64>,
}

/// Tool schema for function calling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Parameters schema (JSON Schema)
    pub parameters: serde_json::Value,
}

impl ToolSchema {
    /// Create schema from metadata
    pub fn from_metadata(metadata: &ToolMetadata) -> Self {
        Self {
            name: metadata.name.clone(),
            description: metadata.description.clone(),
            parameters: metadata.input_schema.clone(),
        }
    }
}

impl ToolInput {
    /// Create a new tool input
    pub fn new(parameters: HashMap<String, serde_json::Value>) -> Self {
        Self {
            parameters,
            context: ToolContext::default(),
            config: HashMap::new(),
        }
    }

    /// Create tool input with context
    pub fn with_context(
        parameters: HashMap<String, serde_json::Value>,
        context: ToolContext,
    ) -> Self {
        Self {
            parameters,
            context,
            config: HashMap::new(),
        }
    }

    /// Get a parameter value
    pub fn get_param<T>(&self, key: &str) -> CoreResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let value = self.parameters.get(key)
            .ok_or_else(|| CoreError::validation_error(format!("Missing parameter: {}", key)))?;
        
        serde_json::from_value(value.clone())
            .map_err(|e| CoreError::validation_error(format!("Invalid parameter {}: {}", key, e)))
    }

    /// Get an optional parameter value
    pub fn get_optional_param<T>(&self, key: &str) -> CoreResult<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        match self.parameters.get(key) {
            Some(value) => {
                let parsed = serde_json::from_value(value.clone())
                    .map_err(|e| CoreError::validation_error(format!("Invalid parameter {}: {}", key, e)))?;
                Ok(Some(parsed))
            }
            None => Ok(None),
        }
    }
}

impl ToolOutput {
    /// Create a successful output
    pub fn success(data: Option<serde_json::Value>) -> Self {
        Self {
            success: true,
            data,
            error: None,
            metrics: ToolMetrics::default(),
            metadata: HashMap::new(),
        }
    }

    /// Create a failure output
    pub fn failure(error: ToolError) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            metrics: ToolMetrics::default(),
            metadata: HashMap::new(),
        }
    }

    /// Create output with metrics
    pub fn with_metrics(mut self, metrics: ToolMetrics) -> Self {
        self.metrics = metrics;
        self
    }
}

impl ToolError {
    /// Create a new tool error
    pub fn new(code: String, message: String) -> Self {
        Self {
            code,
            message,
            details: None,
            retryable: false,
        }
    }

    /// Create a retryable error
    pub fn retryable(code: String, message: String) -> Self {
        Self {
            code,
            message,
            details: None,
            retryable: true,
        }
    }

    /// Add details to the error
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

impl Default for ToolContext {
    fn default() -> Self {
        Self {
            execution_id: uuid::Uuid::new_v4().to_string(),
            user_id: None,
            session_id: None,
            timeout: Some(Duration::from_secs(30)),
            security_context: SecurityContext::default(),
            metadata: HashMap::new(),
        }
    }
}

impl Default for SecurityContext {
    fn default() -> Self {
        Self {
            permissions: Vec::new(),
            sandboxed: true,
            resource_limits: ResourceLimits::default(),
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: Some(100 * 1024 * 1024), // 100MB
            max_execution_time: Some(Duration::from_secs(30)),
            max_file_size_bytes: Some(10 * 1024 * 1024), // 10MB
            max_network_requests: Some(10),
        }
    }
}

/// Tool registry for managing available tools
#[derive(Debug)]
pub struct ToolRegistry {
    /// Registered tools
    tools: HashMap<String, Box<dyn Tool>>,
    /// Tool categories
    categories: HashMap<ToolCategory, Vec<String>>,
}

impl ToolRegistry {
    /// Create a new tool registry
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            categories: HashMap::new(),
        }
    }

    /// Register a tool
    pub fn register(&mut self, tool: Box<dyn Tool>) -> CoreResult<()> {
        let metadata = tool.metadata();
        let tool_id = metadata.id.clone();
        let category = metadata.category.clone();

        // Validate tool before registration
        tool.validate()?;

        // Add to tools
        self.tools.insert(tool_id.clone(), tool);

        // Add to category index
        self.categories.entry(category).or_insert_with(Vec::new).push(tool_id);

        Ok(())
    }

    /// Get a tool by ID
    pub fn get(&self, tool_id: &str) -> Option<&dyn Tool> {
        self.tools.get(tool_id).map(|t| t.as_ref())
    }

    /// List all tool IDs
    pub fn list_tools(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }

    /// List tools by category
    pub fn list_by_category(&self, category: &ToolCategory) -> Vec<String> {
        self.categories.get(category).cloned().unwrap_or_default()
    }

    /// Get all tool schemas for function calling
    pub fn get_schemas(&self) -> Vec<ToolSchema> {
        self.tools.values().map(|tool| tool.schema()).collect()
    }

    /// Search tools by tags
    pub fn search_by_tags(&self, tags: &[String]) -> Vec<String> {
        self.tools
            .iter()
            .filter(|(_, tool)| {
                let tool_tags = &tool.metadata().tags;
                tags.iter().any(|tag| tool_tags.contains(tag))
            })
            .map(|(id, _)| id.clone())
            .collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}