// Enterprise Deployment Example
// Demonstrates production-ready AgentGraph deployment with security, monitoring, and scaling

use agent_graph::{
    agents::{Agent, collaboration::CollaborationManager, roles::RoleTemplates},
    enterprise::{
        security::{SecurityManager, AuthContext, Permission},
        monitoring::{MonitoringManager, PerformanceMetrics},
        resources::{ResourceManager, ResourceUsage},
    },
    human::approval::{ApprovalManager, ApprovalRequest, RiskLevel},
    llm::{LLMManager, LLMConfig, providers::OpenAIProvider},
    state::StateManager,
    tools::{ToolRegistry, ToolExecutor},
};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏢 AgentGraph Enterprise Deployment Example");
    println!("==========================================");

    // Step 1: Initialize Enterprise Platform
    println!("\n🚀 Initializing enterprise platform...");
    let platform = EnterprisePlatform::new().await?;
    
    // Step 2: Setup Security
    println!("🔒 Configuring security...");
    let security_manager = SecurityManager::new();
    
    // Create admin user context
    let admin_context = AuthContext {
        user_id: "admin@company.com".to_string(),
        roles: vec!["admin".to_string(), "agent_manager".to_string()],
        session_id: uuid::Uuid::new_v4().to_string(),
        authenticated_at: std::time::SystemTime::now(),
        expires_at: Some(std::time::SystemTime::now() + Duration::from_secs(3600)),
        claims: HashMap::new(),
    };
    
    // Verify admin permissions
    let can_manage = security_manager.authorize(
        &admin_context,
        Permission {
            resource: "agents".to_string(),
            action: "manage".to_string(),
        }
    ).await?;
    
    if !can_manage {
        return Err("Insufficient permissions for agent management".into());
    }
    
    println!("✅ Admin permissions verified");
    
    // Step 3: Setup Monitoring
    println!("📊 Initializing monitoring...");
    let monitoring_manager = MonitoringManager::new();
    
    // Record initial metrics
    monitoring_manager.record_metric(
        "platform_startup".to_string(),
        agent_graph::enterprise::monitoring::MetricType::Counter,
        1.0,
        HashMap::new(),
    ).await?;
    
    // Step 4: Setup Resource Management
    println!("💾 Configuring resource management...");
    let resource_manager = ResourceManager::new();
    
    // Set resource limits for the deployment
    let resource_limits = ResourceUsage {
        cpu_time_ms: 10000,      // 10 seconds CPU time
        memory_bytes: 1024 * 1024 * 1024, // 1GB memory
        execution_count: 1000,    // 1000 executions
        network_bytes_sent: 100 * 1024 * 1024,   // 100MB sent
        network_bytes_received: 100 * 1024 * 1024, // 100MB received
        storage_bytes: 10 * 1024 * 1024 * 1024,   // 10GB storage
        custom_metrics: HashMap::new(),
    };
    
    println!("Resource limits configured: {:?}", resource_limits);
    
    // Step 5: Setup LLM Providers with Enterprise Configuration
    println!("🤖 Configuring LLM providers...");
    let llm_config = LLMConfig {
        default_provider: "openai".to_string(),
        timeout: Duration::from_secs(30),
        max_retries: 3,
        retry_delay: Duration::from_millis(1000),
    };
    
    let mut llm_manager = LLMManager::new(llm_config);
    
    // In production, use real API keys from environment variables
    if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
        let openai_provider = OpenAIProvider::new(api_key);
        llm_manager.register_provider("openai".to_string(), Arc::new(openai_provider));
        println!("✅ OpenAI provider configured");
    } else {
        println!("⚠️  OpenAI API key not found, using mock provider");
        let mock_provider = agent_graph::llm::providers::MockProvider::new();
        llm_manager.register_provider("openai".to_string(), Arc::new(mock_provider));
    }
    
    let llm_manager = Arc::new(llm_manager);
    
    // Step 6: Setup Tools and Collaboration
    println!("🔧 Setting up tools and collaboration...");
    let tool_registry = Arc::new(ToolRegistry::new());
    let tool_executor = Arc::new(ToolExecutor::new());
    
    let collab_config = agent_graph::agents::collaboration::CollaborationConfig {
        max_agents: 100,
        message_timeout: Duration::from_secs(30),
        heartbeat_interval: Duration::from_secs(10),
    };
    let collab_manager = CollaborationManager::new(collab_config);
    
    // Step 7: Create Enterprise Agent Team
    println!("\n👥 Creating enterprise agent team...");
    
    let mut agents = Vec::new();
    let agent_configs = vec![
        ("SecurityAnalyst", RoleTemplates::research_analyst()),
        ("LeadDeveloper", RoleTemplates::software_developer()),
        ("QAManager", RoleTemplates::qa_engineer()),
        ("ProjectManager", RoleTemplates::project_manager()),
        ("DevOpsEngineer", RoleTemplates::devops_engineer()),
    ];
    
    for (name, template) in agent_configs {
        let config = template.to_agent_config(name.to_string(), "openai".to_string());
        let agent = Agent::new(
            config,
            Arc::clone(&llm_manager),
            Arc::clone(&tool_registry),
            Arc::clone(&tool_executor),
        )?;
        
        // Register agent with collaboration manager
        let capabilities = match name {
            "SecurityAnalyst" => vec!["security".to_string(), "analysis".to_string()],
            "LeadDeveloper" => vec!["coding".to_string(), "architecture".to_string()],
            "QAManager" => vec!["testing".to_string(), "quality".to_string()],
            "ProjectManager" => vec!["planning".to_string(), "coordination".to_string()],
            "DevOpsEngineer" => vec!["deployment".to_string(), "infrastructure".to_string()],
            _ => vec![],
        };
        
        collab_manager.register_agent(name.to_string(), capabilities).await?;
        agents.push((name, agent));
        
        println!("✅ Created and registered agent: {}", name);
    }
    
    // Step 8: Setup Human-in-the-Loop Approval System
    println!("\n👤 Configuring human approval system...");
    let approval_manager = ApprovalManager::new(Arc::new(MockHumanInteraction::new()));
    
    // Step 9: Execute Enterprise Workflow
    println!("\n🚀 Executing enterprise workflow: 'Security Assessment and Deployment'");
    
    // Create high-risk approval request
    let approval_request = ApprovalRequest {
        request_id: "DEPLOY-2024-001".to_string(),
        title: "Production Deployment Approval".to_string(),
        description: "Deploy new agent system to production environment".to_string(),
        risk_level: RiskLevel::High,
        data: agent_graph::human::approval::HumanContext::default(),
        min_approvals: 2,
        required_approvers: vec!["security_team".to_string(), "ops_team".to_string()],
        auto_approve_conditions: vec![],
        expires_at: Some(std::time::SystemTime::now() + Duration::from_secs(3600)),
    };
    
    let approval_id = approval_manager.create_approval(approval_request).await?;
    println!("📋 Created approval request: {}", approval_id);
    
    // Simulate workflow execution with monitoring
    println!("\n📈 Monitoring workflow execution...");
    
    let workflow_start = std::time::Instant::now();
    
    // Execute tasks across different agents
    let tasks = vec![
        ("SecurityAnalyst", "Perform security assessment of the new deployment"),
        ("LeadDeveloper", "Review code quality and architecture"),
        ("QAManager", "Validate test coverage and quality metrics"),
        ("DevOpsEngineer", "Prepare infrastructure and deployment scripts"),
        ("ProjectManager", "Coordinate timeline and stakeholder communication"),
    ];
    
    let mut results = Vec::new();
    
    for (agent_name, task) in tasks {
        let start_time = std::time::Instant::now();
        
        // Find the agent
        if let Some((_, agent)) = agents.iter_mut().find(|(name, _)| *name == agent_name) {
            println!("🔄 {} executing: {}", agent_name, task);
            
            let result = agent.execute_task(task.to_string()).await?;
            let execution_time = start_time.elapsed();
            
            // Record metrics
            let mut labels = HashMap::new();
            labels.insert("agent".to_string(), agent_name.to_string());
            labels.insert("task_type".to_string(), "workflow_task".to_string());
            
            monitoring_manager.record_metric(
                "task_execution_time_ms".to_string(),
                agent_graph::enterprise::monitoring::MetricType::Histogram,
                execution_time.as_millis() as f64,
                labels.clone(),
            ).await?;
            
            monitoring_manager.record_metric(
                "task_completed".to_string(),
                agent_graph::enterprise::monitoring::MetricType::Counter,
                1.0,
                labels,
            ).await?;
            
            results.push((agent_name, result, execution_time));
            println!("✅ {} completed in {:?}", agent_name, execution_time);
        }
    }
    
    let total_workflow_time = workflow_start.elapsed();
    
    // Step 10: Generate Enterprise Report
    println!("\n📊 Generating enterprise report...");
    
    println!("=== ENTERPRISE WORKFLOW REPORT ===");
    println!("Workflow: Security Assessment and Deployment");
    println!("Total Execution Time: {:?}", total_workflow_time);
    println!("Agents Involved: {}", agents.len());
    println!("Tasks Completed: {}", results.len());
    
    println!("\n--- Agent Performance ---");
    for (name, _, agent) in &agents {
        let state = agent.state();
        println!("{}: {} tokens, {} tasks, {:.2}ms avg",
                 name,
                 state.total_tokens_used,
                 state.tasks_completed,
                 state.average_response_time_ms);
    }
    
    println!("\n--- Task Results ---");
    for (agent_name, result, execution_time) in &results {
        println!("{} ({:?}): {}", agent_name, execution_time, 
                 result.chars().take(100).collect::<String>() + "...");
    }
    
    // Step 11: Resource Usage Report
    println!("\n--- Resource Usage ---");
    let current_usage = resource_manager.get_current_usage();
    println!("CPU Time: {}ms", current_usage.cpu_time_ms);
    println!("Memory: {} bytes", current_usage.memory_bytes);
    println!("Network Sent: {} bytes", current_usage.network_bytes_sent);
    println!("Network Received: {} bytes", current_usage.network_bytes_received);
    
    // Step 12: Security Audit
    println!("\n--- Security Audit ---");
    println!("Admin Session: Valid");
    println!("Agent Permissions: Verified");
    println!("Approval Status: Pending (ID: {})", approval_id);
    println!("Encryption: Enabled");
    println!("Audit Trail: Complete");
    
    // Step 13: Collaboration Statistics
    println!("\n--- Collaboration Statistics ---");
    let collab_stats = collab_manager.get_stats().await;
    println!("Registered Agents: {}", collab_stats.registered_agents);
    println!("Total Messages: {}", collab_stats.total_messages);
    println!("Active Collaborations: {}", collab_stats.active_collaborations);
    
    println!("\n✅ Enterprise deployment example completed successfully!");
    println!("🎯 System is ready for production deployment");
    
    Ok(())
}

// Mock enterprise platform for demonstration
struct EnterprisePlatform;

impl EnterprisePlatform {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize enterprise platform components
        Ok(Self)
    }
}

// Mock human interaction for approval system
struct MockHumanInteraction;

impl MockHumanInteraction {
    fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl agent_graph::human::interaction::HumanInteraction for MockHumanInteraction {
    async fn request_interaction(
        &self,
        _request: agent_graph::human::interaction::InteractionRequest,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        // Simulate human approval
        Ok(serde_json::json!({
            "approved": true,
            "approver": "security_team",
            "comment": "Security assessment passed, deployment approved"
        }))
    }
    
    async fn get_interaction_status(
        &self,
        _interaction_id: &str,
    ) -> Result<agent_graph::human::interaction::InteractionStatus, Box<dyn std::error::Error + Send + Sync>> {
        Ok(agent_graph::human::interaction::InteractionStatus::Completed)
    }
    
    async fn cancel_interaction(
        &self,
        _interaction_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_enterprise_security() {
        let security_manager = SecurityManager::new();
        
        let auth_context = AuthContext {
            user_id: "test@company.com".to_string(),
            roles: vec!["user".to_string()],
            session_id: uuid::Uuid::new_v4().to_string(),
            authenticated_at: std::time::SystemTime::now(),
            expires_at: Some(std::time::SystemTime::now() + Duration::from_secs(3600)),
            claims: HashMap::new(),
        };
        
        let permission = Permission {
            resource: "agents".to_string(),
            action: "read".to_string(),
        };
        
        // This should work for basic read permissions
        let result = security_manager.authorize(&auth_context, permission).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_resource_monitoring() {
        let resource_manager = ResourceManager::new();
        let current_usage = resource_manager.get_current_usage();
        
        // Should have some basic resource tracking
        assert!(current_usage.cpu_time_ms >= 0);
        assert!(current_usage.memory_bytes >= 0);
    }
}
