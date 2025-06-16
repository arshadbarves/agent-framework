// OpenRouter LLM provider for AgentGraph
// Provides access to multiple LLM models through OpenRouter's unified API

#![allow(missing_docs)]

use super::{LLMProvider, LLMError};
use crate::llm::{
    CompletionRequest, CompletionResponse, Choice, Message, MessageRole,
    FunctionCall, TokenUsage
};
use async_trait::async_trait;
use reqwest::{Client, header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE}};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;

/// OpenRouter provider configuration
#[derive(Debug, Clone)]
pub struct OpenRouterConfig {
    /// API key for OpenRouter
    pub api_key: String,
    /// Base URL for OpenRouter API
    pub base_url: String,
    /// HTTP client timeout
    pub timeout: Duration,
    /// Your app name (for OpenRouter analytics)
    pub app_name: Option<String>,
    /// Your site URL (for OpenRouter analytics)
    pub site_url: Option<String>,
}

impl Default for OpenRouterConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("OPENROUTER_API_KEY").unwrap_or_default(),
            base_url: "https://openrouter.ai/api/v1".to_string(),
            timeout: Duration::from_secs(120),
            app_name: Some("AgentGraph".to_string()),
            site_url: Some("https://github.com/agent-graph/agent-graph".to_string()),
        }
    }
}

/// OpenRouter LLM provider
#[derive(Debug)]
pub struct OpenRouterProvider {
    config: OpenRouterConfig,
    client: Client,
}

impl OpenRouterProvider {
    /// Create a new OpenRouter provider
    pub fn new(config: OpenRouterConfig) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", config.api_key))
                .expect("Invalid API key format"),
        );
        
        // Add OpenRouter-specific headers
        if let Some(app_name) = &config.app_name {
            headers.insert(
                "HTTP-Referer",
                HeaderValue::from_str(app_name).unwrap_or_else(|_| HeaderValue::from_static("AgentGraph")),
            );
        }
        
        if let Some(site_url) = &config.site_url {
            headers.insert(
                "X-Title",
                HeaderValue::from_str(site_url).unwrap_or_else(|_| HeaderValue::from_static("AgentGraph")),
            );
        }
        
        let client = Client::builder()
            .timeout(config.timeout)
            .default_headers(headers)
            .build()
            .expect("Failed to create HTTP client");
        
        Self { config, client }
    }
    
    /// Get available models from OpenRouter
    pub async fn get_models(&self) -> Result<Vec<OpenRouterModel>, LLMError> {
        let url = format!("{}/models", self.config.base_url);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| LLMError::NetworkError { message: e.to_string() })?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(LLMError::ServerError {
                provider: "openrouter".to_string(),
                message: format!("OpenRouter API error: {}", error_text),
            });
        }
        
        let models_response: OpenRouterModelsResponse = response
            .json()
            .await
            .map_err(|e| LLMError::SystemError { message: e.to_string() })?;
        
        Ok(models_response.data)
    }
    
    /// Convert AgentGraph message to OpenRouter format
    fn convert_message(&self, message: &Message) -> OpenRouterMessage {
        OpenRouterMessage {
            role: match message.role {
                MessageRole::System => "system".to_string(),
                MessageRole::User => "user".to_string(),
                MessageRole::Assistant => "assistant".to_string(),
                MessageRole::Function => "function".to_string(),
            },
            content: Some(message.content.clone()),
            name: None, // OpenRouter doesn't use name field in the same way
            function_call: message.function_call.as_ref().map(|fc| OpenRouterFunctionCall {
                name: fc.name.clone(),
                arguments: fc.arguments.clone(),
            }),
        }
    }
    
    /// Convert OpenRouter response to AgentGraph format
    fn convert_response(&self, response: OpenRouterResponse) -> Result<CompletionResponse, LLMError> {
        let choices = response.choices
            .into_iter()
            .map(|choice| {
                let function_call = choice.message.function_call.map(|fc| FunctionCall {
                    id: Some(uuid::Uuid::new_v4().to_string()),
                    name: fc.name,
                    arguments: fc.arguments,
                });

                Choice {
                    index: choice.index,
                    message: Message {
                        role: match choice.message.role.as_str() {
                            "system" => MessageRole::System,
                            "user" => MessageRole::User,
                            "assistant" => MessageRole::Assistant,
                            "function" => MessageRole::Function,
                            _ => MessageRole::Assistant,
                        },
                        content: choice.message.content.unwrap_or_default(),
                        function_call,
                        metadata: std::collections::HashMap::new(),
                        timestamp: std::time::SystemTime::now(),
                    },
                    finish_reason: crate::llm::FinishReason::Stop, // Default, should be mapped properly
                }
            })
            .collect();

        let usage = TokenUsage {
            prompt_tokens: response.usage.prompt_tokens,
            completion_tokens: response.usage.completion_tokens,
            total_tokens: response.usage.total_tokens,
            estimated_cost: self.calculate_cost(&response.model, &response.usage),
        };

        Ok(CompletionResponse {
            id: response.id,
            model: response.model,
            choices,
            usage,
            metadata: std::collections::HashMap::new(),
            timestamp: std::time::SystemTime::now(),
        })
    }
    
    /// Calculate estimated cost based on model and usage
    fn calculate_cost(&self, model: &str, usage: &OpenRouterUsage) -> Option<f64> {
        // OpenRouter pricing varies by model - this is a simplified calculation
        // In practice, you'd want to fetch current pricing from OpenRouter's API
        let cost_per_1k_tokens = match model {
            // GPT models
            m if m.contains("gpt-4") => 0.03,
            m if m.contains("gpt-3.5") => 0.002,
            
            // Claude models
            m if m.contains("claude-3-opus") => 0.015,
            m if m.contains("claude-3-sonnet") => 0.003,
            m if m.contains("claude-3-haiku") => 0.00025,
            
            // Gemini models
            m if m.contains("gemini-pro") => 0.0005,
            
            // Other models - rough estimate
            _ => 0.001,
        };
        
        Some((usage.total_tokens as f64 / 1000.0) * cost_per_1k_tokens)
    }
}

#[async_trait]
impl LLMProvider for OpenRouterProvider {
    fn name(&self) -> &str {
        "openrouter"
    }

    fn supported_models(&self) -> Vec<String> {
        // OpenRouter supports many models - this is a subset of popular ones
        vec![
            // OpenAI models
            "openai/gpt-4".to_string(),
            "openai/gpt-4-turbo".to_string(),
            "openai/gpt-3.5-turbo".to_string(),

            // Anthropic models
            "anthropic/claude-3-opus".to_string(),
            "anthropic/claude-3-sonnet".to_string(),
            "anthropic/claude-3-haiku".to_string(),

            // Google models
            "google/gemini-pro".to_string(),
            "google/gemini-pro-vision".to_string(),

            // Meta models
            "meta-llama/llama-2-70b-chat".to_string(),
            "meta-llama/llama-2-13b-chat".to_string(),

            // Mistral models
            "mistralai/mistral-7b-instruct".to_string(),
            "mistralai/mixtral-8x7b-instruct".to_string(),
        ]
    }

    async fn count_tokens(&self, text: &str, _model: &str) -> Result<u32, LLMError> {
        // Simple approximation: ~4 characters per token
        // In practice, you'd use a proper tokenizer
        Ok((text.len() / 4) as u32)
    }

    fn get_pricing(&self, model: &str) -> Option<crate::llm::ModelPricing> {
        // OpenRouter pricing varies by model - these are approximate values
        match model {
            // OpenAI models
            m if m.contains("gpt-4-turbo") => Some(crate::llm::ModelPricing {
                prompt_cost_per_1k: 0.01,
                completion_cost_per_1k: 0.03,
                currency: "USD".to_string(),
            }),
            m if m.contains("gpt-4") => Some(crate::llm::ModelPricing {
                prompt_cost_per_1k: 0.03,
                completion_cost_per_1k: 0.06,
                currency: "USD".to_string(),
            }),
            m if m.contains("gpt-3.5") => Some(crate::llm::ModelPricing {
                prompt_cost_per_1k: 0.001,
                completion_cost_per_1k: 0.002,
                currency: "USD".to_string(),
            }),

            // Anthropic models
            m if m.contains("claude-3-opus") => Some(crate::llm::ModelPricing {
                prompt_cost_per_1k: 0.015,
                completion_cost_per_1k: 0.075,
                currency: "USD".to_string(),
            }),
            m if m.contains("claude-3-sonnet") => Some(crate::llm::ModelPricing {
                prompt_cost_per_1k: 0.003,
                completion_cost_per_1k: 0.015,
                currency: "USD".to_string(),
            }),
            m if m.contains("claude-3-haiku") => Some(crate::llm::ModelPricing {
                prompt_cost_per_1k: 0.00025,
                completion_cost_per_1k: 0.00125,
                currency: "USD".to_string(),
            }),

            // Google models
            m if m.contains("gemini") => Some(crate::llm::ModelPricing {
                prompt_cost_per_1k: 0.0005,
                completion_cost_per_1k: 0.0015,
                currency: "USD".to_string(),
            }),

            // Default for other models
            _ => Some(crate::llm::ModelPricing {
                prompt_cost_per_1k: 0.001,
                completion_cost_per_1k: 0.002,
                currency: "USD".to_string(),
            }),
        }
    }
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LLMError> {
        let url = format!("{}/chat/completions", self.config.base_url);
        
        let messages: Vec<OpenRouterMessage> = request.messages
            .iter()
            .map(|m| self.convert_message(m))
            .collect();
        
        let mut body = json!({
            "model": request.model,
            "messages": messages,
            "stream": false,
        });
        
        // Add optional parameters
        if let Some(max_tokens) = request.max_tokens {
            body["max_tokens"] = json!(max_tokens);
        }
        
        if let Some(temperature) = request.temperature {
            body["temperature"] = json!(temperature);
        }
        
        if let Some(top_p) = request.top_p {
            body["top_p"] = json!(top_p);
        }
        
        if let Some(functions) = request.functions {
            let openrouter_functions: Vec<Value> = functions
                .iter()
                .map(|f| json!({
                    "name": f.name,
                    "description": f.description,
                    "parameters": f.parameters
                }))
                .collect();
            body["functions"] = json!(openrouter_functions);
        }
        
        if let Some(function_call) = request.function_call {
            let function_call_value = match function_call {
                crate::llm::FunctionCallBehavior::None => "none".to_string(),
                crate::llm::FunctionCallBehavior::Auto => "auto".to_string(),
                crate::llm::FunctionCallBehavior::Force(name) => name,
            };
            body["function_call"] = json!(function_call_value);
        }
        
        let response = self.client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| LLMError::NetworkError { message: e.to_string() })?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(LLMError::ServerError {
                provider: "openrouter".to_string(),
                message: format!("OpenRouter API error: {}", error_text),
            });
        }
        
        let openrouter_response: OpenRouterResponse = response
            .json()
            .await
            .map_err(|e| LLMError::SystemError { message: e.to_string() })?;
        
        self.convert_response(openrouter_response)
    }

    fn supports_function_calling(&self) -> bool {
        true
    }

    fn supports_streaming(&self) -> bool {
        false // Will be true once streaming is implemented
    }

    
    async fn stream(&self, _request: CompletionRequest) -> Result<Box<dyn futures::Stream<Item = Result<CompletionResponse, LLMError>> + Unpin + Send>, LLMError> {
        // For now, return an error as streaming implementation is complex
        // In a full implementation, you'd handle Server-Sent Events from OpenRouter
        Err(LLMError::SystemError {
            message: "Streaming not yet implemented for OpenRouter".to_string(),
        })
    }

}

/// OpenRouter API message format
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenRouterMessage {
    role: String,
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    function_call: Option<OpenRouterFunctionCall>,
}

/// OpenRouter function call format
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenRouterFunctionCall {
    name: String,
    arguments: Value,
}

/// OpenRouter API response format
#[derive(Debug, Clone, Deserialize)]
struct OpenRouterResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<OpenRouterChoice>,
    usage: OpenRouterUsage,
}

/// OpenRouter choice format
#[derive(Debug, Clone, Deserialize)]
struct OpenRouterChoice {
    index: u32,
    message: OpenRouterResponseMessage,
    finish_reason: Option<String>,
}

/// OpenRouter response message format
#[derive(Debug, Clone, Deserialize)]
struct OpenRouterResponseMessage {
    role: String,
    content: Option<String>,
    name: Option<String>,
    function_call: Option<OpenRouterFunctionCall>,
}

/// OpenRouter usage information
#[derive(Debug, Clone, Deserialize)]
struct OpenRouterUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// OpenRouter model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterModel {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub context_length: Option<u32>,
    pub pricing: Option<OpenRouterPricing>,
    pub top_provider: Option<OpenRouterTopProvider>,
}

/// OpenRouter pricing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterPricing {
    pub prompt: String,
    pub completion: String,
    pub request: Option<String>,
    pub image: Option<String>,
}

/// OpenRouter top provider information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterTopProvider {
    pub max_completion_tokens: Option<u32>,
    pub is_moderated: Option<bool>,
}

/// OpenRouter models response
#[derive(Debug, Clone, Deserialize)]
struct OpenRouterModelsResponse {
    data: Vec<OpenRouterModel>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openrouter_config_default() {
        let config = OpenRouterConfig::default();
        assert_eq!(config.base_url, "https://openrouter.ai/api/v1");
        assert_eq!(config.timeout, Duration::from_secs(120));
        assert!(config.app_name.is_some());
    }

    #[test]
    fn test_openrouter_provider_creation() {
        let config = OpenRouterConfig {
            api_key: "test_key".to_string(),
            ..Default::default()
        };
        
        let provider = OpenRouterProvider::new(config);
        assert_eq!(provider.config.api_key, "test_key");
    }

    #[test]
    fn test_message_conversion() {
        let config = OpenRouterConfig::default();
        let provider = OpenRouterProvider::new(config);
        
        let message = Message::user("Hello, world!".to_string());
        let converted = provider.convert_message(&message);
        
        assert_eq!(converted.role, "user");
        assert_eq!(converted.content, Some("Hello, world!".to_string()));
    }

    #[test]
    fn test_supported_models() {
        let config = OpenRouterConfig::default();
        let provider = OpenRouterProvider::new(config);

        let models = provider.supported_models();
        assert!(models.contains(&"openai/gpt-4".to_string()));
        assert!(models.contains(&"anthropic/claude-3-opus".to_string()));
        assert!(models.contains(&"google/gemini-pro".to_string()));
    }

    #[test]
    fn test_cost_calculation() {
        let config = OpenRouterConfig::default();
        let provider = OpenRouterProvider::new(config);
        
        let usage = OpenRouterUsage {
            prompt_tokens: 100,
            completion_tokens: 50,
            total_tokens: 150,
        };
        
        let cost = provider.calculate_cost("gpt-4", &usage);
        assert!(cost.is_some());
        assert!(cost.unwrap() > 0.0);
    }

    #[test]
    fn test_supported_features() {
        let config = OpenRouterConfig::default();
        let provider = OpenRouterProvider::new(config);

        assert!(provider.supports_function_calling());
        assert!(!provider.supports_streaming()); // Not implemented yet
    }
}
