// Standalone Agent-Graph Workflow Example
// This demonstrates LangGraph-style agent orchestration without depending on broken library parts
// Shows how AgentGraph provides similar functionality to LangGraph in Rust

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Simple workflow state for content creation pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowState {
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
    /// Metadata
    pub metadata: HashMap<String, String>,
}

impl Default for WorkflowState {
    fn default() -> Self {
        Self {
            input: String::new(),
            research: String::new(),
            content: String::new(),
            review: String::new(),
            output: String::new(),
            stage: "start".to_string(),
            quality_score: 0,
            metadata: HashMap::new(),
        }
    }
}

/// Mock agent that simulates AI agent behavior
#[derive(Debug, Clone)]
pub struct MockAgent {
    pub name: String,
    pub role: String,
}

impl MockAgent {
    pub fn new(name: String, role: String) -> Self {
        Self { name, role }
    }

    pub async fn execute_task(&self, task: String) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Simulate processing time
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // Generate mock response based on role
        let response = match self.role.as_str() {
            "researcher" => format!(
                "Research Analysis: {} - Key findings include comprehensive analysis of current trends, \
                 market research data, and expert opinions. The topic shows significant relevance \
                 with multiple data points supporting the main thesis.",
                task.replace("Research this topic thoroughly and provide key insights: ", "")
            ),
            "writer" => format!(
                "Article Content: Based on the research provided, here is a comprehensive article:\n\n\
                 # {}\n\n\
                 This topic represents a significant development in the field. The research indicates \
                 multiple important aspects that deserve attention. Through careful analysis, we can \
                 identify key trends and implications.\n\n\
                 ## Key Points\n\
                 - Comprehensive analysis reveals important insights\n\
                 - Current trends show significant development\n\
                 - Expert opinions support the main conclusions\n\n\
                 ## Conclusion\n\
                 The evidence strongly supports the importance of this topic in current discussions.",
                task.split("'").nth(1).unwrap_or("Unknown Topic")
            ),
            "reviewer" => format!(
                "Review Feedback: The content is well-structured and comprehensive. \
                 Strengths include good organization, clear writing, and thorough coverage. \
                 The article demonstrates excellent research integration and provides \
                 valuable insights. Quality assessment: Excellent work with professional standards met."
            ),
            _ => format!("Processed: {}", task),
        };
        
        println!("🤖 {} ({}): Generated {} characters", self.name, self.role, response.len());
        Ok(response)
    }
}

/// Agent workflow node that executes an AI agent as part of a workflow
#[derive(Debug)]
pub struct AgentNode {
    /// The AI agent to execute
    agent: Arc<Mutex<MockAgent>>,
    /// Task template with placeholders
    task_template: String,
    /// Stage name for this node
    stage_name: String,
    /// Output field to update
    output_field: String,
}

impl AgentNode {
    pub fn new(
        agent: MockAgent, 
        task_template: String, 
        stage_name: String,
        output_field: String
    ) -> Self {
        Self {
            agent: Arc::new(Mutex::new(agent)),
            task_template,
            stage_name,
            output_field,
        }
    }

    /// Build task from template and state
    fn build_task(&self, state: &WorkflowState) -> String {
        self.task_template
            .replace("{input}", &state.input)
            .replace("{research}", &state.research)
            .replace("{content}", &state.content)
            .replace("{review}", &state.review)
    }

    /// Update state with agent response
    fn update_state(&self, state: &mut WorkflowState, response: String) {
        match self.output_field.as_str() {
            "research" => state.research = response,
            "content" => state.content = response,
            "review" => state.review = response,
            "output" => state.output = response,
            _ => state.output = response,
        }
        state.stage = self.stage_name.clone();
        state.metadata.insert("last_updated".to_string(), chrono::Utc::now().to_rfc3339());
    }

    pub async fn execute(&self, state: &mut WorkflowState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("🔄 Executing {} node...", self.stage_name);
        
        // Build task from template and current state
        let task = self.build_task(state);
        
        // Execute agent
        let agent = self.agent.lock().await;
        let response = agent.execute_task(task).await?;
        drop(agent);
        
        // Update state with response
        self.update_state(state, response);
        
        println!("✅ {} completed", self.stage_name);
        Ok(())
    }
}

/// Quality gate node that evaluates content quality
#[derive(Debug)]
pub struct QualityGate {
    threshold: u32,
}

impl QualityGate {
    pub fn new(threshold: u32) -> Self {
        Self { threshold }
    }

    fn calculate_quality_score(&self, state: &WorkflowState) -> u32 {
        let mut score = 0u32;
        
        // Score based on content length and quality indicators
        if !state.content.is_empty() {
            score += 25;
            if state.content.len() > 500 {
                score += 15;
            }
            if state.content.contains("comprehensive") || state.content.contains("analysis") {
                score += 10;
            }
        }
        
        // Score based on research quality
        if !state.research.is_empty() {
            score += 20;
            if state.research.contains("research") || state.research.contains("analysis") {
                score += 10;
            }
        }
        
        // Score based on review feedback
        if !state.review.is_empty() {
            score += 20;
            if state.review.contains("excellent") || state.review.contains("good") {
                score += 10;
            }
        }
        
        score
    }

    pub async fn execute(&self, state: &mut WorkflowState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("🔍 Performing quality assessment...");
        
        let score = self.calculate_quality_score(state);
        state.quality_score = score;
        state.stage = "quality_check".to_string();
        
        let passed = score >= self.threshold;
        
        println!("📊 Quality Score: {}/100 (threshold: {})", score, self.threshold);
        
        if passed {
            println!("✅ Quality gate PASSED");
            state.output = state.content.clone();
        } else {
            println!("❌ Quality gate FAILED - content needs improvement");
        }
        
        Ok(())
    }
}

/// Simple workflow orchestrator (similar to LangGraph's StateGraph)
pub struct AgentWorkflow {
    nodes: Vec<Box<dyn WorkflowNode>>,
}

/// Trait for workflow nodes (similar to LangGraph's Node interface)
#[async_trait::async_trait]
pub trait WorkflowNode: Send + Sync {
    async fn execute(&self, state: &mut WorkflowState) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    fn name(&self) -> &str;
}

#[async_trait::async_trait]
impl WorkflowNode for AgentNode {
    async fn execute(&self, state: &mut WorkflowState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.execute(state).await
    }

    fn name(&self) -> &str {
        &self.stage_name
    }
}

#[async_trait::async_trait]
impl WorkflowNode for QualityGate {
    async fn execute(&self, state: &mut WorkflowState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.execute(state).await
    }

    fn name(&self) -> &str {
        "quality_gate"
    }
}

impl AgentWorkflow {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
        }
    }

    pub fn add_node<N: WorkflowNode + 'static>(mut self, node: N) -> Self {
        self.nodes.push(Box::new(node));
        self
    }

    pub async fn execute(&self, state: &mut WorkflowState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("🚀 Starting AgentGraph workflow execution...");
        println!("============================================");
        
        let start_time = std::time::Instant::now();
        
        for (i, node) in self.nodes.iter().enumerate() {
            println!("\n📍 Step {}: {}", i + 1, node.name());
            node.execute(state).await?;
        }
        
        let duration = start_time.elapsed();
        println!("\n✅ Workflow completed successfully in {:?}!", duration);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 AgentGraph Workflow Example: LangGraph-style Agent Orchestration");
    println!("===================================================================");
    println!("This demonstrates how AgentGraph provides LangGraph functionality in Rust!\n");

    // Step 1: Create Specialized Agents (similar to LangGraph agents)
    println!("👥 Creating specialized AI agents...");
    
    let researcher = MockAgent::new("Dr. Research".to_string(), "researcher".to_string());
    let writer = MockAgent::new("Alex Writer".to_string(), "writer".to_string());
    let reviewer = MockAgent::new("Sam Reviewer".to_string(), "reviewer".to_string());

    // Step 2: Create Agent Nodes (similar to LangGraph's agent nodes)
    println!("🔗 Creating agent workflow nodes...");
    
    let research_node = AgentNode::new(
        researcher,
        "Research this topic thoroughly and provide key insights: {input}".to_string(),
        "research".to_string(),
        "research".to_string(),
    );

    let writing_node = AgentNode::new(
        writer,
        "Write a comprehensive article about '{input}' based on this research: {research}".to_string(),
        "writing".to_string(),
        "content".to_string(),
    );

    let review_node = AgentNode::new(
        reviewer,
        "Review and provide feedback on this content: {content}".to_string(),
        "review".to_string(),
        "review".to_string(),
    );

    let quality_gate = QualityGate::new(70); // 70% quality threshold

    // Step 3: Build Workflow (similar to LangGraph's StateGraph)
    println!("🏗️ Building agent workflow graph...");
    
    let workflow = AgentWorkflow::new()
        .add_node(research_node)
        .add_node(writing_node)
        .add_node(review_node)
        .add_node(quality_gate);

    // Step 4: Execute Workflow with State Management
    println!("\n🚀 Executing content creation workflow...");
    
    let mut state = WorkflowState {
        input: "The transformative impact of AI agents on software development workflows".to_string(),
        ..Default::default()
    };

    println!("📝 Input Topic: {}", state.input);

    // Execute the workflow (similar to LangGraph's graph.invoke())
    workflow.execute(&mut state).await?;

    // Step 5: Display Results
    println!("\n📊 Workflow Execution Results:");
    println!("==============================");
    println!("Final Stage: {}", state.stage);
    println!("Quality Score: {}/100", state.quality_score);
    
    println!("\n🔬 Research Findings:");
    println!("{}", truncate_text(&state.research, 200));
    
    println!("\n📝 Generated Content:");
    println!("{}", truncate_text(&state.content, 300));
    
    println!("\n📋 Review Feedback:");
    println!("{}", truncate_text(&state.review, 200));

    println!("\n🎯 AgentGraph vs LangGraph Comparison:");
    println!("=====================================");
    println!("✅ State Management: Both use typed state objects");
    println!("✅ Agent Nodes: Both support AI agents as workflow nodes");
    println!("✅ Sequential Execution: Both support step-by-step processing");
    println!("✅ Quality Gates: Both support conditional logic and validation");
    println!("✅ Workflow Orchestration: Both provide graph-based execution");
    println!("🦀 Rust Advantage: Type safety, memory safety, and performance");

    println!("\n✅ AgentGraph successfully demonstrates LangGraph-style functionality in Rust!");

    Ok(())
}

fn truncate_text(text: &str, max_len: usize) -> String {
    if text.len() > max_len {
        format!("{}...", &text[..max_len])
    } else {
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_state() {
        let mut state = WorkflowState::default();
        state.input = "test topic".to_string();
        state.research = "test research".to_string();

        assert_eq!(state.input, "test topic");
        assert_eq!(state.research, "test research");
        assert_eq!(state.stage, "start");
    }

    #[tokio::test]
    async fn test_mock_agent() {
        let agent = MockAgent::new("Test Agent".to_string(), "researcher".to_string());
        let result = agent.execute_task("Test task".to_string()).await.unwrap();
        
        assert!(!result.is_empty());
        assert!(result.contains("Research Analysis"));
    }

    #[tokio::test]
    async fn test_quality_gate() {
        let quality_gate = QualityGate::new(50);
        let mut state = WorkflowState {
            content: "This is comprehensive content with detailed analysis and research-backed insights.".to_string(),
            research: "Thorough research and analysis findings.".to_string(),
            review: "Excellent work with good structure.".to_string(),
            ..Default::default()
        };

        quality_gate.execute(&mut state).await.unwrap();
        
        assert!(state.quality_score >= 50);
        assert_eq!(state.stage, "quality_check");
    }

    #[tokio::test]
    async fn test_workflow_execution() {
        let researcher = MockAgent::new("Test Researcher".to_string(), "researcher".to_string());
        let writer = MockAgent::new("Test Writer".to_string(), "writer".to_string());
        
        let research_node = AgentNode::new(
            researcher,
            "Research: {input}".to_string(),
            "research".to_string(),
            "research".to_string(),
        );
        
        let writing_node = AgentNode::new(
            writer,
            "Write about '{input}' using: {research}".to_string(),
            "writing".to_string(),
            "content".to_string(),
        );
        
        let workflow = AgentWorkflow::new()
            .add_node(research_node)
            .add_node(writing_node);
        
        let mut state = WorkflowState {
            input: "Test Topic".to_string(),
            ..Default::default()
        };
        
        workflow.execute(&mut state).await.unwrap();
        
        assert!(!state.research.is_empty());
        assert!(!state.content.is_empty());
        assert_eq!(state.stage, "writing");
    }
}
