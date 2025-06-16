// Working Agent-Graph Workflow Example
// This demonstrates the integration of AI agents into graph workflows
// Similar to LangGraph's agent orchestration but in Rust

use agent_graph::{
    agents::{Agent, roles::RoleTemplates},
    llm::{LLMManager, LLMConfig, providers::MockProvider},
    tools::{ToolRegistry, ToolExecutor},
    error::GraphResult,
    node::{Node, NodeMetadata},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;

/// Simple workflow state for content creation pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentState {
    /// Input topic or request
    pub input: String,
    /// Research findings
    pub research: String,
    /// Written content
    pub content: String,
    /// Review feedback
    pub review: String,
    /// Final output
    pub output: String,
    /// Current stage
    pub stage: String,
    /// Quality metrics
    pub quality_score: u32,
}

impl Default for ContentState {
    fn default() -> Self {
        Self {
            input: String::new(),
            research: String::new(),
            content: String::new(),
            review: String::new(),
            output: String::new(),
            stage: "start".to_string(),
            quality_score: 0,
        }
    }
}

/// Agent wrapper node that executes an AI agent as part of a workflow
#[derive(Debug)]
pub struct AgentWorkflowNode {
    /// The AI agent to execute
    agent: Arc<tokio::sync::Mutex<Agent>>,
    /// Task template with placeholders
    task_template: String,
    /// Stage name for this node
    stage_name: String,
    /// Output field to update
    output_field: String,
}

impl AgentWorkflowNode {
    pub fn new(
        agent: Agent, 
        task_template: String, 
        stage_name: String,
        output_field: String
    ) -> Self {
        Self {
            agent: Arc::new(tokio::sync::Mutex::new(agent)),
            task_template,
            stage_name,
            output_field,
        }
    }

    /// Build task from template and state
    fn build_task(&self, state: &ContentState) -> String {
        self.task_template
            .replace("{input}", &state.input)
            .replace("{research}", &state.research)
            .replace("{content}", &state.content)
            .replace("{review}", &state.review)
    }

    /// Update state with agent response
    fn update_state(&self, state: &mut ContentState, response: String) {
        match self.output_field.as_str() {
            "research" => state.research = response,
            "content" => state.content = response,
            "review" => state.review = response,
            "output" => state.output = response,
            _ => state.output = response,
        }
        state.stage = self.stage_name.clone();
    }
}

#[async_trait]
impl Node<ContentState> for AgentWorkflowNode {
    async fn invoke(&self, state: &mut ContentState) -> GraphResult<()> {
        println!("🤖 Executing {} agent...", self.stage_name);
        
        // Build task from template and current state
        let task = self.build_task(state);
        println!("📝 Task: {}", task);
        
        // Execute agent
        let mut agent = self.agent.lock().await;
        let response = agent.execute_task(task).await
            .map_err(|e| agent_graph::error::GraphError::node_error(
                self.stage_name.clone(),
                format!("Agent execution failed: {}", e),
                Some(Box::new(e)),
            ))?;
        
        // Update state with response
        self.update_state(state, response);
        
        println!("✅ {} completed: {}", self.stage_name, 
                 if state.output.len() > 100 { 
                     format!("{}...", &state.output[..100]) 
                 } else { 
                     state.output.clone() 
                 });
        
        Ok(())
    }

    fn metadata(&self) -> NodeMetadata {
        NodeMetadata::new(&self.stage_name)
            .with_description(&format!("AI agent node for {}", self.stage_name))
            .with_tag("agent")
            .with_tag(&self.stage_name)
            .with_parallel_safe(false) // Agents should run sequentially for this example
    }
}

/// Quality gate node that evaluates content quality
#[derive(Debug)]
pub struct QualityGateNode {
    threshold: u32,
}

impl QualityGateNode {
    pub fn new(threshold: u32) -> Self {
        Self { threshold }
    }

    fn calculate_quality_score(&self, state: &ContentState) -> u32 {
        let mut score = 0u32;
        
        // Score based on content length
        if !state.content.is_empty() {
            score += 20;
            if state.content.len() > 200 {
                score += 15;
            }
        }
        
        // Score based on research quality
        if !state.research.is_empty() {
            score += 20;
            if state.research.contains("analysis") || state.research.contains("research") {
                score += 10;
            }
        }
        
        // Score based on review feedback
        if !state.review.is_empty() {
            score += 20;
            if state.review.contains("good") || state.review.contains("excellent") {
                score += 15;
            }
        }
        
        // Bonus for comprehensive content
        if state.content.len() > 500 && state.research.len() > 200 {
            score += 10;
        }
        
        score
    }
}

#[async_trait]
impl Node<ContentState> for QualityGateNode {
    async fn invoke(&self, state: &mut ContentState) -> GraphResult<()> {
        println!("🔍 Performing quality assessment...");
        
        let score = self.calculate_quality_score(state);
        state.quality_score = score;
        state.stage = "quality_check".to_string();
        
        let passed = score >= self.threshold;
        
        println!("📊 Quality Score: {}/100 (threshold: {})", score, self.threshold);
        
        if passed {
            println!("✅ Quality gate PASSED");
            state.output = state.content.clone(); // Approve the content
        } else {
            println!("❌ Quality gate FAILED - content needs improvement");
            // In a real system, this might trigger a retry or revision workflow
        }
        
        Ok(())
    }

    fn metadata(&self) -> NodeMetadata {
        NodeMetadata::new("QualityGate")
            .with_description("Evaluates content quality against threshold")
            .with_tag("quality")
            .with_tag("gate")
    }
}

/// Simple sequential workflow executor
pub struct SimpleWorkflow {
    nodes: Vec<Box<dyn Node<ContentState>>>,
}

impl SimpleWorkflow {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
        }
    }

    pub fn add_node<N: Node<ContentState> + 'static>(mut self, node: N) -> Self {
        self.nodes.push(Box::new(node));
        self
    }

    pub async fn execute(&self, state: &mut ContentState) -> GraphResult<()> {
        println!("🚀 Starting workflow execution...");
        
        for (i, node) in self.nodes.iter().enumerate() {
            println!("\n📍 Step {}: {}", i + 1, node.metadata().name);
            node.invoke(state).await?;
        }
        
        println!("\n✅ Workflow completed successfully!");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 AgentGraph Workflow Example: Content Creation Pipeline");
    println!("=========================================================");
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
    
    // Research Agent
    let researcher_template = RoleTemplates::research_analyst();
    let researcher_config = researcher_template.to_agent_config("Researcher".to_string(), "mock".to_string());
    let researcher_agent = Agent::new(
        researcher_config,
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

    // Step 3: Create Workflow Nodes
    println!("🔗 Creating workflow nodes...");
    
    let research_node = AgentWorkflowNode::new(
        researcher_agent,
        "Research this topic thoroughly and provide key insights: {input}".to_string(),
        "research".to_string(),
        "research".to_string(),
    );

    let writing_node = AgentWorkflowNode::new(
        writer_agent,
        "Write a comprehensive article about '{input}' based on this research: {research}".to_string(),
        "writing".to_string(),
        "content".to_string(),
    );

    let review_node = AgentWorkflowNode::new(
        reviewer_agent,
        "Review and provide feedback on this content: {content}".to_string(),
        "review".to_string(),
        "review".to_string(),
    );

    let quality_gate = QualityGateNode::new(60); // 60% quality threshold

    // Step 4: Build and Execute Workflow
    println!("🏗️ Building workflow...");
    
    let workflow = SimpleWorkflow::new()
        .add_node(research_node)
        .add_node(writing_node)
        .add_node(review_node)
        .add_node(quality_gate);

    // Step 5: Execute with Sample Input
    println!("\n🚀 Executing content creation workflow...");
    
    let mut state = ContentState {
        input: "The impact of artificial intelligence on modern software development practices".to_string(),
        ..Default::default()
    };

    println!("📝 Input Topic: {}", state.input);

    // Execute the workflow
    workflow.execute(&mut state).await?;

    // Step 6: Display Results
    println!("\n📊 Workflow Results:");
    println!("====================");
    println!("Final Stage: {}", state.stage);
    println!("Quality Score: {}/100", state.quality_score);
    
    println!("\n🔬 Research Findings:");
    println!("{}", if state.research.len() > 300 { 
        format!("{}...", &state.research[..300]) 
    } else { 
        state.research.clone() 
    });
    
    println!("\n📝 Generated Content:");
    println!("{}", if state.content.len() > 300 { 
        format!("{}...", &state.content[..300]) 
    } else { 
        state.content.clone() 
    });
    
    println!("\n📋 Review Feedback:");
    println!("{}", if state.review.len() > 300 { 
        format!("{}...", &state.review[..300]) 
    } else { 
        state.review.clone() 
    });

    println!("\n✅ Agent-Graph workflow completed successfully!");
    println!("\n🎯 Key Features Demonstrated:");
    println!("   • AI agents as workflow nodes");
    println!("   • Sequential agent execution with state passing");
    println!("   • Template-based task generation");
    println!("   • Quality gates and evaluation");
    println!("   • LangGraph-style workflow orchestration in Rust");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_state() {
        let mut state = ContentState::default();
        state.input = "test topic".to_string();
        state.research = "test research".to_string();
        state.content = "test content".to_string();

        assert_eq!(state.input, "test topic");
        assert_eq!(state.research, "test research");
        assert_eq!(state.content, "test content");
    }

    #[tokio::test]
    async fn test_quality_gate() {
        let quality_gate = QualityGateNode::new(50);
        let mut state = ContentState {
            content: "This is a comprehensive article with detailed analysis and research-backed insights that demonstrates high quality content creation.".to_string(),
            research: "Thorough research and analysis of the topic with multiple sources and comprehensive coverage.".to_string(),
            review: "Excellent work with good structure and comprehensive coverage.".to_string(),
            ..Default::default()
        };

        quality_gate.invoke(&mut state).await.unwrap();
        
        assert!(state.quality_score >= 50);
        assert_eq!(state.stage, "quality_check");
    }
}
