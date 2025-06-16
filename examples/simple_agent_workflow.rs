// Simple Agent-Graph Workflow Example
// Demonstrates the integration of AI agents into graph workflows
// This shows how AgentGraph compares to LangGraph's agent orchestration

use agent_graph::{
    agents::{Agent, roles::RoleTemplates},
    graph::{GraphBuilder, engine::GraphEngine},
    edge::Edge,
    llm::{LLMManager, LLMConfig, providers::MockProvider},
    tools::{ToolRegistry, ToolExecutor},
    error::GraphResult,
    node::Node,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Simple workflow state that implements the original State trait
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowState {
    /// Input text to process
    pub input: String,
    /// Processed output
    pub output: String,
    /// Current stage of processing
    pub stage: String,
    /// Processing results
    pub results: Vec<String>,
}

impl Default for WorkflowState {
    fn default() -> Self {
        Self {
            input: String::new(),
            output: String::new(),
            stage: "start".to_string(),
            results: Vec::new(),
        }
    }
}

/// Agent wrapper node that executes an AI agent as part of a workflow
#[derive(Debug)]
pub struct SimpleAgentNode {
    /// The AI agent to execute
    agent: Arc<tokio::sync::Mutex<Agent>>,
    /// Task template
    task_template: String,
    /// Stage name for this node
    stage_name: String,
}

impl SimpleAgentNode {
    pub fn new(agent: Agent, task_template: String, stage_name: String) -> Self {
        Self {
            agent: Arc::new(tokio::sync::Mutex::new(agent)),
            task_template,
            stage_name,
        }
    }
}

#[async_trait]
impl Node<WorkflowState> for SimpleAgentNode {
    async fn invoke(&self, state: &mut WorkflowState) -> GraphResult<()> {
        println!("🤖 Executing {} agent...", self.stage_name);
        
        // Update stage
        state.stage = self.stage_name.clone();
        
        // Build task from template
        let task = self.task_template.replace("{input}", &state.input);
        
        // Execute agent
        let mut agent = self.agent.lock().await;
        let response = agent.execute_task(task).await
            .map_err(|e| agent_graph::error::GraphError::node_error(
                self.stage_name.clone(),
                format!("Agent execution failed: {}", e),
                Some(Box::new(e)),
            ))?;
        
        // Update state
        state.output = response.clone();
        state.results.push(format!("{}: {}", self.stage_name, response));
        
        println!("✅ {} completed", self.stage_name);
        Ok(())
    }
}

/// Quality check node
#[derive(Debug)]
pub struct QualityCheckNode {
    threshold: usize,
}

impl QualityCheckNode {
    pub fn new(threshold: usize) -> Self {
        Self { threshold }
    }
}

#[async_trait]
impl Node<WorkflowState> for QualityCheckNode {
    async fn invoke(&self, state: &mut WorkflowState) -> GraphResult<()> {
        println!("🔍 Performing quality check...");
        
        state.stage = "quality_check".to_string();
        
        // Simple quality check based on output length
        let quality_score = state.output.len();
        let passed = quality_score >= self.threshold;
        
        let result = if passed {
            format!("Quality check PASSED (score: {})", quality_score)
        } else {
            format!("Quality check FAILED (score: {}, threshold: {})", quality_score, self.threshold)
        };
        
        state.results.push(result.clone());
        println!("📊 {}", result);
        
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Simple Agent-Graph Workflow Example");
    println!("=====================================");
    println!("This demonstrates LangGraph-style agent orchestration in Rust!\n");

    // Step 1: Setup Infrastructure
    println!("🔧 Setting up infrastructure...");
    let llm_config = LLMConfig::default();
    let mut llm_manager = LLMManager::new(llm_config);
    let mock_provider = MockProvider::new();
    llm_manager.register_provider("mock".to_string(), Arc::new(mock_provider));
    let llm_manager = Arc::new(llm_manager);
    
    let tool_registry = Arc::new(ToolRegistry::new());
    let tool_executor = Arc::new(ToolExecutor::new());

    // Step 2: Create Specialized Agents
    println!("👥 Creating specialized agents...");
    
    // Analyst Agent
    let analyst_template = RoleTemplates::research_analyst();
    let analyst_config = analyst_template.to_agent_config("Analyst".to_string(), "mock".to_string());
    let analyst_agent = Agent::new(
        analyst_config,
        Arc::clone(&llm_manager),
        Arc::clone(&tool_registry),
        Arc::clone(&tool_executor),
    )?;

    // Writer Agent
    let writer_template = RoleTemplates::content_creator();
    let writer_config = writer_template.to_agent_config("Writer".to_string(), "mock".to_string());
    let writer_agent = Agent::new(
        writer_config,
        Arc::clone(&llm_manager),
        Arc::clone(&tool_registry),
        Arc::clone(&tool_executor),
    )?;

    // Reviewer Agent
    let reviewer_template = RoleTemplates::qa_engineer();
    let reviewer_config = reviewer_template.to_agent_config("Reviewer".to_string(), "mock".to_string());
    let reviewer_agent = Agent::new(
        reviewer_config,
        Arc::clone(&llm_manager),
        Arc::clone(&tool_registry),
        Arc::clone(&tool_executor),
    )?;

    // Step 3: Create Agent Nodes
    println!("🔗 Creating agent workflow nodes...");
    
    let analyst_node = SimpleAgentNode::new(
        analyst_agent,
        "Analyze this topic and provide key insights: {input}".to_string(),
        "analysis".to_string(),
    );

    let writer_node = SimpleAgentNode::new(
        writer_agent,
        "Write a comprehensive article about: {input}".to_string(),
        "writing".to_string(),
    );

    let reviewer_node = SimpleAgentNode::new(
        reviewer_agent,
        "Review and improve this content: {input}".to_string(),
        "review".to_string(),
    );

    let quality_check = QualityCheckNode::new(50); // Minimum 50 characters

    // Step 4: Build Workflow Graph
    println!("🏗️ Building workflow graph...");
    
    let graph = GraphBuilder::new()
        .add_node("analyst".to_string(), analyst_node)?
        .add_node("writer".to_string(), writer_node)?
        .add_node("reviewer".to_string(), reviewer_node)?
        .add_node("quality_check".to_string(), quality_check)?
        
        // Define workflow: analyst → writer → reviewer → quality_check
        .add_edge(Edge::simple("analyst", "writer"))?
        .add_edge(Edge::simple("writer", "reviewer"))?
        .add_edge(Edge::simple("reviewer", "quality_check"))?
        
        .with_entry_point("analyst".to_string())?
        .add_finish_point("quality_check".to_string())?
        .build()?;

    println!("✅ Workflow graph created with {} nodes", graph.node_ids().len());

    // Step 5: Execute Workflow
    println!("\n🚀 Executing content creation workflow...");
    
    let mut state = WorkflowState {
        input: "The future of artificial intelligence in software development".to_string(),
        ..Default::default()
    };

    println!("📝 Input Topic: {}", state.input);

    // Execute the graph
    let mut engine = GraphEngine::new();
    let execution_context = engine.execute(&graph, &mut state).await?;

    // Step 6: Display Results
    println!("\n📊 Workflow Execution Results:");
    println!("==============================");
    println!("Execution ID: {}", execution_context.execution_id);
    println!("Duration: {}ms", execution_context.duration_ms());
    println!("Steps: {}", execution_context.current_step);
    println!("Path: {:?}", execution_context.execution_path);

    println!("\n📝 Final State:");
    println!("Current Stage: {}", state.stage);
    println!("Final Output: {}", if state.output.len() > 200 { 
        format!("{}...", &state.output[..200]) 
    } else { 
        state.output.clone() 
    });

    println!("\n📋 Processing Results:");
    for (i, result) in state.results.iter().enumerate() {
        println!("{}. {}", i + 1, result);
    }

    println!("\n✅ Agent-Graph workflow completed successfully!");
    println!("🎯 This demonstrates how AgentGraph provides LangGraph-style functionality in Rust!");
    println!("\n🔍 Key Features Demonstrated:");
    println!("   • AI agents as workflow nodes");
    println!("   • Sequential agent execution");
    println!("   • State management across agents");
    println!("   • Graph-based workflow orchestration");
    println!("   • Execution context and monitoring");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_state() {
        let mut state = WorkflowState::default();
        state.input = "test input".to_string();
        state.output = "test output".to_string();
        state.stage = "testing".to_string();
        state.results.push("test result".to_string());

        assert_eq!(state.input, "test input");
        assert_eq!(state.output, "test output");
        assert_eq!(state.stage, "testing");
        assert_eq!(state.results.len(), 1);
    }

    #[tokio::test]
    async fn test_quality_check_node() {
        let quality_check = QualityCheckNode::new(10);
        let mut state = WorkflowState {
            output: "This is a test output that should pass".to_string(),
            ..Default::default()
        };

        quality_check.invoke(&mut state).await.unwrap();
        
        assert_eq!(state.stage, "quality_check");
        assert!(!state.results.is_empty());
        assert!(state.results[0].contains("PASSED"));
    }
}
