//! Built-in tools for common operations.

use crate::{CoreError, CoreResult};
use crate::core::{Tool, ToolMetadata, ToolInput, ToolOutput, ToolCategory, ToolError, ToolMetrics};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// HTTP request tool
#[derive(Debug)]
pub struct HttpRequestTool {
    metadata: ToolMetadata,
    client: reqwest::Client,
}

impl HttpRequestTool {
    /// Create a new HTTP request tool
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        let metadata = ToolMetadata {
            id: "http_request".to_string(),
            name: "HTTP Request".to_string(),
            description: "Make HTTP requests to external APIs".to_string(),
            version: "1.0.0".to_string(),
            category: ToolCategory::Network,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "The URL to make the request to"
                    },
                    "method": {
                        "type": "string",
                        "enum": ["GET", "POST", "PUT", "DELETE", "PATCH"],
                        "default": "GET",
                        "description": "HTTP method"
                    },
                    "headers": {
                        "type": "object",
                        "description": "HTTP headers to include"
                    },
                    "body": {
                        "type": "string",
                        "description": "Request body (for POST/PUT/PATCH)"
                    },
                    "timeout_seconds": {
                        "type": "number",
                        "default": 30,
                        "description": "Request timeout in seconds"
                    }
                },
                "required": ["url"]
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "status": {
                        "type": "number",
                        "description": "HTTP status code"
                    },
                    "headers": {
                        "type": "object",
                        "description": "Response headers"
                    },
                    "body": {
                        "type": "string",
                        "description": "Response body"
                    }
                }
            }),
            parallel_safe: true,
            estimated_duration: Some(Duration::from_secs(5)),
            required_permissions: vec!["network.http".to_string()],
            tags: vec!["http".to_string(), "network".to_string(), "api".to_string()],
            properties: HashMap::new(),
        };

        Self { metadata, client }
    }
}

impl Default for HttpRequestTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for HttpRequestTool {
    async fn execute(&self, input: ToolInput) -> CoreResult<ToolOutput> {
        let start_time = Instant::now();

        // Extract parameters
        let url: String = input.get_param("url")?;
        let method: String = input.get_optional_param("method")?.unwrap_or_else(|| "GET".to_string());
        let headers: Option<HashMap<String, String>> = input.get_optional_param("headers")?;
        let body: Option<String> = input.get_optional_param("body")?;
        let timeout_seconds: Option<f64> = input.get_optional_param("timeout_seconds")?;

        // Validate URL
        let parsed_url = url::Url::parse(&url)
            .map_err(|e| CoreError::validation_error(format!("Invalid URL: {}", e)))?;

        // Check permissions
        if !input.context.security_context.permissions.contains(&"network.http".to_string()) {
            return Ok(ToolOutput::failure(ToolError::new(
                "PERMISSION_DENIED".to_string(),
                "HTTP requests not permitted".to_string(),
            )));
        }

        // Build request
        let mut request_builder = match method.to_uppercase().as_str() {
            "GET" => self.client.get(&url),
            "POST" => self.client.post(&url),
            "PUT" => self.client.put(&url),
            "DELETE" => self.client.delete(&url),
            "PATCH" => self.client.patch(&url),
            _ => {
                return Ok(ToolOutput::failure(ToolError::new(
                    "INVALID_METHOD".to_string(),
                    format!("Unsupported HTTP method: {}", method),
                )));
            }
        };

        // Add headers
        if let Some(headers) = headers {
            for (key, value) in headers {
                request_builder = request_builder.header(&key, &value);
            }
        }

        // Add body
        if let Some(body) = body {
            request_builder = request_builder.body(body);
        }

        // Set timeout
        if let Some(timeout) = timeout_seconds {
            request_builder = request_builder.timeout(Duration::from_secs_f64(timeout));
        }

        // Execute request
        match request_builder.send().await {
            Ok(response) => {
                let status = response.status().as_u16();
                let headers: HashMap<String, String> = response
                    .headers()
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                    .collect();

                let body = response.text().await.unwrap_or_else(|_| "".to_string());

                let metrics = ToolMetrics {
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    network_requests: Some(1),
                    ..Default::default()
                };

                Ok(ToolOutput::success(Some(serde_json::json!({
                    "status": status,
                    "headers": headers,
                    "body": body
                }))).with_metrics(metrics))
            }
            Err(error) => {
                let metrics = ToolMetrics {
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    network_requests: Some(1),
                    ..Default::default()
                };

                Ok(ToolOutput::failure(ToolError::retryable(
                    "HTTP_ERROR".to_string(),
                    format!("HTTP request failed: {}", error),
                )).with_metrics(metrics))
            }
        }
    }

    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }
}

/// Text processing tool
#[derive(Debug)]
pub struct TextProcessingTool {
    metadata: ToolMetadata,
}

impl TextProcessingTool {
    /// Create a new text processing tool
    pub fn new() -> Self {
        let metadata = ToolMetadata {
            id: "text_processing".to_string(),
            name: "Text Processing".to_string(),
            description: "Process and manipulate text data".to_string(),
            version: "1.0.0".to_string(),
            category: ToolCategory::TextProcessing,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "The text to process"
                    },
                    "operation": {
                        "type": "string",
                        "enum": ["uppercase", "lowercase", "word_count", "char_count", "reverse", "trim"],
                        "description": "The operation to perform"
                    }
                },
                "required": ["text", "operation"]
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "result": {
                        "description": "The result of the text processing operation"
                    },
                    "original_length": {
                        "type": "number",
                        "description": "Length of the original text"
                    }
                }
            }),
            parallel_safe: true,
            estimated_duration: Some(Duration::from_millis(100)),
            required_permissions: vec![],
            tags: vec!["text".to_string(), "processing".to_string(), "string".to_string()],
            properties: HashMap::new(),
        };

        Self { metadata }
    }
}

impl Default for TextProcessingTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for TextProcessingTool {
    async fn execute(&self, input: ToolInput) -> CoreResult<ToolOutput> {
        let start_time = Instant::now();

        // Extract parameters
        let text: String = input.get_param("text")?;
        let operation: String = input.get_param("operation")?;

        let original_length = text.len();

        let result = match operation.as_str() {
            "uppercase" => serde_json::Value::String(text.to_uppercase()),
            "lowercase" => serde_json::Value::String(text.to_lowercase()),
            "word_count" => serde_json::Value::Number(
                serde_json::Number::from(text.split_whitespace().count())
            ),
            "char_count" => serde_json::Value::Number(
                serde_json::Number::from(text.chars().count())
            ),
            "reverse" => serde_json::Value::String(text.chars().rev().collect()),
            "trim" => serde_json::Value::String(text.trim().to_string()),
            _ => {
                return Ok(ToolOutput::failure(ToolError::new(
                    "INVALID_OPERATION".to_string(),
                    format!("Unsupported operation: {}", operation),
                )));
            }
        };

        let metrics = ToolMetrics {
            execution_time_ms: start_time.elapsed().as_millis() as u64,
            ..Default::default()
        };

        Ok(ToolOutput::success(Some(serde_json::json!({
            "result": result,
            "original_length": original_length
        }))).with_metrics(metrics))
    }

    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }
}

/// Mathematical operations tool
#[derive(Debug)]
pub struct MathTool {
    metadata: ToolMetadata,
}

impl MathTool {
    /// Create a new math tool
    pub fn new() -> Self {
        let metadata = ToolMetadata {
            id: "math".to_string(),
            name: "Mathematical Operations".to_string(),
            description: "Perform mathematical calculations".to_string(),
            version: "1.0.0".to_string(),
            category: ToolCategory::Math,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["add", "subtract", "multiply", "divide", "power", "sqrt", "sin", "cos", "tan"],
                        "description": "The mathematical operation to perform"
                    },
                    "a": {
                        "type": "number",
                        "description": "First operand"
                    },
                    "b": {
                        "type": "number",
                        "description": "Second operand (not required for unary operations)"
                    }
                },
                "required": ["operation", "a"]
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "result": {
                        "type": "number",
                        "description": "The result of the mathematical operation"
                    }
                }
            }),
            parallel_safe: true,
            estimated_duration: Some(Duration::from_millis(10)),
            required_permissions: vec![],
            tags: vec!["math".to_string(), "calculation".to_string(), "arithmetic".to_string()],
            properties: HashMap::new(),
        };

        Self { metadata }
    }
}

impl Default for MathTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for MathTool {
    async fn execute(&self, input: ToolInput) -> CoreResult<ToolOutput> {
        let start_time = Instant::now();

        // Extract parameters
        let operation: String = input.get_param("operation")?;
        let a: f64 = input.get_param("a")?;
        let b: Option<f64> = input.get_optional_param("b")?;

        let result = match operation.as_str() {
            "add" => {
                let b = b.ok_or_else(|| CoreError::validation_error("Parameter 'b' required for add operation"))?;
                a + b
            }
            "subtract" => {
                let b = b.ok_or_else(|| CoreError::validation_error("Parameter 'b' required for subtract operation"))?;
                a - b
            }
            "multiply" => {
                let b = b.ok_or_else(|| CoreError::validation_error("Parameter 'b' required for multiply operation"))?;
                a * b
            }
            "divide" => {
                let b = b.ok_or_else(|| CoreError::validation_error("Parameter 'b' required for divide operation"))?;
                if b == 0.0 {
                    return Ok(ToolOutput::failure(ToolError::new(
                        "DIVISION_BY_ZERO".to_string(),
                        "Cannot divide by zero".to_string(),
                    )));
                }
                a / b
            }
            "power" => {
                let b = b.ok_or_else(|| CoreError::validation_error("Parameter 'b' required for power operation"))?;
                a.powf(b)
            }
            "sqrt" => {
                if a < 0.0 {
                    return Ok(ToolOutput::failure(ToolError::new(
                        "INVALID_INPUT".to_string(),
                        "Cannot take square root of negative number".to_string(),
                    )));
                }
                a.sqrt()
            }
            "sin" => a.sin(),
            "cos" => a.cos(),
            "tan" => a.tan(),
            _ => {
                return Ok(ToolOutput::failure(ToolError::new(
                    "INVALID_OPERATION".to_string(),
                    format!("Unsupported operation: {}", operation),
                )));
            }
        };

        let metrics = ToolMetrics {
            execution_time_ms: start_time.elapsed().as_millis() as u64,
            ..Default::default()
        };

        Ok(ToolOutput::success(Some(serde_json::json!({
            "result": result
        }))).with_metrics(metrics))
    }

    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }
}

// Re-export additional tool modules
pub mod database;
pub mod file;

pub use database::*;
pub use file::*;

/// Create a registry with all built-in tools
pub fn create_builtin_registry() -> CoreResult<crate::core::ToolRegistry> {
    let mut registry = crate::core::ToolRegistry::new();

    // Register built-in tools
    registry.register(Box::new(HttpRequestTool::new()))?;
    registry.register(Box::new(TextProcessingTool::new()))?;
    registry.register(Box::new(MathTool::new()))?;
    
    // Database tools
    registry.register(Box::new(SqlQueryTool::new()))?;
    registry.register(Box::new(JsonQueryTool::new()))?;
    registry.register(Box::new(CsvQueryTool::new()))?;
    
    // File system tools
    registry.register(Box::new(FileReadTool::new()))?;
    registry.register(Box::new(FileWriteTool::new()))?;
    registry.register(Box::new(DirectoryListTool::new()))?;

    Ok(registry)
}