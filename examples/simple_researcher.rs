//! Simple researcher example demonstrating basic graph execution.

use agent_graph::{
    GraphBuilder, Node, State, GraphResult, Edge,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Research state containing the query and accumulated results
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ResearchState {
    /// The research query
    query: String,
    /// Research results
    results: Vec<String>,
    /// Research metadata
    metadata: HashMap<String, String>,
    /// Current research step
    step: u32,
    /// Whether research is complete
    complete: bool,
}



/// Node that initializes the research process
#[derive(Debug)]
struct InitializeResearchNode;

#[async_trait]
impl Node<ResearchState> for InitializeResearchNode {
    async fn invoke(&self, state: &mut ResearchState) -> GraphResult<()> {
        println!("🔍 Initializing research for query: '{}'", state.query);
        
        state.metadata.insert("start_time".to_string(), chrono::Utc::now().to_rfc3339());
        state.metadata.insert("status".to_string(), "initialized".to_string());
        state.step = 1;
        
        // Simulate some initialization work
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        println!("✅ Research initialized successfully");
        Ok(())
    }

    fn metadata(&self) -> agent_graph::NodeMetadata {
        agent_graph::NodeMetadata::new("InitializeResearch")
            .with_description("Initialize the research process")
            .with_tag("initialization")
            .with_expected_duration(100)
    }
}

/// Node that performs web search
#[derive(Debug)]
struct WebSearchNode;

#[async_trait]
impl Node<ResearchState> for WebSearchNode {
    async fn invoke(&self, state: &mut ResearchState) -> GraphResult<()> {
        println!("🌐 Performing web search for: '{}'", state.query);
        
        // Simulate web search
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        // Add mock search results
        let search_results = vec![
            format!("Web result 1 for '{}'", state.query),
            format!("Web result 2 for '{}'", state.query),
            format!("Web result 3 for '{}'", state.query),
        ];
        
        state.results.extend(search_results);
        state.metadata.insert("web_search_completed".to_string(), chrono::Utc::now().to_rfc3339());
        state.step += 1;
        
        println!("✅ Web search completed, found {} results", state.results.len());
        Ok(())
    }

    fn metadata(&self) -> agent_graph::NodeMetadata {
        agent_graph::NodeMetadata::new("WebSearch")
            .with_description("Perform web search for the research query")
            .with_tag("search")
            .with_expected_duration(500)
    }
}

/// Node that analyzes academic papers
#[derive(Debug)]
struct AcademicSearchNode;

#[async_trait]
impl Node<ResearchState> for AcademicSearchNode {
    async fn invoke(&self, state: &mut ResearchState) -> GraphResult<()> {
        println!("🎓 Searching academic papers for: '{}'", state.query);
        
        // Simulate academic search
        tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;
        
        // Add mock academic results
        let academic_results = vec![
            format!("Academic paper 1: '{}' - Journal of AI Research", state.query),
            format!("Academic paper 2: '{}' - Nature Machine Intelligence", state.query),
        ];
        
        state.results.extend(academic_results);
        state.metadata.insert("academic_search_completed".to_string(), chrono::Utc::now().to_rfc3339());
        state.step += 1;
        
        println!("✅ Academic search completed, found {} total results", state.results.len());
        Ok(())
    }

    fn metadata(&self) -> agent_graph::NodeMetadata {
        agent_graph::NodeMetadata::new("AcademicSearch")
            .with_description("Search academic papers and journals")
            .with_tag("academic")
            .with_tag("search")
            .with_expected_duration(800)
    }
}

/// Node that synthesizes research results
#[derive(Debug)]
struct SynthesizeResultsNode;

#[async_trait]
impl Node<ResearchState> for SynthesizeResultsNode {
    async fn invoke(&self, state: &mut ResearchState) -> GraphResult<()> {
        println!("🧠 Synthesizing research results...");
        
        // Simulate synthesis work
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        
        // Create a synthesis summary
        let synthesis = format!(
            "Research synthesis for '{}': Found {} sources including web results and academic papers. \
             Key findings suggest significant developments in this area.",
            state.query,
            state.results.len()
        );
        
        state.results.push(synthesis);
        state.metadata.insert("synthesis_completed".to_string(), chrono::Utc::now().to_rfc3339());
        state.complete = true;
        state.step += 1;
        
        println!("✅ Research synthesis completed");
        Ok(())
    }

    fn metadata(&self) -> agent_graph::NodeMetadata {
        agent_graph::NodeMetadata::new("SynthesizeResults")
            .with_description("Synthesize and summarize research findings")
            .with_tag("synthesis")
            .with_tag("analysis")
            .with_expected_duration(300)
    }
}

/// Node that generates the final report
#[derive(Debug)]
struct GenerateReportNode;

#[async_trait]
impl Node<ResearchState> for GenerateReportNode {
    async fn invoke(&self, state: &mut ResearchState) -> GraphResult<()> {
        println!("📄 Generating final research report...");
        
        // Simulate report generation
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        
        println!("\n{}", "=".repeat(60));
        println!("📊 RESEARCH REPORT");
        println!("{}", "=".repeat(60));
        println!("Query: {}", state.query);
        println!("Total Results: {}", state.results.len());
        println!("Research Steps: {}", state.step);
        println!("\nFindings:");
        for (i, result) in state.results.iter().enumerate() {
            println!("  {}. {}", i + 1, result);
        }
        println!("\nMetadata:");
        for (key, value) in &state.metadata {
            println!("  {}: {}", key, value);
        }
        println!("{}", "=".repeat(60));
        
        state.metadata.insert("report_generated".to_string(), chrono::Utc::now().to_rfc3339());
        
        println!("✅ Research report generated successfully");
        Ok(())
    }

    fn metadata(&self) -> agent_graph::NodeMetadata {
        agent_graph::NodeMetadata::new("GenerateReport")
            .with_description("Generate the final research report")
            .with_tag("reporting")
            .with_tag("output")
            .with_expected_duration(200)
    }
}

#[tokio::main]
async fn main() -> GraphResult<()> {
    // Initialize tracing
    agent_graph::init_tracing();
    
    println!("🚀 Starting Simple Researcher Example");
    
    // Create the research state
    let mut state = ResearchState {
        query: "artificial intelligence in healthcare".to_string(),
        results: Vec::new(),
        metadata: HashMap::new(),
        step: 0,
        complete: false,
    };
    
    // Build the research graph
    let graph = GraphBuilder::new()
        .add_node("initialize".to_string(), InitializeResearchNode)?
        .add_node("web_search".to_string(), WebSearchNode)?
        .add_node("academic_search".to_string(), AcademicSearchNode)?
        .add_node("synthesize".to_string(), SynthesizeResultsNode)?
        .add_node("generate_report".to_string(), GenerateReportNode)?
        .with_entry_point("initialize".to_string())?
        .add_edge(Edge::simple("initialize", "web_search"))?
        .add_edge(Edge::simple("web_search", "academic_search"))?
        .add_edge(Edge::simple("academic_search", "synthesize"))?
        .add_edge(Edge::simple("synthesize", "generate_report"))?
        .add_finish_point("generate_report".to_string())?
        .build()?;
    
    println!("\n📋 Graph Summary: {}", graph.summary());
    
    // Execute the research graph
    println!("\n🔄 Starting research execution...\n");
    let start_time = std::time::Instant::now();
    
    let context = graph.run(&mut state).await?;
    
    let execution_time = start_time.elapsed();
    
    println!("\n🎉 Research completed successfully!");
    println!("⏱️  Total execution time: {:?}", execution_time);
    println!("📊 Execution steps: {}", context.current_step);
    println!("🛤️  Execution path: {:?}", context.execution_path);
    println!("🆔 Execution ID: {}", context.execution_id);
    
    // Display final state
    println!("\n📈 Final Research State:");
    println!("  Query: {}", state.query);
    println!("  Results count: {}", state.results.len());
    println!("  Research complete: {}", state.complete);
    println!("  Final step: {}", state.step);
    
    Ok(())
}
