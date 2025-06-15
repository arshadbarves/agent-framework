// Agent System demonstration example
// Shows role-based agents, memory, collaboration, and multi-agent workflows

use agent_graph::{
    agents::{
        Agent, AgentRole,
        collaboration::{CollaborationManager, CollaborationConfig, CollaborationPattern, MessageUrgency},
        roles::RoleTemplates,
    },
    llm::{LLMManager, LLMConfig, providers::MockProvider},
    tools::{ToolRegistry, ToolExecutor},
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 AgentGraph Agent System Demo");
    println!("===============================");
    
    // Setup infrastructure
    let llm_config = LLMConfig::default();
    let mut llm_manager = LLMManager::new(llm_config);
    
    // Register mock provider
    let mock_provider = MockProvider::new();
    llm_manager.register_provider("mock".to_string(), Arc::new(mock_provider));
    let llm_manager = Arc::new(llm_manager);
    
    // Setup tool system
    let tool_registry = Arc::new(ToolRegistry::new());
    let tool_executor = Arc::new(ToolExecutor::new());
    
    // Demo 1: Role-Based Agent Creation
    println!("\n🎭 Demo 1: Role-Based Agent Creation");
    println!("===================================");
    
    // Create agents with different roles
    let developer_template = RoleTemplates::software_developer();
    let developer_config = developer_template.to_agent_config("Alice_Developer".to_string(), "mock".to_string());
    
    let researcher_template = RoleTemplates::research_analyst();
    let researcher_config = researcher_template.to_agent_config("Bob_Researcher".to_string(), "mock".to_string());
    
    let writer_template = RoleTemplates::content_writer();
    let writer_config = writer_template.to_agent_config("Carol_Writer".to_string(), "mock".to_string());
    
    println!("✅ Created role templates:");
    println!("   - Developer: {} tools, temp={}", developer_config.available_tools.len(), developer_config.temperature.unwrap_or(0.0));
    println!("   - Researcher: {} tools, temp={}", researcher_config.available_tools.len(), researcher_config.temperature.unwrap_or(0.0));
    println!("   - Writer: {} tools, temp={}", writer_config.available_tools.len(), writer_config.temperature.unwrap_or(0.0));
    
    // Create agent instances
    let mut developer_agent = Agent::new(
        developer_config,
        Arc::clone(&llm_manager),
        Arc::clone(&tool_registry),
        Arc::clone(&tool_executor),
    )?;
    
    let mut researcher_agent = Agent::new(
        researcher_config,
        Arc::clone(&llm_manager),
        Arc::clone(&tool_registry),
        Arc::clone(&tool_executor),
    )?;
    
    let mut writer_agent = Agent::new(
        writer_config,
        Arc::clone(&llm_manager),
        Arc::clone(&tool_registry),
        Arc::clone(&tool_executor),
    )?;
    
    println!("✅ Created agent instances:");
    println!("   - {}: {}", developer_agent.config().name, developer_agent.config().role.default_system_prompt().split('.').next().unwrap_or(""));
    println!("   - {}: {}", researcher_agent.config().name, researcher_agent.config().role.default_system_prompt().split('.').next().unwrap_or(""));
    println!("   - {}: {}", writer_agent.config().name, writer_agent.config().role.default_system_prompt().split('.').next().unwrap_or(""));
    
    // Demo 2: Agent Task Execution
    println!("\n💼 Demo 2: Agent Task Execution");
    println!("===============================");
    
    // Developer task
    let dev_task = "Review this code snippet and suggest improvements: function add(a, b) { return a + b; }";
    println!("🔧 Developer task: {}", dev_task);
    
    let dev_response = developer_agent.execute_task(dev_task.to_string()).await?;
    println!("✅ Developer response: {}", dev_response);
    println!("   Tokens used: {}", developer_agent.state().total_tokens_used);
    println!("   Status: {:?}", developer_agent.state().status);
    
    // Researcher task
    let research_task = "Research the latest trends in artificial intelligence and machine learning";
    println!("\n🔍 Researcher task: {}", research_task);
    
    let research_response = researcher_agent.execute_task(research_task.to_string()).await?;
    println!("✅ Researcher response: {}", research_response);
    println!("   Tokens used: {}", researcher_agent.state().total_tokens_used);
    
    // Writer task
    let writing_task = "Write a brief introduction to quantum computing for beginners";
    println!("\n✍️  Writer task: {}", writing_task);
    
    let writing_response = writer_agent.execute_task(writing_task.to_string()).await?;
    println!("✅ Writer response: {}", writing_response);
    println!("   Tokens used: {}", writer_agent.state().total_tokens_used);
    
    // Demo 3: Agent Memory System
    println!("\n🧠 Demo 3: Agent Memory System");
    println!("==============================");
    
    // Store additional interactions in developer agent
    developer_agent.memory_mut().store_interaction(
        "What's the best way to handle errors in Rust?",
        "Use Result<T, E> for recoverable errors and panic! for unrecoverable errors."
    ).await?;

    developer_agent.memory_mut().store_task(
        "Implement error handling",
        "Added proper Result types and error propagation",
        true
    ).await?;

    // Test memory retrieval
    let context = developer_agent.memory_mut().get_relevant_context("error handling").await?;
    println!("✅ Retrieved relevant context for 'error handling':");
    println!("   Context length: {} characters", context.len());
    if !context.is_empty() {
        println!("   Sample: {}...", context.chars().take(100).collect::<String>());
    }
    
    // Memory statistics
    let memory_stats = developer_agent.memory().get_stats();
    println!("📊 Developer agent memory stats:");
    println!("   Short-term entries: {}", memory_stats.short_term_entries);
    println!("   Long-term entries: {}", memory_stats.long_term_entries);
    println!("   Average importance: {:.2}", memory_stats.average_importance);
    
    // Demo 4: Multi-Agent Collaboration
    println!("\n🤝 Demo 4: Multi-Agent Collaboration");
    println!("====================================");
    
    // Setup collaboration manager
    let collab_config = CollaborationConfig::default();
    let collab_manager = CollaborationManager::new(collab_config);
    
    // Register agents for collaboration
    let dev_capabilities = vec!["coding".to_string(), "debugging".to_string(), "architecture".to_string()];
    let research_capabilities = vec!["research".to_string(), "analysis".to_string(), "documentation".to_string()];
    let writing_capabilities = vec!["writing".to_string(), "editing".to_string(), "communication".to_string()];
    
    let _dev_receiver = collab_manager.register_agent("Alice_Developer".to_string(), dev_capabilities).await?;
    let _research_receiver = collab_manager.register_agent("Bob_Researcher".to_string(), research_capabilities).await?;
    let _writer_receiver = collab_manager.register_agent("Carol_Writer".to_string(), writing_capabilities).await?;
    
    println!("✅ Registered agents for collaboration");
    
    // Find agents with specific capabilities
    let coding_agents = collab_manager.find_agents_with_capabilities(&["coding".to_string()]).await?;
    let writing_agents = collab_manager.find_agents_with_capabilities(&["writing".to_string()]).await?;
    
    println!("🔍 Agent capability discovery:");
    println!("   Coding agents: {:?}", coding_agents);
    println!("   Writing agents: {:?}", writing_agents);
    
    // Start a collaboration session
    let participants = vec!["Alice_Developer".to_string(), "Bob_Researcher".to_string(), "Carol_Writer".to_string()];
    let session_id = collab_manager.start_collaboration(
        participants,
        CollaborationPattern::Sequential,
        "Create a technical blog post about Rust programming".to_string(),
    ).await?;
    
    println!("✅ Started collaboration session: {}", session_id);
    
    // Request assistance
    let assistance_candidates = collab_manager.request_assistance(
        "Alice_Developer",
        "Need help with technical writing".to_string(),
        "Working on a blog post about Rust best practices".to_string(),
        MessageUrgency::Normal,
        vec!["writing".to_string()],
    ).await?;
    
    println!("📞 Assistance request sent to: {:?}", assistance_candidates);
    
    // Get collaboration statistics
    let collab_stats = collab_manager.get_stats().await;
    println!("📊 Collaboration statistics:");
    println!("   Registered agents: {}", collab_stats.registered_agents);
    println!("   Active sessions: {}", collab_stats.active_sessions);
    println!("   Total sessions: {}", collab_stats.total_sessions);
    
    // Demo 5: Agent Role Specialization
    println!("\n🎯 Demo 5: Agent Role Specialization");
    println!("====================================");
    
    // Show all available role templates
    let all_templates = RoleTemplates::all_templates();
    println!("✅ Available role templates ({}):", all_templates.len());
    for template in &all_templates {
        println!("   - {}: {} tools, {}°C temperature", 
                 template.name, 
                 template.tools.len(), 
                 template.temperature);
    }
    
    // Create a custom QA agent
    let qa_template = RoleTemplates::quality_assurance();
    let qa_config = qa_template.to_agent_config("Dave_QA".to_string(), "mock".to_string());
    
    let mut qa_agent = Agent::new(
        qa_config,
        Arc::clone(&llm_manager),
        Arc::clone(&tool_registry),
        Arc::clone(&tool_executor),
    )?;
    
    // QA task
    let qa_task = "Review this code for potential issues: let mut x = 5; x = x + 1; println!(\"{}\", x);";
    println!("🔍 QA Agent task: {}", qa_task);
    
    let qa_response = qa_agent.execute_task(qa_task.to_string()).await?;
    println!("✅ QA Agent response: {}", qa_response);
    
    // Demo 6: Agent State Management
    println!("\n📊 Demo 6: Agent State Management");
    println!("=================================");
    
    // Show agent states
    println!("🤖 Agent Status Summary:");
    println!("   Developer: {:?} | Tokens: {} | Cost: ${:.4}", 
             developer_agent.state().status,
             developer_agent.state().total_tokens_used,
             developer_agent.state().total_cost);
    
    println!("   Researcher: {:?} | Tokens: {} | Cost: ${:.4}", 
             researcher_agent.state().status,
             researcher_agent.state().total_tokens_used,
             researcher_agent.state().total_cost);
    
    println!("   Writer: {:?} | Tokens: {} | Cost: ${:.4}", 
             writer_agent.state().status,
             writer_agent.state().total_tokens_used,
             writer_agent.state().total_cost);
    
    println!("   QA: {:?} | Tokens: {} | Cost: ${:.4}", 
             qa_agent.state().status,
             qa_agent.state().total_tokens_used,
             qa_agent.state().total_cost);
    
    // Show conversation history
    println!("\n💬 Developer Agent Conversation History:");
    for (i, message) in developer_agent.get_conversation().iter().enumerate() {
        println!("   {}. {:?}: {}", i + 1, message.role, 
                 message.content.chars().take(50).collect::<String>() + 
                 if message.content.len() > 50 { "..." } else { "" });
    }
    
    // Demo 7: Agent Configuration and Customization
    println!("\n⚙️  Demo 7: Agent Configuration");
    println!("==============================");
    
    // Show role-specific configurations
    println!("🎭 Role-Specific Configurations:");
    
    let dev_role = AgentRole::Developer;
    println!("   Developer tools: {:?}", dev_role.recommended_tools());
    
    let researcher_role = AgentRole::Researcher;
    println!("   Researcher tools: {:?}", researcher_role.recommended_tools());
    
    let writer_role = AgentRole::Writer;
    println!("   Writer tools: {:?}", writer_role.recommended_tools());
    
    // Demonstrate agent tool management
    println!("\n🔧 Agent Tool Management:");
    println!("   Developer tools before: {}", developer_agent.config().available_tools.len());
    
    developer_agent.add_tool("new_debugging_tool".to_string());
    println!("   Developer tools after adding: {}", developer_agent.config().available_tools.len());
    
    developer_agent.remove_tool("new_debugging_tool");
    println!("   Developer tools after removing: {}", developer_agent.config().available_tools.len());
    
    // Demo 8: Memory Configuration and Optimization
    println!("\n🧠 Demo 8: Memory Optimization");
    println!("==============================");
    
    // Show memory configurations for different roles
    println!("📝 Memory Configurations by Role:");
    
    let dev_memory_config = RoleTemplates::software_developer().memory_config;
    println!("   Developer: {} short-term, {} long-term, {}d retention", 
             dev_memory_config.max_short_term_entries,
             dev_memory_config.max_long_term_entries,
             dev_memory_config.retention_period.as_secs() / 86400);
    
    let researcher_memory_config = RoleTemplates::research_analyst().memory_config;
    println!("   Researcher: {} short-term, {} long-term, {}d retention", 
             researcher_memory_config.max_short_term_entries,
             researcher_memory_config.max_long_term_entries,
             researcher_memory_config.retention_period.as_secs() / 86400);
    
    // Working memory demonstration
    developer_agent.memory_mut().set_working_memory("current_project".to_string(), "AgentGraph Demo");
    developer_agent.memory_mut().set_working_memory("priority".to_string(), "high");
    
    let project = developer_agent.memory().get_working_memory("current_project");
    let priority = developer_agent.memory().get_working_memory("priority");
    
    println!("💾 Working Memory Example:");
    println!("   Current project: {:?}", project);
    println!("   Priority: {:?}", priority);
    
    println!("\n🎉 Agent System Demo Complete!");
    println!("===============================");
    println!("The AgentGraph Agent System provides:");
    println!("✅ Role-based agents with specialized capabilities");
    println!("✅ Advanced memory system with context retrieval");
    println!("✅ Multi-agent collaboration and communication");
    println!("✅ Comprehensive role templates and customization");
    println!("✅ State management and lifecycle tracking");
    println!("✅ Tool integration and execution");
    println!("✅ Cost tracking and resource management");
    println!("✅ Production-ready scalability and reliability");
    
    Ok(())
}
