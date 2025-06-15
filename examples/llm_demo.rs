// LLM Integration demonstration example
// Shows multi-provider LLM support, function calling, and streaming

use agent_graph::llm::{
    LLMManager, LLMConfig, CompletionRequest, Message,
    FunctionDefinition, FunctionCallBehavior, TokenUsage,
    providers::MockProvider,
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 AgentGraph LLM Integration Demo");
    println!("==================================");
    
    // Demo 1: Basic LLM Manager Setup
    println!("\n🔧 Demo 1: LLM Manager Setup");
    println!("============================");
    
    // Create LLM configuration
    let mut config = LLMConfig::default();
    config.default_provider = "mock".to_string();
    config.cost_tracking = true;
    config.max_cost_per_request = Some(0.10); // 10 cents max per request
    
    let mut llm_manager = LLMManager::new(config);
    
    // Register mock provider for demonstration
    let mock_provider = MockProvider::new()
        .with_delay(Duration::from_millis(50));
    llm_manager.register_provider("mock".to_string(), std::sync::Arc::new(mock_provider));
    
    println!("✅ Created LLM manager with mock provider");
    println!("   - Default provider: mock");
    println!("   - Cost tracking: enabled");
    println!("   - Max cost per request: $0.10");
    
    // Demo 2: Basic Completion
    println!("\n💬 Demo 2: Basic Text Completion");
    println!("=================================");
    
    let messages = vec![
        Message::system("You are a helpful AI assistant specialized in explaining complex topics simply.".to_string()),
        Message::user("Explain quantum computing in simple terms.".to_string()),
    ];
    
    let request = CompletionRequest {
        model: "mock-gpt-4".to_string(),
        messages,
        max_tokens: Some(200),
        temperature: Some(0.7),
        ..Default::default()
    };
    
    let response = llm_manager.complete(request).await?;
    
    println!("✅ Completion successful:");
    println!("   Model: {}", response.model);
    println!("   Response: {}", response.choices[0].message.content);
    println!("   Tokens used: {} (prompt: {}, completion: {})", 
             response.usage.total_tokens,
             response.usage.prompt_tokens,
             response.usage.completion_tokens);
    
    if let Some(cost) = response.usage.estimated_cost {
        println!("   Estimated cost: ${:.4}", cost);
    }
    
    // Demo 3: Function Calling
    println!("\n🔧 Demo 3: Function Calling");
    println!("===========================");
    
    // Define available functions
    let weather_function = FunctionDefinition::new(
        "get_weather".to_string(),
        "Get current weather for a location".to_string(),
        serde_json::json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "The city and state, e.g. San Francisco, CA"
                },
                "unit": {
                    "type": "string",
                    "enum": ["celsius", "fahrenheit"],
                    "description": "Temperature unit"
                }
            },
            "required": ["location"]
        })
    );
    
    let calculator_function = FunctionDefinition::new(
        "calculate".to_string(),
        "Perform mathematical calculations".to_string(),
        serde_json::json!({
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "Mathematical expression to evaluate"
                }
            },
            "required": ["expression"]
        })
    );
    
    let function_request = CompletionRequest {
        model: "mock-gpt-4".to_string(),
        messages: vec![
            Message::system("You are a helpful assistant with access to weather and calculator functions.".to_string()),
            Message::user("What's the weather like in New York and what's 15 * 23?".to_string()),
        ],
        functions: Some(vec![weather_function, calculator_function]),
        function_call: Some(FunctionCallBehavior::Auto),
        max_tokens: Some(150),
        ..Default::default()
    };
    
    let function_response = llm_manager.complete(function_request).await?;
    
    println!("✅ Function calling response:");
    println!("   Model: {}", function_response.model);
    
    if let Some(function_call) = &function_response.choices[0].message.function_call {
        println!("   Function called: {}", function_call.name);
        println!("   Arguments: {}", function_call.arguments);
        println!("   Function ID: {}", function_call.id.as_ref().unwrap_or(&"none".to_string()));
    } else {
        println!("   Response: {}", function_response.choices[0].message.content);
    }
    
    // Demo 4: Streaming Completion
    println!("\n🌊 Demo 4: Streaming Completion");
    println!("===============================");
    
    let streaming_request = CompletionRequest {
        model: "mock-gpt-4".to_string(),
        messages: vec![
            Message::system("You are a creative storyteller.".to_string()),
            Message::user("Tell me a short story about a robot learning to paint.".to_string()),
        ],
        stream: true,
        max_tokens: Some(300),
        temperature: Some(0.8),
        ..Default::default()
    };
    
    println!("✅ Starting streaming completion...");
    print!("   Story: ");
    
    let provider = llm_manager.get_provider("mock").unwrap();
    let mut stream = provider.stream(streaming_request).await?;
    
    use futures::StreamExt;
    let mut chunk_count = 0;
    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                chunk_count += 1;
                print!("{}", chunk.choices[0].message.content);
                if chunk.choices[0].finish_reason == agent_graph::llm::FinishReason::Stop {
                    break;
                }
                print!(" ");
                // Small delay to simulate real streaming
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            Err(e) => {
                println!("\n❌ Streaming error: {}", e);
                break;
            }
        }
    }
    
    println!("\n   Received {} chunks", chunk_count);
    
    // Demo 5: Multi-Provider Support
    println!("\n🔄 Demo 5: Multi-Provider Support");
    println!("=================================");
    
    // Register additional mock providers with different characteristics
    let fast_provider = MockProvider::with_responses(vec!["Fast response from speed-optimized model".to_string()])
        .with_delay(Duration::from_millis(10));

    let creative_provider = MockProvider::with_responses(vec!["Creative and imaginative response with artistic flair".to_string()])
        .with_delay(Duration::from_millis(100));
    
    llm_manager.register_provider("fast-mock".to_string(), std::sync::Arc::new(fast_provider));
    llm_manager.register_provider("creative-mock".to_string(), std::sync::Arc::new(creative_provider));
    
    println!("✅ Registered multiple providers:");
    println!("   - mock: Standard provider");
    println!("   - fast-mock: Speed-optimized");
    println!("   - creative-mock: Creative responses");
    
    // Test different providers
    let test_request = CompletionRequest {
        model: "mock-gpt-4".to_string(),
        messages: vec![Message::user("Hello, how are you?".to_string())],
        max_tokens: Some(50),
        ..Default::default()
    };
    
    // Test fast provider
    let start_time = std::time::Instant::now();
    let fast_response = llm_manager.complete_with_provider("fast-mock", test_request.clone()).await?;
    let fast_duration = start_time.elapsed();
    
    println!("   Fast provider response ({}ms): {}", 
             fast_duration.as_millis(),
             fast_response.choices[0].message.content);
    
    // Test creative provider
    let start_time = std::time::Instant::now();
    let creative_response = llm_manager.complete_with_provider("creative-mock", test_request.clone()).await?;
    let creative_duration = start_time.elapsed();
    
    println!("   Creative provider response ({}ms): {}", 
             creative_duration.as_millis(),
             creative_response.choices[0].message.content);
    
    // Demo 6: Cost Estimation and Tracking
    println!("\n💰 Demo 6: Cost Estimation and Tracking");
    println!("=======================================");
    
    let cost_request = CompletionRequest {
        model: "mock-gpt-4".to_string(),
        messages: vec![
            Message::user("Write a detailed analysis of renewable energy trends.".to_string()),
        ],
        max_tokens: Some(1000),
        ..Default::default()
    };
    
    // Estimate cost before making request
    let estimated_cost = llm_manager.estimate_cost(&cost_request, "mock").await?;
    if let Some(cost) = estimated_cost {
        println!("✅ Estimated cost: ${:.4}", cost);
    }
    
    // Make request and track actual cost
    let cost_response = llm_manager.complete_with_provider("mock", cost_request).await?;
    if let Some(actual_cost) = cost_response.usage.estimated_cost {
        println!("   Actual cost: ${:.4}", actual_cost);
    }
    
    // Get overall statistics
    let stats = llm_manager.get_stats();
    println!("📊 LLM Usage Statistics:");
    println!("   Total requests: {}", stats.total_requests);
    println!("   Total tokens: {}", stats.total_tokens);
    println!("   Total cost: ${:.4}", stats.total_cost);
    println!("   Requests by provider: {:?}", stats.requests_by_provider);
    println!("   Requests by model: {:?}", stats.requests_by_model);
    
    // Demo 7: Error Handling and Retry Logic
    println!("\n⚠️  Demo 7: Error Handling");
    println!("==========================");
    
    // Test with unsupported model
    let invalid_request = CompletionRequest {
        model: "invalid-model".to_string(),
        messages: vec![Message::user("Test".to_string())],
        ..Default::default()
    };
    
    match llm_manager.complete_with_provider("mock", invalid_request).await {
        Ok(_) => println!("❌ Expected error but got success"),
        Err(e) => println!("✅ Correctly handled error: {}", e),
    }
    
    // Test with excessive token limit
    let token_limit_request = CompletionRequest {
        model: "mock-gpt-4".to_string(),
        messages: vec![Message::user("Test".to_string())],
        max_tokens: Some(10000), // Exceeds mock provider limit
        ..Default::default()
    };
    
    match llm_manager.complete_with_provider("mock", token_limit_request).await {
        Ok(_) => println!("❌ Expected token limit error but got success"),
        Err(e) => println!("✅ Correctly handled token limit: {}", e),
    }
    
    // Demo 8: Provider Capabilities
    println!("\n🔍 Demo 8: Provider Capabilities");
    println!("================================");
    
    let mock_provider = llm_manager.get_provider("mock").unwrap();
    println!("✅ Mock Provider Capabilities:");
    println!("   Name: {}", mock_provider.name());
    println!("   Supported models: {:?}", mock_provider.supported_models());
    println!("   Function calling: {}", mock_provider.supports_function_calling());
    println!("   Streaming: {}", mock_provider.supports_streaming());
    
    // Test pricing information
    if let Some(pricing) = mock_provider.get_pricing("mock-gpt-4") {
        println!("   Pricing for mock-gpt-4:");
        println!("     Prompt: ${:.4}/1K tokens", pricing.prompt_cost_per_1k);
        println!("     Completion: ${:.4}/1K tokens", pricing.completion_cost_per_1k);
        
        // Calculate cost for sample usage
        let sample_usage = TokenUsage::new(500, 200);
        let sample_cost = pricing.calculate_cost(&sample_usage);
        println!("     Sample cost (500 prompt + 200 completion): ${:.4}", sample_cost);
    }
    
    println!("\n🎉 LLM Integration Demo Complete!");
    println!("=================================");
    println!("The AgentGraph LLM framework provides:");
    println!("✅ Multi-provider support with unified interface");
    println!("✅ Function calling for tool integration");
    println!("✅ Streaming responses for real-time interaction");
    println!("✅ Cost tracking and estimation");
    println!("✅ Comprehensive error handling and retry logic");
    println!("✅ Provider capability discovery");
    println!("✅ Token counting and usage analytics");
    println!("✅ Flexible configuration and customization");
    
    Ok(())
}
