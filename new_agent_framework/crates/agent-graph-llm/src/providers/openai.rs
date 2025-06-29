//! OpenAI provider implementation for GPT models.

use crate::{CoreError, CoreResult};
use crate::types::{CompletionRequest, CompletionResponse, Message, MessageRole, Choice, Usage, FinishReason, ModelInfo, ModelCapability, ModelPricing, FunctionCallBehavior};
use crate::providers::{LLMProvider, ProviderConfig};
use async_trait::async_trait;
use reqwest::{Client, header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE}};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// OpenAI provider for GPT models
#[derive(Debug)]
pub struct OpenAIProvider {
    config: OpenAIConfig,
    client: Client,
}

/// Configuration for OpenAI provider
#[derive(Debug, Clone)]
pub struct OpenAIConfig {
    pub api_key: String,
    pub endpoint: String,
    pub timeout_ms: u64,
    pub max_retries: u32,
    pub organization: Option<String>,
}

impl Default for OpenAIConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default(),
            endpoint: "https://api.openai.com/v1".to_string(),
            timeout_ms: 60000,
            max_retries: 3,
            organization: std::env::var("OPENAI_ORG_ID").ok(),
        }
    }
}

impl ProviderConfig for OpenAIConfig {
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

/// OpenAI API request format
#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
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
    functions: Option<Vec<OpenAIFunction>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    function_call: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

/// OpenAI message format
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    function_call: Option<OpenAIFunctionCall>,
}

/// OpenAI function call format
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIFunctionCall {
    name: String,
    arguments: String,
}

/// OpenAI function definition format
#[derive(Debug, Serialize)]
struct OpenAIFunction {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

/// OpenAI API response format
#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

/// OpenAI choice format
#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    index: u32,
    message: OpenAIMessage,
    finish_reason: Option<String>,
}

/// OpenAI usage format
#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// OpenAI models list response
#[derive(Debug, Deserialize)]
struct OpenAIModelsResponse {
    data: Vec<OpenAIModel>,
}

/// OpenAI model information
#[derive(Debug, Deserialize)]
struct OpenAIModel {
    id: String,
    object: String,
    owned_by: String,
}

impl OpenAIProvider {
    /// Create a new OpenAI provider
    pub fn new(config: OpenAIConfig) -> CoreResult<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", config.api_key))
                .map_err(|e| CoreError::configuration_error(format!("Invalid API key: {}", e)))?
        );

        // Add organization header if provided
        if let Some(ref org) = config.organization {
            headers.insert(
                "OpenAI-Organization",
                HeaderValue::from_str(org)
                    .map_err(|e| CoreError::configuration_error(format!("Invalid organization: {}", e)))?
            );
        }

        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_millis(config.timeout_ms))
            .build()
            .map_err(|e| CoreError::configuration_error(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { config, client })
    }

    /// Convert our message format to OpenAI format
    fn convert_messages(&self, messages: &[Message]) -> Vec<OpenAIMessage> {
        messages
            .iter()
            .map(|message| {
                let role = match message.role {
                    MessageRole::System => "system",
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                    MessageRole::Function => "function",
                };

                let function_call = message.function_call.as_ref().map(|fc| OpenAIFunctionCall {
                    name: fc.name.clone(),
                    arguments: fc.arguments.clone(),
                });

                OpenAIMessage {
                    role: role.to_string(),
                    content: message.content.clone(),
                    name: message.name.clone(),
                    function_call,
                }
            })
            .collect()
    }

    /// Convert our function definitions to OpenAI format
    fn convert_functions(&self, functions: &[crate::types::FunctionDefinition]) -> Vec<OpenAIFunction> {
        functions
            .iter()
            .map(|f| OpenAIFunction {
                name: f.name.clone(),
                description: f.description.clone(),
                parameters: f.parameters.clone(),
            })
            .collect()
    }

    /// Convert function call behavior to OpenAI format
    fn convert_function_call_behavior(&self, behavior: &FunctionCallBehavior) -> serde_json::Value {
        match behavior {
            FunctionCallBehavior::None => serde_json::Value::String("none".to_string()),
            FunctionCallBehavior::Auto => serde_json::Value::String("auto".to_string()),
            FunctionCallBehavior::Force { name } => serde_json::json!({"name": name}),
        }
    }

    /// Convert OpenAI response to our format
    fn convert_response(&self, openai_response: OpenAIResponse) -> CompletionResponse {
        let choices = openai_response
            .choices
            .into_iter()
            .map(|choice| {
                let finish_reason = choice.finish_reason.as_deref().map(|reason| {
                    match reason {
                        "stop" => FinishReason::Stop,
                        "length" => FinishReason::Length,
                        "function_call" => FinishReason::FunctionCall,
                        "content_filter" => FinishReason::ContentFilter,
                        other => FinishReason::Other(other.to_string()),
                    }
                });

                let message = if let Some(function_call) = choice.message.function_call {
                    Message::assistant_with_function_call(crate::types::FunctionCall {
                        name: function_call.name,
                        arguments: function_call.arguments,
                    })
                } else {
                    Message::assistant(choice.message.content.unwrap_or_default())
                };

                Choice {
                    index: choice.index,
                    message,
                    finish_reason,
                    logprobs: None,
                }
            })
            .collect();

        CompletionResponse {
            id: openai_response.id,
            object: openai_response.object,
            created: openai_response.created,
            model: openai_response.model,
            choices,
            usage: Usage::new(
                openai_response.usage.prompt_tokens,
                openai_response.usage.completion_tokens,
            ),
            metadata: HashMap::new(),
        }
    }

    /// Get model pricing information
    fn get_model_pricing(&self, model_id: &str) -> Option<ModelPricing> {
        match model_id {
            "gpt-4" | "gpt-4-0613" => Some(ModelPricing {
                input_cost_per_1k_tokens: Some(0.03),
                output_cost_per_1k_tokens: Some(0.06),
                currency: "USD".to_string(),
            }),
            "gpt-4-32k" | "gpt-4-32k-0613" => Some(ModelPricing {
                input_cost_per_1k_tokens: Some(0.06),
                output_cost_per_1k_tokens: Some(0.12),
                currency: "USD".to_string(),
            }),
            "gpt-4-1106-preview" | "gpt-4-turbo-preview" => Some(ModelPricing {
                input_cost_per_1k_tokens: Some(0.01),
                output_cost_per_1k_tokens: Some(0.03),
                currency: "USD".to_string(),
            }),
            "gpt-3.5-turbo" | "gpt-3.5-turbo-0613" => Some(ModelPricing {
                input_cost_per_1k_tokens: Some(0.0015),
                output_cost_per_1k_tokens: Some(0.002),
                currency: "USD".to_string(),
            }),
            "gpt-3.5-turbo-16k" | "gpt-3.5-turbo-16k-0613" => Some(ModelPricing {
                input_cost_per_1k_tokens: Some(0.003),
                output_cost_per_1k_tokens: Some(0.004),
                currency: "USD".to_string(),
            }),
            _ => None,
        }
    }

    /// Get model capabilities
    fn get_model_info(&self, model: &OpenAIModel) -> Option<ModelInfo> {
        let id = &model.id;
        
        // Only include chat models
        if !id.contains("gpt") || id.contains("instruct") || id.contains("edit") || id.contains("embedding") {
            return None;
        }

        let (max_tokens, supports_functions) = match id.as_str() {
            "gpt-4" | "gpt-4-0613" => (8192, true),
            "gpt-4-32k" | "gpt-4-32k-0613" => (32768, true),
            "gpt-4-1106-preview" | "gpt-4-turbo-preview" => (128000, true),
            "gpt-3.5-turbo" | "gpt-3.5-turbo-0613" => (4096, true),
            "gpt-3.5-turbo-16k" | "gpt-3.5-turbo-16k-0613" => (16384, true),
            "gpt-3.5-turbo-1106" => (16384, true),
            _ => (4096, false),
        };

        let mut capabilities = vec![
            ModelCapability::TextCompletion,
            ModelCapability::ChatCompletion,
        ];

        if supports_functions {
            capabilities.push(ModelCapability::FunctionCalling);
        }

        if id.contains("gpt-4") {
            capabilities.push(ModelCapability::CodeGeneration);
        }

        Some(ModelInfo {
            id: id.clone(),
            name: format!("OpenAI {}", id),
            description: Some(format!("OpenAI {} model", id)),
            max_tokens: Some(max_tokens),
            supports_functions,
            supports_streaming: true,
            capabilities,
            pricing: self.get_model_pricing(id),
        })
    }
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    async fn complete(&self, request: CompletionRequest) -> CoreResult<CompletionResponse> {
        let messages = self.convert_messages(&request.messages);

        let mut openai_request = OpenAIRequest {
            model: request.model.clone(),
            messages,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            top_p: request.top_p,
            frequency_penalty: request.frequency_penalty,
            presence_penalty: request.presence_penalty,
            stop: request.stop,
            functions: None,
            function_call: None,
            stream: Some(false),
        };

        // Add function calling support
        if let Some(ref functions) = request.functions {
            openai_request.functions = Some(self.convert_functions(functions));
            
            if let Some(ref function_call) = request.function_call {
                openai_request.function_call = Some(self.convert_function_call_behavior(function_call));
            }
        }

        let url = format!("{}/chat/completions", self.config.endpoint);
        let response = self.client
            .post(&url)
            .json(&openai_request)
            .send()
            .await
            .map_err(|e| CoreError::execution_error(format!("OpenAI request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(CoreError::execution_error(format!(
                "OpenAI API error: {} - {}",
                response.status(),
                error_text
            )));
        }

        let openai_response: OpenAIResponse = response
            .json()
            .await
            .map_err(|e| CoreError::execution_error(format!("Failed to parse OpenAI response: {}", e)))?;

        Ok(self.convert_response(openai_response))
    }

    async fn get_models(&self) -> CoreResult<Vec<ModelInfo>> {
        let url = format!("{}/models", self.config.endpoint);
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| CoreError::execution_error(format!("OpenAI request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(CoreError::execution_error(format!(
                "OpenAI API error: {} - {}",
                response.status(),
                error_text
            )));
        }

        let models_response: OpenAIModelsResponse = response
            .json()
            .await
            .map_err(|e| CoreError::execution_error(format!("Failed to parse OpenAI response: {}", e)))?;

        let models = models_response
            .data
            .iter()
            .filter_map(|model| self.get_model_info(model))
            .collect();

        Ok(models)
    }

    fn name(&self) -> &str {
        "openai"
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
            return Err(CoreError::configuration_error("OpenAI API key is required"));
        }
        Ok(())
    }
}