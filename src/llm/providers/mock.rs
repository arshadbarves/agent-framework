// Mock provider implementation for testing AgentGraph LLM framework

#![allow(missing_docs)]

use super::super::*;
use std::time::SystemTime;

/// Mock provider for testing LLM operations
#[derive(Debug)]
pub struct MockProvider {
    /// Simulated delay
    delay: std::time::Duration,
    /// Simulated responses
    responses: Vec<String>,
    /// Current response index
    response_index: std::sync::Arc<std::sync::Mutex<usize>>,
}

impl MockProvider {
    /// Create a new mock provider
    pub fn new() -> Self {
        Self {
            delay: std::time::Duration::from_millis(100),
            responses: vec![
                "This is a mock response from the LLM.".to_string(),
                "Here's another simulated response.".to_string(),
                "Mock provider generating test content.".to_string(),
            ],
            response_index: std::sync::Arc::new(std::sync::Mutex::new(0)),
        }
    }

    /// Create mock provider with custom responses
    pub fn with_responses(responses: Vec<String>) -> Self {
        Self {
            delay: std::time::Duration::from_millis(100),
            responses,
            response_index: std::sync::Arc::new(std::sync::Mutex::new(0)),
        }
    }

    /// Set simulated delay
    pub fn with_delay(mut self, delay: std::time::Duration) -> Self {
        self.delay = delay;
        self
    }

    /// Get next response
    fn get_next_response(&self) -> String {
        let mut index = self.response_index.lock().unwrap();
        let response = self.responses[*index % self.responses.len()].clone();
        *index += 1;
        response
    }

    /// Simulate function call response
    fn create_function_call_response(&self, request: &CompletionRequest) -> Option<FunctionCall> {
        if let Some(functions) = &request.functions {
            if !functions.is_empty() {
                // Simulate calling the first available function
                let function = &functions[0];
                return Some(FunctionCall::new(
                    function.name.clone(),
                    serde_json::json!({"result": "mock_function_result"}),
                ));
            }
        }
        None
    }
}

impl Default for MockProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl LLMProvider for MockProvider {
    fn name(&self) -> &str {
        "mock"
    }

    fn supported_models(&self) -> Vec<String> {
        vec![
            "mock-gpt-4".to_string(),
            "mock-gpt-3.5-turbo".to_string(),
            "mock-claude-3".to_string(),
            "mock-llama-2".to_string(),
        ]
    }

    fn supports_function_calling(&self) -> bool {
        true // Mock provider supports everything for testing
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LLMError> {
        // Simulate network delay
        tokio::time::sleep(self.delay).await;

        // Check if model is supported
        if !self.supports_model(&request.model) {
            return Err(LLMError::ModelNotSupported {
                model: request.model,
                provider: self.name().to_string(),
            });
        }

        // Simulate token limit check
        if let Some(max_tokens) = request.max_tokens {
            if max_tokens > 4000 {
                return Err(LLMError::TokenLimitExceeded {
                    tokens: max_tokens,
                    limit: 4000,
                });
            }
        }

        let content = self.get_next_response();
        let mut message = Message::assistant(content);

        // Check for function calling
        let function_call = self.create_function_call_response(&request);
        let finish_reason = if function_call.is_some() {
            message.function_call = function_call;
            FinishReason::FunctionCall
        } else {
            FinishReason::Stop
        };

        let choice = Choice {
            index: 0,
            message,
            finish_reason,
        };

        // Simulate token usage
        let prompt_tokens = request.messages.iter()
            .map(|m| m.content.split_whitespace().count() as u32)
            .sum::<u32>();
        let completion_tokens = request.max_tokens.unwrap_or(100).min(200);

        let usage = TokenUsage::new(prompt_tokens, completion_tokens);

        Ok(CompletionResponse {
            id: format!("mock-{}", uuid::Uuid::new_v4()),
            model: request.model,
            choices: vec![choice],
            usage,
            metadata: HashMap::new(),
            timestamp: SystemTime::now(),
        })
    }

    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> Result<Box<dyn futures::Stream<Item = Result<CompletionResponse, LLMError>> + Unpin + Send>, LLMError> {
        // For mock streaming, we'll split the response into chunks
        let response = self.complete(request).await?;
        let content = &response.choices[0].message.content;
        
        // Split content into words for streaming simulation
        let words: Vec<&str> = content.split_whitespace().collect();
        let chunks: Vec<String> = words.chunks(3)
            .map(|chunk| chunk.join(" "))
            .collect();

        let chunk_count = chunks.len();
        let stream = futures::stream::iter(chunks.into_iter().enumerate().map(move |(i, chunk)| {
            let mut chunk_response = response.clone();
            chunk_response.choices[0].message.content = chunk;
            chunk_response.choices[0].finish_reason = if i == chunk_count - 1 {
                FinishReason::Stop
            } else {
                FinishReason::Length // Use Length to indicate partial response
            };
            Ok(chunk_response)
        }));

        Ok(Box::new(Box::pin(stream)))
    }

    async fn count_tokens(&self, text: &str, _model: &str) -> Result<u32, LLMError> {
        // Simple word-based token counting for mock
        Ok(text.split_whitespace().count() as u32)
    }

    fn get_pricing(&self, model: &str) -> Option<ModelPricing> {
        match model {
            "mock-gpt-4" => Some(ModelPricing {
                prompt_cost_per_1k: 0.001, // Very cheap for testing
                completion_cost_per_1k: 0.002,
                currency: "USD".to_string(),
            }),
            "mock-gpt-3.5-turbo" => Some(ModelPricing {
                prompt_cost_per_1k: 0.0005,
                completion_cost_per_1k: 0.001,
                currency: "USD".to_string(),
            }),
            "mock-claude-3" => Some(ModelPricing {
                prompt_cost_per_1k: 0.0008,
                completion_cost_per_1k: 0.0015,
                currency: "USD".to_string(),
            }),
            "mock-llama-2" => Some(ModelPricing {
                prompt_cost_per_1k: 0.0001, // Very cheap open source model
                completion_cost_per_1k: 0.0002,
                currency: "USD".to_string(),
            }),
            _ => None,
        }
    }
}

/// Mock provider builder for testing scenarios
pub struct MockProviderBuilder {
    responses: Vec<String>,
    delay: std::time::Duration,
    should_fail: bool,
    failure_error: Option<LLMError>,
}

impl MockProviderBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            responses: vec!["Mock response".to_string()],
            delay: std::time::Duration::from_millis(10),
            should_fail: false,
            failure_error: None,
        }
    }

    /// Add a response
    pub fn with_response(mut self, response: String) -> Self {
        self.responses.push(response);
        self
    }

    /// Set responses
    pub fn with_responses(mut self, responses: Vec<String>) -> Self {
        self.responses = responses;
        self
    }

    /// Set delay
    pub fn with_delay(mut self, delay: std::time::Duration) -> Self {
        self.delay = delay;
        self
    }

    /// Make provider fail with specific error
    pub fn with_failure(mut self, error: LLMError) -> Self {
        self.should_fail = true;
        self.failure_error = Some(error);
        self
    }

    /// Build the mock provider
    pub fn build(self) -> MockProvider {
        MockProvider::with_responses(self.responses).with_delay(self.delay)
    }
}

impl Default for MockProviderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_provider_creation() {
        let provider = MockProvider::new();
        assert_eq!(provider.name(), "mock");
        assert!(provider.supports_function_calling());
        assert!(provider.supports_streaming());
    }

    #[test]
    fn test_supported_models() {
        let provider = MockProvider::new();
        let models = provider.supported_models();
        
        assert!(models.contains(&"mock-gpt-4".to_string()));
        assert!(models.contains(&"mock-claude-3".to_string()));
        assert!(provider.supports_model("mock-gpt-4"));
        assert!(!provider.supports_model("real-gpt-4"));
    }

    #[test]
    fn test_custom_responses() {
        let responses = vec![
            "First response".to_string(),
            "Second response".to_string(),
        ];
        let provider = MockProvider::with_responses(responses.clone());
        
        // Test that responses cycle
        assert_eq!(provider.get_next_response(), "First response");
        assert_eq!(provider.get_next_response(), "Second response");
        assert_eq!(provider.get_next_response(), "First response"); // Cycles back
    }

    #[tokio::test]
    async fn test_mock_completion() {
        let provider = MockProvider::new();
        let request = CompletionRequest {
            model: "mock-gpt-4".to_string(),
            messages: vec![Message::user("Hello".to_string())],
            ..Default::default()
        };

        let response = provider.complete(request).await.unwrap();
        
        assert_eq!(response.model, "mock-gpt-4");
        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.choices[0].finish_reason, FinishReason::Stop);
        assert!(!response.choices[0].message.content.is_empty());
    }

    #[tokio::test]
    async fn test_function_calling() {
        let provider = MockProvider::new();
        let function = FunctionDefinition::new(
            "test_function".to_string(),
            "A test function".to_string(),
            serde_json::json!({"type": "object"}),
        );

        let request = CompletionRequest {
            model: "mock-gpt-4".to_string(),
            messages: vec![Message::user("Call a function".to_string())],
            functions: Some(vec![function]),
            ..Default::default()
        };

        let response = provider.complete(request).await.unwrap();
        
        assert_eq!(response.choices[0].finish_reason, FinishReason::FunctionCall);
        assert!(response.choices[0].message.function_call.is_some());
        
        let function_call = response.choices[0].message.function_call.as_ref().unwrap();
        assert_eq!(function_call.name, "test_function");
    }

    #[tokio::test]
    async fn test_token_counting() {
        let provider = MockProvider::new();
        let tokens = provider.count_tokens("Hello world test", "mock-gpt-4").await.unwrap();
        
        assert_eq!(tokens, 3); // Three words
    }

    #[test]
    fn test_pricing() {
        let provider = MockProvider::new();
        
        let gpt4_pricing = provider.get_pricing("mock-gpt-4").unwrap();
        assert_eq!(gpt4_pricing.prompt_cost_per_1k, 0.001);
        
        let llama_pricing = provider.get_pricing("mock-llama-2").unwrap();
        assert_eq!(llama_pricing.prompt_cost_per_1k, 0.0001);
        
        assert!(provider.get_pricing("invalid-model").is_none());
    }

    #[test]
    fn test_builder_pattern() {
        let provider = MockProviderBuilder::new()
            .with_response("Custom response".to_string())
            .with_delay(std::time::Duration::from_millis(50))
            .build();
        
        assert_eq!(provider.delay, std::time::Duration::from_millis(50));
        assert_eq!(provider.get_next_response(), "Mock response"); // Default first
        assert_eq!(provider.get_next_response(), "Custom response");
    }
}
