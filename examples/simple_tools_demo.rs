// Simple tools demonstration example
// Shows basic usage of the new tools framework

use agent_graph::tools::{
    registry::ToolRegistry,
    execution::{ToolExecutor, ToolExecutionContext},
    traits::ToolInput,
    common::{create_common_tools_registry, categories},
    ToolConfig,
};
use serde_json::json;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 AgentGraph Tools Framework - Simple Demo");
    println!("===========================================");
    
    // Create the tools registry
    let registry = create_common_tools_registry()?;
    let mut executor = ToolExecutor::new().with_cache(Duration::from_secs(300));
    
    // Show registry statistics
    let stats = registry.stats();
    println!("✅ Registry contains {} tools in {} categories", 
            stats.total_tools, stats.total_categories);
    
    // Show tools by category
    for category in [categories::HTTP, categories::TEXT, categories::MATH] {
        let tools = registry.get_by_category(category);
        println!("   {} category: {} tools", category, tools.len());
        for tool in tools {
            println!("     - {} ({})", tool.metadata().name, tool.metadata().id);
        }
    }
    
    println!("\n📝 Testing Text Processing Tool");
    println!("==============================");
    
    // Get the text processor tool
    if let Some(text_tool) = registry.get("text_processor") {
        let sample_text = "Hello, AgentGraph Tools Framework!";
        let operations = vec!["uppercase", "lowercase", "reverse"];
        
        for operation in operations {
            let input = ToolInput::new(json!(sample_text))
                .with_parameter("operation", operation);
            
            let config = ToolConfig::default();
            let context = ToolExecutionContext::new(format!("text_demo_{}", operation));
            
            match executor.execute(text_tool.clone(), input, &config, &context).await {
                Ok(result) => {
                    let processed_text = result.output.data.get("result")
                        .and_then(|v| v.as_str())
                        .unwrap_or("N/A");
                    
                    println!("✅ {} operation: '{}'", operation, processed_text);
                }
                Err(e) => {
                    println!("❌ {} operation failed: {}", operation, e);
                }
            }
        }
    }
    
    println!("\n🧮 Testing Math Tools");
    println!("====================");
    
    // Test calculator
    if let Some(calc_tool) = registry.get("calculator") {
        let expressions = vec!["10 + 5", "20 - 8", "6 * 7", "100 / 4"];
        
        for expr in expressions {
            let input = ToolInput::new(json!(expr));
            let config = ToolConfig::default();
            let context = ToolExecutionContext::new(format!("calc_{}", expr.replace(' ', "_")));
            
            match executor.execute(calc_tool.clone(), input, &config, &context).await {
                Ok(result) => {
                    let result_value = result.output.data.get("result")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    
                    println!("✅ {} = {}", expr, result_value);
                }
                Err(e) => {
                    println!("❌ {} failed: {}", expr, e);
                }
            }
        }
    }
    
    // Test statistics
    if let Some(stats_tool) = registry.get("statistics") {
        let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let input = ToolInput::new(json!(numbers));
        let config = ToolConfig::default();
        let context = ToolExecutionContext::new("stats_demo".to_string());
        
        match executor.execute(stats_tool, input, &config, &context).await {
            Ok(result) => {
                let mean = result.output.data.get("mean").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let std_dev = result.output.data.get("standard_deviation").and_then(|v| v.as_f64()).unwrap_or(0.0);
                
                println!("✅ Statistics for {:?}:", numbers);
                println!("   Mean: {:.2}", mean);
                println!("   Std Dev: {:.2}", std_dev);
            }
            Err(e) => {
                println!("❌ Statistics calculation failed: {}", e);
            }
        }
    }
    
    println!("\n🔍 Testing Tool Search");
    println!("=====================");
    
    // Search for tools
    let search_results = registry.search("http");
    println!("✅ Search for 'http' found {} tools:", search_results.len());
    for tool in search_results {
        println!("   - {} ({})", tool.metadata().name, tool.metadata().id);
    }
    
    let search_results = registry.search("math");
    println!("✅ Search for 'math' found {} tools:", search_results.len());
    for tool in search_results {
        println!("   - {} ({})", tool.metadata().name, tool.metadata().id);
    }
    
    println!("\n📊 Executor Statistics");
    println!("=====================");
    
    // Show executor statistics
    let all_stats = executor.get_all_stats();
    for (tool_id, stats) in all_stats {
        println!("Tool: {}", tool_id);
        println!("  Executions: {}", stats.execution_count);
        println!("  Success rate: {:.1}%", stats.success_rate());
        println!("  Avg time: {:.1}ms", stats.avg_execution_time_ms);
    }
    
    println!("\n🎉 Demo Complete!");
    println!("================");
    println!("The AgentGraph Tools Framework provides:");
    println!("✅ Tool registry for managing and discovering tools");
    println!("✅ Tool executor with retry, timeout, and caching");
    println!("✅ Common tools for HTTP, file, database, text, and math operations");
    println!("✅ Comprehensive error handling and statistics");
    println!("✅ Type-safe tool interfaces with validation");
    
    Ok(())
}
