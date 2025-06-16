// Basic Agent Example
// Demonstrates creating and using a simple agent with AgentGraph

use agent_graph::{
    agents::{Agent, roles::RoleTemplates},
    llm::{LLMManager, LLMConfig, providers::MockProvider},
    tools::{ToolRegistry, ToolExecutor},
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 AgentGraph Basic Agent Example");
    println!("=================================");

    // Step 1: Setup LLM Manager with Mock Provider
    println!("\n📡 Setting up LLM provider...");
    let llm_config = LLMConfig::default();
    let mut llm_manager = LLMManager::new(llm_config);
    
    // Register mock provider for testing
    let mock_provider = MockProvider::new();
    llm_manager.register_provider("mock".to_string(), Arc::new(mock_provider));
    let llm_manager = Arc::new(llm_manager);
    
    // Step 2: Setup Tools
    println!("🔧 Setting up tools...");
    let tool_registry = Arc::new(ToolRegistry::new());
    let tool_executor = Arc::new(ToolExecutor::new());
    
    // List available tools
    let available_tools = tool_registry.list_tools();
    println!("Available tools: {:?}", available_tools);
    
    // Step 3: Create Agent with Software Developer Role
    println!("\n👨‍💻 Creating software developer agent...");
    let developer_template = RoleTemplates::software_developer();
    let config = developer_template.to_agent_config("Alice".to_string(), "mock".to_string());
    
    println!("Agent config:");
    println!("  Name: {}", config.name);
    println!("  Role: {}", config.role);
    println!("  Model: {}", config.model);
    println!("  Temperature: {}", config.temperature);
    println!("  Tools: {:?}", config.tools);
    
    let mut agent = Agent::new(
        config,
        Arc::clone(&llm_manager),
        Arc::clone(&tool_registry),
        Arc::clone(&tool_executor),
    )?;
    
    // Step 4: Execute Tasks
    println!("\n🚀 Executing tasks...");
    
    let tasks = vec![
        "Write a hello world function in Python",
        "Explain the benefits of using Rust for systems programming",
        "Create a simple REST API endpoint in Python using FastAPI",
        "What are the best practices for error handling in Rust?",
    ];
    
    for (i, task) in tasks.iter().enumerate() {
        println!("\n--- Task {} ---", i + 1);
        println!("Task: {}", task);
        
        let response = agent.execute_task(task.to_string()).await?;
        println!("Response: {}", response);
        
        // Show agent state
        let state = agent.state();
        println!("Tokens used: {}", state.total_tokens_used);
        println!("Tasks completed: {}", state.tasks_completed);
    }
    
    // Step 5: Test Memory System
    println!("\n🧠 Testing memory system...");
    
    // Store some interactions
    agent.memory_mut().store_interaction(
        "What is the best way to handle errors in Python?",
        "Use try-except blocks for exception handling. For more robust error handling, consider using custom exception classes and proper logging."
    ).await?;
    
    agent.memory_mut().store_interaction(
        "How do I create a virtual environment in Python?",
        "Use 'python -m venv myenv' to create a virtual environment, then activate it with 'source myenv/bin/activate' on Unix or 'myenv\\Scripts\\activate' on Windows."
    ).await?;
    
    // Retrieve relevant context
    let context = agent.memory_mut().get_relevant_context("error handling").await?;
    println!("Relevant context for 'error handling': {}", context);
    
    let context = agent.memory_mut().get_relevant_context("virtual environment").await?;
    println!("Relevant context for 'virtual environment': {}", context);
    
    // Show memory statistics
    let memory_stats = agent.memory().get_stats();
    println!("\nMemory Statistics:");
    println!("  Total entries: {}", memory_stats.total_entries);
    println!("  Memory usage: {} bytes", memory_stats.memory_usage_bytes);
    
    // Step 6: Final Agent State
    println!("\n📊 Final Agent State:");
    let final_state = agent.state();
    println!("  Total tokens used: {}", final_state.total_tokens_used);
    println!("  Tasks completed: {}", final_state.tasks_completed);
    println!("  Average response time: {:.2}ms", final_state.average_response_time_ms);
    
    println!("\n✅ Basic agent example completed successfully!");
    
    Ok(())
}

// Helper function to demonstrate error handling
async fn demonstrate_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n⚠️  Demonstrating error handling...");
    
    let llm_config = LLMConfig::default();
    let llm_manager = LLMManager::new(llm_config);
    // Note: No provider registered
    
    let tool_registry = Arc::new(ToolRegistry::new());
    let tool_executor = Arc::new(ToolExecutor::new());
    
    let developer_template = RoleTemplates::software_developer();
    let mut config = developer_template.to_agent_config("TestAgent".to_string(), "nonexistent".to_string());
    config.provider = "nonexistent_provider".to_string();
    
    let result = Agent::new(
        config,
        Arc::new(llm_manager),
        tool_registry,
        tool_executor,
    );
    
    match result {
        Ok(_) => println!("Agent created successfully (unexpected)"),
        Err(e) => println!("Expected error: {:?}", e),
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_basic_agent_creation() {
        let llm_config = LLMConfig::default();
        let mut llm_manager = LLMManager::new(llm_config);
        let mock_provider = MockProvider::new();
        llm_manager.register_provider("mock".to_string(), Arc::new(mock_provider));
        
        let tool_registry = Arc::new(ToolRegistry::new());
        let tool_executor = Arc::new(ToolExecutor::new());
        
        let developer_template = RoleTemplates::software_developer();
        let config = developer_template.to_agent_config("TestAgent".to_string(), "mock".to_string());
        
        let agent = Agent::new(
            config,
            Arc::new(llm_manager),
            tool_registry,
            tool_executor,
        );
        
        assert!(agent.is_ok());
    }
    
    #[tokio::test]
    async fn test_agent_task_execution() {
        let llm_config = LLMConfig::default();
        let mut llm_manager = LLMManager::new(llm_config);
        let mock_provider = MockProvider::new();
        llm_manager.register_provider("mock".to_string(), Arc::new(mock_provider));
        
        let tool_registry = Arc::new(ToolRegistry::new());
        let tool_executor = Arc::new(ToolExecutor::new());
        
        let developer_template = RoleTemplates::software_developer();
        let config = developer_template.to_agent_config("TestAgent".to_string(), "mock".to_string());
        
        let mut agent = Agent::new(
            config,
            Arc::new(llm_manager),
            tool_registry,
            tool_executor,
        ).unwrap();
        
        let response = agent.execute_task("Write a hello world function".to_string()).await;
        assert!(response.is_ok());
        assert!(!response.unwrap().is_empty());
    }
    
    #[tokio::test]
    async fn test_memory_system() {
        let llm_config = LLMConfig::default();
        let mut llm_manager = LLMManager::new(llm_config);
        let mock_provider = MockProvider::new();
        llm_manager.register_provider("mock".to_string(), Arc::new(mock_provider));
        
        let tool_registry = Arc::new(ToolRegistry::new());
        let tool_executor = Arc::new(ToolExecutor::new());
        
        let developer_template = RoleTemplates::software_developer();
        let config = developer_template.to_agent_config("TestAgent".to_string(), "mock".to_string());
        
        let mut agent = Agent::new(
            config,
            Arc::new(llm_manager),
            tool_registry,
            tool_executor,
        ).unwrap();
        
        // Test memory storage
        let result = agent.memory_mut().store_interaction(
            "Test question",
            "Test answer"
        ).await;
        assert!(result.is_ok());
        
        // Test memory retrieval
        let context = agent.memory_mut().get_relevant_context("Test").await;
        assert!(context.is_ok());
        
        // Test memory stats
        let stats = agent.memory().get_stats();
        assert!(stats.total_entries > 0);
    }
}
