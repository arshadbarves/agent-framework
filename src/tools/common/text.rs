// Text processing tools

use crate::tools::traits::{Tool, ToolError, ToolInput, ToolMetadata, ToolOutput, ToolResult};
use async_trait::async_trait;
use serde_json::json;

/// Tool for processing text
#[derive(Debug)]
pub struct TextProcessorTool {
    metadata: ToolMetadata,
}

impl TextProcessorTool {
    /// Create a new text processor tool
    pub fn new() -> Self {
        let metadata = ToolMetadata::new(
            "text_processor",
            "Text Processor",
            "Process and transform text data"
        )
        .with_tag("text")
        .with_tag("processing")
        .with_tag("utility")
        .with_deterministic(true)
        .with_side_effects(false)
        .with_estimated_duration_ms(50);
        
        Self { metadata }
    }
}

#[async_trait]
impl Tool for TextProcessorTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(&self, input: ToolInput) -> ToolResult<ToolOutput> {
        let text = input.data.as_str()
            .ok_or_else(|| ToolError::ValidationError {
                message: "Text input is required".to_string(),
            })?;

        let operation = input.get_parameter::<String>("operation")
            .unwrap_or_else(|| "identity".to_string());

        let result = match operation.as_str() {
            "uppercase" => text.to_uppercase(),
            "lowercase" => text.to_lowercase(),
            "trim" => text.trim().to_string(),
            "reverse" => text.chars().rev().collect(),
            _ => text.to_string(),
        };

        let output = ToolOutput::new(json!({
            "original": text,
            "result": result,
            "operation": operation,
            "length": result.len()
        }))
        .with_metadata("operation", &operation)
        .with_metric("input_length", text.len() as f64)
        .with_metric("output_length", result.len() as f64);

        Ok(output)
    }

    async fn validate_input(&self, input: &ToolInput) -> ToolResult<()> {
        if input.data.as_str().is_none() {
            return Err(ToolError::ValidationError {
                message: "Text input is required".to_string(),
            });
        }
        Ok(())
    }
}

/// Tool for regex operations
#[derive(Debug)]
pub struct RegexTool {
    metadata: ToolMetadata,
}

impl RegexTool {
    /// Create a new regex tool
    pub fn new() -> Self {
        let metadata = ToolMetadata::new(
            "regex",
            "Regex Tool",
            "Perform regular expression operations on text"
        )
        .with_tag("text")
        .with_tag("regex")
        .with_tag("pattern")
        .with_deterministic(true)
        .with_side_effects(false)
        .with_estimated_duration_ms(100);
        
        Self { metadata }
    }
}

#[async_trait]
impl Tool for RegexTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(&self, input: ToolInput) -> ToolResult<ToolOutput> {
        let text = input.data.as_str()
            .ok_or_else(|| ToolError::ValidationError {
                message: "Text input is required".to_string(),
            })?;

        let pattern = input.get_parameter::<String>("pattern")
            .ok_or_else(|| ToolError::ValidationError {
                message: "Regex pattern parameter is required".to_string(),
            })?;

        // Stub implementation - in a real implementation, this would use the regex crate
        let output = ToolOutput::new(json!({
            "text": text,
            "pattern": pattern,
            "matches": [],
            "match_count": 0
        }))
        .with_metadata("pattern", &pattern)
        .with_metric("match_count", 0.0);

        Ok(output)
    }

    async fn validate_input(&self, input: &ToolInput) -> ToolResult<()> {
        if input.data.as_str().is_none() {
            return Err(ToolError::ValidationError {
                message: "Text input is required".to_string(),
            });
        }

        if input.get_parameter::<String>("pattern").is_none() {
            return Err(ToolError::ValidationError {
                message: "Regex pattern parameter is required".to_string(),
            });
        }

        Ok(())
    }
}

/// Tool for template rendering
#[derive(Debug)]
pub struct TemplateRenderTool {
    metadata: ToolMetadata,
}

impl TemplateRenderTool {
    /// Create a new template render tool
    pub fn new() -> Self {
        let metadata = ToolMetadata::new(
            "template_render",
            "Template Renderer",
            "Render templates with variable substitution"
        )
        .with_tag("text")
        .with_tag("template")
        .with_tag("rendering")
        .with_deterministic(true)
        .with_side_effects(false)
        .with_estimated_duration_ms(100);
        
        Self { metadata }
    }
}

#[async_trait]
impl Tool for TemplateRenderTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(&self, input: ToolInput) -> ToolResult<ToolOutput> {
        let template = input.data.as_str()
            .ok_or_else(|| ToolError::ValidationError {
                message: "Template string is required".to_string(),
            })?;

        let variables = input.get_parameter::<serde_json::Map<String, serde_json::Value>>("variables")
            .unwrap_or_default();

        let variables_count = variables.len();

        // Simple template rendering - replace {{variable}} with values
        let mut result = template.to_string();
        for (key, value) in &variables {
            let placeholder = format!("{{{{{}}}}}", key);
            let replacement = match value {
                serde_json::Value::String(s) => s.clone(),
                _ => value.to_string(),
            };
            result = result.replace(&placeholder, &replacement);
        }

        let output = ToolOutput::new(json!({
            "template": template,
            "result": result,
            "variables_used": variables_count
        }))
        .with_metadata("template", template)
        .with_metric("variables_count", variables_count as f64);

        Ok(output)
    }

    async fn validate_input(&self, input: &ToolInput) -> ToolResult<()> {
        if input.data.as_str().is_none() {
            return Err(ToolError::ValidationError {
                message: "Template string is required".to_string(),
            });
        }
        Ok(())
    }
}
