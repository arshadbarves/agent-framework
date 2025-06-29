//! Common types for LLM interactions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Request for LLM completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// Model to use for completion
    pub model: String,
    /// Messages in the conversation
    pub messages: Vec<Message>,
    /// Maximum number of tokens to generate
    pub max_tokens: Option<u32>,
    /// Temperature for randomness (0.0 to 2.0)
    pub temperature: Option<f32>,
    /// Top-p sampling parameter
    pub top_p: Option<f32>,
    /// Number of completions to generate
    pub n: Option<u32>,
    /// Whether to stream the response
    pub stream: Option<bool>,
    /// Stop sequences
    pub stop: Option<Vec<String>>,
    /// Presence penalty (-2.0 to 2.0)
    pub presence_penalty: Option<f32>,
    /// Frequency penalty (-2.0 to 2.0)
    pub frequency_penalty: Option<f32>,
    /// Functions available for calling
    pub functions: Option<Vec<FunctionDefinition>>,
    /// Function call behavior
    pub function_call: Option<FunctionCallBehavior>,
    /// Additional provider-specific parameters
    pub extra_params: HashMap<String, serde_json::Value>,
}

/// Response from LLM completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// Unique identifier for the completion
    pub id: String,
    /// Object type (usually "chat.completion")
    pub object: String,
    /// Unix timestamp of creation
    pub created: u64,
    /// Model used for completion
    pub model: String,
    /// Generated choices
    pub choices: Vec<Choice>,
    /// Token usage information
    pub usage: Usage,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// A message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Role of the message sender
    pub role: MessageRole,
    /// Content of the message
    pub content: Option<String>,
    /// Name of the sender (optional)
    pub name: Option<String>,
    /// Function call information (for assistant messages)
    pub function_call: Option<FunctionCall>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Role of a message sender
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// System message (instructions)
    System,
    /// User message
    User,
    /// Assistant message
    Assistant,
    /// Function call result
    Function,
}

/// Function call information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    /// Name of the function to call
    pub name: String,
    /// Arguments to pass to the function (JSON string)
    pub arguments: String,
}

/// Function definition for LLM function calling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    /// Name of the function
    pub name: String,
    /// Description of what the function does
    pub description: String,
    /// Parameters schema (JSON Schema)
    pub parameters: serde_json::Value,
}

/// Function call behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FunctionCallBehavior {
    /// No function calling
    None,
    /// Automatic function calling
    Auto,
    /// Force a specific function call
    Force { name: String },
}

/// A choice in the completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    /// Index of this choice
    pub index: u32,
    /// The generated message
    pub message: Message,
    /// Finish reason
    pub finish_reason: Option<FinishReason>,
    /// Log probabilities (if requested)
    pub logprobs: Option<serde_json::Value>,
}

/// Reason why the completion finished
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    /// Natural completion
    Stop,
    /// Hit token limit
    Length,
    /// Function call was made
    FunctionCall,
    /// Content was filtered
    ContentFilter,
    /// Other reason
    Other(String),
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    /// Number of tokens in the prompt
    pub prompt_tokens: u32,
    /// Number of tokens in the completion
    pub completion_tokens: u32,
    /// Total number of tokens used
    pub total_tokens: u32,
}

/// Information about an available model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Model description
    pub description: Option<String>,
    /// Maximum context length in tokens
    pub max_tokens: Option<u32>,
    /// Whether the model supports function calling
    pub supports_functions: bool,
    /// Whether the model supports streaming
    pub supports_streaming: bool,
    /// Model capabilities
    pub capabilities: Vec<ModelCapability>,
    /// Pricing information (if available)
    pub pricing: Option<ModelPricing>,
}

/// Model capability
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModelCapability {
    /// Text completion
    TextCompletion,
    /// Chat completion
    ChatCompletion,
    /// Function calling
    FunctionCalling,
    /// Code generation
    CodeGeneration,
    /// Image understanding
    ImageUnderstanding,
    /// Audio processing
    AudioProcessing,
    /// Embeddings
    Embeddings,
}

/// Model pricing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    /// Cost per 1K input tokens
    pub input_cost_per_1k_tokens: Option<f64>,
    /// Cost per 1K output tokens
    pub output_cost_per_1k_tokens: Option<f64>,
    /// Currency (e.g., "USD")
    pub currency: String,
}

impl Message {
    /// Create a new system message
    pub fn system(content: String) -> Self {
        Self {
            role: MessageRole::System,
            content: Some(content),
            name: None,
            function_call: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a new user message
    pub fn user(content: String) -> Self {
        Self {
            role: MessageRole::User,
            content: Some(content),
            name: None,
            function_call: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a new assistant message
    pub fn assistant(content: String) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: Some(content),
            name: None,
            function_call: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a new function message
    pub fn function(name: String, content: String) -> Self {
        Self {
            role: MessageRole::Function,
            content: Some(content),
            name: Some(name),
            function_call: None,
            metadata: HashMap::new(),
        }
    }

    /// Create an assistant message with a function call
    pub fn assistant_with_function_call(function_call: FunctionCall) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: None,
            name: None,
            function_call: Some(function_call),
            metadata: HashMap::new(),
        }
    }
}

impl CompletionRequest {
    /// Create a simple completion request
    pub fn simple(model: String, messages: Vec<Message>) -> Self {
        Self {
            model,
            messages,
            max_tokens: None,
            temperature: None,
            top_p: None,
            n: None,
            stream: None,
            stop: None,
            presence_penalty: None,
            frequency_penalty: None,
            functions: None,
            function_call: None,
            extra_params: HashMap::new(),
        }
    }

    /// Create a completion request with function calling
    pub fn with_functions(
        model: String,
        messages: Vec<Message>,
        functions: Vec<FunctionDefinition>,
    ) -> Self {
        Self {
            model,
            messages,
            functions: Some(functions),
            function_call: Some(FunctionCallBehavior::Auto),
            max_tokens: None,
            temperature: None,
            top_p: None,
            n: None,
            stream: None,
            stop: None,
            presence_penalty: None,
            frequency_penalty: None,
            extra_params: HashMap::new(),
        }
    }
}

impl FunctionDefinition {
    /// Create a new function definition
    pub fn new(name: String, description: String, parameters: serde_json::Value) -> Self {
        Self {
            name,
            description,
            parameters,
        }
    }
}

impl Usage {
    /// Create new usage information
    pub fn new(prompt_tokens: u32, completion_tokens: u32) -> Self {
        Self {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        }
    }
}