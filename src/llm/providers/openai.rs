// OpenAI provider implementation for AgentGraph LLM framework

#![allow(missing_docs)]

use super::super::*;
use reqwest::{Client, header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE}};
use serde_json::json;
use std::time::SystemTime;

/// OpenAI provider for LLM operations
#[derive(Debug)]
pub struct OpenAIProvider {
    /// HTTP client
    client: Client,
    /// API key
    api_key: String,
    /// Base URL
    base_url: String,
    /// Organization ID
    organization: Option<String>,
}

impl OpenAIProvider {
    /// Create a new OpenAI provider
    pub fn new(api_key: String) -> Result<Self, LLMError> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", api_key))
                .map_err(|e| LLMError::ConfigurationError {
                    message: format!("Invalid API key format: {}", e),
                })?,
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
            base_url: "https://api.openai.com/v1".to_string(),
            organization: None,
        })
    }

    /// Create provider with custom configuration
    pub fn with_config(config: ProviderConfig) -> Result<Self, LLMError> {
        let api_key = config.api_key
            .ok_or_else(|| LLMError::ConfigurationError {
                message: "OpenAI API key is required".to_string(),
            })?;

        let mut provider = Self::new(api_key)?;
        
        if let Some(base_url) = config.base_url {
            provider.base_url = base_url;
        }
        
        provider.organization = config.organization;
        
        Ok(provider)
    }

    /// Convert internal message to OpenAI format
    fn convert_message(&self, message: &Message) -> serde_json::Value {
        let role = match message.role {
            MessageRole::System => "system",
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
            MessageRole::Function => "function",
        };

        let mut msg = json!({
            "role": role,
            "content": message.content
        });

        if let Some(function_call) = &message.function_call {
            msg["function_call"] = json!({
                "name": function_call.name,
                "arguments": serde_json::to_string(&function_call.arguments).unwrap_or_default()
            });
        }

        msg
    }

    /// Convert function definition to OpenAI format
    fn convert_function(&self, function: &FunctionDefinition) -> serde_json::Value {
        json!({
            "name": function.name,
            "description": function.description,
            "parameters": function.parameters
        })
    }

    /// Parse OpenAI response
    fn parse_response(&self, response: serde_json::Value) -> Result<CompletionResponse, LLMError> {
        let id = response["id"].as_str()
            .unwrap_or("unknown")
            .to_string();

        let model = response["model"].as_str()
            .unwrap_or("unknown")
            .to_string();

        let choices = response["choices"].as_array()
            .ok_or_else(|| LLMError::ServerError {
                provider: "openai".to_string(),
                message: "No choices in response".to_string(),
            })?;

        let mut parsed_choices = Vec::new();
        for (index, choice) in choices.iter().enumerate() {
            let message_data = &choice["message"];
            
            let role = match message_data["role"].as_str().unwrap_or("assistant") {
                "system" => MessageRole::System,
                "user" => MessageRole::User,
                "assistant" => MessageRole::Assistant,
                "function" => MessageRole::Function,
                _ => MessageRole::Assistant,
            };

            let content = message_data["content"].as_str()
                .unwrap_or("")
                .to_string();

            let mut message = Message::new(role, content);

            // Parse function call if present
            if let Some(function_call_data) = message_data.get("function_call") {
                let name = function_call_data["name"].as_str()
                    .unwrap_or("")
                    .to_string();
                
                let arguments_str = function_call_data["arguments"].as_str()
                    .unwrap_or("{}");
                
                let arguments: serde_json::Value = serde_json::from_str(arguments_str)
                    .unwrap_or(json!({}));

                message.function_call = Some(FunctionCall::new(name, arguments));
            }

            let finish_reason = match choice["finish_reason"].as_str() {
                Some("stop") => FinishReason::Stop,
                Some("length") => FinishReason::Length,
                Some("function_call") => FinishReason::FunctionCall,
                Some("content_filter") => FinishReason::ContentFilter,
                _ => FinishReason::Stop,
            };

            parsed_choices.push(Choice {
                index: index as u32,
                message,
                finish_reason,
            });
        }

        // Parse usage information
        let usage_data = &response["usage"];
        let usage = TokenUsage::new(
            usage_data["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            usage_data["completion_tokens"].as_u64().unwrap_or(0) as u32,
        );

        Ok(CompletionResponse {
            id,
            model,
            choices: parsed_choices,
            usage,
            metadata: HashMap::new(),
            timestamp: SystemTime::now(),
        })
    }
}

#[async_trait::async_trait]
impl LLMProvider for OpenAIProvider {
    fn name(&self) -> &str {
        "openai"
    }

    fn supported_models(&self) -> Vec<String> {
        vec![
            "gpt-4".to_string(),
            "gpt-4-0613".to_string(),
            "gpt-4-32k".to_string(),
            "gpt-4-32k-0613".to_string(),
            "gpt-3.5-turbo".to_string(),
            "gpt-3.5-turbo-0613".to_string(),
            "gpt-3.5-turbo-16k".to_string(),
            "gpt-3.5-turbo-16k-0613".to_string(),
            "text-davinci-003".to_string(),
            "text-davinci-002".to_string(),
            "code-davinci-002".to_string(),
        ]
    }

    fn supports_function_calling(&self) -> bool {
        true
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

        // Build request body
        let mut body = json!({
            "model": request.model,
            "messages": request.messages.iter().map(|m| self.convert_message(m)).collect::<Vec<_>>(),
        });

        if let Some(max_tokens) = request.max_tokens {
            body["max_tokens"] = json!(max_tokens);
        }

        if let Some(temperature) = request.temperature {
            body["temperature"] = json!(temperature);
        }

        if let Some(top_p) = request.top_p {
            body["top_p"] = json!(top_p);
        }

        if let Some(stop) = &request.stop {
            body["stop"] = json!(stop);
        }

        if request.stream {
            body["stream"] = json!(true);
        }

        // Add function calling if specified
        if let Some(functions) = &request.functions {
            body["functions"] = json!(functions.iter().map(|f| self.convert_function(f)).collect::<Vec<_>>());
            
            if let Some(function_call) = &request.function_call {
                body["function_call"] = match function_call {
                    FunctionCallBehavior::None => json!("none"),
                    FunctionCallBehavior::Auto => json!("auto"),
                    FunctionCallBehavior::Force(name) => json!({"name": name}),
                };
            }
        }

        // Make request
        let url = format!("{}/chat/completions", self.base_url);
        let mut req_builder = self.client.post(&url).json(&body);

        if let Some(org) = &self.organization {
            req_builder = req_builder.header("OpenAI-Organization", org);
        }

        let response = req_builder.send().await
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

        self.parse_response(response_json)
    }

    async fn count_tokens(&self, text: &str, _model: &str) -> Result<u32, LLMError> {
        // Simplified token counting (rough approximation)
        // In production, you'd use tiktoken or similar
        let words = text.split_whitespace().count();
        Ok((words as f32 * 1.3) as u32) // Rough approximation: 1.3 tokens per word
    }

    fn get_pricing(&self, model: &str) -> Option<ModelPricing> {
        match model {
            "gpt-4" | "gpt-4-0613" => Some(ModelPricing {
                prompt_cost_per_1k: 0.03,
                completion_cost_per_1k: 0.06,
                currency: "USD".to_string(),
            }),
            "gpt-4-32k" | "gpt-4-32k-0613" => Some(ModelPricing {
                prompt_cost_per_1k: 0.06,
                completion_cost_per_1k: 0.12,
                currency: "USD".to_string(),
            }),
            "gpt-3.5-turbo" | "gpt-3.5-turbo-0613" => Some(ModelPricing {
                prompt_cost_per_1k: 0.0015,
                completion_cost_per_1k: 0.002,
                currency: "USD".to_string(),
            }),
            "gpt-3.5-turbo-16k" | "gpt-3.5-turbo-16k-0613" => Some(ModelPricing {
                prompt_cost_per_1k: 0.003,
                completion_cost_per_1k: 0.004,
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
    fn test_openai_provider_creation() {
        let provider = OpenAIProvider::new("test-key".to_string()).unwrap();
        assert_eq!(provider.name(), "openai");
        assert!(provider.supports_function_calling());
        assert!(provider.supports_streaming());
    }

    #[test]
    fn test_supported_models() {
        let provider = OpenAIProvider::new("test-key".to_string()).unwrap();
        let models = provider.supported_models();
        
        assert!(models.contains(&"gpt-4".to_string()));
        assert!(models.contains(&"gpt-3.5-turbo".to_string()));
        assert!(provider.supports_model("gpt-4"));
        assert!(!provider.supports_model("invalid-model"));
    }

    #[test]
    fn test_message_conversion() {
        let provider = OpenAIProvider::new("test-key".to_string()).unwrap();
        let message = Message::user("Hello, world!".to_string());
        let converted = provider.convert_message(&message);
        
        assert_eq!(converted["role"], "user");
        assert_eq!(converted["content"], "Hello, world!");
    }

    #[test]
    fn test_function_conversion() {
        let provider = OpenAIProvider::new("test-key".to_string()).unwrap();
        let function = FunctionDefinition::new(
            "test_function".to_string(),
            "A test function".to_string(),
            json!({"type": "object", "properties": {}})
        );
        let converted = provider.convert_function(&function);
        
        assert_eq!(converted["name"], "test_function");
        assert_eq!(converted["description"], "A test function");
    }

    #[test]
    fn test_pricing() {
        let provider = OpenAIProvider::new("test-key".to_string()).unwrap();
        
        let gpt4_pricing = provider.get_pricing("gpt-4").unwrap();
        assert_eq!(gpt4_pricing.prompt_cost_per_1k, 0.03);
        assert_eq!(gpt4_pricing.completion_cost_per_1k, 0.06);
        
        let gpt35_pricing = provider.get_pricing("gpt-3.5-turbo").unwrap();
        assert_eq!(gpt35_pricing.prompt_cost_per_1k, 0.0015);
        assert_eq!(gpt35_pricing.completion_cost_per_1k, 0.002);
        
        assert!(provider.get_pricing("invalid-model").is_none());
    }

    #[tokio::test]
    async fn test_token_counting() {
        let provider = OpenAIProvider::new("test-key".to_string()).unwrap();
        let tokens = provider.count_tokens("Hello world", "gpt-3.5-turbo").await.unwrap();
        
        // Should be approximately 2-3 tokens for "Hello world"
        assert!(tokens >= 2 && tokens <= 4);
    }
}
