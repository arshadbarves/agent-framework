//! Parallel processing example demonstrating concurrent node execution.

use agent_graph::{
    GraphBuilder, Node, GraphResult, Edge,
    ExecutionConfig,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

/// Processing state for parallel operations
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProcessingState {
    /// Input data to process
    input_data: Vec<i32>,
    /// Results from different processing stages
    results: HashMap<String, Vec<i32>>,
    /// Processing metadata
    metadata: HashMap<String, String>,
    /// Processing statistics
    stats: ProcessingStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProcessingStats {
    total_items: usize,
    processed_items: usize,
    processing_time_ms: u64,
    parallel_operations: u32,
}

impl Default for ProcessingStats {
    fn default() -> Self {
        Self {
            total_items: 0,
            processed_items: 0,
            processing_time_ms: 0,
            parallel_operations: 0,
        }
    }
}



/// Node that initializes the data processing
#[derive(Debug)]
struct InitializeDataNode;

#[async_trait]
impl Node<ProcessingState> for InitializeDataNode {
    async fn invoke(&self, state: &mut ProcessingState) -> GraphResult<()> {
        println!("🔧 Initializing data processing...");
        
        // Generate sample data
        state.input_data = (1..=100).collect();
        state.stats.total_items = state.input_data.len();
        state.metadata.insert("initialized_at".to_string(), chrono::Utc::now().to_rfc3339());
        
        println!("✅ Initialized with {} data items", state.input_data.len());
        Ok(())
    }

    fn metadata(&self) -> agent_graph::NodeMetadata {
        agent_graph::NodeMetadata::new("InitializeData")
            .with_description("Initialize data for parallel processing")
            .with_tag("initialization")
            .with_parallel_safe(true)
    }
}

/// Node that performs mathematical transformations
#[derive(Debug)]
struct MathProcessorNode {
    operation: String,
}

impl MathProcessorNode {
    fn new(operation: &str) -> Self {
        Self {
            operation: operation.to_string(),
        }
    }
}

#[async_trait]
impl Node<ProcessingState> for MathProcessorNode {
    async fn invoke(&self, state: &mut ProcessingState) -> GraphResult<()> {
        println!("🧮 Starting {} processing...", self.operation);
        let start_time = std::time::Instant::now();
        
        // Simulate processing time
        sleep(Duration::from_millis(500)).await;
        
        let processed_data = match self.operation.as_str() {
            "square" => state.input_data.iter().map(|x| x * x).collect(),
            "double" => state.input_data.iter().map(|x| x * 2).collect(),
            "increment" => state.input_data.iter().map(|x| x + 1).collect(),
            _ => state.input_data.clone(),
        };
        
        state.results.insert(self.operation.clone(), processed_data);
        state.stats.parallel_operations += 1;
        
        let duration = start_time.elapsed().as_millis() as u64;
        println!("✅ {} processing completed in {}ms", self.operation, duration);
        
        Ok(())
    }

    fn metadata(&self) -> agent_graph::NodeMetadata {
        agent_graph::NodeMetadata::new(&format!("MathProcessor_{}", self.operation))
            .with_description(&format!("Perform {} operation on data", self.operation))
            .with_tag("math")
            .with_tag("processing")
            .with_parallel_safe(true)
            .with_expected_duration(500)
    }
}

/// Node that performs statistical analysis
#[derive(Debug)]
struct StatisticsNode;

#[async_trait]
impl Node<ProcessingState> for StatisticsNode {
    async fn invoke(&self, state: &mut ProcessingState) -> GraphResult<()> {
        println!("📊 Computing statistics...");
        let start_time = std::time::Instant::now();
        
        // Simulate statistical computation
        sleep(Duration::from_millis(300)).await;
        
        // Calculate statistics for original data
        let sum: i32 = state.input_data.iter().sum();
        let mean = sum as f64 / state.input_data.len() as f64;
        let min = *state.input_data.iter().min().unwrap_or(&0);
        let max = *state.input_data.iter().max().unwrap_or(&0);
        
        let stats_result = vec![sum, mean as i32, min, max];
        state.results.insert("statistics".to_string(), stats_result);
        state.stats.parallel_operations += 1;
        
        let duration = start_time.elapsed().as_millis() as u64;
        println!("✅ Statistics computed in {}ms", duration);
        
        Ok(())
    }

    fn metadata(&self) -> agent_graph::NodeMetadata {
        agent_graph::NodeMetadata::new("Statistics")
            .with_description("Compute statistical measures")
            .with_tag("statistics")
            .with_tag("analysis")
            .with_parallel_safe(true)
            .with_expected_duration(300)
    }
}

/// Node that filters data
#[derive(Debug)]
struct FilterNode {
    filter_type: String,
}

impl FilterNode {
    fn new(filter_type: &str) -> Self {
        Self {
            filter_type: filter_type.to_string(),
        }
    }
}

#[async_trait]
impl Node<ProcessingState> for FilterNode {
    async fn invoke(&self, state: &mut ProcessingState) -> GraphResult<()> {
        println!("🔍 Applying {} filter...", self.filter_type);
        let start_time = std::time::Instant::now();
        
        // Simulate filtering time
        sleep(Duration::from_millis(200)).await;
        
        let filtered_data = match self.filter_type.as_str() {
            "even" => state.input_data.iter().filter(|&x| x % 2 == 0).cloned().collect(),
            "odd" => state.input_data.iter().filter(|&x| x % 2 != 0).cloned().collect(),
            "large" => state.input_data.iter().filter(|&x| *x > 50).cloned().collect(),
            _ => state.input_data.clone(),
        };
        
        state.results.insert(format!("filter_{}", self.filter_type), filtered_data);
        state.stats.parallel_operations += 1;
        
        let duration = start_time.elapsed().as_millis() as u64;
        println!("✅ {} filter applied in {}ms", self.filter_type, duration);
        
        Ok(())
    }

    fn metadata(&self) -> agent_graph::NodeMetadata {
        agent_graph::NodeMetadata::new(&format!("Filter_{}", self.filter_type))
            .with_description(&format!("Apply {} filter to data", self.filter_type))
            .with_tag("filtering")
            .with_tag("processing")
            .with_parallel_safe(true)
            .with_expected_duration(200)
    }
}

/// Node that aggregates all results
#[derive(Debug)]
struct AggregateResultsNode;

#[async_trait]
impl Node<ProcessingState> for AggregateResultsNode {
    async fn invoke(&self, state: &mut ProcessingState) -> GraphResult<()> {
        println!("📋 Aggregating all processing results...");
        
        // Simulate aggregation work
        sleep(Duration::from_millis(100)).await;
        
        state.stats.processed_items = state.results.values()
            .map(|v| v.len())
            .sum();
        
        state.metadata.insert("aggregated_at".to_string(), chrono::Utc::now().to_rfc3339());
        
        println!("✅ Results aggregated successfully");
        Ok(())
    }

    fn metadata(&self) -> agent_graph::NodeMetadata {
        agent_graph::NodeMetadata::new("AggregateResults")
            .with_description("Aggregate all processing results")
            .with_tag("aggregation")
            .with_tag("finalization")
            .with_parallel_safe(false) // This needs to run after all parallel operations
    }
}

#[tokio::main]
async fn main() -> GraphResult<()> {
    // Initialize tracing
    agent_graph::init_tracing();
    
    println!("🚀 Starting Parallel Processing Example");
    
    // Create processing state
    let mut state = ProcessingState {
        input_data: Vec::new(),
        results: HashMap::new(),
        metadata: HashMap::new(),
        stats: ProcessingStats::default(),
    };
    
    // Configure for parallel execution
    let config = ExecutionConfig {
        enable_parallel: true,
        max_execution_time_seconds: Some(60),
        max_steps: Some(100),
        ..Default::default()
    };
    
    // Build the parallel processing graph
    let graph = GraphBuilder::new()
        .with_config(config)
        .add_node("initialize".to_string(), InitializeDataNode)?
        .add_node("math_square".to_string(), MathProcessorNode::new("square"))?
        .add_node("math_double".to_string(), MathProcessorNode::new("double"))?
        .add_node("math_increment".to_string(), MathProcessorNode::new("increment"))?
        .add_node("statistics".to_string(), StatisticsNode)?
        .add_node("filter_even".to_string(), FilterNode::new("even"))?
        .add_node("filter_odd".to_string(), FilterNode::new("odd"))?
        .add_node("filter_large".to_string(), FilterNode::new("large"))?
        .add_node("aggregate".to_string(), AggregateResultsNode)?
        .with_entry_point("initialize".to_string())?
        // Sequential initialization
        .add_edge(Edge::simple("initialize", "math_square"))?
        // Parallel processing branches
        .add_edge(Edge::parallel("math_square", vec![
            "math_double".to_string(),
            "math_increment".to_string(),
            "statistics".to_string(),
            "filter_even".to_string(),
            "filter_odd".to_string(),
            "filter_large".to_string(),
        ]))?
        // All parallel branches converge to aggregation
        .add_edge(Edge::simple("math_double", "aggregate"))?
        .add_edge(Edge::simple("math_increment", "aggregate"))?
        .add_edge(Edge::simple("statistics", "aggregate"))?
        .add_edge(Edge::simple("filter_even", "aggregate"))?
        .add_edge(Edge::simple("filter_odd", "aggregate"))?
        .add_edge(Edge::simple("filter_large", "aggregate"))?
        .add_finish_point("aggregate".to_string())?
        .build()?;
    
    println!("\n📋 Graph Summary: {}", graph.summary());
    println!("🔧 Parallel execution enabled: {}", graph.config().enable_parallel);
    
    // Execute the parallel processing graph
    println!("\n🔄 Starting parallel processing execution...\n");
    let start_time = std::time::Instant::now();
    
    let context = graph.run(&mut state).await?;
    
    let execution_time = start_time.elapsed();
    
    println!("\n🎉 Parallel processing completed successfully!");
    println!("⏱️  Total execution time: {:?}", execution_time);
    println!("📊 Execution steps: {}", context.current_step);
    println!("🛤️  Execution path: {:?}", context.execution_path);
    
    // Display results
    println!("\n📈 Processing Results:");
    println!("  Input data size: {}", state.input_data.len());
    println!("  Parallel operations: {}", state.stats.parallel_operations);
    println!("  Total processed items: {}", state.stats.processed_items);
    
    println!("\n📊 Result Details:");
    for (operation, results) in &state.results {
        println!("  {}: {} items (first 5: {:?})", 
                operation, 
                results.len(), 
                results.iter().take(5).collect::<Vec<_>>());
    }
    
    // Calculate efficiency
    let theoretical_sequential_time = 500 + 500 + 500 + 300 + 200 + 200 + 200; // Sum of all parallel operations
    let actual_parallel_time = execution_time.as_millis() as u64;
    let speedup = theoretical_sequential_time as f64 / actual_parallel_time as f64;
    
    println!("\n⚡ Performance Analysis:");
    println!("  Theoretical sequential time: {}ms", theoretical_sequential_time);
    println!("  Actual parallel time: {}ms", actual_parallel_time);
    println!("  Speedup factor: {:.2}x", speedup);
    
    Ok(())
}
