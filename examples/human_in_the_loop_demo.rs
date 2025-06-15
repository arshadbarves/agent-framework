// Human-in-the-Loop demonstration example
// Shows basic usage of the human interaction framework

use agent_graph::human::{
    HumanContext, HumanConfig,
    input::{InputCollector, ConsoleInteraction},
    approval::{ApprovalManager, ApprovalRequest, ApprovalResponse, ApprovalDecision, RiskLevel},
    interrupt::{InterruptManager, InterruptPoint},
    traits::{HumanInput, HumanInteraction},
};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct DemoState {
    step: String,
    data: serde_json::Value,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 AgentGraph Human-in-the-Loop Demo");
    println!("====================================");
    
    // Create human interaction provider
    let interaction_provider = Arc::new(ConsoleInteraction::new());
    
    // Demo 1: Basic Input Collection
    println!("\n📝 Demo 1: Input Collection");
    println!("===========================");
    
    let input_collector = InputCollector::new(interaction_provider.clone());
    let context = HumanContext::new("demo_input".to_string())
        .with_user_id("demo_user".to_string())
        .with_session_id("demo_session".to_string());
    
    // Text input
    let text_request = input_collector.text_input(
        "text_input_1".to_string(),
        "Please enter your name:".to_string(),
        context.clone(),
    );
    
    match input_collector.collect_input(text_request).await {
        Ok(response) => {
            println!("✅ Name received: {}", response.as_string().unwrap_or("N/A".to_string()));
            println!("   Response time: {}ms", response.response_time_ms);
        }
        Err(e) => println!("❌ Failed to collect name: {}", e),
    }
    
    // Multiple choice input
    let choice_request = input_collector.multiple_choice(
        "choice_input_1".to_string(),
        "What's your favorite programming language?".to_string(),
        vec!["Rust".to_string(), "Python".to_string(), "JavaScript".to_string(), "Go".to_string()],
        context.clone(),
    );
    
    match input_collector.collect_input(choice_request).await {
        Ok(response) => {
            println!("✅ Language choice: {}", response.as_string().unwrap_or("N/A".to_string()));
        }
        Err(e) => println!("❌ Failed to collect choice: {}", e),
    }
    
    // Demo 2: Approval Workflow
    println!("\n✋ Demo 2: Approval Workflow");
    println!("============================");
    
    let approval_manager = ApprovalManager::new(interaction_provider.clone());
    
    // Create approval request
    let approval_context = HumanContext::new("approval_demo".to_string())
        .with_user_id("admin_user".to_string())
        .with_graph_context("operation".to_string(), "database_migration".to_string());
    
    let approval_request = ApprovalRequest::new(
        "migration_approval".to_string(),
        "Database Migration Approval".to_string(),
        "This operation will migrate the user database to a new schema. This is irreversible.".to_string(),
        approval_context.clone(),
    )
    .with_risk_level(RiskLevel::High)
    .with_approver("admin_user".to_string())
    .with_min_approvals(1)
    .with_expiration(Duration::from_secs(300)); // 5 minutes
    
    // Submit approval request
    match approval_manager.submit_request(approval_request).await {
        Ok(request_id) => {
            println!("✅ Approval request submitted: {}", request_id);
            
            // Simulate human approval decision
            println!("\n🤔 Approval Required:");
            println!("   Operation: Database Migration");
            println!("   Risk Level: High");
            println!("   Description: This operation will migrate the user database to a new schema. This is irreversible.");
            
            // Create approval input
            let approval_input = HumanInput::approval("Do you approve this database migration?".to_string())
                .with_context("This is a high-risk operation that cannot be undone.".to_string());
            
            let config = HumanConfig::default();
            
            match interaction_provider.request_input(approval_input, &approval_context, &config).await {
                Ok(response) => {
                    let approved = response.as_bool().unwrap_or(false);
                    let decision = if approved { ApprovalDecision::Approved } else { ApprovalDecision::Rejected };
                    
                    let approval_response = ApprovalResponse::new(
                        request_id.clone(),
                        "admin_user".to_string(),
                        decision,
                    ).with_comments(format!("Decision made via console interface"));
                    
                    match approval_manager.submit_response(approval_response) {
                        Ok(status) => {
                            println!("✅ Approval decision recorded: {:?}", status);
                            
                            if approved {
                                println!("🚀 Migration approved! Proceeding with operation...");
                            } else {
                                println!("🛑 Migration rejected. Operation cancelled.");
                            }
                        }
                        Err(e) => println!("❌ Failed to record approval: {}", e),
                    }
                }
                Err(e) => println!("❌ Failed to get approval: {}", e),
            }
        }
        Err(e) => println!("❌ Failed to submit approval request: {}", e),
    }
    
    // Demo 3: Interrupt and Resume
    println!("\n⏸️  Demo 3: Interrupt and Resume");
    println!("===============================");
    
    let interrupt_manager = InterruptManager::<DemoState>::new();
    
    // Register interrupt points
    let checkpoint = InterruptPoint::checkpoint("data_processing".to_string())
        .with_timeout(Duration::from_secs(60));
    
    let human_input_point = InterruptPoint::input("user_verification".to_string())
        .with_timeout(Duration::from_secs(120));
    
    interrupt_manager.register_interrupt_point(checkpoint)?;
    interrupt_manager.register_interrupt_point(human_input_point)?;
    
    // Simulate workflow execution with interrupts
    let initial_state = DemoState {
        step: "processing_data".to_string(),
        data: serde_json::json!({"records_processed": 1000, "errors": 0}),
    };
    
    // Create interrupt at checkpoint
    let token = interrupt_manager.create_interrupt(
        "workflow_123".to_string(),
        "data_processing".to_string(),
        initial_state.clone(),
        "Checkpoint reached - data processing complete".to_string(),
    )?;
    
    println!("✅ Workflow interrupted at checkpoint");
    println!("   Interrupt ID: {}", token.interrupt_id);
    println!("   Node: {}", token.node_id);
    println!("   State preserved: {:?}", initial_state);
    
    // List active interrupts
    let active_interrupts = interrupt_manager.list_active_interrupts()?;
    println!("📋 Active interrupts: {}", active_interrupts.len());
    
    // Simulate some time passing
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Resume execution
    match interrupt_manager.resume_execution(&token) {
        Ok(interrupt_state) => {
            println!("▶️  Workflow resumed successfully");
            println!("   Restored state: {:?}", interrupt_state.state);
            println!("   Interrupt reason: {}", interrupt_state.reason);
        }
        Err(e) => println!("❌ Failed to resume workflow: {}", e),
    }
    
    // Demo 4: Statistics and Monitoring
    println!("\n📊 Demo 4: Statistics");
    println!("====================");
    
    // Get approval statistics
    match approval_manager.get_stats() {
        Ok(stats) => {
            println!("Approval Statistics:");
            println!("  Total interactions: {}", stats.total_interactions);
            println!("  Successful: {}", stats.successful_interactions);
            println!("  Success rate: {:.1}%", stats.success_rate());
            println!("  Avg response time: {:.1}ms", stats.avg_response_time_ms);
        }
        Err(e) => println!("❌ Failed to get approval stats: {}", e),
    }
    
    // Get interrupt statistics
    match interrupt_manager.get_stats() {
        Ok(stats) => {
            println!("\nInterrupt Statistics:");
            println!("  Total interrupts: {}", stats.total_interrupts);
            println!("  Active: {}", stats.active_interrupts);
            println!("  Resumed: {}", stats.resumed_interrupts);
            println!("  Cancelled: {}", stats.cancelled_interrupts);
        }
        Err(e) => println!("❌ Failed to get interrupt stats: {}", e),
    }
    
    println!("\n🎉 Human-in-the-Loop Demo Complete!");
    println!("===================================");
    println!("The AgentGraph Human-in-the-Loop system provides:");
    println!("✅ Interactive input collection with validation");
    println!("✅ Approval workflows with risk-based policies");
    println!("✅ Interrupt/resume capabilities for long-running processes");
    println!("✅ Comprehensive statistics and monitoring");
    println!("✅ Type-safe interfaces with async support");
    println!("✅ Extensible architecture for custom interaction providers");
    
    Ok(())
}
