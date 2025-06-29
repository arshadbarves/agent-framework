//! LLM client abstraction for unified access to multiple providers.

use crate::{CoreError, CoreResult};
use crate::types::{CompletionRequest, CompletionResponse, Message, FunctionCall, ModelInfo};
use crate::providers::LLMProvider;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Unified LLM client that manages multiple providers
#[derive(Debug)]
pub struct LLMClient {
    /// Registered providers
    providers: Arc<RwLock<HashMap<String, Arc<dyn LLMProvider>>>>,
    /// Client configuration
    config: LLMClientConfig,
    /// Default provider name
    default_provider: Option<String>,
}

/// LLM client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMClientConfig {
    /// Default timeout for requests in milliseconds
    pub default_timeout_ms: u64,
    /// Maximum retries for failed requests
    pub max_retries: u32,
    /// Enable request/response logging
    pub enable_logging: bool,
    /// Rate limiting configuration
    pub rate_limit: Option<RateLimitConfig>,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum requests per minute
    pub requests_per_minute: u32,
    /// Maximum tokens per minute
    pub tokens_per_minute: u32,
}

impl Default for LLMClientConfig {
    fn default() -> Self {
        Self {
            default_timeout_ms: 30000, // 30 seconds
            max_retries: 3,
            enable_logging: true,
            rate_limit: Some(RateLimitConfig {
                requests_per_minute: 60,
                tokens_per_minute: 10000,
            }),
        }
    }
}

impl LLMClient {
    /// Create a new LLM client
    pub fn new(config: LLMClientConfig) -> Self {
        Self {
            providers: Arc::new(RwLock::new(HashMap::new())),
            config,
            default_provider: None,
        }
    }

    /// Register a new LLM provider
    pub async fn register_provider(
        &mut self,
        name: String,
        provider: Arc<dyn LLMProvider>,
    ) -> CoreResult<()> {
        let mut providers = self.providers.write().await;
        providers.insert(name.clone(), provider);
        
        // Set as default if it's the first provider
        if self.default_provider.is_none() {
            self.default_provider = Some(name);
        }
        
        Ok(())
    }

    /// Get a provider by name
    pub async fn get_provider(&self, name: &str) -> Option<Arc<dyn LLMProvider>> {
        let providers = self.providers.read().await;
        providers.get(name).cloned()
    }

    /// List all registered providers
    pub async fn list_providers(&self) -> Vec<String> {
        let providers = self.providers.read().await;
        providers.keys().cloned().collect()
    }

    /// Set the default provider
    pub fn set_default_provider(&mut self, name: String) -> CoreResult<()> {
        self.default_provider = Some(name);
        Ok(())
    }

    /// Get the default provider name
    pub fn default_provider(&self) -> Option<&String> {
        self.default_provider.as_ref()
    }

    /// Complete a chat conversation using the specified provider
    pub async fn complete(
        &self,
        request: CompletionRequest,
        provider_name: Option<&str>,
    ) -> CoreResult<CompletionResponse> {
        let provider_name = provider_name
            .or(self.default_provider.as_deref())
            .ok_or_else(|| CoreError::configuration_error("No provider specified and no default provider set"))?;

        let provider = self.get_provider(provider_name).await
            .ok_or_else(|| CoreError::configuration_error(format!("Provider not found: {}", provider_name)))?;

        // Apply rate limiting if configured
        if let Some(rate_limit) = &self.config.rate_limit {
            self.apply_rate_limit(rate_limit).await?;
        }

        // Execute with retries
        let mut last_error = None;
        for attempt in 0..=self.config.max_retries {
            match provider.complete(request.clone()).await {
                Ok(response) => {
                    if self.config.enable_logging {
                        tracing::info!(
                            provider = provider_name,
                            attempt = attempt + 1,
                            tokens_used = response.usage.total_tokens,
                            "LLM completion successful"
                        );
                    }
                    return Ok(response);
                }
                Err(error) => {
                    last_error = Some(error);
                    if attempt < self.config.max_retries {
                        if self.config.enable_logging {
                            tracing::warn!(
                                provider = provider_name,
                                attempt = attempt + 1,
                                error = %last_error.as_ref().unwrap(),
                                "LLM completion failed, retrying"
                            );
                        }
                        // Exponential backoff
                        let delay = std::time::Duration::from_millis(100 * (2_u64.pow(attempt)));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| CoreError::execution_error("All retry attempts failed")))
    }

    /// Stream a chat completion (if supported by provider)
    pub async fn stream(
        &self,
        request: CompletionRequest,
        provider_name: Option<&str>,
    ) -> CoreResult<Box<dyn futures::Stream<Item = CoreResult<CompletionResponse>> + Unpin + Send>> {
        let provider_name = provider_name
            .or(self.default_provider.as_deref())
            .ok_or_else(|| CoreError::configuration_error("No provider specified and no default provider set"))?;

        let provider = self.get_provider(provider_name).await
            .ok_or_else(|| CoreError::configuration_error(format!("Provider not found: {}", provider_name)))?;

        provider.stream(request).await
    }

    /// Get available models from a provider
    pub async fn get_models(&self, provider_name: Option<&str>) -> CoreResult<Vec<ModelInfo>> {
        let provider_name = provider_name
            .or(self.default_provider.as_deref())
            .ok_or_else(|| CoreError::configuration_error("No provider specified and no default provider set"))?;

        let provider = self.get_provider(provider_name).await
            .ok_or_else(|| CoreError::configuration_error(format!("Provider not found: {}", provider_name)))?;

        provider.get_models().await
    }

    /// Check if a provider supports function calling
    pub async fn supports_function_calling(&self, provider_name: Option<&str>) -> CoreResult<bool> {
        let provider_name = provider_name
            .or(self.default_provider.as_deref())
            .ok_or_else(|| CoreError::configuration_error("No provider specified and no default provider set"))?;

        let provider = self.get_provider(provider_name).await
            .ok_or_else(|| CoreError::configuration_error(format!("Provider not found: {}", provider_name)))?;

        Ok(provider.supports_function_calling())
    }

    /// Check if a provider supports streaming
    pub async fn supports_streaming(&self, provider_name: Option<&str>) -> CoreResult<bool> {
        let provider_name = provider_name
            .or(self.default_provider.as_deref())
            .ok_or_else(|| CoreError::configuration_error("No provider specified and no default provider set"))?;

        let provider = self.get_provider(provider_name).await
            .ok_or_else(|| CoreError::configuration_error(format!("Provider not found: {}", provider_name)))?;

        Ok(provider.supports_streaming())
    }

    /// Get client configuration
    pub fn config(&self) -> &LLMClientConfig {
        &self.config
    }

    /// Apply rate limiting (placeholder implementation)
    async fn apply_rate_limit(&self, _rate_limit: &RateLimitConfig) -> CoreResult<()> {
        // TODO: Implement actual rate limiting logic
        // This would typically involve tracking request counts and timing
        Ok(())
    }
}

/// Builder for LLM client
#[derive(Debug)]
pub struct LLMClientBuilder {
    config: LLMClientConfig,
    providers: Vec<(String, Arc<dyn LLMProvider>)>,
    default_provider: Option<String>,
}

impl LLMClientBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: LLMClientConfig::default(),
            providers: Vec::new(),
            default_provider: None,
        }
    }

    /// Set the configuration
    pub fn with_config(mut self, config: LLMClientConfig) -> Self {
        self.config = config;
        self
    }

    /// Add a provider
    pub fn with_provider(mut self, name: String, provider: Arc<dyn LLMProvider>) -> Self {
        self.providers.push((name, provider));
        self
    }

    /// Set the default provider
    pub fn with_default_provider(mut self, name: String) -> Self {
        self.default_provider = Some(name);
        self
    }

    /// Build the client
    pub async fn build(self) -> CoreResult<LLMClient> {
        let mut client = LLMClient::new(self.config);
        
        // Register all providers
        for (name, provider) in self.providers {
            client.register_provider(name, provider).await?;
        }
        
        // Set default provider if specified
        if let Some(default) = self.default_provider {
            client.set_default_provider(default)?;
        }
        
        Ok(client)
    }
}

impl Default for LLMClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}