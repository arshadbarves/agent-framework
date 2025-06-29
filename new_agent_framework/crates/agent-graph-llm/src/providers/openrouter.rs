//! OpenRouter provider implementation for accessing multiple LLM models.

use crate::{CoreError, CoreResult};
use crate::types::{CompletionRequest, CompletionResponse, Message, MessageRole, Choice, Usage, FinishReason, ModelInfo, ModelCapability, ModelPricing};
use crate::providers::{LLMProvider, ProviderConfig};
use async_trait::async_trait;
use reqwest::{Client, header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE}};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// OpenRouter provider for accessing multiple LLM models
#[derive(Debug)]
pub struct OpenRouterProvider {
    config: OpenRouterConfig,
    client: Client,
}

/// Configuration for OpenRouter provider
#[derive(Debug, Clone)]
pub struct OpenRouterConfig {
    pub api_key: String,
    pub endpoint: String,
    pub timeout_ms: u64,
    pub max_retries: u32,
    pub app_name: Option<String>,
    pub site_url: Option<String>,
}

impl Default for OpenRouterConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("OPENROUTER_API_KEY").unwrap_or_default(),
            endpoint: "https://openrouter.ai/api/v1".to_string(),
            timeout_ms: 120000, // 2 minutes for longer model responses
            max_retries: 3,
            app_name: Some("AgentGraph".to_string()),
            site_url: Some("https://github.com/agent-graph/agent-graph".to_string()),
        }
    }
}

impl ProviderConfig for OpenRouterConfig {
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

/// OpenRouter API request format (OpenAI-compatible)
#[derive(Debug, Serialize)]
struct OpenRouterRequest {
    model: String,
    messages: Vec<OpenRouterMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

/// OpenRouter message format
#[derive(Debug, Serialize, Deserialize)]
struct OpenRouterMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

/// OpenRouter API response format
#[derive(Debug, Deserialize)]
struct OpenRouterResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<OpenRouterChoice>,
    usage: OpenRouterUsage,
}

/// OpenRouter choice format
#[derive(Debug, Deserialize)]
struct OpenRouterChoice {
    index: u32,
    message: OpenRouterMessage,
    finish_reason: Option<String>,
}

/// OpenRouter usage format
#[derive(Debug, Deserialize)]
struct OpenRouterUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// OpenRouter models list response
#[derive(Debug, Deserialize)]
struct OpenRouterModelsResponse {
    data: Vec<OpenRouterModel>,
}

/// OpenRouter model information
#[derive(Debug, Deserialize)]
struct OpenRouterModel {
    id: String,
    name: Option<String>,
    description: Option<String>,
    context_length: Option<u32>,
    pricing: Option<OpenRouterModelPricing>,
    top_provider: Option<OpenRouterTopProvider>,
}

/// OpenRouter model pricing
#[derive(Debug, Deserialize)]
struct OpenRouterModelPricing {
    prompt: Option<String>,
    completion: Option<String>,
}

/// OpenRouter top provider info
#[derive(Debug, Deserialize)]
struct OpenRouterTopProvider {
    max_completion_tokens: Option<u32>,
}

impl OpenRouterProvider {
    /// Create a new OpenRouter provider
    pub fn new(config: OpenRouterConfig) -> CoreResult<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", config.api_key))
                .map_err(|e| CoreError::configuration_error(format!("Invalid API key: {}", e)))?
        );

        // Add optional headers for OpenRouter analytics
        if let Some(ref app_name) = config.app_name {
            headers.insert(
                "HTTP-Referer",
                HeaderValue::from_str(app_name)
                    .map_err(|e| CoreError::configuration_error(format!("Invalid app name: {}", e)))?
            );
        }

        if let Some(ref site_url) = config.site_url {
            headers.insert(
                "X-Title",
                HeaderValue::from_str(site_url)
                    .map_err(|e| CoreError::configuration_error(format!("Invalid site URL: {}", e)))?
            );
        }

        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_millis(config.timeout_ms))
            .build()
            .map_err(|e| CoreError::configuration_error(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { config, client })
    }

    /// Convert our message format to OpenRouter format
    fn convert_messages(&self, messages: &[Message]) -> Vec<OpenRouterMessage> {
        messages
            .iter()
            .filter_map(|message| {
                message.content.as_ref().map(|content| {
                    let role = match message.role {
                        MessageRole::System => "system",
                        MessageRole::User => "user",
                        MessageRole::Assistant => "assistant",
                        MessageRole::Function => "user", // Convert function results to user messages
                    };

                    let content = if message.role == MessageRole::Function {
                        format!(
                            "Function result from {}: {}",
                            message.name.as_deref().unwrap_or("unknown"),
                            content
                        )
                    } else {
                        content.clone()
                    };

                    OpenRouterMessage {
                        role: role.to_string(),
                        content,
                        name: message.name.clone(),
                    }
                })
            })
            .collect()
    }

    /// Convert OpenRouter response to our format
    fn convert_response(&self, openrouter_response: OpenRouterResponse) -> CompletionResponse {
        let choices = openrouter_response
            .choices
            .into_iter()
            .map(|choice| {
                let finish_reason = match choice.finish_reason.as_deref() {
                    Some("stop") => Some(FinishReason::Stop),
                    Some("length") => Some(FinishReason::Length),
                    Some("content_filter") => Some(FinishReason::ContentFilter),
                    Some("function_call") => Some(FinishReason::FunctionCall),
                    _ => None,
                };

                Choice {
                    index: choice.index,
                    message: Message::assistant(choice.message.content),
                    finish_reason,
                    logprobs: None,
                }
            })
            .collect();

        CompletionResponse {
            id: openrouter_response.id,
            object: openrouter_response.object,
            created: openrouter_response.created,
            model: openrouter_response.model,
            choices,
            usage: Usage::new(
                openrouter_response.usage.prompt_tokens,
                openrouter_response.usage.completion_tokens,
            ),
            metadata: HashMap::new(),
        }
    }

    /// Parse pricing string to float
    fn parse_pricing(&self, pricing_str: &str) -> Option<f64> {
        // OpenRouter pricing is in format like "0.000002" (per token)
        // Convert to per 1K tokens
        pricing_str.parse::<f64>().ok().map(|price| price * 1000.0)
    }
}

#[async_trait]
impl LLMProvider for OpenRouterProvider {
    async fn complete(&self, request: CompletionRequest) -> CoreResult<CompletionResponse> {
        let messages = self.convert_messages(&request.messages);

        let openrouter_request = OpenRouterRequest {
            model: request.model.clone(),
            messages,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            top_p: request.top_p,
            frequency_penalty: request.frequency_penalty,
            presence_penalty: request.presence_penalty,
            stop: request.stop,
            stream: Some(false),
        };

        let url = format!("{}/chat/completions", self.config.endpoint);
        let response = self.client
            .post(&url)
            .json(&openrouter_request)
            .send()
            .await
            .map_err(|e| CoreError::execution_error(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(CoreError::execution_error(format!(
                "OpenRouter API error: {} - {}",
                response.status(),
                error_text
            )));
        }

        let openrouter_response: OpenRouterResponse = response
            .json()
            .await
            .map_err(|e| CoreError::execution_error(format!("Failed to parse response: {}", e)))?;

        Ok(self.convert_response(openrouter_response))
    }

    async fn get_models(&self) -> CoreResult<Vec<ModelInfo>> {
        let url = format!("{}/models", self.config.endpoint);
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| CoreError::execution_error(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(CoreError::execution_error(format!(
                "OpenRouter API error: {} - {}",
                response.status(),
                error_text
            )));
        }

        let models_response: OpenRouterModelsResponse = response
            .json()
            .await
            .map_err(|e| CoreError::execution_error(format!("Failed to parse response: {}", e)))?;

        let models = models_response
            .data
            .into_iter()
            .map(|model| {
                let pricing = model.pricing.and_then(|p| {
                    let input_cost = p.prompt.and_then(|price| self.parse_pricing(&price));
                    let output_cost = p.completion.and_then(|price| self.parse_pricing(&price));
                    
                    if input_cost.is_some() || output_cost.is_some() {
                        Some(ModelPricing {
                            input_cost_per_1k_tokens: input_cost,
                            output_cost_per_1k_tokens: output_cost,
                            currency: "USD".to_string(),
                        })
                    } else {
                        None
                    }
                });

                ModelInfo {
                    id: model.id.clone(),
                    name: model.name.unwrap_or_else(|| model.id.clone()),
                    description: model.description,
                    max_tokens: model.context_length,
                    supports_functions: true, // Most models on OpenRouter support functions
                    supports_streaming: true,
                    capabilities: vec![
                        ModelCapability::TextCompletion,
                        ModelCapability::ChatCompletion,
                        ModelCapability::FunctionCalling,
                    ],
                    pricing,
                }
            })
            .collect();

        Ok(models)
    }

    fn name(&self) -> &str {
        "openrouter"
    }

    fn supports_function_calling(&self) -> bool {
        true
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn config(&self) -> &dyn ProviderConfig {
        &self.config
    }

    async fn validate_config(&self) -> CoreResult<()> {
        if self.config.api_key.is_empty() {
            return Err(CoreError::configuration_error("OpenRouter API key is required"));
        }
        Ok(())
    }
}