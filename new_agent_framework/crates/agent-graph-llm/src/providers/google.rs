//! Google provider implementation for Gemini models.

use crate::{CoreError, CoreResult};
use crate::types::{CompletionRequest, CompletionResponse, Message, MessageRole, Choice, Usage, FinishReason, ModelInfo, ModelCapability, ModelPricing};
use crate::providers::{LLMProvider, ProviderConfig};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Google provider for Gemini models
#[derive(Debug)]
pub struct GoogleProvider {
    config: GoogleConfig,
    client: Client,
}

/// Configuration for Google provider
#[derive(Debug, Clone)]
pub struct GoogleConfig {
    pub api_key: String,
    pub endpoint: String,
    pub timeout_ms: u64,
    pub max_retries: u32,
}

impl Default for GoogleConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("GOOGLE_API_KEY").unwrap_or_default(),
            endpoint: "https://generativelanguage.googleapis.com".to_string(),
            timeout_ms: 60000,
            max_retries: 3,
        }
    }
}

impl ProviderConfig for GoogleConfig {
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

/// Google Gemini API request format
#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GeminiGenerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    safety_settings: Option<Vec<GeminiSafetySetting>>,
}

/// Gemini content structure
#[derive(Debug, Serialize, Deserialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

/// Gemini content part
#[derive(Debug, Serialize, Deserialize)]
struct GeminiPart {
    text: String,
}

/// Gemini generation configuration
#[derive(Debug, Serialize)]
struct GeminiGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
}

/// Gemini safety setting
#[derive(Debug, Serialize)]
struct GeminiSafetySetting {
    category: String,
    threshold: String,
}

/// Gemini API response format
#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: Option<GeminiUsageMetadata>,
}

/// Gemini candidate response
#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
    #[serde(rename = "finishReason")]
    finish_reason: Option<String>,
    index: Option<u32>,
    #[serde(rename = "safetyRatings")]
    safety_ratings: Option<Vec<GeminiSafetyRating>>,
}

/// Gemini safety rating
#[derive(Debug, Deserialize)]
struct GeminiSafetyRating {
    category: String,
    probability: String,
}

/// Gemini usage metadata
#[derive(Debug, Deserialize)]
struct GeminiUsageMetadata {
    #[serde(rename = "promptTokenCount")]
    prompt_token_count: Option<u32>,
    #[serde(rename = "candidatesTokenCount")]
    candidates_token_count: Option<u32>,
    #[serde(rename = "totalTokenCount")]
    total_token_count: Option<u32>,
}

impl GoogleProvider {
    /// Create a new Google provider
    pub fn new(config: GoogleConfig) -> CoreResult<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_millis(config.timeout_ms))
            .build()
            .map_err(|e| CoreError::configuration_error(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { config, client })
    }

    /// Convert our message format to Gemini format
    fn convert_messages(&self, messages: &[Message]) -> Vec<GeminiContent> {
        let mut gemini_contents = Vec::new();

        for message in messages {
            if let Some(content) = &message.content {
                let role = match message.role {
                    MessageRole::System => "user", // Gemini doesn't have system role, convert to user
                    MessageRole::User => "user",
                    MessageRole::Assistant => "model",
                    MessageRole::Function => "user", // Convert function results to user messages
                };

                let text = if message.role == MessageRole::System {
                    format!("System: {}", content)
                } else if message.role == MessageRole::Function {
                    format!(
                        "Function result from {}: {}",
                        message.name.as_deref().unwrap_or("unknown"),
                        content
                    )
                } else {
                    content.clone()
                };

                gemini_contents.push(GeminiContent {
                    role: role.to_string(),
                    parts: vec![GeminiPart { text }],
                });
            }
        }

        gemini_contents
    }

    /// Convert Gemini response to our format
    fn convert_response(&self, gemini_response: GeminiResponse, model: String) -> CoreResult<CompletionResponse> {
        if gemini_response.candidates.is_empty() {
            return Err(CoreError::execution_error("No candidates in response"));
        }

        let candidate = &gemini_response.candidates[0];
        let content = candidate.content.parts
            .iter()
            .map(|part| part.text.clone())
            .collect::<Vec<_>>()
            .join("");

        let finish_reason = match candidate.finish_reason.as_deref() {
            Some("STOP") => Some(FinishReason::Stop),
            Some("MAX_TOKENS") => Some(FinishReason::Length),
            Some("SAFETY") => Some(FinishReason::ContentFilter),
            Some("RECITATION") => Some(FinishReason::ContentFilter),
            _ => None,
        };

        let usage = if let Some(usage_meta) = &gemini_response.usage_metadata {
            Usage::new(
                usage_meta.prompt_token_count.unwrap_or(0),
                usage_meta.candidates_token_count.unwrap_or(0),
            )
        } else {
            Usage::new(0, 0)
        };

        Ok(CompletionResponse {
            id: format!("gemini-{}", uuid::Uuid::new_v4()),
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp() as u64,
            model,
            choices: vec![Choice {
                index: candidate.index.unwrap_or(0),
                message: Message::assistant(content),
                finish_reason,
                logprobs: None,
            }],
            usage,
            metadata: HashMap::new(),
        })
    }
}

#[async_trait]
impl LLMProvider for GoogleProvider {
    async fn complete(&self, request: CompletionRequest) -> CoreResult<CompletionResponse> {
        let contents = self.convert_messages(&request.messages);

        let generation_config = GeminiGenerationConfig {
            temperature: request.temperature,
            top_p: request.top_p,
            top_k: None,
            max_output_tokens: request.max_tokens,
            stop_sequences: request.stop,
        };

        let gemini_request = GeminiRequest {
            contents,
            generation_config: Some(generation_config),
            safety_settings: Some(vec![
                GeminiSafetySetting {
                    category: "HARM_CATEGORY_HARASSMENT".to_string(),
                    threshold: "BLOCK_MEDIUM_AND_ABOVE".to_string(),
                },
                GeminiSafetySetting {
                    category: "HARM_CATEGORY_HATE_SPEECH".to_string(),
                    threshold: "BLOCK_MEDIUM_AND_ABOVE".to_string(),
                },
                GeminiSafetySetting {
                    category: "HARM_CATEGORY_SEXUALLY_EXPLICIT".to_string(),
                    threshold: "BLOCK_MEDIUM_AND_ABOVE".to_string(),
                },
                GeminiSafetySetting {
                    category: "HARM_CATEGORY_DANGEROUS_CONTENT".to_string(),
                    threshold: "BLOCK_MEDIUM_AND_ABOVE".to_string(),
                },
            ]),
        };

        let url = format!(
            "{}/v1beta/models/{}:generateContent?key={}",
            self.config.endpoint,
            request.model,
            self.config.api_key
        );

        let response = self.client
            .post(&url)
            .json(&gemini_request)
            .send()
            .await
            .map_err(|e| CoreError::execution_error(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(CoreError::execution_error(format!(
                "Google API error: {} - {}",
                response.status(),
                error_text
            )));
        }

        let gemini_response: GeminiResponse = response
            .json()
            .await
            .map_err(|e| CoreError::execution_error(format!("Failed to parse response: {}", e)))?;

        self.convert_response(gemini_response, request.model)
    }

    async fn get_models(&self) -> CoreResult<Vec<ModelInfo>> {
        // Google doesn't have a models endpoint for Gemini, so we return known models
        Ok(vec![
            ModelInfo {
                id: "gemini-1.5-pro".to_string(),
                name: "Gemini 1.5 Pro".to_string(),
                description: Some("Most capable multimodal model".to_string()),
                max_tokens: Some(2097152), // 2M tokens
                supports_functions: true,
                supports_streaming: true,
                capabilities: vec![
                    ModelCapability::TextCompletion,
                    ModelCapability::ChatCompletion,
                    ModelCapability::CodeGeneration,
                    ModelCapability::ImageUnderstanding,
                ],
                pricing: Some(ModelPricing {
                    input_cost_per_1k_tokens: Some(0.00125),
                    output_cost_per_1k_tokens: Some(0.00375),
                    currency: "USD".to_string(),
                }),
            },
            ModelInfo {
                id: "gemini-1.5-flash".to_string(),
                name: "Gemini 1.5 Flash".to_string(),
                description: Some("Fast and versatile multimodal model".to_string()),
                max_tokens: Some(1048576), // 1M tokens
                supports_functions: true,
                supports_streaming: true,
                capabilities: vec![
                    ModelCapability::TextCompletion,
                    ModelCapability::ChatCompletion,
                    ModelCapability::CodeGeneration,
                    ModelCapability::ImageUnderstanding,
                ],
                pricing: Some(ModelPricing {
                    input_cost_per_1k_tokens: Some(0.000075),
                    output_cost_per_1k_tokens: Some(0.0003),
                    currency: "USD".to_string(),
                }),
            },
            ModelInfo {
                id: "gemini-pro".to_string(),
                name: "Gemini Pro".to_string(),
                description: Some("Best model for scaling across a wide range of tasks".to_string()),
                max_tokens: Some(32768),
                supports_functions: true,
                supports_streaming: true,
                capabilities: vec![
                    ModelCapability::TextCompletion,
                    ModelCapability::ChatCompletion,
                    ModelCapability::CodeGeneration,
                ],
                pricing: Some(ModelPricing {
                    input_cost_per_1k_tokens: Some(0.0005),
                    output_cost_per_1k_tokens: Some(0.0015),
                    currency: "USD".to_string(),
                }),
            },
        ])
    }

    fn name(&self) -> &str {
        "google"
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
            return Err(CoreError::configuration_error("Google API key is required"));
        }
        Ok(())
    }
}