// OpenRouter LLM provider demonstration
// Shows how to use OpenRouter to access multiple LLM models through a single API

use agent_graph::{
    llm::{LLMManager, LLMConfig, CompletionRequest, Message, FunctionDefinition, FunctionCallBehavior},
    llm::providers::{OpenRouterProvider, openrouter::OpenRouterConfig},
};
use std::sync::Arc;
use std::time::Duration;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 AgentGraph OpenRouter Integration Demo");
    println!("=========================================");
    
    // Demo 1: OpenRouter Configuration
    println!("\n🔧 Demo 1: OpenRouter Configuration");
    println!("===================================");
    
    // Create OpenRouter configuration
    let openrouter_config = OpenRouterConfig {
        api_key: std::env::var("OPENROUTER_API_KEY")
            .unwrap_or_else(|_| "demo_key_for_testing".to_string()),
        base_url: "https://openrouter.ai/api/v1".to_string(),
        timeout: Duration::from_secs(120),
        app_name: Some("AgentGraph Demo".to_string()),
        site_url: Some("https://github.com/agent-graph/agent-graph".to_string()),
    };
    
    println!("✅ OpenRouter Configuration:");
    println!("   Base URL: {}", openrouter_config.base_url);
    println!("   Timeout: {:?}", openrouter_config.timeout);
    println!("   App Name: {:?}", openrouter_config.app_name);
    println!("   Site URL: {:?}", openrouter_config.site_url);
    
    // Create OpenRouter provider
    let openrouter_provider = OpenRouterProvider::new(openrouter_config);
    
    // Demo 2: Provider Capabilities
    println!("\n🎯 Demo 2: Provider Capabilities");
    println!("================================");
    
    println!("✅ Provider Name: {}", openrouter_provider.name());
    println!("✅ Function Calling: {}", openrouter_provider.supports_function_calling());
    println!("✅ Streaming: {}", openrouter_provider.supports_streaming());
    
    // Show supported models
    let models = openrouter_provider.supported_models();
    println!("✅ Supported Models ({}):", models.len());
    for (i, model) in models.iter().take(10).enumerate() {
        println!("   {}. {}", i + 1, model);
    }
    if models.len() > 10 {
        println!("   ... and {} more models", models.len() - 10);
    }
    
    // Demo 3: Model Pricing Information
    println!("\n💰 Demo 3: Model Pricing Information");
    println!("====================================");
    
    let test_models = vec![
        "openai/gpt-4",
        "openai/gpt-3.5-turbo",
        "anthropic/claude-3-opus",
        "anthropic/claude-3-sonnet",
        "google/gemini-pro",
        "meta-llama/llama-2-70b-chat",
    ];
    
    for model in &test_models {
        if let Some(pricing) = openrouter_provider.get_pricing(model) {
            println!("💵 {}", model);
            println!("   Prompt: ${:.4}/1k tokens", pricing.prompt_cost_per_1k);
            println!("   Completion: ${:.4}/1k tokens", pricing.completion_cost_per_1k);
            println!("   Currency: {}", pricing.currency);
        }
    }
    
    // Demo 4: Token Counting
    println!("\n🔢 Demo 4: Token Counting");
    println!("=========================");
    
    let test_texts = vec![
        "Hello, world!",
        "This is a longer text that should have more tokens than the previous one.",
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.",
    ];
    
    for text in &test_texts {
        let token_count = openrouter_provider.count_tokens(text, "openai/gpt-4").await?;
        println!("📝 Text: \"{}...\"", text.chars().take(50).collect::<String>());
        println!("   Tokens: {}", token_count);
        println!("   Characters: {}", text.len());
        println!("   Ratio: {:.2} chars/token", text.len() as f32 / token_count as f32);
    }
    
    // Demo 5: LLM Manager Integration
    println!("\n🔗 Demo 5: LLM Manager Integration");
    println!("==================================");
    
    // Create LLM manager and register OpenRouter provider
    let llm_config = LLMConfig::default();
    let mut llm_manager = LLMManager::new(llm_config);
    
    // Register OpenRouter provider
    let openrouter_provider_arc = Arc::new(openrouter_provider);
    llm_manager.register_provider("openrouter".to_string(), openrouter_provider_arc);
    
    println!("✅ Registered OpenRouter provider with LLM Manager");
    
    // Check if provider is registered
    if llm_manager.get_provider("openrouter").is_some() {
        println!("📋 OpenRouter provider successfully registered");
    }
    
    // Demo 6: Message Conversion
    println!("\n💬 Demo 6: Message Conversion");
    println!("=============================");
    
    let messages = vec![
        Message::system("You are a helpful AI assistant specialized in explaining complex topics.".to_string()),
        Message::user("Explain quantum computing in simple terms.".to_string()),
        Message::assistant("Quantum computing is a revolutionary approach to computation that leverages quantum mechanics...".to_string()),
        Message::user("Can you give me a practical example?".to_string()),
    ];
    
    println!("✅ Created conversation with {} messages:", messages.len());
    for (i, message) in messages.iter().enumerate() {
        println!("   {}. {:?}: {}", i + 1, message.role, 
                 message.content.chars().take(50).collect::<String>() + 
                 if message.content.len() > 50 { "..." } else { "" });
    }
    
    // Demo 7: Function Calling Setup
    println!("\n⚙️  Demo 7: Function Calling Setup");
    println!("==================================");
    
    let functions = vec![
        FunctionDefinition::new(
            "get_weather".to_string(),
            "Get current weather information for a location".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "The city and state, e.g. San Francisco, CA"
                    },
                    "unit": {
                        "type": "string",
                        "enum": ["celsius", "fahrenheit"],
                        "description": "The temperature unit"
                    }
                },
                "required": ["location"]
            }),
        ),
        FunctionDefinition::new(
            "calculate_tip".to_string(),
            "Calculate tip amount for a bill".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "bill_amount": {
                        "type": "number",
                        "description": "The total bill amount"
                    },
                    "tip_percentage": {
                        "type": "number",
                        "description": "The tip percentage (e.g., 15, 18, 20)"
                    }
                },
                "required": ["bill_amount", "tip_percentage"]
            }),
        ),
    ];
    
    println!("✅ Created {} function definitions:", functions.len());
    for function in &functions {
        println!("   - {}: {}", function.name, function.description);
    }
    
    // Demo 8: Completion Request Setup
    println!("\n📝 Demo 8: Completion Request Setup");
    println!("===================================");
    
    let completion_request = CompletionRequest {
        model: "openai/gpt-3.5-turbo".to_string(),
        messages: messages.clone(),
        max_tokens: Some(150),
        temperature: Some(0.7),
        top_p: Some(0.9),
        functions: Some(functions.clone()),
        function_call: Some(FunctionCallBehavior::Auto),
        ..Default::default()
    };
    
    println!("✅ Completion Request Configuration:");
    println!("   Model: {}", completion_request.model);
    println!("   Messages: {}", completion_request.messages.len());
    println!("   Max Tokens: {:?}", completion_request.max_tokens);
    println!("   Temperature: {:?}", completion_request.temperature);
    println!("   Top P: {:?}", completion_request.top_p);
    println!("   Functions: {}", completion_request.functions.as_ref().map_or(0, |f| f.len()));
    println!("   Function Call: {:?}", completion_request.function_call);
    
    // Demo 9: Model Comparison
    println!("\n🔍 Demo 9: Model Comparison");
    println!("===========================");
    
    let comparison_models = vec![
        ("OpenAI GPT-4", "openai/gpt-4"),
        ("OpenAI GPT-3.5 Turbo", "openai/gpt-3.5-turbo"),
        ("Claude 3 Opus", "anthropic/claude-3-opus"),
        ("Claude 3 Sonnet", "anthropic/claude-3-sonnet"),
        ("Gemini Pro", "google/gemini-pro"),
        ("Llama 2 70B", "meta-llama/llama-2-70b-chat"),
    ];
    
    println!("📊 Model Comparison Table:");
    println!("┌─────────────────────────┬─────────────────────────┬──────────────┬──────────────┐");
    println!("│ Model Name              │ OpenRouter ID           │ Prompt Cost  │ Completion   │");
    println!("├─────────────────────────┼─────────────────────────┼──────────────┼──────────────┤");
    
    for (name, model_id) in &comparison_models {
        let provider = llm_manager.get_provider("openrouter").unwrap();
        if let Some(pricing) = provider.get_pricing(model_id) {
            println!("│ {:<23} │ {:<23} │ ${:<11.4} │ ${:<11.4} │", 
                     name, model_id, pricing.prompt_cost_per_1k, pricing.completion_cost_per_1k);
        }
    }
    println!("└─────────────────────────┴─────────────────────────┴──────────────┴──────────────┘");
    
    // Demo 10: Cost Estimation
    println!("\n💲 Demo 10: Cost Estimation");
    println!("===========================");
    
    let sample_prompt = "Write a comprehensive guide about machine learning for beginners";
    let estimated_prompt_tokens = 15; // Rough estimate
    let estimated_completion_tokens = 500; // Rough estimate for a comprehensive guide
    
    println!("📋 Cost Estimation for Sample Task:");
    println!("   Prompt: \"{}\"", sample_prompt);
    println!("   Estimated Prompt Tokens: {}", estimated_prompt_tokens);
    println!("   Estimated Completion Tokens: {}", estimated_completion_tokens);
    println!();
    
    for (name, model_id) in &comparison_models {
        let provider = llm_manager.get_provider("openrouter").unwrap();
        if let Some(pricing) = provider.get_pricing(model_id) {
            let prompt_cost = (estimated_prompt_tokens as f64 / 1000.0) * pricing.prompt_cost_per_1k;
            let completion_cost = (estimated_completion_tokens as f64 / 1000.0) * pricing.completion_cost_per_1k;
            let total_cost = prompt_cost + completion_cost;
            
            println!("💰 {}: ${:.4} total", name, total_cost);
            println!("   (${:.4} prompt + ${:.4} completion)", prompt_cost, completion_cost);
        }
    }
    
    // Demo 11: OpenRouter Advantages
    println!("\n🌟 Demo 11: OpenRouter Advantages");
    println!("=================================");
    
    println!("✅ OpenRouter provides several key advantages:");
    println!("   🔄 Single API for multiple LLM providers");
    println!("   💰 Transparent pricing across all models");
    println!("   🚀 No need to manage multiple API keys");
    println!("   📊 Built-in usage analytics and monitoring");
    println!("   🔧 Consistent API interface across providers");
    println!("   ⚡ Automatic failover and load balancing");
    println!("   🌍 Access to latest models from all major providers");
    println!("   💡 Cost optimization through model comparison");
    
    // Demo 12: Integration Summary
    println!("\n📋 Demo 12: Integration Summary");
    println!("===============================");
    
    println!("🎉 AgentGraph OpenRouter Integration Complete!");
    println!();
    println!("✅ Features Demonstrated:");
    println!("   - OpenRouter provider configuration");
    println!("   - Multi-model support (OpenAI, Anthropic, Google, Meta, Mistral)");
    println!("   - Pricing information and cost estimation");
    println!("   - Token counting and text analysis");
    println!("   - LLM Manager integration");
    println!("   - Function calling setup");
    println!("   - Message conversion and handling");
    println!("   - Model comparison and selection");
    println!();
    println!("🚀 Ready for Production Use:");
    println!("   - Set OPENROUTER_API_KEY environment variable");
    println!("   - Choose appropriate models for your use case");
    println!("   - Monitor costs and usage through OpenRouter dashboard");
    println!("   - Leverage AgentGraph's advanced agent capabilities");
    
    println!("\n🔗 Next Steps:");
    println!("   1. Get OpenRouter API key: https://openrouter.ai/");
    println!("   2. Explore available models: https://openrouter.ai/models");
    println!("   3. Set up monitoring and alerts");
    println!("   4. Integrate with AgentGraph agents and workflows");
    
    Ok(())
}
