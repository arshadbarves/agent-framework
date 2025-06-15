//! Streaming chat example demonstrating real-time execution events.

use agent_graph::{
    GraphBuilder, Node, GraphResult, Edge,
    ExecutionConfig,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use futures::StreamExt;

/// Chat state containing conversation history and context
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatState {
    /// Conversation messages
    messages: Vec<ChatMessage>,
    /// Current user input
    current_input: String,
    /// Chat context and metadata
    context: HashMap<String, String>,
    /// Processing statistics
    stats: ChatStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    role: String,      // "user", "assistant", "system"
    content: String,
    timestamp: String,
    metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatStats {
    total_messages: u32,
    processing_time_ms: u64,
    tokens_processed: u32,
    response_quality: f64,
}

impl Default for ChatStats {
    fn default() -> Self {
        Self {
            total_messages: 0,
            processing_time_ms: 0,
            tokens_processed: 0,
            response_quality: 0.0,
        }
    }
}



impl Default for ChatState {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            current_input: String::new(),
            context: HashMap::new(),
            stats: ChatStats::default(),
        }
    }
}

/// Node that processes user input
#[derive(Debug)]
struct InputProcessingNode;

#[async_trait]
impl Node<ChatState> for InputProcessingNode {
    async fn invoke(&self, state: &mut ChatState) -> GraphResult<()> {
        println!("🔤 Processing user input...");
        
        // Simulate input processing delay
        sleep(Duration::from_millis(200)).await;
        
        // Add user message to conversation
        let user_message = ChatMessage {
            role: "user".to_string(),
            content: state.current_input.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("processed_by".to_string(), "InputProcessingNode".to_string());
                meta
            },
        };
        
        state.messages.push(user_message);
        state.stats.total_messages += 1;
        state.stats.tokens_processed += state.current_input.len() as u32;
        
        // Update context
        state.context.insert("last_input_length".to_string(), state.current_input.len().to_string());
        state.context.insert("processing_stage".to_string(), "input_processed".to_string());
        
        println!("✅ User input processed: '{}'", state.current_input);
        Ok(())
    }

    fn metadata(&self) -> agent_graph::NodeMetadata {
        agent_graph::NodeMetadata::new("InputProcessing")
            .with_description("Processes and validates user input")
            .with_tag("input")
            .with_tag("preprocessing")
            .with_expected_duration(200)
    }
}

/// Node that performs intent recognition
#[derive(Debug)]
struct IntentRecognitionNode;

#[async_trait]
impl Node<ChatState> for IntentRecognitionNode {
    async fn invoke(&self, state: &mut ChatState) -> GraphResult<()> {
        println!("🧠 Analyzing user intent...");
        
        // Simulate intent analysis
        sleep(Duration::from_millis(300)).await;
        
        let input = &state.current_input.to_lowercase();
        let intent = if input.contains("hello") || input.contains("hi") {
            "greeting"
        } else if input.contains("help") || input.contains("?") {
            "help_request"
        } else if input.contains("bye") || input.contains("goodbye") {
            "farewell"
        } else if input.contains("weather") {
            "weather_query"
        } else if input.contains("time") {
            "time_query"
        } else {
            "general_conversation"
        };
        
        state.context.insert("detected_intent".to_string(), intent.to_string());
        state.context.insert("confidence".to_string(), "0.85".to_string());
        
        println!("🎯 Intent detected: {} (confidence: 85%)", intent);
        Ok(())
    }

    fn metadata(&self) -> agent_graph::NodeMetadata {
        agent_graph::NodeMetadata::new("IntentRecognition")
            .with_description("Analyzes user intent from input")
            .with_tag("nlp")
            .with_tag("intent")
            .with_expected_duration(300)
    }
}

/// Node that generates responses based on intent
#[derive(Debug)]
struct ResponseGenerationNode;

#[async_trait]
impl Node<ChatState> for ResponseGenerationNode {
    async fn invoke(&self, state: &mut ChatState) -> GraphResult<()> {
        println!("💭 Generating response...");
        
        // Simulate response generation delay
        sleep(Duration::from_millis(500)).await;
        
        let default_intent = "general_conversation".to_string();
        let intent = state.context.get("detected_intent").unwrap_or(&default_intent);
        
        let response_content = match intent.as_str() {
            "greeting" => "Hello! How can I help you today?",
            "help_request" => "I'm here to assist you! You can ask me about the weather, time, or just have a conversation.",
            "farewell" => "Goodbye! Have a great day!",
            "weather_query" => "I'd love to help with weather information! However, I don't have access to real-time weather data in this demo.",
            "time_query" => &format!("The current time is: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")),
            _ => "That's interesting! Tell me more about what you're thinking.",
        };
        
        // Add assistant response to conversation
        let assistant_message = ChatMessage {
            role: "assistant".to_string(),
            content: response_content.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("intent".to_string(), intent.clone());
                meta.insert("generated_by".to_string(), "ResponseGenerationNode".to_string());
                meta
            },
        };
        
        state.messages.push(assistant_message);
        state.stats.total_messages += 1;
        state.stats.tokens_processed += response_content.len() as u32;
        state.stats.response_quality = 0.8; // Mock quality score
        
        state.context.insert("response_generated".to_string(), "true".to_string());
        
        println!("🤖 Response generated: '{}'", response_content);
        Ok(())
    }

    fn metadata(&self) -> agent_graph::NodeMetadata {
        agent_graph::NodeMetadata::new("ResponseGeneration")
            .with_description("Generates contextual responses")
            .with_tag("nlp")
            .with_tag("generation")
            .with_expected_duration(500)
    }
}

/// Node that handles conversation logging and analytics
#[derive(Debug)]
struct ConversationLoggingNode;

#[async_trait]
impl Node<ChatState> for ConversationLoggingNode {
    async fn invoke(&self, state: &mut ChatState) -> GraphResult<()> {
        println!("📊 Logging conversation analytics...");
        
        // Simulate logging delay
        sleep(Duration::from_millis(100)).await;
        
        // Calculate conversation metrics
        let total_chars: usize = state.messages.iter()
            .map(|msg| msg.content.len())
            .sum();
        
        let avg_message_length = if state.messages.is_empty() {
            0.0
        } else {
            total_chars as f64 / state.messages.len() as f64
        };
        
        // Update context with analytics
        state.context.insert("total_characters".to_string(), total_chars.to_string());
        state.context.insert("avg_message_length".to_string(), format!("{:.1}", avg_message_length));
        state.context.insert("conversation_turns".to_string(), (state.messages.len() / 2).to_string());
        
        println!("📈 Analytics updated - Total chars: {}, Avg length: {:.1}", total_chars, avg_message_length);
        Ok(())
    }

    fn metadata(&self) -> agent_graph::NodeMetadata {
        agent_graph::NodeMetadata::new("ConversationLogging")
            .with_description("Logs conversation data and analytics")
            .with_tag("logging")
            .with_tag("analytics")
            .with_expected_duration(100)
    }
}

#[tokio::main]
async fn main() -> GraphResult<()> {
    // Initialize tracing
    agent_graph::init_tracing();
    
    println!("🚀 Starting Streaming Chat Example");
    println!("==================================");
    
    // Configure for streaming
    let config = ExecutionConfig {
        enable_streaming: true,
        enable_parallel: false, // Sequential for chat flow
        max_execution_time_seconds: Some(30),
        ..Default::default()
    };
    
    // Build the chat processing graph
    let mut graph = GraphBuilder::new()
        .with_config(config)
        .add_node("input_processing".to_string(), InputProcessingNode)?
        .add_node("intent_recognition".to_string(), IntentRecognitionNode)?
        .add_node("response_generation".to_string(), ResponseGenerationNode)?
        .add_node("conversation_logging".to_string(), ConversationLoggingNode)?
        .add_edge(Edge::simple("input_processing", "intent_recognition"))?
        .add_edge(Edge::simple("intent_recognition", "response_generation"))?
        .add_edge(Edge::simple("response_generation", "conversation_logging"))?
        .with_entry_point("input_processing".to_string())?
        .add_finish_point("conversation_logging".to_string())?
        .build()?;
    
    println!("\n📋 Chat Graph Summary: {}", graph.summary());
    
    // Simulate a conversation with multiple turns
    let conversation_inputs = vec![
        "Hello there!",
        "Can you help me with something?",
        "What time is it?",
        "How's the weather today?",
        "Thanks, goodbye!",
    ];
    
    println!("\n💬 Starting conversation simulation...\n");
    
    for (turn, input) in conversation_inputs.iter().enumerate() {
        println!("🔄 Turn {} - Processing: '{}'", turn + 1, input);
        println!("{}", "─".repeat(50));
        
        // Create state for this turn
        let mut state = ChatState {
            current_input: input.to_string(),
            ..Default::default()
        };
        
        // Execute with streaming (in a real implementation, you'd handle the stream)
        let start_time = std::time::Instant::now();
        let context = graph.run(&mut state).await?;
        let execution_time = start_time.elapsed();
        
        // Display results
        println!("\n📊 Turn {} Results:", turn + 1);
        println!("  ⏱️  Execution time: {:?}", execution_time);
        println!("  🎯 Detected intent: {}", state.context.get("detected_intent").unwrap_or(&"unknown".to_string()));
        println!("  💬 Messages in conversation: {}", state.messages.len());
        println!("  🔤 Tokens processed: {}", state.stats.tokens_processed);
        
        // Show the conversation
        println!("\n💭 Conversation:");
        for message in &state.messages {
            let emoji = match message.role.as_str() {
                "user" => "👤",
                "assistant" => "🤖",
                _ => "📝",
            };
            println!("  {} {}: {}", emoji, message.role, message.content);
        }
        
        println!("\n");
    }
    
    println!("🎉 Streaming chat example completed successfully!");
    println!("\n📈 Key Features Demonstrated:");
    println!("  ✅ Real-time chat processing pipeline");
    println!("  ✅ Intent recognition and response generation");
    println!("  ✅ Conversation state management");
    println!("  ✅ Analytics and logging");
    println!("  ✅ Sequential execution flow");
    println!("  ✅ Comprehensive metadata tracking");
    
    Ok(())
}
