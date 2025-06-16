// Multi-Agent Collaboration Example
// Demonstrates how multiple agents can work together on complex tasks

use agent_graph::{
    agents::{Agent, collaboration::CollaborationManager, roles::RoleTemplates},
    llm::{LLMManager, LLMConfig, providers::MockProvider},
    tools::{ToolRegistry, ToolExecutor},
};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤝 AgentGraph Multi-Agent Collaboration Example");
    println!("===============================================");

    // Step 1: Setup Infrastructure
    println!("\n🏗️  Setting up infrastructure...");
    let llm_config = LLMConfig::default();
    let mut llm_manager = LLMManager::new(llm_config);
    let mock_provider = MockProvider::new();
    llm_manager.register_provider("mock".to_string(), Arc::new(mock_provider));
    let llm_manager = Arc::new(llm_manager);
    
    let tool_registry = Arc::new(ToolRegistry::new());
    let tool_executor = Arc::new(ToolExecutor::new());
    
    // Step 2: Setup Collaboration Manager
    println!("🤝 Setting up collaboration manager...");
    let collab_config = agent_graph::agents::collaboration::CollaborationConfig::default();
    let collab_manager = CollaborationManager::new(collab_config);
    
    // Step 3: Create Specialized Agents
    println!("\n👥 Creating specialized agents...");
    
    // Software Developer Agent
    let developer_template = RoleTemplates::software_developer();
    let developer_config = developer_template.to_agent_config("Alice".to_string(), "mock".to_string());
    let mut developer_agent = Agent::new(
        developer_config,
        Arc::clone(&llm_manager),
        Arc::clone(&tool_registry),
        Arc::clone(&tool_executor),
    )?;
    
    // QA Engineer Agent
    let qa_template = RoleTemplates::qa_engineer();
    let qa_config = qa_template.to_agent_config("Bob".to_string(), "mock".to_string());
    let mut qa_agent = Agent::new(
        qa_config,
        Arc::clone(&llm_manager),
        Arc::clone(&tool_registry),
        Arc::clone(&tool_executor),
    )?;
    
    // Project Manager Agent
    let pm_template = RoleTemplates::project_manager();
    let pm_config = pm_template.to_agent_config("Carol".to_string(), "mock".to_string());
    let mut pm_agent = Agent::new(
        pm_config,
        Arc::clone(&llm_manager),
        Arc::clone(&tool_registry),
        Arc::clone(&tool_executor),
    )?;
    
    // DevOps Engineer Agent
    let devops_template = RoleTemplates::devops_engineer();
    let devops_config = devops_template.to_agent_config("Dave".to_string(), "mock".to_string());
    let mut devops_agent = Agent::new(
        devops_config,
        Arc::clone(&llm_manager),
        Arc::clone(&tool_registry),
        Arc::clone(&tool_executor),
    )?;
    
    // Step 4: Register Agents with Collaboration Manager
    println!("📝 Registering agents with collaboration manager...");
    
    let _dev_receiver = collab_manager.register_agent(
        "Alice".to_string(),
        vec!["coding".to_string(), "debugging".to_string(), "architecture".to_string()]
    ).await?;
    
    let _qa_receiver = collab_manager.register_agent(
        "Bob".to_string(),
        vec!["testing".to_string(), "quality_assurance".to_string(), "automation".to_string()]
    ).await?;
    
    let _pm_receiver = collab_manager.register_agent(
        "Carol".to_string(),
        vec!["planning".to_string(), "coordination".to_string(), "requirements".to_string()]
    ).await?;
    
    let _devops_receiver = collab_manager.register_agent(
        "Dave".to_string(),
        vec!["deployment".to_string(), "infrastructure".to_string(), "monitoring".to_string()]
    ).await?;
    
    // Step 5: Demonstrate Collaboration
    println!("\n🚀 Starting collaborative project: 'Build a Web API'");
    
    // Project Manager starts by defining requirements
    println!("\n--- Phase 1: Requirements Gathering ---");
    let requirements_task = "Define requirements for a REST API that manages user accounts with authentication, CRUD operations, and rate limiting";
    let requirements = pm_agent.execute_task(requirements_task.to_string()).await?;
    println!("📋 Project Manager (Carol): {}", requirements);
    
    // Find agents with coding capabilities
    let coding_agents = collab_manager.find_agents_with_capabilities(&["coding".to_string()]).await?;
    println!("🔍 Found coding agents: {:?}", coding_agents);
    
    // Developer implements the API
    println!("\n--- Phase 2: Development ---");
    let dev_task = format!("Based on these requirements: '{}', implement a Python FastAPI application with user authentication", requirements);
    let implementation = developer_agent.execute_task(dev_task).await?;
    println!("💻 Developer (Alice): {}", implementation);
    
    // QA Engineer creates test plan
    println!("\n--- Phase 3: Quality Assurance ---");
    let qa_task = format!("Create a comprehensive test plan for this API implementation: '{}'", implementation);
    let test_plan = qa_agent.execute_task(qa_task).await?;
    println!("🧪 QA Engineer (Bob): {}", test_plan);
    
    // DevOps Engineer plans deployment
    println!("\n--- Phase 4: Deployment Planning ---");
    let devops_task = format!("Create a deployment strategy for this API: '{}'. Include containerization, CI/CD, and monitoring", implementation);
    let deployment_plan = devops_agent.execute_task(devops_task).await?;
    println!("🚀 DevOps Engineer (Dave): {}", deployment_plan);
    
    // Step 6: Collaboration Statistics
    println!("\n📊 Collaboration Statistics:");
    let collab_stats = collab_manager.get_stats().await;
    println!("  Registered agents: {}", collab_stats.registered_agents);
    println!("  Total messages: {}", collab_stats.total_messages);
    println!("  Active collaborations: {}", collab_stats.active_collaborations);
    
    // Step 7: Agent Performance Summary
    println!("\n📈 Agent Performance Summary:");
    
    let agents = vec![
        ("Alice (Developer)", &developer_agent),
        ("Bob (QA Engineer)", &qa_agent),
        ("Carol (Project Manager)", &pm_agent),
        ("Dave (DevOps Engineer)", &devops_agent),
    ];
    
    for (name, agent) in agents {
        let state = agent.state();
        println!("  {}: {} tokens, {} tasks, {:.2}ms avg response time",
                 name,
                 state.total_tokens_used,
                 state.tasks_completed,
                 state.average_response_time_ms);
    }
    
    // Step 8: Demonstrate Cross-Agent Memory Sharing
    println!("\n🧠 Demonstrating cross-agent memory sharing...");
    
    // Store shared knowledge
    developer_agent.memory_mut().store_interaction(
        "What are the API endpoints we implemented?",
        "We implemented /users (GET, POST), /users/{id} (GET, PUT, DELETE), /auth/login (POST), /auth/register (POST)"
    ).await?;
    
    qa_agent.memory_mut().store_interaction(
        "What testing frameworks should we use?",
        "For Python FastAPI, use pytest for unit tests, httpx for API testing, and pytest-asyncio for async tests"
    ).await?;
    
    // Retrieve shared context
    let api_context = developer_agent.memory_mut().get_relevant_context("API endpoints").await?;
    println!("🔍 Developer's API knowledge: {}", api_context);
    
    let testing_context = qa_agent.memory_mut().get_relevant_context("testing frameworks").await?;
    println!("🔍 QA's testing knowledge: {}", testing_context);
    
    println!("\n✅ Multi-agent collaboration example completed successfully!");
    
    Ok(())
}

// Simulate a complex workflow with multiple agents
async fn simulate_complex_workflow() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 Simulating complex workflow...");
    
    // This would involve:
    // 1. Task decomposition by PM
    // 2. Parallel development by multiple developers
    // 3. Continuous testing by QA
    // 4. Infrastructure setup by DevOps
    // 5. Coordination and status updates
    
    // Simulate some async work
    sleep(Duration::from_millis(100)).await;
    
    println!("Complex workflow simulation completed");
    Ok(())
}

// Demonstrate agent specialization
async fn demonstrate_agent_specialization() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🎯 Demonstrating agent specialization...");
    
    // Show how different role templates have different:
    // - System prompts
    // - Tool access
    // - Temperature settings
    // - Capabilities
    
    let templates = RoleTemplates::all_templates();
    
    for template in templates {
        println!("Role: {}", template.name);
        println!("  Temperature: {}", template.temperature);
        println!("  Tools: {:?}", template.tools);
        println!("  Capabilities: {:?}", template.capabilities);
        println!();
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_collaboration_manager() {
        let collab_config = agent_graph::agents::collaboration::CollaborationConfig::default();
        let collab_manager = CollaborationManager::new(collab_config);
        
        // Register test agents
        let _receiver1 = collab_manager.register_agent(
            "Agent1".to_string(),
            vec!["coding".to_string()]
        ).await.unwrap();
        
        let _receiver2 = collab_manager.register_agent(
            "Agent2".to_string(),
            vec!["testing".to_string()]
        ).await.unwrap();
        
        // Test finding agents
        let coding_agents = collab_manager.find_agents_with_capabilities(&["coding".to_string()]).await.unwrap();
        assert!(coding_agents.contains(&"Agent1".to_string()));
        
        let testing_agents = collab_manager.find_agents_with_capabilities(&["testing".to_string()]).await.unwrap();
        assert!(testing_agents.contains(&"Agent2".to_string()));
        
        // Test stats
        let stats = collab_manager.get_stats().await;
        assert_eq!(stats.registered_agents, 2);
    }
    
    #[tokio::test]
    async fn test_multi_agent_creation() {
        let llm_config = LLMConfig::default();
        let mut llm_manager = LLMManager::new(llm_config);
        let mock_provider = MockProvider::new();
        llm_manager.register_provider("mock".to_string(), Arc::new(mock_provider));
        let llm_manager = Arc::new(llm_manager);
        
        let tool_registry = Arc::new(ToolRegistry::new());
        let tool_executor = Arc::new(ToolExecutor::new());
        
        // Create multiple agents with different roles
        let roles = vec![
            RoleTemplates::software_developer(),
            RoleTemplates::qa_engineer(),
            RoleTemplates::project_manager(),
            RoleTemplates::devops_engineer(),
        ];
        
        for (i, template) in roles.iter().enumerate() {
            let config = template.to_agent_config(format!("Agent{}", i), "mock".to_string());
            let agent = Agent::new(
                config,
                Arc::clone(&llm_manager),
                Arc::clone(&tool_registry),
                Arc::clone(&tool_executor),
            );
            assert!(agent.is_ok());
        }
    }
}
