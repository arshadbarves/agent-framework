//! Complete workflow example demonstrating the new AgentGraph architecture.
//! 
//! This example shows how to use the modular crate structure to build
//! a sophisticated multi-agent system with LLM integration, tools,
//! human-in-the-loop workflows, and enterprise features.

use agent_graph::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio;

/// Example state for our workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkflowState {
    /// Current task being processed
    current_task: String,
    /// Results from each step
    results: HashMap<String, serde_json::Value>,
    /// Workflow status
    status: String,
    /// Error messages if any
    errors: Vec<String>,
}

impl State for WorkflowState {}

impl Default for WorkflowState {
    fn default() -> Self {
        Self {
            current_task: "start".to_string(),
            results: HashMap::new(),
            status: "initialized".to_string(),
            errors: Vec::new(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the framework
    agent_graph::init();
    
    println!("🚀 AgentGraph Complete Workflow Example");
    println!("======================================");

    // 1. Set up LLM client with mock provider
    #[cfg(feature = "llm")]
    let llm_client = {
        use agent_graph::llm::{LLMClientBuilder, MockProvider};
        use std::sync::Arc;

        LLMClientBuilder::new()
            .with_provider("mock".to_string(), Arc::new(MockProvider::new()))
            .with_default_provider("mock".to_string())
            .build()
            .await?
    };

    // 2. Set up tool registry with built-in tools
    #[cfg(feature = "tools")]
    let tool_registry = {
        use agent_graph::tools::builtin::create_builtin_registry;
        create_builtin_registry()?
    };

    // 3. Set up human input system
    #[cfg(feature = "human")]
    let mut input_manager = {
        use agent_graph::human::{InputManager, ConsoleInputCollector};
        let mut manager = InputManager::new();
        manager.register_collector(Box::new(ConsoleInputCollector::new()));
        manager
    };

    // 4. Create agents with different roles
    #[cfg(feature = "agents")]
    let agents = {
        use agent_graph::agents::{AgentBuilder, AgentRole};
        use agent_graph::agents::memory::MemoryConfig;

        let researcher = AgentBuilder::new("researcher".to_string())
            .with_role(AgentRole::Researcher)
            .with_description("Research and information gathering agent".to_string())
            .with_capability("web_search".to_string())
            .with_capability("data_analysis".to_string())
            .with_memory_config(MemoryConfig::default())
            .build()?;

        let analyst = AgentBuilder::new("analyst".to_string())
            .with_role(AgentRole::Analyst)
            .with_description("Data analysis and insights agent".to_string())
            .with_capability("statistical_analysis".to_string())
            .with_capability("report_generation".to_string())
            .with_memory_config(MemoryConfig::default())
            .build()?;

        vec![researcher, analyst]
    };

    // 5. Set up agent runtime
    #[cfg(feature = "agents")]
    let mut agent_runtime = {
        use agent_graph::agents::AgentRuntime;
        use agent_graph::agents::runtime::RuntimeConfig;

        let config = RuntimeConfig::default();
        let mut runtime = AgentRuntime::new(config);

        // Register agents
        for agent in agents {
            runtime.register_agent(agent).await?;
        }

        runtime
    };

    // 6. Create the main graph
    let mut graph = GraphBuilder::new()
        .with_name("Complete Workflow".to_string())
        .with_description("Demonstrates all AgentGraph features".to_string())
        .build()?;

    // 7. Create custom nodes for the workflow
    let research_node = ResearchNode::new();
    let analysis_node = AnalysisNode::new();
    let approval_node = ApprovalNode::new();
    let completion_node = CompletionNode::new();

    // 8. Add nodes to graph
    graph.add_node("research".to_string(), Box::new(research_node))?;
    graph.add_node("analysis".to_string(), Box::new(analysis_node))?;
    graph.add_node("approval".to_string(), Box::new(approval_node))?;
    graph.add_node("completion".to_string(), Box::new(completion_node))?;

    // 9. Add edges
    graph.add_edge("research".to_string(), "analysis".to_string())?;
    graph.add_edge("analysis".to_string(), "approval".to_string())?;
    graph.add_edge("approval".to_string(), "completion".to_string())?;

    // 10. Set entry and exit points
    graph.set_entry_point("research".to_string())?;
    graph.add_finish_point("completion".to_string())?;

    // 11. Execute the workflow
    let mut state = WorkflowState::default();
    state.current_task = "Analyze market trends for Q4 2024".to_string();

    println!("\n📊 Starting workflow execution...");
    println!("Task: {}", state.current_task);

    let execution_result = graph.execute(&mut state).await;

    match execution_result {
        Ok(_) => {
            println!("\n✅ Workflow completed successfully!");
            println!("Final status: {}", state.status);
            println!("Results: {:#?}", state.results);
        }
        Err(e) => {
            println!("\n❌ Workflow failed: {}", e);
            println!("Errors: {:?}", state.errors);
        }
    }

    // 12. Display runtime metrics
    #[cfg(feature = "agents")]
    {
        let metrics = agent_runtime.metrics().await;
        println!("\n📈 Runtime Metrics:");
        println!("  Total agents: {}", metrics.active_agents);
        println!("  Successful executions: {}", metrics.successful_executions);
        println!("  Failed executions: {}", metrics.failed_executions);
        println!("  Total execution time: {}ms", metrics.total_execution_time_ms);
    }

    Ok(())
}

// Custom node implementations

/// Research node that simulates data gathering
#[derive(Debug)]
struct ResearchNode {
    metadata: NodeMetadata,
}

impl ResearchNode {
    fn new() -> Self {
        let metadata = NodeMetadata {
            name: "Research Node".to_string(),
            description: Some("Gathers research data for analysis".to_string()),
            version: "1.0.0".to_string(),
            parallel_safe: true,
            expected_duration_ms: Some(2000),
            tags: vec!["research".to_string(), "data".to_string()],
            custom_properties: HashMap::new(),
        };

        Self { metadata }
    }
}

#[async_trait::async_trait]
impl Node<WorkflowState> for ResearchNode {
    async fn execute(&self, state: &mut WorkflowState) -> CoreResult<NodeOutput> {
        println!("🔍 Executing research phase...");
        
        // Simulate research work
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        
        // Store research results
        state.results.insert(
            "research".to_string(),
            serde_json::json!({
                "data_sources": ["market_reports", "industry_analysis", "competitor_data"],
                "key_findings": ["trend_1", "trend_2", "trend_3"],
                "confidence": 0.85
            })
        );
        
        state.status = "research_completed".to_string();
        
        Ok(NodeOutput::success_with_data(serde_json::json!({
            "phase": "research",
            "status": "completed"
        })))
    }

    fn id(&self) -> &str {
        "research"
    }

    fn metadata(&self) -> &NodeMetadata {
        &self.metadata
    }
}

/// Analysis node that processes research data
#[derive(Debug)]
struct AnalysisNode {
    metadata: NodeMetadata,
}

impl AnalysisNode {
    fn new() -> Self {
        let metadata = NodeMetadata {
            name: "Analysis Node".to_string(),
            description: Some("Analyzes research data and generates insights".to_string()),
            version: "1.0.0".to_string(),
            parallel_safe: true,
            expected_duration_ms: Some(3000),
            tags: vec!["analysis".to_string(), "insights".to_string()],
            custom_properties: HashMap::new(),
        };

        Self { metadata }
    }
}

#[async_trait::async_trait]
impl Node<WorkflowState> for AnalysisNode {
    async fn execute(&self, state: &mut WorkflowState) -> CoreResult<NodeOutput> {
        println!("📊 Executing analysis phase...");
        
        // Check if research data is available
        if !state.results.contains_key("research") {
            return Err(CoreError::execution_error("Research data not available"));
        }
        
        // Simulate analysis work
        tokio::time::sleep(std::time::Duration::from_millis(800)).await;
        
        // Store analysis results
        state.results.insert(
            "analysis".to_string(),
            serde_json::json!({
                "insights": ["Market growth expected", "New opportunities identified"],
                "recommendations": ["Invest in sector A", "Monitor sector B"],
                "risk_assessment": "Medium",
                "confidence": 0.92
            })
        );
        
        state.status = "analysis_completed".to_string();
        
        Ok(NodeOutput::success_with_data(serde_json::json!({
            "phase": "analysis",
            "status": "completed"
        })))
    }

    fn id(&self) -> &str {
        "analysis"
    }

    fn metadata(&self) -> &NodeMetadata {
        &self.metadata
    }
}

/// Approval node that requests human approval
#[derive(Debug)]
struct ApprovalNode {
    metadata: NodeMetadata,
}

impl ApprovalNode {
    fn new() -> Self {
        let metadata = NodeMetadata {
            name: "Approval Node".to_string(),
            description: Some("Requests human approval for recommendations".to_string()),
            version: "1.0.0".to_string(),
            parallel_safe: false,
            expected_duration_ms: Some(30000), // 30 seconds for human response
            tags: vec!["approval".to_string(), "human".to_string()],
            custom_properties: HashMap::new(),
        };

        Self { metadata }
    }
}

#[async_trait::async_trait]
impl Node<WorkflowState> for ApprovalNode {
    async fn execute(&self, state: &mut WorkflowState) -> CoreResult<NodeOutput> {
        println!("✋ Requesting human approval...");
        
        // Check if analysis data is available
        if !state.results.contains_key("analysis") {
            return Err(CoreError::execution_error("Analysis data not available"));
        }
        
        // For this example, we'll simulate approval
        // In a real implementation, this would use the human input system
        println!("📋 Analysis complete. Recommendations ready for review.");
        println!("   - Market growth expected");
        println!("   - New opportunities identified");
        println!("   - Risk assessment: Medium");
        
        // Simulate human thinking time
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        
        // Auto-approve for demo purposes
        let approved = true;
        
        state.results.insert(
            "approval".to_string(),
            serde_json::json!({
                "approved": approved,
                "approver": "demo_user",
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "comments": "Recommendations look good, proceed with implementation"
            })
        );
        
        state.status = if approved { "approved" } else { "rejected" }.to_string();
        
        if approved {
            println!("✅ Recommendations approved!");
            Ok(NodeOutput::success_with_data(serde_json::json!({
                "phase": "approval",
                "status": "approved"
            })))
        } else {
            println!("❌ Recommendations rejected!");
            Ok(NodeOutput::stop())
        }
    }

    fn id(&self) -> &str {
        "approval"
    }

    fn metadata(&self) -> &NodeMetadata {
        &self.metadata
    }
}

/// Completion node that finalizes the workflow
#[derive(Debug)]
struct CompletionNode {
    metadata: NodeMetadata,
}

impl CompletionNode {
    fn new() -> Self {
        let metadata = NodeMetadata {
            name: "Completion Node".to_string(),
            description: Some("Finalizes the workflow and generates final report".to_string()),
            version: "1.0.0".to_string(),
            parallel_safe: true,
            expected_duration_ms: Some(1000),
            tags: vec!["completion".to_string(), "report".to_string()],
            custom_properties: HashMap::new(),
        };

        Self { metadata }
    }
}

#[async_trait::async_trait]
impl Node<WorkflowState> for CompletionNode {
    async fn execute(&self, state: &mut WorkflowState) -> CoreResult<NodeOutput> {
        println!("🎯 Finalizing workflow...");
        
        // Generate final report
        let report = serde_json::json!({
            "workflow_id": uuid::Uuid::new_v4().to_string(),
            "task": state.current_task,
            "completed_at": chrono::Utc::now().to_rfc3339(),
            "phases_completed": ["research", "analysis", "approval"],
            "final_status": "completed",
            "summary": "Market analysis completed successfully with approved recommendations"
        });
        
        state.results.insert("final_report".to_string(), report);
        state.status = "completed".to_string();
        
        println!("📄 Final report generated!");
        
        Ok(NodeOutput::success_with_data(serde_json::json!({
            "phase": "completion",
            "status": "completed"
        })))
    }

    fn id(&self) -> &str {
        "completion"
    }

    fn metadata(&self) -> &NodeMetadata {
        &self.metadata
    }
}