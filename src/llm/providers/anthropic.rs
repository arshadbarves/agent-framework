// Anthropic provider implementation for AgentGraph LLM framework

#![allow(missing_docs)]

use super::super::*;
use reqwest::{Client, header::{HeaderMap, HeaderValue, CONTENT_TYPE}};
use serde_json::json;
use std::time::SystemTime;

/// Anthropic provider for LLM operations
#[derive(Debug)]
pub struct AnthropicProvider {
    /// HTTP client
    client: Client,
    /// API key
    api_key: String,
    /// Base URL
    base_url: String,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider
    pub fn new(api_key: String) -> Result<Self, LLMError> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            "x-api-key",
            HeaderValue::from_str(&api_key)
                .map_err(|e| LLMError::ConfigurationError {
                    message: format!("Invalid API key format: {}", e),
                })?,
        );
        headers.insert(
            "anthropic-version",
            HeaderValue::from_static("2023-06-01"),
        );

        let client = Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| LLMError::ConfigurationError {
                message: format!("Failed to create HTTP client: {}", e),
            })?;

        Ok(Self {
            client,
            api_key,
            base_url: "https://api.anthropic.com/v1".to_string(),
        })
    }

    /// Create provider with custom configuration
    pub fn with_config(config: ProviderConfig) -> Result<Self, LLMError> {
        let api_key = config.api_key
            .ok_or_else(|| LLMError::ConfigurationError {
                message: "Anthropic API key is required".to_string(),
            })?;

        let mut provider = Self::new(api_key)?;
        
        if let Some(base_url) = config.base_url {
            provider.base_url = base_url;
        }
        
        Ok(provider)
    }

    /// Convert messages to Anthropic format
    fn convert_messages(&self, messages: &[Message]) -> (Option<String>, Vec<serde_json::Value>) {
        let mut system_message = None;
        let mut converted_messages = Vec::new();

        for message in messages {
            match message.role {
                MessageRole::System => {
                    system_message = Some(message.content.clone());
                }
                MessageRole::User => {
                    converted_messages.push(json!({
                        "role": "user",
                        "content": message.content
                    }));
                }
                MessageRole::Assistant => {
                    converted_messages.push(json!({
                        "role": "assistant",
                        "content": message.content
                    }));
                }
                MessageRole::Function => {
                    // Anthropic doesn't support function messages directly
                    // Convert to user message with context
                    converted_messages.push(json!({
                        "role": "user",
                        "content": format!("Function result: {}", message.content)
                    }));
                }
            }
        }

        (system_message, converted_messages)
    }

    /// Parse Anthropic response
    fn parse_response(&self, response: serde_json::Value, model: &str) -> Result<CompletionResponse, LLMError> {
        let id = response["id"].as_str()
            .unwrap_or("unknown")
            .to_string();

        let content = response["content"].as_array()
            .ok_or_else(|| LLMError::ServerError {
                provider: "anthropic".to_string(),
                message: "No content in response".to_string(),
            })?;

        let text_content = content.iter()
            .find(|c| c["type"] == "text")
            .and_then(|c| c["text"].as_str())
            .unwrap_or("")
            .to_string();

        let message = Message::assistant(text_content);

        let finish_reason = match response["stop_reason"].as_str() {
            Some("end_turn") => FinishReason::Stop,
            Some("max_tokens") => FinishReason::Length,
            Some("stop_sequence") => FinishReason::Stop,
            _ => FinishReason::Stop,
        };

        let choice = Choice {
            index: 0,
            message,
            finish_reason,
        };

        // Parse usage information
        let usage_data = &response["usage"];
        let usage = TokenUsage::new(
            usage_data["input_tokens"].as_u64().unwrap_or(0) as u32,
            usage_data["output_tokens"].as_u64().unwrap_or(0) as u32,
        );

        Ok(CompletionResponse {
            id,
            model: model.to_string(),
            choices: vec![choice],
            usage,
            metadata: HashMap::new(),
            timestamp: SystemTime::now(),
        })
    }
}

#[async_trait::async_trait]
impl LLMProvider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    fn supported_models(&self) -> Vec<String> {
        vec![
            "claude-3-opus-20240229".to_string(),
            "claude-3-sonnet-20240229".to_string(),
            "claude-3-haiku-20240307".to_string(),
            "claude-2.1".to_string(),
            "claude-2.0".to_string(),
            "claude-instant-1.2".to_string(),
        ]
    }

    fn supports_function_calling(&self) -> bool {
        false // Anthropic doesn't support function calling in the same way as OpenAI
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LLMError> {
        if !self.supports_model(&request.model) {
            return Err(LLMError::ModelNotSupported {
                model: request.model,
                provider: self.name().to_string(),
            });
        }

        // Function calling not supported
        if request.functions.is_some() {
            return Err(LLMError::FunctionCallError {
                message: "Anthropic provider does not support function calling".to_string(),
            });
        }

        let (system_message, messages) = self.convert_messages(&request.messages);

        // Build request body
        let mut body = json!({
            "model": request.model,
            "messages": messages,
            "max_tokens": request.max_tokens.unwrap_or(1000),
        });

        if let Some(system) = system_message {
            body["system"] = json!(system);
        }

        if let Some(temperature) = request.temperature {
            body["temperature"] = json!(temperature);
        }

        if let Some(top_p) = request.top_p {
            body["top_p"] = json!(top_p);
        }

        if let Some(stop) = &request.stop {
            body["stop_sequences"] = json!(stop);
        }

        if request.stream {
            body["stream"] = json!(true);
        }

        // Make request
        let url = format!("{}/messages", self.base_url);
        let response = self.client.post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| LLMError::NetworkError {
                message: format!("Request failed: {}", e),
            })?;

        let status = response.status();
        let response_text = response.text().await
            .map_err(|e| LLMError::NetworkError {
                message: format!("Failed to read response: {}", e),
            })?;

        if !status.is_success() {
            return match status.as_u16() {
                401 => Err(LLMError::AuthenticationError {
                    provider: self.name().to_string(),
                    message: "Invalid API key".to_string(),
                }),
                429 => Err(LLMError::RateLimitExceeded {
                    provider: self.name().to_string(),
                }),
                _ => Err(LLMError::ServerError {
                    provider: self.name().to_string(),
                    message: format!("HTTP {}: {}", status, response_text),
                }),
            };
        }

        let response_json: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| LLMError::ServerError {
                provider: self.name().to_string(),
                message: format!("Invalid JSON response: {}", e),
            })?;

        self.parse_response(response_json, &request.model)
    }

    async fn count_tokens(&self, text: &str, _model: &str) -> Result<u32, LLMError> {
        // Simplified token counting (rough approximation)
        // Anthropic uses a different tokenizer than OpenAI
        let words = text.split_whitespace().count();
        Ok((words as f32 * 1.2) as u32) // Rough approximation: 1.2 tokens per word
    }

    fn get_pricing(&self, model: &str) -> Option<ModelPricing> {
        match model {
            "claude-3-opus-20240229" => Some(ModelPricing {
                prompt_cost_per_1k: 0.015,
                completion_cost_per_1k: 0.075,
                currency: "USD".to_string(),
            }),
            "claude-3-sonnet-20240229" => Some(ModelPricing {
                prompt_cost_per_1k: 0.003,
                completion_cost_per_1k: 0.015,
                currency: "USD".to_string(),
            }),
            "claude-3-haiku-20240307" => Some(ModelPricing {
                prompt_cost_per_1k: 0.00025,
                completion_cost_per_1k: 0.00125,
                currency: "USD".to_string(),
            }),
            "claude-2.1" | "claude-2.0" => Some(ModelPricing {
                prompt_cost_per_1k: 0.008,
                completion_cost_per_1k: 0.024,
                currency: "USD".to_string(),
            }),
            "claude-instant-1.2" => Some(ModelPricing {
                prompt_cost_per_1k: 0.0008,
                completion_cost_per_1k: 0.0024,
                currency: "USD".to_string(),
            }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anthropic_provider_creation() {
        let provider = AnthropicProvider::new("test-key".to_string()).unwrap();
        assert_eq!(provider.name(), "anthropic");
        assert!(!provider.supports_function_calling());
        assert!(provider.supports_streaming());
    }

    #[test]
    fn test_supported_models() {
        let provider = AnthropicProvider::new("test-key".to_string()).unwrap();
        let models = provider.supported_models();
        
        assert!(models.contains(&"claude-3-opus-20240229".to_string()));
        assert!(models.contains(&"claude-3-sonnet-20240229".to_string()));
        assert!(provider.supports_model("claude-3-opus-20240229"));
        assert!(!provider.supports_model("gpt-4"));
    }

    #[test]
    fn test_message_conversion() {
        let provider = AnthropicProvider::new("test-key".to_string()).unwrap();
        let messages = vec![
            Message::system("You are a helpful assistant".to_string()),
            Message::user("Hello!".to_string()),
            Message::assistant("Hi there!".to_string()),
        ];
        
        let (system, converted) = provider.convert_messages(&messages);
        
        assert_eq!(system, Some("You are a helpful assistant".to_string()));
        assert_eq!(converted.len(), 2);
        assert_eq!(converted[0]["role"], "user");
        assert_eq!(converted[1]["role"], "assistant");
    }

    #[test]
    fn test_pricing() {
        let provider = AnthropicProvider::new("test-key".to_string()).unwrap();
        
        let opus_pricing = provider.get_pricing("claude-3-opus-20240229").unwrap();
        assert_eq!(opus_pricing.prompt_cost_per_1k, 0.015);
        assert_eq!(opus_pricing.completion_cost_per_1k, 0.075);
        
        let haiku_pricing = provider.get_pricing("claude-3-haiku-20240307").unwrap();
        assert_eq!(haiku_pricing.prompt_cost_per_1k, 0.00025);
        assert_eq!(haiku_pricing.completion_cost_per_1k, 0.00125);
        
        assert!(provider.get_pricing("invalid-model").is_none());
    }

    #[tokio::test]
    async fn test_token_counting() {
        let provider = AnthropicProvider::new("test-key".to_string()).unwrap();
        let tokens = provider.count_tokens("Hello world", "claude-3-sonnet-20240229").await.unwrap();
        
        // Should be approximately 2-3 tokens for "Hello world"
        assert!(tokens >= 2 && tokens <= 4);
    }
}
