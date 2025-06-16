// Agent-Graph Workflow Example
// Demonstrates the integration of AI agents into graph workflows
// This is similar to LangGraph's agent-based workflow orchestration

use agent_graph::{
    agents::{Agent, roles::RoleTemplates},
    graph::{agent_node::AgentNode, Graph, GraphBuilder, ExecutionContext, engine::GraphEngine},
    edge::Edge,
    llm::{LLMManager, LLMConfig, providers::MockProvider},
    tools::{ToolRegistry, ToolExecutor},
    error::GraphResult,
    node::Node,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Workflow state for software development process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareDevState {
    /// Project requirements
    pub requirements: String,
    /// Generated code
    pub code: String,
    /// Test results
    pub test_results: String,
    /// Security audit results
    pub security_audit: String,
    /// Deployment status
    pub deployment_status: String,
    /// Current workflow stage
    pub stage: String,
    /// Quality score (0-100)
    pub quality_score: u32,
    /// Approval status
    pub approved: bool,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl agent_graph::state::State for SoftwareDevState {
    fn get_value(&self, key: &str) -> Option<serde_json::Value> {
        match key {
            "requirements" => Some(serde_json::Value::String(self.requirements.clone())),
            "code" => Some(serde_json::Value::String(self.code.clone())),
            "test_results" => Some(serde_json::Value::String(self.test_results.clone())),
            "security_audit" => Some(serde_json::Value::String(self.security_audit.clone())),
            "deployment_status" => Some(serde_json::Value::String(self.deployment_status.clone())),
            "stage" => Some(serde_json::Value::String(self.stage.clone())),
            "quality_score" => Some(serde_json::Value::Number(serde_json::Number::from(self.quality_score))),
            "approved" => Some(serde_json::Value::Bool(self.approved)),
            _ => self.metadata.get(key).cloned(),
        }
    }

    fn set_value(&mut self, key: &str, value: serde_json::Value) -> GraphResult<()> {
        match key {
            "requirements" => {
                if let serde_json::Value::String(s) = value {
                    self.requirements = s;
                }
            }
            "code" => {
                if let serde_json::Value::String(s) = value {
                    self.code = s;
                }
            }
            "test_results" => {
                if let serde_json::Value::String(s) = value {
                    self.test_results = s;
                }
            }
            "security_audit" => {
                if let serde_json::Value::String(s) = value {
                    self.security_audit = s;
                }
            }
            "deployment_status" => {
                if let serde_json::Value::String(s) = value {
                    self.deployment_status = s;
                }
            }
            "stage" => {
                if let serde_json::Value::String(s) = value {
                    self.stage = s;
                }
            }
            "quality_score" => {
                if let serde_json::Value::Number(n) = value {
                    if let Some(score) = n.as_u64() {
                        self.quality_score = score as u32;
                    }
                }
            }
            "approved" => {
                if let serde_json::Value::Bool(b) = value {
                    self.approved = b;
                }
            }
            "output" => {
                // Handle generic output mapping
                if let serde_json::Value::String(s) = value {
                    match self.stage.as_str() {
                        "development" => self.code = s,
                        "testing" => self.test_results = s,
                        "security" => self.security_audit = s,
                        "deployment" => self.deployment_status = s,
                        _ => {
                            self.metadata.insert("output".to_string(), serde_json::Value::String(s));
                        }
                    }
                }
            }
            _ => {
                self.metadata.insert(key.to_string(), value);
            }
        }
        Ok(())
    }

    fn clone_state(&self) -> Box<dyn agent_graph::state::State> {
        Box::new(self.clone())
    }

    fn to_json(&self) -> GraphResult<serde_json::Value> {
        serde_json::to_value(self).map_err(|e| {
            agent_graph::error::GraphError::state_error(format!("Failed to serialize state: {}", e))
        })
    }

    fn keys(&self) -> Vec<String> {
        vec![
            "requirements".to_string(),
            "code".to_string(),
            "test_results".to_string(),
            "security_audit".to_string(),
            "deployment_status".to_string(),
            "stage".to_string(),
            "quality_score".to_string(),
            "approved".to_string(),
        ]
    }
}

impl agent_graph::state::CloneableState for SoftwareDevState {}

impl Default for SoftwareDevState {
    fn default() -> Self {
        Self {
            requirements: String::new(),
            code: String::new(),
            test_results: String::new(),
            security_audit: String::new(),
            deployment_status: String::new(),
            stage: "requirements".to_string(),
            quality_score: 0,
            approved: false,
            metadata: HashMap::new(),
        }
    }
}

/// Quality gate node that checks if quality score meets threshold
#[derive(Debug)]
pub struct QualityGateNode {
    threshold: u32,
}

impl QualityGateNode {
    pub fn new(threshold: u32) -> Self {
        Self { threshold }
    }
}

#[async_trait]
impl Node<SoftwareDevState> for QualityGateNode {
    async fn invoke(&self, state: &mut SoftwareDevState) -> GraphResult<()> {
        // Simple quality scoring based on content length and keywords
        let mut score = 0u32;
        
        // Score based on code quality indicators
        if !state.code.is_empty() {
            score += 20;
            if state.code.contains("error handling") || state.code.contains("try") || state.code.contains("catch") {
                score += 15;
            }
            if state.code.contains("test") || state.code.contains("assert") {
                score += 15;
            }
            if state.code.len() > 100 {
                score += 10;
            }
        }
        
        // Score based on test results
        if !state.test_results.is_empty() {
            score += 20;
            if state.test_results.contains("passed") || state.test_results.contains("success") {
                score += 15;
            }
        }
        
        // Score based on security audit
        if !state.security_audit.is_empty() {
            score += 15;
            if state.security_audit.contains("secure") || state.security_audit.contains("no vulnerabilities") {
                score += 10;
            }
        }
        
        state.quality_score = score;
        state.approved = score >= self.threshold;
        
        println!("🎯 Quality Gate: Score {} (threshold: {}), Approved: {}", 
                 score, self.threshold, state.approved);
        
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 AgentGraph Workflow Example: Software Development Pipeline");
    println!("==============================================================");

    // Step 1: Setup LLM and Tools Infrastructure
    println!("\n🔧 Setting up infrastructure...");
    let llm_config = LLMConfig::default();
    let mut llm_manager = LLMManager::new(llm_config);
    let mock_provider = MockProvider::new();
    llm_manager.register_provider("mock".to_string(), Arc::new(mock_provider));
    let llm_manager = Arc::new(llm_manager);
    
    let tool_registry = Arc::new(ToolRegistry::new());
    let tool_executor = Arc::new(ToolExecutor::new());

    // Step 2: Create Specialized Agents
    println!("👥 Creating specialized agents...");
    
    // Developer Agent
    let developer_template = RoleTemplates::software_developer();
    let developer_config = developer_template.to_agent_config("Alice".to_string(), "mock".to_string());
    let developer_agent = Agent::new(
        developer_config,
        Arc::clone(&llm_manager),
        Arc::clone(&tool_registry),
        Arc::clone(&tool_executor),
    )?;

    // QA Agent
    let qa_template = RoleTemplates::qa_engineer();
    let qa_config = qa_template.to_agent_config("Bob".to_string(), "mock".to_string());
    let qa_agent = Agent::new(
        qa_config,
        Arc::clone(&llm_manager),
        Arc::clone(&tool_registry),
        Arc::clone(&tool_executor),
    )?;

    // Security Agent (using research analyst template)
    let security_template = RoleTemplates::research_analyst();
    let security_config = security_template.to_agent_config("Carol".to_string(), "mock".to_string());
    let security_agent = Agent::new(
        security_config,
        Arc::clone(&llm_manager),
        Arc::clone(&tool_registry),
        Arc::clone(&tool_executor),
    )?;

    // DevOps Agent
    let devops_template = RoleTemplates::devops_engineer();
    let devops_config = devops_template.to_agent_config("Dave".to_string(), "mock".to_string());
    let devops_agent = Agent::new(
        devops_config,
        Arc::clone(&llm_manager),
        Arc::clone(&tool_registry),
        Arc::clone(&tool_executor),
    )?;

    // Step 3: Create Agent Nodes for Graph Workflow
    println!("🔗 Creating agent nodes for workflow...");
    
    let developer_node = AgentNode::new(
        developer_agent,
        "Implement the following requirements with proper error handling and documentation: {requirements}"
    )
    .map_input("requirements".to_string(), "requirements".to_string())
    .map_output("response".to_string(), "code".to_string());

    let qa_node = AgentNode::new(
        qa_agent,
        "Create comprehensive tests for this code and analyze its quality: {code}"
    )
    .map_input("code".to_string(), "code".to_string())
    .map_output("response".to_string(), "test_results".to_string());

    let security_node = AgentNode::new(
        security_agent,
        "Perform a security audit on this code, identify vulnerabilities and suggest improvements: {code}"
    )
    .map_input("code".to_string(), "code".to_string())
    .map_output("response".to_string(), "security_audit".to_string());

    let devops_node = AgentNode::new(
        devops_agent,
        "Create deployment strategy and infrastructure setup for this code: {code}"
    )
    .map_input("code".to_string(), "code".to_string())
    .map_output("response".to_string(), "deployment_status".to_string());

    let quality_gate = QualityGateNode::new(70); // 70% quality threshold

    // Step 4: Build the Workflow Graph
    println!("🏗️ Building workflow graph...");
    
    let graph = GraphBuilder::new()
        // Add agent nodes
        .add_node("developer".to_string(), developer_node)?
        .add_node("qa".to_string(), qa_node)?
        .add_node("security".to_string(), security_node)?
        .add_node("quality_gate".to_string(), quality_gate)?
        .add_node("devops".to_string(), devops_node)?
        
        // Define workflow edges
        .add_edge(Edge::simple("developer", "qa"))?
        .add_edge(Edge::parallel("qa", vec!["security", "quality_gate"]))?
        .add_edge(Edge::conditional("quality_gate", "approved", "devops", "developer"))?
        
        // Set entry and finish points
        .with_entry_point("developer".to_string())?
        .add_finish_point("devops".to_string())?
        .add_finish_point("developer".to_string())? // In case of quality gate failure
        
        .build()?;

    println!("✅ Workflow graph created with {} nodes", graph.node_ids().len());

    // Step 5: Execute the Workflow
    println!("\n🚀 Executing software development workflow...");
    
    let mut state = SoftwareDevState {
        requirements: "Create a REST API for user authentication with JWT tokens, input validation, rate limiting, and comprehensive error handling. Include unit tests and security best practices.".to_string(),
        stage: "development".to_string(),
        ..Default::default()
    };

    println!("📋 Initial Requirements: {}", state.requirements);

    // Execute the graph workflow
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
    println!("Stage: {}", state.stage);
    println!("Quality Score: {}/100", state.quality_score);
    println!("Approved: {}", state.approved);
    
    println!("\n💻 Generated Code:");
    println!("{}", if state.code.len() > 200 { 
        format!("{}...", &state.code[..200]) 
    } else { 
        state.code.clone() 
    });
    
    println!("\n🧪 Test Results:");
    println!("{}", if state.test_results.len() > 200 { 
        format!("{}...", &state.test_results[..200]) 
    } else { 
        state.test_results.clone() 
    });
    
    println!("\n🔒 Security Audit:");
    println!("{}", if state.security_audit.len() > 200 { 
        format!("{}...", &state.security_audit[..200]) 
    } else { 
        state.security_audit.clone() 
    });
    
    if state.approved {
        println!("\n🚀 Deployment Status:");
        println!("{}", if state.deployment_status.len() > 200 { 
            format!("{}...", &state.deployment_status[..200]) 
        } else { 
            state.deployment_status.clone() 
        });
    }

    println!("\n✅ Agent-Graph workflow completed successfully!");
    println!("🎯 This demonstrates LangGraph-style agent orchestration in Rust!");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_software_dev_state() {
        let mut state = SoftwareDevState::default();
        
        // Test state value setting and getting
        state.set_value("requirements", serde_json::Value::String("Test requirement".to_string())).unwrap();
        assert_eq!(state.get_value("requirements"), Some(serde_json::Value::String("Test requirement".to_string())));
        
        state.set_value("quality_score", serde_json::Value::Number(serde_json::Number::from(85))).unwrap();
        assert_eq!(state.quality_score, 85);
        
        state.set_value("approved", serde_json::Value::Bool(true)).unwrap();
        assert!(state.approved);
    }

    #[tokio::test]
    async fn test_quality_gate_node() {
        let quality_gate = QualityGateNode::new(50);
        let mut state = SoftwareDevState {
            code: "function test() { try { /* error handling */ } catch(e) { /* handle */ } }".to_string(),
            test_results: "All tests passed successfully".to_string(),
            security_audit: "Code is secure with no vulnerabilities found".to_string(),
            ..Default::default()
        };
        
        quality_gate.invoke(&mut state).await.unwrap();
        
        assert!(state.quality_score > 50);
        assert!(state.approved);
    }
}
