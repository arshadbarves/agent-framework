# OpenRouter Integration

AgentGraph now includes comprehensive support for OpenRouter, providing access to multiple LLM providers through a single, unified API. This integration allows you to seamlessly switch between models from OpenAI, Anthropic, Google, Meta, Mistral, and other providers without managing multiple API keys or different interfaces.

## Overview

OpenRouter is a unified API that provides access to multiple LLM providers through a single endpoint. This integration offers several key advantages:

- **Single API**: Access models from multiple providers with one API key
- **Transparent Pricing**: Clear, upfront pricing for all models
- **Model Diversity**: Choose from the latest models across all major providers
- **Cost Optimization**: Compare costs and performance across models
- **Simplified Management**: No need to manage multiple API keys and endpoints

## Quick Start

### 1. Installation

The OpenRouter provider is included with AgentGraph by default:

```toml
[dependencies]
agent_graph = "0.3.0"
```

### 2. Get OpenRouter API Key

1. Visit [OpenRouter](https://openrouter.ai/)
2. Sign up for an account
3. Generate an API key from your dashboard
4. Set the environment variable:

```bash
export OPENROUTER_API_KEY="your_api_key_here"
```

### 3. Basic Usage

```rust
use agent_graph::{
    llm::{LLMManager, LLMConfig, CompletionRequest, Message},
    llm::providers::{OpenRouterProvider, openrouter::OpenRouterConfig},
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create OpenRouter configuration
    let config = OpenRouterConfig {
        api_key: std::env::var("OPENROUTER_API_KEY")?,
        base_url: "https://openrouter.ai/api/v1".to_string(),
        app_name: Some("My AgentGraph App".to_string()),
        site_url: Some("https://myapp.com".to_string()),
        ..Default::default()
    };
    
    // Create provider
    let provider = OpenRouterProvider::new(config);
    
    // Create LLM manager and register provider
    let mut llm_manager = LLMManager::new(LLMConfig::default());
    llm_manager.register_provider("openrouter".to_string(), Arc::new(provider));
    
    // Create a completion request
    let request = CompletionRequest {
        model: "openai/gpt-4".to_string(),
        messages: vec![
            Message::system("You are a helpful assistant.".to_string()),
            Message::user("Explain quantum computing briefly.".to_string()),
        ],
        max_tokens: Some(150),
        temperature: Some(0.7),
        ..Default::default()
    };
    
    // Get completion
    let response = llm_manager.complete_with_provider("openrouter", request).await?;
    println!("Response: {}", response.choices[0].message.content);
    
    Ok(())
}
```

## Configuration

### OpenRouterConfig

```rust
pub struct OpenRouterConfig {
    /// OpenRouter API key
    pub api_key: String,
    /// API base URL (default: "https://openrouter.ai/api/v1")
    pub base_url: String,
    /// Request timeout (default: 30 seconds)
    pub timeout: Duration,
    /// Application name for OpenRouter analytics
    pub app_name: Option<String>,
    /// Website URL for OpenRouter analytics
    pub site_url: Option<String>,
}
```

### Environment Variables

- `OPENROUTER_API_KEY`: Your OpenRouter API key (required)
- `OPENROUTER_BASE_URL`: Custom base URL (optional)
- `OPENROUTER_APP_NAME`: Application name for analytics (optional)
- `OPENROUTER_SITE_URL`: Website URL for analytics (optional)

## Supported Models

OpenRouter provides access to models from multiple providers:

### OpenAI Models
- `openai/gpt-4` - GPT-4 (8K context)
- `openai/gpt-4-turbo` - GPT-4 Turbo (128K context)
- `openai/gpt-3.5-turbo` - GPT-3.5 Turbo (16K context)

### Anthropic Models
- `anthropic/claude-3-opus` - Claude 3 Opus (200K context)
- `anthropic/claude-3-sonnet` - Claude 3 Sonnet (200K context)
- `anthropic/claude-3-haiku` - Claude 3 Haiku (200K context)

### Google Models
- `google/gemini-pro` - Gemini Pro (1M context)
- `google/gemini-pro-vision` - Gemini Pro Vision

### Meta Models
- `meta-llama/llama-2-70b-chat` - Llama 2 70B Chat
- `meta-llama/llama-2-13b-chat` - Llama 2 13B Chat

### Mistral Models
- `mistralai/mistral-7b-instruct` - Mistral 7B Instruct
- `mistralai/mixtral-8x7b-instruct` - Mixtral 8x7B Instruct

For the complete list of available models, visit [OpenRouter Models](https://openrouter.ai/models).

## Features

### Function Calling

OpenRouter supports function calling for compatible models:

```rust
use agent_graph::llm::{FunctionDefinition, FunctionCallBehavior};
use serde_json::json;

let functions = vec![
    FunctionDefinition::new(
        "get_weather".to_string(),
        "Get current weather".to_string(),
        json!({
            "type": "object",
            "properties": {
                "location": {"type": "string", "description": "City name"}
            },
            "required": ["location"]
        }),
    ),
];

let request = CompletionRequest {
    model: "openai/gpt-4".to_string(),
    messages: vec![Message::user("What's the weather in Paris?".to_string())],
    functions: Some(functions),
    function_call: Some(FunctionCallBehavior::Auto),
    ..Default::default()
};
```

### Cost Estimation

Get pricing information for any model:

```rust
let provider = OpenRouterProvider::new(config);

// Get pricing for a specific model
if let Some(pricing) = provider.get_pricing("openai/gpt-4") {
    println!("Prompt cost: ${:.4}/1k tokens", pricing.prompt_cost_per_1k);
    println!("Completion cost: ${:.4}/1k tokens", pricing.completion_cost_per_1k);
}

// Estimate cost for a request
let estimated_cost = llm_manager.estimate_cost(&request, "openrouter").await?;
```

### Token Counting

Count tokens for text with any model:

```rust
let token_count = provider.count_tokens("Hello, world!", "openai/gpt-4").await?;
println!("Token count: {}", token_count);
```

## Model Selection Guide

### For General Tasks
- **GPT-3.5 Turbo**: Cost-effective for most applications
- **GPT-4**: Higher quality reasoning and complex tasks
- **Claude 3 Haiku**: Fast and efficient for simple tasks

### For Long Context
- **Claude 3 Opus/Sonnet**: 200K context window
- **Gemini Pro**: 1M context window
- **GPT-4 Turbo**: 128K context window

### For Code Generation
- **GPT-4**: Excellent code understanding and generation
- **Claude 3 Opus**: Strong reasoning for complex algorithms
- **Mixtral 8x7B**: Good open-source alternative

### For Cost Optimization
- **GPT-3.5 Turbo**: Lowest cost OpenAI model
- **Claude 3 Haiku**: Fastest and cheapest Claude model
- **Llama 2**: Open-source models with competitive pricing

## Error Handling

```rust
use agent_graph::llm::LLMError;

match llm_manager.complete_with_provider("openrouter", request).await {
    Ok(response) => {
        println!("Success: {}", response.choices[0].message.content);
    }
    Err(LLMError::AuthenticationError { provider, message }) => {
        eprintln!("Auth error for {}: {}", provider, message);
    }
    Err(LLMError::RateLimitExceeded { provider }) => {
        eprintln!("Rate limit exceeded for {}", provider);
    }
    Err(LLMError::CostLimitExceeded { estimated_cost, limit }) => {
        eprintln!("Cost limit exceeded: ${:.2} > ${:.2}", estimated_cost, limit);
    }
    Err(e) => {
        eprintln!("Other error: {}", e);
    }
}
```

## Best Practices

### 1. Model Selection
- Start with cost-effective models for development
- Use higher-tier models for production critical tasks
- Consider context length requirements
- Test multiple models for your specific use case

### 2. Cost Management
- Set cost limits in LLMConfig
- Monitor usage through OpenRouter dashboard
- Use token counting to estimate costs
- Consider model switching based on task complexity

### 3. Error Handling
- Implement retry logic for transient errors
- Handle rate limits gracefully
- Monitor authentication status
- Log errors for debugging

### 4. Performance Optimization
- Cache responses when appropriate
- Use streaming for long responses
- Batch requests when possible
- Monitor response times

## Examples

See the complete demo in `examples/openrouter_demo.rs` for comprehensive usage examples including:

- Provider configuration and setup
- Model comparison and selection
- Cost estimation and optimization
- Function calling integration
- Error handling patterns
- LLM Manager integration

## Troubleshooting

### Common Issues

1. **Authentication Errors**
   - Verify OPENROUTER_API_KEY is set correctly
   - Check API key permissions on OpenRouter dashboard

2. **Rate Limiting**
   - Implement exponential backoff
   - Monitor usage limits
   - Consider upgrading OpenRouter plan

3. **Model Not Found**
   - Verify model name format (provider/model)
   - Check model availability on OpenRouter

4. **High Costs**
   - Set cost limits in configuration
   - Use cost estimation before requests
   - Choose appropriate models for tasks

### Support

- OpenRouter Documentation: https://openrouter.ai/docs
- AgentGraph Issues: https://github.com/agent-graph/agent-graph/issues
- OpenRouter Support: https://openrouter.ai/support
