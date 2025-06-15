// Tools demonstration example
// Shows how to use the new tools framework with AgentGraph

use agent_graph::{
    GraphBuilder, Node, GraphResult, Edge,
    tools::{
        registry::ToolRegistry,
        execution::{ToolExecutor, ToolExecutionContext},
        traits::ToolInput,
        common::{create_common_tools_registry, categories},
        ToolConfig,
    },
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ToolDemoState {
    step: String,
    data: serde_json::Value,
    results: Vec<serde_json::Value>,
}

impl Default for ToolDemoState {
    fn default() -> Self {
        Self {
            step: "start".to_string(),
            data: json!({}),
            results: Vec::new(),
        }
    }
}

/// Node that demonstrates HTTP tool usage
#[derive(Debug)]
struct HttpDemoNode {
    registry: ToolRegistry,
    executor: Arc<Mutex<ToolExecutor>>,
}

impl HttpDemoNode {
    fn new() -> GraphResult<Self> {
        let registry = create_common_tools_registry()
            .map_err(|e| agent_graph::error::GraphError::ConfigurationError(
                format!("Failed to create tools registry: {}", e)
            ))?;
        let executor = ToolExecutor::new().with_cache(Duration::from_secs(300));

        Ok(Self {
            registry,
            executor: Arc::new(Mutex::new(executor))
        })
    }
}

#[async_trait]
impl Node<ToolDemoState> for HttpDemoNode {
    async fn invoke(&self, state: &mut ToolDemoState) -> GraphResult<()> {
        println!("🌐 HTTP Tool Demo");
        
        // Get the HTTP GET tool
        let http_get_tool = self.registry.get("http_get")
            .ok_or_else(|| agent_graph::error::GraphError::NodeError {
                node_id: "http_demo".to_string(),
                message: "HTTP GET tool not found".to_string(),
                source: None,
            })?;
        
        // Create tool input for a simple HTTP request
        let input = ToolInput::new(json!("https://httpbin.org/json"))
            .with_parameter("headers", json!({"User-Agent": "AgentGraph/0.3.0"}));
        
        let config = ToolConfig::default();
        let context = ToolExecutionContext::new("http_demo_1".to_string())
            .with_user_id("demo_user".to_string());
        
        // Execute the tool
        let executor_clone = Arc::clone(&self.executor);
        let result = {
            let mut executor = executor_clone.lock().unwrap();
            executor.execute(http_get_tool, input, &config, &context).await
        };
        match result {
            Ok(result) => {
                println!("✅ HTTP request successful!");
                println!("   Status: {}", result.output.get_metadata::<u16>("status_code").unwrap_or(0));
                println!("   Duration: {}ms", result.metadata.duration_ms);
                
                state.step = "http_complete".to_string();
                state.results.push(json!({
                    "tool": "http_get",
                    "success": true,
                    "duration_ms": result.metadata.duration_ms
                }));
            }
            Err(e) => {
                println!("❌ HTTP request failed: {}", e);
                state.results.push(json!({
                    "tool": "http_get",
                    "success": false,
                    "error": e.to_string()
                }));
            }
        }
        
        Ok(())
    }
}

/// Node that demonstrates text processing tools
#[derive(Debug)]
struct TextDemoNode {
    registry: ToolRegistry,
    executor: Arc<Mutex<ToolExecutor>>,
}

impl TextDemoNode {
    fn new() -> GraphResult<Self> {
        let registry = create_common_tools_registry()
            .map_err(|e| agent_graph::error::GraphError::ConfigurationError(
                format!("Failed to create tools registry: {}", e)
            ))?;
        let executor = ToolExecutor::new();

        Ok(Self {
            registry,
            executor: Arc::new(Mutex::new(executor))
        })
    }
}

#[async_trait]
impl Node<ToolDemoState> for TextDemoNode {
    async fn invoke(&self, state: &mut ToolDemoState) -> GraphResult<()> {
        println!("📝 Text Processing Tool Demo");
        
        // Get the text processor tool
        let text_tool = self.registry.get("text_processor")
            .ok_or_else(|| agent_graph::error::GraphError::NodeError {
                node_id: "text_demo".to_string(),
                message: "Text processor tool not found".to_string(),
                source: None,
            })?;
        
        let sample_text = "Hello, AgentGraph Tools Framework!";
        
        // Test different text operations
        let operations = vec!["uppercase", "lowercase", "reverse"];
        let executor_clone = Arc::clone(&self.executor);

        for operation in operations {
            let input = ToolInput::new(json!(sample_text))
                .with_parameter("operation", operation);
            
            let config = ToolConfig::default();
            let context = ToolExecutionContext::new(format!("text_demo_{}", operation));
            
            let result = {
                let mut executor = executor_clone.lock().unwrap();
                executor.execute(text_tool.clone(), input, &config, &context).await
            };
            match result {
                Ok(result) => {
                    let processed_text = result.output.data.get("result")
                        .and_then(|v| v.as_str())
                        .unwrap_or("N/A");
                    
                    println!("✅ {} operation: '{}'", operation, processed_text);
                    
                    state.results.push(json!({
                        "tool": "text_processor",
                        "operation": operation,
                        "input": sample_text,
                        "output": processed_text,
                        "success": true
                    }));
                }
                Err(e) => {
                    println!("❌ {} operation failed: {}", operation, e);
                    state.results.push(json!({
                        "tool": "text_processor",
                        "operation": operation,
                        "success": false,
                        "error": e.to_string()
                    }));
                }
            }
        }
        
        state.step = "text_complete".to_string();
        Ok(())
    }
}

/// Node that demonstrates math tools
#[derive(Debug)]
struct MathDemoNode {
    registry: ToolRegistry,
    executor: Arc<Mutex<ToolExecutor>>,
}

impl MathDemoNode {
    fn new() -> GraphResult<Self> {
        let registry = create_common_tools_registry()
            .map_err(|e| agent_graph::error::GraphError::ConfigurationError(
                format!("Failed to create tools registry: {}", e)
            ))?;
        let executor = ToolExecutor::new();

        Ok(Self {
            registry,
            executor: Arc::new(Mutex::new(executor))
        })
    }
}

#[async_trait]
impl Node<ToolDemoState> for MathDemoNode {
    async fn invoke(&self, state: &mut ToolDemoState) -> GraphResult<()> {
        println!("🧮 Math Tools Demo");
        
        // Calculator demo
        let calc_tool = self.registry.get("calculator")
            .ok_or_else(|| agent_graph::error::GraphError::NodeError {
                node_id: "math_demo".to_string(),
                message: "Calculator tool not found".to_string(),
                source: None,
            })?;
        
        let expressions = vec!["10 + 5", "20 - 8", "6 * 7", "100 / 4"];
        let executor_clone = Arc::clone(&self.executor);

        for expr in expressions {
            let input = ToolInput::new(json!(expr));
            let config = ToolConfig::default();
            let context = ToolExecutionContext::new(format!("calc_{}", expr.replace(' ', "_")));
            
            let result = {
                let mut executor = executor_clone.lock().unwrap();
                executor.execute(calc_tool.clone(), input, &config, &context).await
            };
            match result {
                Ok(result) => {
                    let result_value = result.output.data.get("result")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    
                    println!("✅ {} = {}", expr, result_value);
                    
                    state.results.push(json!({
                        "tool": "calculator",
                        "expression": expr,
                        "result": result_value,
                        "success": true
                    }));
                }
                Err(e) => {
                    println!("❌ {} failed: {}", expr, e);
                }
            }
        }
        
        // Statistics demo
        let stats_tool = self.registry.get("statistics")
            .ok_or_else(|| agent_graph::error::GraphError::NodeError {
                node_id: "math_demo".to_string(),
                message: "Statistics tool not found".to_string(),
                source: None,
            })?;
        
        let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let input = ToolInput::new(json!(numbers));
        let config = ToolConfig::default();
        let context = ToolExecutionContext::new("stats_demo".to_string());
        
        let result = {
            let mut executor = executor_clone.lock().unwrap();
            executor.execute(stats_tool, input, &config, &context).await
        };
        match result {
            Ok(result) => {
                let mean = result.output.data.get("mean").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let std_dev = result.output.data.get("standard_deviation").and_then(|v| v.as_f64()).unwrap_or(0.0);
                
                println!("✅ Statistics for {:?}:", numbers);
                println!("   Mean: {:.2}", mean);
                println!("   Std Dev: {:.2}", std_dev);
                
                state.results.push(json!({
                    "tool": "statistics",
                    "data": numbers,
                    "mean": mean,
                    "std_dev": std_dev,
                    "success": true
                }));
            }
            Err(e) => {
                println!("❌ Statistics calculation failed: {}", e);
            }
        }
        
        state.step = "math_complete".to_string();
        Ok(())
    }
}

/// Node that shows tool registry capabilities
#[derive(Debug)]
struct RegistryDemoNode {
    registry: ToolRegistry,
}

impl RegistryDemoNode {
    fn new() -> GraphResult<Self> {
        let registry = create_common_tools_registry()
            .map_err(|e| agent_graph::error::GraphError::ConfigurationError(
                format!("Failed to create tools registry: {}", e)
            ))?;
        Ok(Self { registry })
    }
}

#[async_trait]
impl Node<ToolDemoState> for RegistryDemoNode {
    async fn invoke(&self, state: &mut ToolDemoState) -> GraphResult<()> {
        println!("📚 Tool Registry Demo");
        
        // Show registry statistics
        let stats = self.registry.stats();
        println!("✅ Registry contains {} tools in {} categories", 
                stats.total_tools, stats.total_categories);
        
        // Show tools by category
        for category in [categories::HTTP, categories::TEXT, categories::MATH] {
            let tools = self.registry.get_by_category(category);
            println!("   {} category: {} tools", category, tools.len());
            for tool in tools {
                println!("     - {} ({})", tool.metadata().name, tool.metadata().id);
            }
        }
        
        // Search for tools
        let search_results = self.registry.search("http");
        println!("✅ Search for 'http' found {} tools", search_results.len());
        
        state.step = "registry_complete".to_string();
        state.data = json!({
            "total_tools": stats.total_tools,
            "total_categories": stats.total_categories,
            "search_results": search_results.len()
        });
        
        Ok(())
    }
}

#[tokio::main]
async fn main() -> GraphResult<()> {
    println!("🚀 AgentGraph Tools Framework Demo");
    println!("=====================================");
    
    // Create the graph
    let graph = GraphBuilder::new()
        .add_node("registry_demo".to_string(), RegistryDemoNode::new()?)?
        .add_node("http_demo".to_string(), HttpDemoNode::new()?)?
        .add_node("text_demo".to_string(), TextDemoNode::new()?)?
        .add_node("math_demo".to_string(), MathDemoNode::new()?)?
        .add_edge(Edge::simple("registry_demo".to_string(), "http_demo".to_string()))?
        .add_edge(Edge::simple("http_demo".to_string(), "text_demo".to_string()))?
        .add_edge(Edge::simple("text_demo".to_string(), "math_demo".to_string()))?
        .with_entry_point("registry_demo".to_string())
        .with_finish_point("math_demo".to_string())
        .build()?;
    
    // Execute the graph
    let mut state = ToolDemoState::default();
    let context = graph.run(&mut state).await?;
    
    println!("\n🎉 Demo Complete!");
    println!("================");
    println!("Final state: {}", state.step);
    println!("Total results: {}", state.results.len());
    println!("Execution time: {}ms", context.total_duration_ms);
    
    // Show summary of results
    println!("\n📊 Results Summary:");
    for (i, result) in state.results.iter().enumerate() {
        if let Some(tool) = result.get("tool").and_then(|v| v.as_str()) {
            let success = result.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
            let status = if success { "✅" } else { "❌" };
            println!("   {}. {} {} - {}", i + 1, status, tool, 
                    if success { "Success" } else { "Failed" });
        }
    }
    
    Ok(())
}
