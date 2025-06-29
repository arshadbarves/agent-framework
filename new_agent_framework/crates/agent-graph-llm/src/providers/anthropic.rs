//! Anthropic provider implementation for Claude models.

use crate::{CoreError, CoreResult};
use crate::types::{CompletionRequest, CompletionResponse, Message, MessageRole, Choice, Usage, FinishReason, ModelInfo, ModelCapability, ModelPricing};
use crate::providers::{LLMProvider, ProviderConfig};
use async_trait::async_trait;
use reqwest::{Client, header::{HeaderMap, HeaderValue, CONTENT_TYPE}};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Anthropic provider for Claude models
#[derive(Debug)]
pub struct AnthropicProvider {
    config: AnthropicConfig,
    client: Client,
}

/// Configuration for Anthropic provider
#[derive(Debug, Clone)]
pub struct AnthropicConfig {
    pub api_key: String,
    pub endpoint: String,
    pub timeout_ms: u64,
    pub max_retries: u32,
    pub anthropic_version: String,
}

impl Default for AnthropicConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("ANTHROPIC_API_KEY").unwrap_or_default(),
            endpoint: "https://api.anthropic.com".to_string(),
            timeout_ms: 60000,
            max_retries: 3,
            anthropic_version: "2023-06-01".to_string(),
        }
    }
}

impl ProviderConfig for AnthropicConfig {
    fn endpoint(&self) -> &str {
        &self.endpoint
    }

    fn api_key(&self) -> Option<&str> {
        Some(&self.api_key)
    }

    fn timeout_ms(&self) -> u64 {
        self.timeout_ms
    }

    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

/// Anthropic API request format
#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

/// Anthropic message format
#[derive(Debug, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

/// Anthropic API response format
#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    id: String,
    #[serde(rename = "type")]
    response_type: String,
    role: String,
    content: Vec<AnthropicContent>,
    model: String,
    stop_reason: Option<String>,
    stop_sequence: Option<String>,
    usage: AnthropicUsage,
}

/// Anthropic content block
#[derive(Debug, Deserialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

/// Anthropic usage information
#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider
    pub fn new(config: AnthropicConfig) -> CoreResult<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            "x-api-key",
            HeaderValue::from_str(&config.api_key)
                .map_err(|e| CoreError::configuration_error(format!("Invalid API key: {}", e)))?
        );
        headers.insert(
            "anthropic-version",
            HeaderValue::from_str(&config.anthropic_version)
                .map_err(|e| CoreError::configuration_error(format!("Invalid version: {}", e)))?
        );

        let client = Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_millis(config.timeout_ms))
            .build()
            .map_err(|e| CoreError::configuration_error(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { config, client })
    }

    /// Convert our message format to Anthropic format
    fn convert_messages(&self, messages: &[Message]) -> (Option<String>, Vec<AnthropicMessage>) {
        let mut system_message = None;
        let mut anthropic_messages = Vec::new();

        for message in messages {
            match message.role {
                MessageRole::System => {
                    if let Some(content) = &message.content {
                        system_message = Some(content.clone());
                    }
                }
                MessageRole::User => {
                    if let Some(content) = &message.content {
                        anthropic_messages.push(AnthropicMessage {
                            role: "user".to_string(),
                            content: content.clone(),
                        });
                    }
                }
                MessageRole::Assistant => {
                    if let Some(content) = &message.content {
                        anthropic_messages.push(AnthropicMessage {
                            role: "assistant".to_string(),
                            content: content.clone(),
                        });
                    }
                }
                MessageRole::Function => {
                    // Anthropic doesn't support function messages directly
                    // Convert to user message with context
                    if let Some(content) = &message.content {
                        let function_content = format!(
                            "Function result from {}: {}",
                            message.name.as_deref().unwrap_or("unknown"),
                            content
                        );
                        anthropic_messages.push(AnthropicMessage {
                            role: "user".to_string(),
                            content: function_content,
                        });
                    }
                }
            }
        }

        (system_message, anthropic_messages)
    }

    /// Convert Anthropic response to our format
    fn convert_response(&self, anthropic_response: AnthropicResponse, model: String) -> CompletionResponse {
        let content = anthropic_response.content
            .into_iter()
            .map(|c| c.text)
            .collect::<Vec<_>>()
            .join("");

        let finish_reason = match anthropic_response.stop_reason.as_deref() {
            Some("end_turn") => Some(FinishReason::Stop),
            Some("max_tokens") => Some(FinishReason::Length),
            Some("stop_sequence") => Some(FinishReason::Stop),
            _ => None,
        };

        CompletionResponse {
            id: anthropic_response.id,
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp() as u64,
            model,
            choices: vec![Choice {
                index: 0,
                message: Message::assistant(content),
                finish_reason,
                logprobs: None,
            }],
            usage: Usage::new(
                anthropic_response.usage.input_tokens,
                anthropic_response.usage.output_tokens,
            ),
            metadata: HashMap::new(),
        }
    }
}

#[async_trait]
impl LLMProvider for AnthropicProvider {
    async fn complete(&self, request: CompletionRequest) -> CoreResult<CompletionResponse> {
        let (system, messages) = self.convert_messages(&request.messages);

        let anthropic_request = AnthropicRequest {
            model: request.model.clone(),
            max_tokens: request.max_tokens.unwrap_or(4096),
            messages,
            system,
            temperature: request.temperature,
            top_p: request.top_p,
            stop_sequences: request.stop,
            stream: Some(false),
        };

        let url = format!("{}/v1/messages", self.config.endpoint);
        let response = self.client
            .post(&url)
            .json(&anthropic_request)
            .send()
            .await
            .map_err(|e| CoreError::execution_error(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(CoreError::execution_error(format!(
                "Anthropic API error: {} - {}",
                response.status(),
                error_text
            )));
        }

        let anthropic_response: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| CoreError::execution_error(format!("Failed to parse response: {}", e)))?;

        Ok(self.convert_response(anthropic_response, request.model))
    }

    async fn get_models(&self) -> CoreResult<Vec<ModelInfo>> {
        // Anthropic doesn't have a models endpoint, so we return known models
        Ok(vec![
            ModelInfo {
                id: "claude-3-opus-20240229".to_string(),
                name: "Claude 3 Opus".to_string(),
                description: Some("Most powerful model for highly complex tasks".to_string()),
                max_tokens: Some(200000),
                supports_functions: false,
                supports_streaming: true,
                capabilities: vec![
                    ModelCapability::TextCompletion,
                    ModelCapability::ChatCompletion,
                    ModelCapability::CodeGeneration,
                ],
                pricing: Some(ModelPricing {
                    input_cost_per_1k_tokens: Some(0.015),
                    output_cost_per_1k_tokens: Some(0.075),
                    currency: "USD".to_string(),
                }),
            },
            ModelInfo {
                id: "claude-3-sonnet-20240229".to_string(),
                name: "Claude 3 Sonnet".to_string(),
                description: Some("Balance of intelligence and speed".to_string()),
                max_tokens: Some(200000),
                supports_functions: false,
                supports_streaming: true,
                capabilities: vec![
                    ModelCapability::TextCompletion,
                    ModelCapability::ChatCompletion,
                    ModelCapability::CodeGeneration,
                ],
                pricing: Some(ModelPricing {
                    input_cost_per_1k_tokens: Some(0.003),
                    output_cost_per_1k_tokens: Some(0.015),
                    currency: "USD".to_string(),
                }),
            },
            ModelInfo {
                id: "claude-3-haiku-20240307".to_string(),
                name: "Claude 3 Haiku".to_string(),
                description: Some("Fastest and most compact model".to_string()),
                max_tokens: Some(200000),
                supports_functions: false,
                supports_streaming: true,
                capabilities: vec![
                    ModelCapability::TextCompletion,
                    ModelCapability::ChatCompletion,
                ],
                pricing: Some(ModelPricing {
                    input_cost_per_1k_tokens: Some(0.00025),
                    output_cost_per_1k_tokens: Some(0.00125),
                    currency: "USD".to_string(),
                }),
            },
        ])
    }

    fn name(&self) -> &str {
        "anthropic"
    }

    fn supports_function_calling(&self) -> bool {
        false // Anthropic doesn't support function calling yet
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn config(&self) -> &dyn ProviderConfig {
        &self.config
    }

    async fn validate_config(&self) -> CoreResult<()> {
        if self.config.api_key.is_empty() {
            return Err(CoreError::configuration_error("Anthropic API key is required"));
        }
        Ok(())
    }
}