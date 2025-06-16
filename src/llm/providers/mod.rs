// LLM providers module for AgentGraph

#![allow(missing_docs)]

pub mod openai;
pub mod anthropic;
pub mod google;
pub mod openrouter;
pub mod mock;

pub use openai::OpenAIProvider;
pub use anthropic::AnthropicProvider;
pub use google::GoogleProvider;
pub use openrouter::OpenRouterProvider;
pub use mock::MockProvider;

// Re-export from parent module
pub use super::ProviderConfig;

use super::*;

/// Create provider from configuration
pub fn create_provider(name: &str, config: ProviderConfig) -> Result<Arc<dyn LLMProvider>, LLMError> {
    match name {
        "openai" => Ok(Arc::new(OpenAIProvider::with_config(config)?)),
        "anthropic" => Ok(Arc::new(AnthropicProvider::with_config(config)?)),
        "google" => Ok(Arc::new(GoogleProvider::with_config(config)?)),
        "openrouter" => Ok(Arc::new(OpenRouterProvider::new(openrouter::OpenRouterConfig {
            api_key: config.api_key.unwrap_or_default(),
            ..Default::default()
        }))),
        "mock" => Ok(Arc::new(MockProvider::new())),
        _ => Err(LLMError::ProviderNotFound {
            provider: name.to_string(),
        }),
    }
}

/// Get all available provider names
pub fn available_providers() -> Vec<&'static str> {
    vec!["openai", "anthropic", "google", "openrouter", "mock"]
}

/// Provider capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCapabilities {
    /// Provider name
    pub name: String,
    /// Supported models
    pub models: Vec<String>,
    /// Supports function calling
    pub function_calling: bool,
    /// Supports streaming
    pub streaming: bool,
    /// Supports embeddings
    pub embeddings: bool,
    /// Maximum context length
    pub max_context_length: Option<u32>,
    /// Supported languages
    pub languages: Vec<String>,
}

/// Get capabilities for a provider
pub fn get_provider_capabilities(provider: &dyn LLMProvider) -> ProviderCapabilities {
    ProviderCapabilities {
        name: provider.name().to_string(),
        models: provider.supported_models(),
        function_calling: provider.supports_function_calling(),
        streaming: provider.supports_streaming(),
        embeddings: false, // TODO: Add embedding support
        max_context_length: get_max_context_length(provider.name()),
        languages: get_supported_languages(provider.name()),
    }
}

/// Get maximum context length for provider
fn get_max_context_length(provider_name: &str) -> Option<u32> {
    match provider_name {
        "openai" => Some(32768), // GPT-4 32k
        "anthropic" => Some(100000), // Claude-2 100k
        "google" => Some(1000000), // Gemini 1M tokens
        "openrouter" => Some(128000), // Varies by model, this is a reasonable default
        _ => None,
    }
}

/// Get supported languages for provider
fn get_supported_languages(provider_name: &str) -> Vec<String> {
    match provider_name {
        "openai" | "anthropic" | "google" => vec![
            "en".to_string(), "es".to_string(), "fr".to_string(),
            "de".to_string(), "it".to_string(), "pt".to_string(),
            "ru".to_string(), "ja".to_string(), "ko".to_string(),
            "zh".to_string(), "ar".to_string(), "hi".to_string(),
        ],
        _ => vec!["en".to_string()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_available_providers() {
        let providers = available_providers();
        assert!(providers.contains(&"openai"));
        assert!(providers.contains(&"anthropic"));
        assert!(providers.contains(&"google"));
        assert!(providers.contains(&"openrouter"));
        assert!(providers.contains(&"mock"));
    }

    #[test]
    fn test_create_mock_provider() {
        let config = ProviderConfig {
            api_key: None,
            base_url: None,
            organization: None,
            headers: HashMap::new(),
            settings: HashMap::new(),
        };

        let provider = create_provider("mock", config).unwrap();
        assert_eq!(provider.name(), "mock");
    }

    #[test]
    fn test_provider_capabilities() {
        let provider = MockProvider::new();
        let capabilities = get_provider_capabilities(&provider);

        assert_eq!(capabilities.name, "mock");
        assert!(capabilities.function_calling); // MockProvider supports function calling
        assert!(capabilities.streaming); // MockProvider supports streaming
    }

    #[test]
    fn test_max_context_length() {
        assert_eq!(get_max_context_length("openai"), Some(32768));
        assert_eq!(get_max_context_length("anthropic"), Some(100000));
        assert_eq!(get_max_context_length("google"), Some(1000000));
        assert_eq!(get_max_context_length("openrouter"), Some(128000));
        assert_eq!(get_max_context_length("unknown"), None);
    }

    #[test]
    fn test_supported_languages() {
        let languages = get_supported_languages("google");
        assert!(languages.contains(&"en".to_string()));
        assert!(languages.contains(&"es".to_string()));
        assert!(languages.contains(&"zh".to_string()));

        let unknown_languages = get_supported_languages("unknown");
        assert_eq!(unknown_languages, vec!["en".to_string()]);
    }
}
