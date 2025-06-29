//! LLM provider trait and implementations.

use crate::{CoreError, CoreResult};
use crate::types::{CompletionRequest, CompletionResponse, ModelInfo};
use async_trait::async_trait;
use futures::Stream;

/// Trait that all LLM providers must implement
#[async_trait]
pub trait LLMProvider: Send + Sync + std::fmt::Debug {
    /// Complete a chat conversation
    async fn complete(&self, request: CompletionRequest) -> CoreResult<CompletionResponse>;

    /// Stream a chat completion (if supported)
    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> CoreResult<Box<dyn Stream<Item = CoreResult<CompletionResponse>> + Unpin + Send>> {
        // Default implementation falls back to non-streaming
        let response = self.complete(request).await?;
        let stream = futures::stream::once(async move { Ok(response) });
        Ok(Box::new(stream))
    }

    /// Get available models
    async fn get_models(&self) -> CoreResult<Vec<ModelInfo>>;

    /// Get provider name
    fn name(&self) -> &str;

    /// Check if provider supports function calling
    fn supports_function_calling(&self) -> bool {
        false
    }

    /// Check if provider supports streaming
    fn supports_streaming(&self) -> bool {
        false
    }

    /// Get provider configuration
    fn config(&self) -> &dyn ProviderConfig;

    /// Validate provider configuration
    async fn validate_config(&self) -> CoreResult<()> {
        Ok(())
    }

    /// Health check for the provider
    async fn health_check(&self) -> CoreResult<ProviderHealth> {
        // Default implementation - providers can override
        Ok(ProviderHealth {
            healthy: true,
            latency_ms: None,
            error_rate: 0.0,
            last_check: chrono::Utc::now(),
        })
    }
}

/// Provider configuration trait
pub trait ProviderConfig: Send + Sync + std::fmt::Debug {
    /// Get the API endpoint
    fn endpoint(&self) -> &str;

    /// Get the API key (if required)
    fn api_key(&self) -> Option<&str>;

    /// Get request timeout in milliseconds
    fn timeout_ms(&self) -> u64;

    /// Get maximum retries
    fn max_retries(&self) -> u32;
}

/// Provider health information
#[derive(Debug, Clone)]
pub struct ProviderHealth {
    /// Whether the provider is healthy
    pub healthy: bool,
    /// Response latency in milliseconds
    pub latency_ms: Option<u64>,
    /// Error rate (0.0 to 1.0)
    pub error_rate: f64,
    /// Last health check timestamp
    pub last_check: chrono::DateTime<chrono::Utc>,
}

/// Mock LLM provider for testing
#[derive(Debug)]
pub struct MockProvider {
    config: MockProviderConfig,
}

#[derive(Debug, Clone)]
pub struct MockProviderConfig {
    pub name: String,
    pub endpoint: String,
    pub timeout_ms: u64,
    pub max_retries: u32,
}

impl Default for MockProviderConfig {
    fn default() -> Self {
        Self {
            name: "mock".to_string(),
            endpoint: "http://localhost:8080".to_string(),
            timeout_ms: 30000,
            max_retries: 3,
        }
    }
}

impl ProviderConfig for MockProviderConfig {
    fn endpoint(&self) -> &str {
        &self.endpoint
    }

    fn api_key(&self) -> Option<&str> {
        None
    }

    fn timeout_ms(&self) -> u64 {
        self.timeout_ms
    }

    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl MockProvider {
    /// Create a new mock provider
    pub fn new() -> Self {
        Self {
            config: MockProviderConfig::default(),
        }
    }

    /// Create a mock provider with custom config
    pub fn with_config(config: MockProviderConfig) -> Self {
        Self { config }
    }
}

impl Default for MockProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LLMProvider for MockProvider {
    async fn complete(&self, request: CompletionRequest) -> CoreResult<CompletionResponse> {
        use crate::types::{Choice, Message, MessageRole, Usage, FinishReason};
        use std::collections::HashMap;

        // Simulate processing delay
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Generate a mock response based on the request
        let content = if request.messages.is_empty() {
            "Hello! I'm a mock AI assistant.".to_string()
        } else {
            let last_message = request.messages.last().unwrap();
            match &last_message.content {
                Some(content) => format!("Mock response to: {}", content),
                None => "Mock response to function call".to_string(),
            }
        };

        let response = CompletionResponse {
            id: format!("mock-{}", uuid::Uuid::new_v4()),
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp() as u64,
            model: request.model.clone(),
            choices: vec![Choice {
                index: 0,
                message: Message::assistant(content.clone()),
                finish_reason: Some(FinishReason::Stop),
                logprobs: None,
            }],
            usage: Usage::new(
                request.messages.iter().map(|m| m.content.as_ref().map_or(0, |c| c.len() / 4)).sum::<usize>() as u32,
                content.len() as u32 / 4,
            ),
            metadata: HashMap::new(),
        };

        Ok(response)
    }

    async fn get_models(&self) -> CoreResult<Vec<ModelInfo>> {
        use crate::types::{ModelCapability, ModelPricing};

        Ok(vec![
            ModelInfo {
                id: "mock-gpt-4".to_string(),
                name: "Mock GPT-4".to_string(),
                description: Some("Mock GPT-4 model for testing".to_string()),
                max_tokens: Some(8192),
                supports_functions: true,
                supports_streaming: true,
                capabilities: vec![
                    ModelCapability::TextCompletion,
                    ModelCapability::ChatCompletion,
                    ModelCapability::FunctionCalling,
                    ModelCapability::CodeGeneration,
                ],
                pricing: Some(ModelPricing {
                    input_cost_per_1k_tokens: Some(0.03),
                    output_cost_per_1k_tokens: Some(0.06),
                    currency: "USD".to_string(),
                }),
            },
            ModelInfo {
                id: "mock-gpt-3.5-turbo".to_string(),
                name: "Mock GPT-3.5 Turbo".to_string(),
                description: Some("Mock GPT-3.5 Turbo model for testing".to_string()),
                max_tokens: Some(4096),
                supports_functions: true,
                supports_streaming: true,
                capabilities: vec![
                    ModelCapability::TextCompletion,
                    ModelCapability::ChatCompletion,
                    ModelCapability::FunctionCalling,
                ],
                pricing: Some(ModelPricing {
                    input_cost_per_1k_tokens: Some(0.001),
                    output_cost_per_1k_tokens: Some(0.002),
                    currency: "USD".to_string(),
                }),
            },
        ])
    }

    fn name(&self) -> &str {
        &self.config.name
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

    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> CoreResult<Box<dyn Stream<Item = CoreResult<CompletionResponse>> + Unpin + Send>> {
        use futures::stream;
        use crate::types::{Choice, Message, MessageRole, Usage, FinishReason};
        use std::collections::HashMap;

        // Simulate streaming by breaking response into chunks
        let content = if request.messages.is_empty() {
            "Hello! I'm a mock AI assistant.".to_string()
        } else {
            let last_message = request.messages.last().unwrap();
            match &last_message.content {
                Some(content) => format!("Mock streaming response to: {}", content),
                None => "Mock streaming response to function call".to_string(),
            }
        };

        let words: Vec<&str> = content.split_whitespace().collect();
        let chunks: Vec<String> = words.chunks(2).map(|chunk| chunk.join(" ")).collect();

        let responses: Vec<CoreResult<CompletionResponse>> = chunks
            .into_iter()
            .enumerate()
            .map(|(i, chunk)| {
                Ok(CompletionResponse {
                    id: format!("mock-stream-{}", uuid::Uuid::new_v4()),
                    object: "chat.completion.chunk".to_string(),
                    created: chrono::Utc::now().timestamp() as u64,
                    model: request.model.clone(),
                    choices: vec![Choice {
                        index: 0,
                        message: Message::assistant(chunk.clone()),
                        finish_reason: if i == words.len() - 1 {
                            Some(FinishReason::Stop)
                        } else {
                            None
                        },
                        logprobs: None,
                    }],
                    usage: Usage::new(0, chunk.len() as u32 / 4),
                    metadata: HashMap::new(),
                })
            })
            .collect();

        let stream = stream::iter(responses);
        Ok(Box::new(stream))
    }
}

/// OpenAI provider configuration
#[derive(Debug, Clone)]
pub struct OpenAIConfig {
    pub api_key: String,
    pub endpoint: String,
    pub timeout_ms: u64,
    pub max_retries: u32,
}

impl Default for OpenAIConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default(),
            endpoint: "https://api.openai.com/v1".to_string(),
            timeout_ms: 30000,
            max_retries: 3,
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

/// OpenAI provider (placeholder implementation)
#[derive(Debug)]
pub struct OpenAIProvider {
    config: OpenAIConfig,
    client: reqwest::Client,
}

impl OpenAIProvider {
    /// Create a new OpenAI provider
    pub fn new(config: OpenAIConfig) -> CoreResult<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(config.timeout_ms))
            .build()
            .map_err(|e| CoreError::configuration_error(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { config, client })
    }
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    async fn complete(&self, _request: CompletionRequest) -> CoreResult<CompletionResponse> {
        // TODO: Implement actual OpenAI API integration
        Err(CoreError::execution_error("OpenAI provider not yet implemented"))
    }

    async fn get_models(&self) -> CoreResult<Vec<ModelInfo>> {
        // TODO: Implement actual model fetching from OpenAI API
        Err(CoreError::execution_error("OpenAI model listing not yet implemented"))
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
}

// Re-export provider implementations
pub mod anthropic;
pub mod google;
pub mod openai;
pub mod openrouter;

pub use anthropic::{AnthropicProvider, AnthropicConfig};
pub use google::{GoogleProvider, GoogleConfig};
pub use openai::{OpenAIProvider, OpenAIConfig};
pub use openrouter::{OpenRouterProvider, OpenRouterConfig};