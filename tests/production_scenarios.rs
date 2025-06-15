//! Production scenario tests for AgentGraph framework
//! 
//! These tests simulate real-world production scenarios to validate
//! the framework's behavior in actual deployment conditions.

use agent_graph::{
    Graph, GraphBuilder, Node, State, GraphResult, Edge, ExecutionConfig,
    node::traits::{RetryableNode, TimeoutNode},
    edge::conditions::FunctionCondition,
    state::checkpointing::MemoryCheckpointer,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, atomic::{AtomicU32, AtomicBool, Ordering}};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use std::collections::HashMap;
use uuid;

/// Production-like state for complex workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProductionState {
    // Data processing fields
    input_data: Vec<String>,
    processed_data: Vec<ProcessedItem>,
    analysis_results: HashMap<String, f64>,
    
    // Workflow tracking
    workflow_id: String,
    current_stage: String,
    completion_percentage: f32,
    
    // Error tracking
    errors: Vec<String>,
    warnings: Vec<String>,
    retry_counts: HashMap<String, u32>,
    
    // Performance metrics
    processing_time_ms: u64,
    items_processed: u32,
    throughput_items_per_sec: f64,
    
    // Business logic fields
    quality_score: f64,
    confidence_level: f64,
    recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProcessedItem {
    id: String,
    content: String,
    score: f64,
    metadata: HashMap<String, String>,
}



impl Default for ProductionState {
    fn default() -> Self {
        Self {
            input_data: Vec::new(),
            processed_data: Vec::new(),
            analysis_results: HashMap::new(),
            workflow_id: uuid::Uuid::new_v4().to_string(),
            current_stage: "initialized".to_string(),
            completion_percentage: 0.0,
            errors: Vec::new(),
            warnings: Vec::new(),
            retry_counts: HashMap::new(),
            processing_time_ms: 0,
            items_processed: 0,
            throughput_items_per_sec: 0.0,
            quality_score: 0.0,
            confidence_level: 0.0,
            recommendations: Vec::new(),
        }
    }
}

/// Data ingestion node that simulates loading data from external sources
#[derive(Debug)]
struct DataIngestionNode {
    source_name: String,
    data_size: usize,
    failure_rate: f32,
}

#[async_trait]
impl Node<ProductionState> for DataIngestionNode {
    async fn invoke(&self, state: &mut ProductionState) -> GraphResult<()> {
        state.current_stage = format!("ingesting_from_{}", self.source_name);
        
        // Simulate network delay
        sleep(Duration::from_millis(100 + (self.data_size / 10) as u64)).await;
        
        // Simulate potential failure
        if rand::random::<f32>() < self.failure_rate {
            let error_msg = format!("Failed to ingest data from {}", self.source_name);
            state.errors.push(error_msg.clone());
            return Err(agent_graph::GraphError::ExternalServiceError(error_msg));
        }
        
        // Generate mock data
        for i in 0..self.data_size {
            state.input_data.push(format!("{}_{}_item_{}", self.source_name, state.workflow_id, i));
        }
        
        state.completion_percentage = 20.0;
        Ok(())
    }
}

#[async_trait]
impl RetryableNode<ProductionState> for DataIngestionNode {
    fn max_retries(&self) -> u32 { 3 }
    fn retry_delay(&self) -> Duration { Duration::from_millis(500) }
}

/// Data processing node that transforms raw data
#[derive(Debug)]
struct DataProcessingNode {
    processing_type: String,
    batch_size: usize,
}

#[async_trait]
impl Node<ProductionState> for DataProcessingNode {
    async fn invoke(&self, state: &mut ProductionState) -> GraphResult<()> {
        state.current_stage = format!("processing_{}", self.processing_type);
        let start_time = Instant::now();
        
        // Process data in batches
        for chunk in state.input_data.chunks(self.batch_size) {
            for (i, item) in chunk.iter().enumerate() {
                // Simulate processing work
                sleep(Duration::from_millis(1)).await;
                
                let processed_item = ProcessedItem {
                    id: format!("processed_{}", i),
                    content: format!("processed_{}", item),
                    score: rand::random::<f64>() * 100.0,
                    metadata: {
                        let mut meta = HashMap::new();
                        meta.insert("processing_type".to_string(), self.processing_type.clone());
                        meta.insert("batch_id".to_string(), format!("batch_{}", i / self.batch_size));
                        meta
                    },
                };
                
                state.processed_data.push(processed_item);
                state.items_processed += 1;
            }
            
            // Update progress
            state.completion_percentage = 20.0 + (state.items_processed as f32 / state.input_data.len() as f32) * 40.0;
        }
        
        state.processing_time_ms += start_time.elapsed().as_millis() as u64;
        Ok(())
    }
}

/// Analysis node that performs complex analysis on processed data
#[derive(Debug)]
struct AnalysisNode {
    analysis_type: String,
    complexity_factor: f32,
}

#[async_trait]
impl Node<ProductionState> for AnalysisNode {
    async fn invoke(&self, state: &mut ProductionState) -> GraphResult<()> {
        state.current_stage = format!("analyzing_{}", self.analysis_type);
        
        if state.processed_data.is_empty() {
            state.warnings.push("No processed data available for analysis".to_string());
            return Ok(());
        }
        
        // Perform analysis
        let mut total_score = 0.0;
        let mut score_variance = 0.0;
        
        for item in &state.processed_data {
            total_score += item.score;
            
            // Simulate complex analysis work
            sleep(Duration::from_millis((self.complexity_factor * 10.0) as u64)).await;
        }
        
        let mean_score = total_score / state.processed_data.len() as f64;
        
        // Calculate variance
        for item in &state.processed_data {
            score_variance += (item.score - mean_score).powi(2);
        }
        score_variance /= state.processed_data.len() as f64;
        
        // Store analysis results
        state.analysis_results.insert("mean_score".to_string(), mean_score);
        state.analysis_results.insert("score_variance".to_string(), score_variance);
        state.analysis_results.insert("total_items".to_string(), state.processed_data.len() as f64);
        
        // Calculate quality metrics
        state.quality_score = mean_score / 100.0; // Normalize to 0-1
        state.confidence_level = 1.0 - (score_variance / 10000.0).min(1.0); // Higher variance = lower confidence
        
        state.completion_percentage = 80.0;
        Ok(())
    }
}

/// Recommendation engine node
#[derive(Debug)]
struct RecommendationNode {
    min_quality_threshold: f64,
    min_confidence_threshold: f64,
}

#[async_trait]
impl Node<ProductionState> for RecommendationNode {
    async fn invoke(&self, state: &mut ProductionState) -> GraphResult<()> {
        state.current_stage = "generating_recommendations".to_string();
        
        // Generate recommendations based on analysis
        if state.quality_score >= self.min_quality_threshold {
            state.recommendations.push("High quality data detected - proceed with deployment".to_string());
        } else {
            state.recommendations.push("Quality below threshold - review data processing pipeline".to_string());
        }
        
        if state.confidence_level >= self.min_confidence_threshold {
            state.recommendations.push("High confidence in results - safe to use for decision making".to_string());
        } else {
            state.recommendations.push("Low confidence - consider collecting more data".to_string());
        }
        
        // Performance recommendations
        if state.processing_time_ms > 5000 {
            state.recommendations.push("Processing time high - consider optimization".to_string());
        }
        
        if state.items_processed > 0 {
            state.throughput_items_per_sec = state.items_processed as f64 / (state.processing_time_ms as f64 / 1000.0);
            
            if state.throughput_items_per_sec < 10.0 {
                state.recommendations.push("Low throughput detected - investigate bottlenecks".to_string());
            }
        }
        
        state.completion_percentage = 100.0;
        state.current_stage = "completed".to_string();
        Ok(())
    }
}

/// Notification node that simulates sending alerts/notifications
#[derive(Debug)]
struct NotificationNode {
    notification_type: String,
    delivery_delay_ms: u64,
}

#[async_trait]
impl Node<ProductionState> for NotificationNode {
    async fn invoke(&self, state: &mut ProductionState) -> GraphResult<()> {
        // Simulate notification delivery delay
        sleep(Duration::from_millis(self.delivery_delay_ms)).await;
        
        // Log notification (in production, this would send actual notifications)
        println!("📧 {} Notification sent for workflow {}", self.notification_type, state.workflow_id);
        println!("   Quality Score: {:.2}", state.quality_score);
        println!("   Confidence: {:.2}", state.confidence_level);
        println!("   Items Processed: {}", state.items_processed);
        println!("   Recommendations: {}", state.recommendations.len());
        
        Ok(())
    }
}

// Production Scenario 1: Data Processing Pipeline
#[tokio::test]
async fn test_data_processing_pipeline() {
    let config = ExecutionConfig {
        enable_parallel: true,
        max_execution_time_seconds: Some(30),
        enable_checkpointing: true,
        ..Default::default()
    };

    let mut graph = GraphBuilder::new()
        .with_config(config)
        // Data ingestion from multiple sources
        .add_node("ingest_api".to_string(), DataIngestionNode { 
            source_name: "api".to_string(), 
            data_size: 100, 
            failure_rate: 0.1 
        }).unwrap()
        .add_node("ingest_db".to_string(), DataIngestionNode { 
            source_name: "database".to_string(), 
            data_size: 150, 
            failure_rate: 0.05 
        }).unwrap()
        .add_node("ingest_files".to_string(), DataIngestionNode { 
            source_name: "files".to_string(), 
            data_size: 75, 
            failure_rate: 0.15 
        }).unwrap()
        // Data processing
        .add_node("process_data".to_string(), DataProcessingNode { 
            processing_type: "nlp".to_string(), 
            batch_size: 50 
        }).unwrap()
        // Analysis
        .add_node("analyze_quality".to_string(), AnalysisNode { 
            analysis_type: "quality".to_string(), 
            complexity_factor: 1.5 
        }).unwrap()
        .add_node("analyze_sentiment".to_string(), AnalysisNode { 
            analysis_type: "sentiment".to_string(), 
            complexity_factor: 2.0 
        }).unwrap()
        // Recommendations and notifications
        .add_node("generate_recommendations".to_string(), RecommendationNode { 
            min_quality_threshold: 0.7, 
            min_confidence_threshold: 0.8 
        }).unwrap()
        .add_node("notify_success".to_string(), NotificationNode { 
            notification_type: "Success".to_string(), 
            delivery_delay_ms: 100 
        }).unwrap()
        .add_node("notify_failure".to_string(), NotificationNode { 
            notification_type: "Failure".to_string(), 
            delivery_delay_ms: 50 
        }).unwrap()
        // Build the pipeline
        .add_edge(Edge::parallel("ingest_api", vec![
            "ingest_db".to_string(), 
            "ingest_files".to_string()
        ])).unwrap()
        .add_edge(Edge::simple("ingest_db", "process_data")).unwrap()
        .add_edge(Edge::simple("ingest_files", "process_data")).unwrap()
        .add_edge(Edge::parallel("process_data", vec![
            "analyze_quality".to_string(), 
            "analyze_sentiment".to_string()
        ])).unwrap()
        .add_edge(Edge::simple("analyze_quality", "generate_recommendations")).unwrap()
        .add_edge(Edge::simple("analyze_sentiment", "generate_recommendations")).unwrap()
        .add_edge(Edge::conditional(
            "generate_recommendations", 
            "quality_check".to_string(), 
            "notify_success", 
            "notify_failure"
        )).unwrap()
        .with_entry_point("ingest_api".to_string()).unwrap()
        .add_finish_point("notify_success".to_string()).unwrap()
        .add_finish_point("notify_failure".to_string()).unwrap()
        .build().unwrap();

    // Add conditional logic
    let quality_condition = FunctionCondition::new("quality_check", |state: &ProductionState| {
        state.quality_score >= 0.7 && state.confidence_level >= 0.8
    });
    graph.edge_registry_mut().register_condition(quality_condition);

    // Add checkpointing
    let checkpointer = MemoryCheckpointer::new();
    graph.set_checkpointer(checkpointer);

    let mut state = ProductionState::default();
    let start = Instant::now();
    let context = graph.run(&mut state).await.unwrap();
    let duration = start.elapsed();

    // Verify pipeline execution
    assert!(state.items_processed > 0);
    assert!(!state.processed_data.is_empty());
    assert!(!state.analysis_results.is_empty());
    assert!(!state.recommendations.is_empty());
    assert_eq!(state.completion_percentage, 100.0);
    assert_eq!(state.current_stage, "completed");
    
    println!("Data Processing Pipeline Results:");
    println!("  Execution time: {:?}", duration);
    println!("  Items processed: {}", state.items_processed);
    println!("  Quality score: {:.2}", state.quality_score);
    println!("  Confidence level: {:.2}", state.confidence_level);
    println!("  Throughput: {:.2} items/sec", state.throughput_items_per_sec);
    println!("  Recommendations: {}", state.recommendations.len());
    println!("  Errors: {}", state.errors.len());
    println!("  Warnings: {}", state.warnings.len());
}

// Production Scenario 2: Real-time Event Processing
#[tokio::test]
async fn test_realtime_event_processing() {
    // Simulate high-frequency event processing
    let event_count = 1000;
    let batch_size = 50;
    
    let config = ExecutionConfig {
        enable_parallel: true,
        max_execution_time_seconds: Some(60),
        ..Default::default()
    };

    let graph = GraphBuilder::new()
        .with_config(config)
        .add_node("ingest_events".to_string(), DataIngestionNode { 
            source_name: "event_stream".to_string(), 
            data_size: batch_size, 
            failure_rate: 0.02 
        }).unwrap()
        .add_node("process_events".to_string(), DataProcessingNode { 
            processing_type: "realtime".to_string(), 
            batch_size: 10 
        }).unwrap()
        .add_node("analyze_events".to_string(), AnalysisNode { 
            analysis_type: "realtime".to_string(), 
            complexity_factor: 0.5 
        }).unwrap()
        .add_edge(Edge::simple("ingest_events", "process_events")).unwrap()
        .add_edge(Edge::simple("process_events", "analyze_events")).unwrap()
        .with_entry_point("ingest_events".to_string()).unwrap()
        .add_finish_point("analyze_events".to_string()).unwrap()
        .build().unwrap();

    let mut total_events_processed = 0;
    let mut total_execution_time = Duration::new(0, 0);
    let start_time = Instant::now();

    // Process events in batches
    for batch in 0..(event_count / batch_size) {
        let mut state = ProductionState::default();
        state.workflow_id = format!("batch_{}", batch);
        
        let batch_start = Instant::now();
        let context = graph.run(&mut state).await.unwrap();
        let batch_duration = batch_start.elapsed();
        
        total_events_processed += state.items_processed;
        total_execution_time += batch_duration;
        
        // Verify batch processing
        assert!(state.items_processed > 0);
        assert!(!state.processed_data.is_empty());
        
        if batch % 5 == 0 {
            println!("Processed batch {}: {} events in {:?}", 
                    batch, state.items_processed, batch_duration);
        }
    }

    let total_time = start_time.elapsed();
    let overall_throughput = total_events_processed as f64 / total_time.as_secs_f64();
    
    println!("Real-time Event Processing Results:");
    println!("  Total events processed: {}", total_events_processed);
    println!("  Total execution time: {:?}", total_time);
    println!("  Average batch time: {:?}", total_execution_time / (event_count / batch_size) as u32);
    println!("  Overall throughput: {:.2} events/sec", overall_throughput);
    
    // Verify performance requirements
    assert!(overall_throughput > 100.0, "Throughput too low: {:.2} events/sec", overall_throughput);
    assert!(total_time.as_secs() < 30, "Processing took too long: {:?}", total_time);
}

// Production Scenario 3: Fault-Tolerant Workflow
#[tokio::test]
async fn test_fault_tolerant_workflow() {
    let config = ExecutionConfig {
        enable_parallel: false, // Sequential for easier fault injection
        max_execution_time_seconds: Some(45),
        max_retries: 3,
        stop_on_error: false, // Continue on errors
        ..Default::default()
    };

    let graph = GraphBuilder::new()
        .with_config(config)
        // High failure rate nodes to test fault tolerance
        .add_node("unreliable_source".to_string(), DataIngestionNode { 
            source_name: "unreliable".to_string(), 
            data_size: 50, 
            failure_rate: 0.7 // High failure rate
        }).unwrap()
        .add_node("backup_source".to_string(), DataIngestionNode { 
            source_name: "backup".to_string(), 
            data_size: 30, 
            failure_rate: 0.1 // Low failure rate
        }).unwrap()
        .add_node("process_data".to_string(), DataProcessingNode { 
            processing_type: "fault_tolerant".to_string(), 
            batch_size: 25 
        }).unwrap()
        .add_node("analyze_data".to_string(), AnalysisNode { 
            analysis_type: "robust".to_string(), 
            complexity_factor: 1.0 
        }).unwrap()
        .add_edge(Edge::simple("unreliable_source", "backup_source")).unwrap() // Fallback pattern
        .add_edge(Edge::simple("backup_source", "process_data")).unwrap()
        .add_edge(Edge::simple("process_data", "analyze_data")).unwrap()
        .with_entry_point("unreliable_source".to_string()).unwrap()
        .add_finish_point("analyze_data".to_string()).unwrap()
        .build().unwrap();

    let mut state = ProductionState::default();
    let start = Instant::now();
    
    // This might fail due to high failure rate, but should demonstrate fault tolerance
    let result = graph.run(&mut state).await;
    let duration = start.elapsed();
    
    println!("Fault-Tolerant Workflow Results:");
    println!("  Execution time: {:?}", duration);
    println!("  Success: {}", result.is_ok());
    println!("  Items processed: {}", state.items_processed);
    println!("  Errors encountered: {}", state.errors.len());
    println!("  Warnings: {}", state.warnings.len());
    println!("  Current stage: {}", state.current_stage);
    
    // Even if the workflow fails, we should have captured error information
    if result.is_err() {
        assert!(!state.errors.is_empty(), "Should have recorded errors");
        println!("  Error details: {:?}", state.errors);
    } else {
        // If successful, verify data was processed
        assert!(state.items_processed > 0);
        assert!(!state.processed_data.is_empty());
    }
}
