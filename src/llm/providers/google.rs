// Google provider implementation for AgentGraph LLM framework

#![allow(missing_docs)]

use super::super::*;
use reqwest::{Client, header::{HeaderMap, HeaderValue, CONTENT_TYPE}};
use serde_json::json;
use std::time::SystemTime;

/// Google provider for LLM operations (Gemini models)
#[derive(Debug)]
pub struct GoogleProvider {
    /// HTTP client
    client: Client,
    /// API key
    api_key: String,
    /// Base URL
    base_url: String,
}

impl GoogleProvider {
    /// Create a new Google provider
    pub fn new(api_key: String) -> Result<Self, LLMError> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

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
            base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
        })
    }

    /// Create provider with custom configuration
    pub fn with_config(config: ProviderConfig) -> Result<Self, LLMError> {
        let api_key = config.api_key
            .ok_or_else(|| LLMError::ConfigurationError {
                message: "Google API key is required".to_string(),
            })?;

        let mut provider = Self::new(api_key)?;
        
        if let Some(base_url) = config.base_url {
            provider.base_url = base_url;
        }
        
        Ok(provider)
    }

    /// Convert internal message to Google format
    fn convert_message(&self, message: &Message) -> serde_json::Value {
        let role = match message.role {
            MessageRole::System => "user", // Google treats system as user with special formatting
            MessageRole::User => "user",
            MessageRole::Assistant => "model",
            MessageRole::Function => "user", // Function results as user messages
        };

        let content = if message.role == MessageRole::System {
            format!("System: {}", message.content)
        } else {
            message.content.clone()
        };

        json!({
            "role": role,
            "parts": [{"text": content}]
        })
    }

    /// Convert function definition to Google format
    fn convert_function(&self, function: &FunctionDefinition) -> serde_json::Value {
        json!({
            "name": function.name,
            "description": function.description,
            "parameters": function.parameters
        })
    }

    /// Parse Google response
    fn parse_response(&self, response: serde_json::Value, model: &str) -> Result<CompletionResponse, LLMError> {
        let candidates = response["candidates"].as_array()
            .ok_or_else(|| LLMError::ServerError {
                provider: "google".to_string(),
                message: "No candidates in response".to_string(),
            })?;

        if candidates.is_empty() {
            return Err(LLMError::ServerError {
                provider: "google".to_string(),
                message: "Empty candidates array".to_string(),
            });
        }

        let candidate = &candidates[0];
        let content = candidate["content"]["parts"][0]["text"].as_str()
            .unwrap_or("")
            .to_string();

        let message = Message::assistant(content);

        let finish_reason = match candidate["finishReason"].as_str() {
            Some("STOP") => FinishReason::Stop,
            Some("MAX_TOKENS") => FinishReason::Length,
            Some("SAFETY") => FinishReason::ContentFilter,
            Some("RECITATION") => FinishReason::ContentFilter,
            _ => FinishReason::Stop,
        };

        let choice = Choice {
            index: 0,
            message,
            finish_reason,
        };

        // Parse usage information if available
        let usage_metadata = &response["usageMetadata"];
        let usage = TokenUsage::new(
            usage_metadata["promptTokenCount"].as_u64().unwrap_or(0) as u32,
            usage_metadata["candidatesTokenCount"].as_u64().unwrap_or(0) as u32,
        );

        Ok(CompletionResponse {
            id: format!("google-{}", uuid::Uuid::new_v4()),
            model: model.to_string(),
            choices: vec![choice],
            usage,
            metadata: HashMap::new(),
            timestamp: SystemTime::now(),
        })
    }
}

#[async_trait::async_trait]
impl LLMProvider for GoogleProvider {
    fn name(&self) -> &str {
        "google"
    }

    fn supported_models(&self) -> Vec<String> {
        vec![
            "gemini-1.5-pro".to_string(),
            "gemini-1.5-flash".to_string(),
            "gemini-1.0-pro".to_string(),
            "gemini-1.0-pro-vision".to_string(),
            "gemini-pro".to_string(),
            "gemini-pro-vision".to_string(),
        ]
    }

    fn supports_function_calling(&self) -> bool {
        true // Gemini supports function calling
    }

    fn supports_streaming(&self) -> bool {
        true // Gemini supports streaming
    }

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LLMError> {
        if !self.supports_model(&request.model) {
            return Err(LLMError::ModelNotSupported {
                model: request.model,
                provider: self.name().to_string(),
            });
        }

        // Convert messages to Google format
        let contents = request.messages.iter()
            .map(|m| self.convert_message(m))
            .collect::<Vec<_>>();

        // Build request body
        let mut body = json!({
            "contents": contents,
        });

        // Add generation config
        let mut generation_config = json!({});
        
        if let Some(max_tokens) = request.max_tokens {
            generation_config["maxOutputTokens"] = json!(max_tokens);
        }

        if let Some(temperature) = request.temperature {
            generation_config["temperature"] = json!(temperature);
        }

        if let Some(top_p) = request.top_p {
            generation_config["topP"] = json!(top_p);
        }

        if let Some(stop) = &request.stop {
            generation_config["stopSequences"] = json!(stop);
        }

        if !generation_config.as_object().unwrap().is_empty() {
            body["generationConfig"] = generation_config;
        }

        // Add function calling if specified
        if let Some(functions) = &request.functions {
            let tools = json!([{
                "functionDeclarations": functions.iter().map(|f| self.convert_function(f)).collect::<Vec<_>>()
            }]);
            body["tools"] = tools;
        }

        // Make request
        let endpoint = if request.stream {
            "streamGenerateContent"
        } else {
            "generateContent"
        };

        let url = format!("{}/models/{}:{}?key={}", 
                         self.base_url, request.model, endpoint, self.api_key);

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
        // Google uses SentencePiece tokenizer
        let words = text.split_whitespace().count();
        Ok((words as f32 * 1.4) as u32) // Rough approximation: 1.4 tokens per word
    }

    fn get_pricing(&self, model: &str) -> Option<ModelPricing> {
        match model {
            "gemini-1.5-pro" => Some(ModelPricing {
                prompt_cost_per_1k: 0.0035,
                completion_cost_per_1k: 0.0105,
                currency: "USD".to_string(),
            }),
            "gemini-1.5-flash" => Some(ModelPricing {
                prompt_cost_per_1k: 0.00035,
                completion_cost_per_1k: 0.00105,
                currency: "USD".to_string(),
            }),
            "gemini-1.0-pro" | "gemini-pro" => Some(ModelPricing {
                prompt_cost_per_1k: 0.0005,
                completion_cost_per_1k: 0.0015,
                currency: "USD".to_string(),
            }),
            "gemini-1.0-pro-vision" | "gemini-pro-vision" => Some(ModelPricing {
                prompt_cost_per_1k: 0.00025,
                completion_cost_per_1k: 0.0005,
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
    fn test_google_provider_creation() {
        let provider = GoogleProvider::new("test-key".to_string()).unwrap();
        assert_eq!(provider.name(), "google");
        assert!(provider.supports_function_calling());
        assert!(provider.supports_streaming());
    }

    #[test]
    fn test_supported_models() {
        let provider = GoogleProvider::new("test-key".to_string()).unwrap();
        let models = provider.supported_models();
        
        assert!(models.contains(&"gemini-1.5-pro".to_string()));
        assert!(models.contains(&"gemini-1.5-flash".to_string()));
        assert!(models.contains(&"gemini-pro".to_string()));
        assert!(provider.supports_model("gemini-1.5-pro"));
        assert!(!provider.supports_model("gpt-4"));
    }

    #[test]
    fn test_message_conversion() {
        let provider = GoogleProvider::new("test-key".to_string()).unwrap();
        
        let user_message = Message::user("Hello, world!".to_string());
        let converted = provider.convert_message(&user_message);
        
        assert_eq!(converted["role"], "user");
        assert_eq!(converted["parts"][0]["text"], "Hello, world!");

        let system_message = Message::system("You are helpful".to_string());
        let converted_system = provider.convert_message(&system_message);
        
        assert_eq!(converted_system["role"], "user");
        assert_eq!(converted_system["parts"][0]["text"], "System: You are helpful");
    }

    #[test]
    fn test_function_conversion() {
        let provider = GoogleProvider::new("test-key".to_string()).unwrap();
        let function = FunctionDefinition::new(
            "get_weather".to_string(),
            "Get weather information".to_string(),
            json!({"type": "object", "properties": {}})
        );
        let converted = provider.convert_function(&function);
        
        assert_eq!(converted["name"], "get_weather");
        assert_eq!(converted["description"], "Get weather information");
    }

    #[test]
    fn test_pricing() {
        let provider = GoogleProvider::new("test-key".to_string()).unwrap();
        
        let gemini_pro_pricing = provider.get_pricing("gemini-1.5-pro").unwrap();
        assert_eq!(gemini_pro_pricing.prompt_cost_per_1k, 0.0035);
        assert_eq!(gemini_pro_pricing.completion_cost_per_1k, 0.0105);
        
        let gemini_flash_pricing = provider.get_pricing("gemini-1.5-flash").unwrap();
        assert_eq!(gemini_flash_pricing.prompt_cost_per_1k, 0.00035);
        assert_eq!(gemini_flash_pricing.completion_cost_per_1k, 0.00105);
        
        assert!(provider.get_pricing("invalid-model").is_none());
    }

    #[tokio::test]
    async fn test_token_counting() {
        let provider = GoogleProvider::new("test-key".to_string()).unwrap();
        let tokens = provider.count_tokens("Hello world test", "gemini-1.5-pro").await.unwrap();
        
        // Should be approximately 4 tokens for "Hello world test" (3 words * 1.4)
        assert!(tokens >= 3 && tokens <= 5);
    }
}
