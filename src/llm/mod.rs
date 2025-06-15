// LLM Integration Framework for AgentGraph
// Provides multi-provider LLM support with function calling and streaming

#![allow(missing_docs)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use thiserror::Error;

pub mod providers;

/// LLM message role
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    /// System message for instructions
    System,
    /// User message from human
    User,
    /// Assistant message from LLM
    Assistant,
    /// Function call result
    Function,
}

/// LLM message content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message role
    pub role: MessageRole,
    /// Message content
    pub content: String,
    /// Optional function call information
    pub function_call: Option<FunctionCall>,
    /// Message metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Message timestamp
    pub timestamp: SystemTime,
}

impl Message {
    /// Create a new message
    pub fn new(role: MessageRole, content: String) -> Self {
        Self {
            role,
            content,
            function_call: None,
            metadata: HashMap::new(),
            timestamp: SystemTime::now(),
        }
    }
    
    /// Create system message
    pub fn system(content: String) -> Self {
        Self::new(MessageRole::System, content)
    }
    
    /// Create user message
    pub fn user(content: String) -> Self {
        Self::new(MessageRole::User, content)
    }
    
    /// Create assistant message
    pub fn assistant(content: String) -> Self {
        Self::new(MessageRole::Assistant, content)
    }
    
    /// Add function call
    pub fn with_function_call(mut self, function_call: FunctionCall) -> Self {
        self.function_call = Some(function_call);
        self
    }
    
    /// Add metadata
    pub fn with_metadata<T: Serialize>(mut self, key: String, value: T) -> Self {
        self.metadata.insert(
            key,
            serde_json::to_value(value).unwrap_or(serde_json::Value::Null),
        );
        self
    }
}

/// Function call information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    /// Function name
    pub name: String,
    /// Function arguments as JSON
    pub arguments: serde_json::Value,
    /// Function call ID for tracking
    pub id: Option<String>,
}

impl FunctionCall {
    /// Create a new function call
    pub fn new(name: String, arguments: serde_json::Value) -> Self {
        Self {
            name,
            arguments,
            id: Some(uuid::Uuid::new_v4().to_string()),
        }
    }
}

/// LLM completion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// Model to use
    pub model: String,
    /// Conversation messages
    pub messages: Vec<Message>,
    /// Maximum tokens to generate
    pub max_tokens: Option<u32>,
    /// Temperature for randomness (0.0-2.0)
    pub temperature: Option<f32>,
    /// Top-p for nucleus sampling
    pub top_p: Option<f32>,
    /// Stop sequences
    pub stop: Option<Vec<String>>,
    /// Enable streaming
    pub stream: bool,
    /// Available functions for calling
    pub functions: Option<Vec<FunctionDefinition>>,
    /// Function call behavior
    pub function_call: Option<FunctionCallBehavior>,
    /// Request metadata
    pub metadata: HashMap<String, String>,
}

impl Default for CompletionRequest {
    fn default() -> Self {
        Self {
            model: "gpt-3.5-turbo".to_string(),
            messages: Vec::new(),
            max_tokens: Some(1000),
            temperature: Some(0.7),
            top_p: Some(1.0),
            stop: None,
            stream: false,
            functions: None,
            function_call: None,
            metadata: HashMap::new(),
        }
    }
}

/// Function call behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FunctionCallBehavior {
    /// No function calling
    None,
    /// Automatic function calling
    Auto,
    /// Force specific function
    Force(String),
}

/// Function definition for LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    /// Function name
    pub name: String,
    /// Function description
    pub description: String,
    /// Function parameters schema
    pub parameters: serde_json::Value,
    /// Whether function is required
    pub required: bool,
}

impl FunctionDefinition {
    /// Create a new function definition
    pub fn new(name: String, description: String, parameters: serde_json::Value) -> Self {
        Self {
            name,
            description,
            parameters,
            required: false,
        }
    }
    
    /// Mark function as required
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
}

/// LLM completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// Response ID
    pub id: String,
    /// Model used
    pub model: String,
    /// Generated choices
    pub choices: Vec<Choice>,
    /// Token usage information
    pub usage: TokenUsage,
    /// Response metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Response timestamp
    pub timestamp: SystemTime,
}

/// Response choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    /// Choice index
    pub index: u32,
    /// Generated message
    pub message: Message,
    /// Finish reason
    pub finish_reason: FinishReason,
}

/// Reason why generation finished
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FinishReason {
    /// Natural completion
    Stop,
    /// Hit token limit
    Length,
    /// Function call triggered
    FunctionCall,
    /// Content filtered
    ContentFilter,
    /// Error occurred
    Error,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Prompt tokens
    pub prompt_tokens: u32,
    /// Completion tokens
    pub completion_tokens: u32,
    /// Total tokens
    pub total_tokens: u32,
    /// Estimated cost in USD
    pub estimated_cost: Option<f64>,
}

impl TokenUsage {
    /// Create new token usage
    pub fn new(prompt_tokens: u32, completion_tokens: u32) -> Self {
        Self {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
            estimated_cost: None,
        }
    }
    
    /// Add cost estimation
    pub fn with_cost(mut self, cost: f64) -> Self {
        self.estimated_cost = Some(cost);
        self
    }
}

/// LLM provider trait
#[async_trait::async_trait]
pub trait LLMProvider: Send + Sync + std::fmt::Debug {
    /// Get provider name
    fn name(&self) -> &str;
    
    /// Get supported models
    fn supported_models(&self) -> Vec<String>;
    
    /// Check if model is supported
    fn supports_model(&self, model: &str) -> bool {
        self.supported_models().contains(&model.to_string())
    }
    
    /// Check if provider supports function calling
    fn supports_function_calling(&self) -> bool {
        false
    }
    
    /// Check if provider supports streaming
    fn supports_streaming(&self) -> bool {
        false
    }
    
    /// Complete a request
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LLMError>;
    
    /// Stream a completion
    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> Result<Box<dyn futures::Stream<Item = Result<CompletionResponse, LLMError>> + Unpin + Send>, LLMError> {
        // Default implementation for non-streaming providers
        let response = self.complete(request).await?;
        let stream = futures::stream::once(async move { Ok(response) });
        Ok(Box::new(Box::pin(stream)))
    }
    
    /// Get token count for text
    async fn count_tokens(&self, text: &str, model: &str) -> Result<u32, LLMError>;
    
    /// Get model pricing information
    fn get_pricing(&self, model: &str) -> Option<ModelPricing>;
}

/// Model pricing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    /// Cost per 1K prompt tokens in USD
    pub prompt_cost_per_1k: f64,
    /// Cost per 1K completion tokens in USD
    pub completion_cost_per_1k: f64,
    /// Currency
    pub currency: String,
}

impl ModelPricing {
    /// Calculate cost for token usage
    pub fn calculate_cost(&self, usage: &TokenUsage) -> f64 {
        let prompt_cost = (usage.prompt_tokens as f64 / 1000.0) * self.prompt_cost_per_1k;
        let completion_cost = (usage.completion_tokens as f64 / 1000.0) * self.completion_cost_per_1k;
        prompt_cost + completion_cost
    }
}

/// LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    /// Default provider
    pub default_provider: String,
    /// Provider configurations
    pub providers: HashMap<String, ProviderConfig>,
    /// Default model per provider
    pub default_models: HashMap<String, String>,
    /// Request timeout
    pub timeout: Duration,
    /// Retry configuration
    pub retry_config: RetryConfig,
    /// Enable cost tracking
    pub cost_tracking: bool,
    /// Maximum cost per request
    pub max_cost_per_request: Option<f64>,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            default_provider: "openai".to_string(),
            providers: HashMap::new(),
            default_models: HashMap::new(),
            timeout: Duration::from_secs(30),
            retry_config: RetryConfig::default(),
            cost_tracking: true,
            max_cost_per_request: Some(1.0), // $1 max per request
        }
    }
}

/// Provider-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// API key
    pub api_key: Option<String>,
    /// API base URL
    pub base_url: Option<String>,
    /// Organization ID
    pub organization: Option<String>,
    /// Additional headers
    pub headers: HashMap<String, String>,
    /// Provider-specific settings
    pub settings: HashMap<String, serde_json::Value>,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum retry attempts
    pub max_attempts: u32,
    /// Base delay between retries
    pub base_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(1000),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
        }
    }
}

/// LLM manager for handling multiple providers
#[derive(Debug)]
pub struct LLMManager {
    /// Configuration
    config: LLMConfig,
    /// Registered providers
    providers: HashMap<String, Arc<dyn LLMProvider>>,
    /// Request statistics
    stats: Arc<std::sync::Mutex<LLMStats>>,
}

impl LLMManager {
    /// Create a new LLM manager
    pub fn new(config: LLMConfig) -> Self {
        Self {
            config,
            providers: HashMap::new(),
            stats: Arc::new(std::sync::Mutex::new(LLMStats::default())),
        }
    }
    
    /// Register a provider
    pub fn register_provider(&mut self, name: String, provider: Arc<dyn LLMProvider>) {
        self.providers.insert(name, provider);
    }
    
    /// Get provider by name
    pub fn get_provider(&self, name: &str) -> Option<&Arc<dyn LLMProvider>> {
        self.providers.get(name)
    }
    
    /// Complete using default provider
    pub async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LLMError> {
        self.complete_with_provider(&self.config.default_provider, request).await
    }
    
    /// Complete using specific provider
    pub async fn complete_with_provider(
        &self,
        provider_name: &str,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, LLMError> {
        let provider = self.get_provider(provider_name)
            .ok_or_else(|| LLMError::ProviderNotFound {
                provider: provider_name.to_string(),
            })?;
        
        // Check cost limits
        if let Some(max_cost) = self.config.max_cost_per_request {
            if let Some(estimated_cost) = self.estimate_cost(&request, provider_name).await? {
                if estimated_cost > max_cost {
                    return Err(LLMError::CostLimitExceeded {
                        estimated_cost,
                        limit: max_cost,
                    });
                }
            }
        }
        
        // Execute with retry logic
        let mut attempts = 0;
        let mut delay = self.config.retry_config.base_delay;
        
        loop {
            attempts += 1;
            
            match provider.complete(request.clone()).await {
                Ok(mut response) => {
                    // Add cost information if tracking enabled
                    if self.config.cost_tracking {
                        if let Some(pricing) = provider.get_pricing(&request.model) {
                            let cost = pricing.calculate_cost(&response.usage);
                            response.usage.estimated_cost = Some(cost);
                        }
                    }
                    
                    // Update statistics
                    self.update_stats(&response, provider_name);
                    
                    return Ok(response);
                }
                Err(e) => {
                    if attempts >= self.config.retry_config.max_attempts {
                        return Err(e);
                    }
                    
                    // Check if error is retryable
                    if !self.is_retryable_error(&e) {
                        return Err(e);
                    }
                    
                    // Wait before retry
                    tokio::time::sleep(delay).await;
                    delay = std::cmp::min(
                        Duration::from_millis(
                            (delay.as_millis() as f64 * self.config.retry_config.backoff_multiplier) as u64
                        ),
                        self.config.retry_config.max_delay,
                    );
                }
            }
        }
    }
    
    /// Estimate cost for a request
    pub async fn estimate_cost(
        &self,
        request: &CompletionRequest,
        provider_name: &str,
    ) -> Result<Option<f64>, LLMError> {
        let provider = self.get_provider(provider_name)
            .ok_or_else(|| LLMError::ProviderNotFound {
                provider: provider_name.to_string(),
            })?;
        
        if let Some(pricing) = provider.get_pricing(&request.model) {
            // Estimate prompt tokens
            let prompt_text = request.messages.iter()
                .map(|m| m.content.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            
            let prompt_tokens = provider.count_tokens(&prompt_text, &request.model).await?;
            let completion_tokens = request.max_tokens.unwrap_or(1000);
            
            let usage = TokenUsage::new(prompt_tokens, completion_tokens);
            Ok(Some(pricing.calculate_cost(&usage)))
        } else {
            Ok(None)
        }
    }
    
    /// Check if error is retryable
    fn is_retryable_error(&self, error: &LLMError) -> bool {
        matches!(error, 
            LLMError::NetworkError { .. } |
            LLMError::RateLimitExceeded { .. } |
            LLMError::ServerError { .. }
        )
    }
    
    /// Update statistics
    fn update_stats(&self, response: &CompletionResponse, provider_name: &str) {
        let mut stats = self.stats.lock().unwrap();
        stats.total_requests += 1;
        stats.total_tokens += response.usage.total_tokens as u64;
        
        if let Some(cost) = response.usage.estimated_cost {
            stats.total_cost += cost;
        }
        
        *stats.requests_by_provider.entry(provider_name.to_string()).or_insert(0) += 1;
        *stats.requests_by_model.entry(response.model.clone()).or_insert(0) += 1;
    }
    
    /// Get statistics
    pub fn get_stats(&self) -> LLMStats {
        self.stats.lock().unwrap().clone()
    }
    
    /// Get configuration
    pub fn config(&self) -> &LLMConfig {
        &self.config
    }
}

/// LLM usage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LLMStats {
    /// Total requests made
    pub total_requests: u64,
    /// Total tokens used
    pub total_tokens: u64,
    /// Total cost incurred
    pub total_cost: f64,
    /// Requests by provider
    pub requests_by_provider: HashMap<String, u64>,
    /// Requests by model
    pub requests_by_model: HashMap<String, u64>,
    /// Average response time
    pub avg_response_time: Duration,
}

/// Errors that can occur in LLM operations
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum LLMError {
    /// Provider not found
    #[error("LLM provider not found: {provider}")]
    ProviderNotFound { provider: String },
    
    /// Model not supported
    #[error("Model not supported: {model} by provider {provider}")]
    ModelNotSupported { model: String, provider: String },
    
    /// Authentication error
    #[error("Authentication failed for provider {provider}: {message}")]
    AuthenticationError { provider: String, message: String },
    
    /// Rate limit exceeded
    #[error("Rate limit exceeded for provider {provider}")]
    RateLimitExceeded { provider: String },
    
    /// Cost limit exceeded
    #[error("Cost limit exceeded: ${estimated_cost:.2} > ${limit:.2}")]
    CostLimitExceeded { estimated_cost: f64, limit: f64 },
    
    /// Network error
    #[error("Network error: {message}")]
    NetworkError { message: String },
    
    /// Server error
    #[error("Server error from {provider}: {message}")]
    ServerError { provider: String, message: String },
    
    /// Invalid request
    #[error("Invalid request: {message}")]
    InvalidRequest { message: String },
    
    /// Function calling error
    #[error("Function calling error: {message}")]
    FunctionCallError { message: String },
    
    /// Token limit exceeded
    #[error("Token limit exceeded: {tokens} > {limit}")]
    TokenLimitExceeded { tokens: u32, limit: u32 },
    
    /// Configuration error
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },
    
    /// System error
    #[error("System error: {message}")]
    SystemError { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = Message::user("Hello, world!".to_string())
            .with_metadata("test".to_string(), "value");
        
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.content, "Hello, world!");
        assert_eq!(msg.metadata.get("test"), Some(&serde_json::json!("value")));
    }

    #[test]
    fn test_function_call() {
        let args = serde_json::json!({"param": "value"});
        let call = FunctionCall::new("test_function".to_string(), args.clone());
        
        assert_eq!(call.name, "test_function");
        assert_eq!(call.arguments, args);
        assert!(call.id.is_some());
    }

    #[test]
    fn test_token_usage() {
        let usage = TokenUsage::new(100, 50).with_cost(0.01);
        
        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.completion_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
        assert_eq!(usage.estimated_cost, Some(0.01));
    }

    #[test]
    fn test_model_pricing() {
        let pricing = ModelPricing {
            prompt_cost_per_1k: 0.001,
            completion_cost_per_1k: 0.002,
            currency: "USD".to_string(),
        };
        
        let usage = TokenUsage::new(1000, 500);
        let cost = pricing.calculate_cost(&usage);
        
        assert_eq!(cost, 0.002); // (1000/1000 * 0.001) + (500/1000 * 0.002)
    }

    #[test]
    fn test_completion_request_default() {
        let request = CompletionRequest::default();
        
        assert_eq!(request.model, "gpt-3.5-turbo");
        assert_eq!(request.max_tokens, Some(1000));
        assert_eq!(request.temperature, Some(0.7));
        assert!(!request.stream);
    }
}
