// HTTP tools for making web requests

use crate::tools::traits::{Tool, ToolError, ToolInput, ToolMetadata, ToolOutput, ToolResult, CacheableTool};
use async_trait::async_trait;
// use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;

/// HTTP GET tool for making GET requests
#[derive(Debug)]
pub struct HttpGetTool {
    metadata: ToolMetadata,
    client: reqwest::Client,
}

impl HttpGetTool {
    /// Create a new HTTP GET tool
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();
            
        let metadata = ToolMetadata::new(
            "http_get",
            "HTTP GET",
            "Make HTTP GET requests to retrieve data from web APIs"
        )
        .with_tag("http")
        .with_tag("network")
        .with_tag("api")
        .with_deterministic(false)
        .with_side_effects(false)
        .with_estimated_duration_ms(1000);
        
        Self { metadata, client }
    }
}

#[async_trait]
impl Tool for HttpGetTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(&self, input: ToolInput) -> ToolResult<ToolOutput> {
        // Extract URL from input
        let url = input.data.as_str()
            .or_else(|| input.data.get("url").and_then(|v| v.as_str()))
            .ok_or_else(|| ToolError::ValidationError {
                message: "URL is required in input data".to_string(),
            })?;

        // Extract headers if provided
        let headers = input.get_parameter::<HashMap<String, String>>("headers")
            .unwrap_or_default();

        // Build request
        let mut request = self.client.get(url);
        for (key, value) in headers {
            request = request.header(&key, &value);
        }

        // Execute request
        let response = request.send().await
            .map_err(|e| ToolError::NetworkError {
                message: format!("HTTP request failed: {}", e),
            })?;

        let status = response.status();
        let headers_map: HashMap<String, String> = response.headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let body = response.text().await
            .map_err(|e| ToolError::NetworkError {
                message: format!("Failed to read response body: {}", e),
            })?;

        // Try to parse as JSON, fallback to text
        let parsed_body = serde_json::from_str::<Value>(&body)
            .unwrap_or_else(|_| Value::String(body));

        let output = ToolOutput::new(json!({
            "status": status.as_u16(),
            "headers": headers_map,
            "body": parsed_body
        }))
        .with_metadata("url", url)
        .with_metadata("method", "GET")
        .with_metric("status_code", status.as_u16() as f64);

        Ok(output)
    }

    async fn validate_input(&self, input: &ToolInput) -> ToolResult<()> {
        // Check if URL is provided
        let url = input.data.as_str()
            .or_else(|| input.data.get("url").and_then(|v| v.as_str()))
            .ok_or_else(|| ToolError::ValidationError {
                message: "URL is required".to_string(),
            })?;

        // Basic URL validation
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(ToolError::ValidationError {
                message: "URL must start with http:// or https://".to_string(),
            });
        }

        Ok(())
    }
}

impl CacheableTool for HttpGetTool {
    fn cache_key(&self, input: &ToolInput) -> String {
        let url = input.data.as_str()
            .or_else(|| input.data.get("url").and_then(|v| v.as_str()))
            .unwrap_or("");
        let headers = input.get_parameter::<HashMap<String, String>>("headers")
            .unwrap_or_default();
        
        format!("http_get:{}:{:?}", url, headers)
    }

    fn should_cache(&self, _input: &ToolInput) -> bool {
        true // Cache GET requests by default
    }

    fn cache_ttl(&self) -> Option<u64> {
        Some(300) // 5 minutes
    }
}

/// HTTP POST tool for making POST requests
#[derive(Debug)]
pub struct HttpPostTool {
    metadata: ToolMetadata,
    client: reqwest::Client,
}

impl HttpPostTool {
    /// Create a new HTTP POST tool
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();
            
        let metadata = ToolMetadata::new(
            "http_post",
            "HTTP POST",
            "Make HTTP POST requests to send data to web APIs"
        )
        .with_tag("http")
        .with_tag("network")
        .with_tag("api")
        .with_deterministic(false)
        .with_side_effects(true)
        .with_estimated_duration_ms(1500);
        
        Self { metadata, client }
    }
}

#[async_trait]
impl Tool for HttpPostTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(&self, input: ToolInput) -> ToolResult<ToolOutput> {
        // Extract URL and body from input
        let url = input.get_parameter::<String>("url")
            .ok_or_else(|| ToolError::ValidationError {
                message: "URL parameter is required".to_string(),
            })?;

        let body = input.data.clone();
        let headers = input.get_parameter::<HashMap<String, String>>("headers")
            .unwrap_or_default();

        // Build request
        let mut request = self.client.post(&url);
        
        // Add headers
        for (key, value) in headers {
            request = request.header(&key, &value);
        }

        // Add body
        request = request.json(&body);

        // Execute request
        let response = request.send().await
            .map_err(|e| ToolError::NetworkError {
                message: format!("HTTP request failed: {}", e),
            })?;

        let status = response.status();
        let headers_map: HashMap<String, String> = response.headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let response_body = response.text().await
            .map_err(|e| ToolError::NetworkError {
                message: format!("Failed to read response body: {}", e),
            })?;

        // Try to parse as JSON, fallback to text
        let parsed_body = serde_json::from_str::<Value>(&response_body)
            .unwrap_or_else(|_| Value::String(response_body));

        let output = ToolOutput::new(json!({
            "status": status.as_u16(),
            "headers": headers_map,
            "body": parsed_body
        }))
        .with_metadata("url", &url)
        .with_metadata("method", "POST")
        .with_metric("status_code", status.as_u16() as f64);

        Ok(output)
    }

    async fn validate_input(&self, input: &ToolInput) -> ToolResult<()> {
        // Check if URL parameter is provided
        let url = input.get_parameter::<String>("url")
            .ok_or_else(|| ToolError::ValidationError {
                message: "URL parameter is required".to_string(),
            })?;

        // Basic URL validation
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(ToolError::ValidationError {
                message: "URL must start with http:// or https://".to_string(),
            });
        }

        Ok(())
    }
}

/// HTTP PUT tool for making PUT requests
#[derive(Debug)]
pub struct HttpPutTool {
    metadata: ToolMetadata,
    client: reqwest::Client,
}

impl HttpPutTool {
    /// Create a new HTTP PUT tool
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();
            
        let metadata = ToolMetadata::new(
            "http_put",
            "HTTP PUT",
            "Make HTTP PUT requests to update data via web APIs"
        )
        .with_tag("http")
        .with_tag("network")
        .with_tag("api")
        .with_deterministic(false)
        .with_side_effects(true)
        .with_estimated_duration_ms(1500);
        
        Self { metadata, client }
    }
}

#[async_trait]
impl Tool for HttpPutTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(&self, input: ToolInput) -> ToolResult<ToolOutput> {
        // Similar implementation to POST but with PUT method
        let url = input.get_parameter::<String>("url")
            .ok_or_else(|| ToolError::ValidationError {
                message: "URL parameter is required".to_string(),
            })?;

        let body = input.data.clone();
        let headers = input.get_parameter::<HashMap<String, String>>("headers")
            .unwrap_or_default();

        let mut request = self.client.put(&url);
        
        for (key, value) in headers {
            request = request.header(&key, &value);
        }

        request = request.json(&body);

        let response = request.send().await
            .map_err(|e| ToolError::NetworkError {
                message: format!("HTTP request failed: {}", e),
            })?;

        let status = response.status();
        let headers_map: HashMap<String, String> = response.headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let response_body = response.text().await
            .map_err(|e| ToolError::NetworkError {
                message: format!("Failed to read response body: {}", e),
            })?;

        let parsed_body = serde_json::from_str::<Value>(&response_body)
            .unwrap_or_else(|_| Value::String(response_body));

        let output = ToolOutput::new(json!({
            "status": status.as_u16(),
            "headers": headers_map,
            "body": parsed_body
        }))
        .with_metadata("url", &url)
        .with_metadata("method", "PUT")
        .with_metric("status_code", status.as_u16() as f64);

        Ok(output)
    }

    async fn validate_input(&self, input: &ToolInput) -> ToolResult<()> {
        let url = input.get_parameter::<String>("url")
            .ok_or_else(|| ToolError::ValidationError {
                message: "URL parameter is required".to_string(),
            })?;

        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(ToolError::ValidationError {
                message: "URL must start with http:// or https://".to_string(),
            });
        }

        Ok(())
    }
}

/// HTTP DELETE tool for making DELETE requests
#[derive(Debug)]
pub struct HttpDeleteTool {
    metadata: ToolMetadata,
    client: reqwest::Client,
}

impl HttpDeleteTool {
    /// Create a new HTTP DELETE tool
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();
            
        let metadata = ToolMetadata::new(
            "http_delete",
            "HTTP DELETE",
            "Make HTTP DELETE requests to remove data via web APIs"
        )
        .with_tag("http")
        .with_tag("network")
        .with_tag("api")
        .with_deterministic(false)
        .with_side_effects(true)
        .with_estimated_duration_ms(1000);
        
        Self { metadata, client }
    }
}

#[async_trait]
impl Tool for HttpDeleteTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(&self, input: ToolInput) -> ToolResult<ToolOutput> {
        let url = if let Some(url_str) = input.data.as_str() {
            url_str.to_string()
        } else if let Some(url_param) = input.get_parameter::<String>("url") {
            url_param
        } else {
            return Err(ToolError::ValidationError {
                message: "URL is required".to_string(),
            });
        };

        let headers = input.get_parameter::<HashMap<String, String>>("headers")
            .unwrap_or_default();

        let mut request = self.client.delete(&url);
        
        for (key, value) in headers {
            request = request.header(&key, &value);
        }

        let response = request.send().await
            .map_err(|e| ToolError::NetworkError {
                message: format!("HTTP request failed: {}", e),
            })?;

        let status = response.status();
        let headers_map: HashMap<String, String> = response.headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let response_body = response.text().await
            .map_err(|e| ToolError::NetworkError {
                message: format!("Failed to read response body: {}", e),
            })?;

        let parsed_body = serde_json::from_str::<Value>(&response_body)
            .unwrap_or_else(|_| Value::String(response_body));

        let output = ToolOutput::new(json!({
            "status": status.as_u16(),
            "headers": headers_map,
            "body": parsed_body
        }))
        .with_metadata("url", &url)
        .with_metadata("method", "DELETE")
        .with_metric("status_code", status.as_u16() as f64);

        Ok(output)
    }

    async fn validate_input(&self, input: &ToolInput) -> ToolResult<()> {
        let url = if let Some(url_str) = input.data.as_str() {
            url_str.to_string()
        } else if let Some(url_param) = input.get_parameter::<String>("url") {
            url_param
        } else {
            return Err(ToolError::ValidationError {
                message: "URL is required".to_string(),
            });
        };

        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(ToolError::ValidationError {
                message: "URL must start with http:// or https://".to_string(),
            });
        }

        Ok(())
    }
}
