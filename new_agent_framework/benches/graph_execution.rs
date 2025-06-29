use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use agent_graph_core::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::runtime::Runtime;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BenchmarkState {
    counter: i32,
    data: HashMap<String, String>,
    messages: Vec<String>,
}

impl State for BenchmarkState {}

impl Default for BenchmarkState {
    fn default() -> Self {
        Self {
            counter: 0,
            data: HashMap::new(),
            messages: Vec::new(),
        }
    }
}

// Simple increment node for benchmarking
#[derive(Debug)]
struct IncrementNode {
    id: String,
    increment_by: i32,
}

impl IncrementNode {
    fn new(id: String, increment_by: i32) -> Self {
        Self { id, increment_by }
    }
}

#[async_trait::async_trait]
impl Node<BenchmarkState> for IncrementNode {
    async fn execute(&self, state: &mut BenchmarkState) -> CoreResult<NodeOutput> {
        state.counter += self.increment_by;
        state.messages.push(format!("Incremented by {} in node {}", self.increment_by, self.id));
        
        // Simulate some work
        tokio::time::sleep(Duration::from_micros(10)).await;
        
        Ok(NodeOutput::success())
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn metadata(&self) -> &NodeMetadata {
        static METADATA: std::sync::OnceLock<NodeMetadata> = std::sync::OnceLock::new();
        METADATA.get_or_init(|| NodeMetadata {
            name: "Increment Node".to_string(),
            description: Some("Increments counter for benchmarking".to_string()),
            version: "1.0.0".to_string(),
            parallel_safe: true,
            expected_duration_ms: Some(1),
            tags: vec!["benchmark".to_string()],
            custom_properties: HashMap::new(),
        })
    }
}

// Data processing node for benchmarking
#[derive(Debug)]
struct DataProcessingNode {
    id: String,
    operations: usize,
}

impl DataProcessingNode {
    fn new(id: String, operations: usize) -> Self {
        Self { id, operations }
    }
}

#[async_trait::async_trait]
impl Node<BenchmarkState> for DataProcessingNode {
    async fn execute(&self, state: &mut BenchmarkState) -> CoreResult<NodeOutput> {
        // Simulate data processing
        for i in 0..self.operations {
            let key = format!("key_{}", i);
            let value = format!("processed_value_{}", i * state.counter);
            state.data.insert(key, value);
        }
        
        // Simulate async work
        tokio::time::sleep(Duration::from_micros(50)).await;
        
        Ok(NodeOutput::success())
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn metadata(&self) -> &NodeMetadata {
        static METADATA: std::sync::OnceLock<NodeMetadata> = std::sync::OnceLock::new();
        METADATA.get_or_init(|| NodeMetadata {
            name: "Data Processing Node".to_string(),
            description: Some("Processes data for benchmarking".to_string()),
            version: "1.0.0".to_string(),
            parallel_safe: true,
            expected_duration_ms: Some(5),
            tags: vec!["benchmark".to_string(), "data".to_string()],
            custom_properties: HashMap::new(),
        })
    }
}

fn create_simple_graph() -> CoreResult<Graph<BenchmarkState>> {
    let mut graph = GraphBuilder::new()
        .with_name("Benchmark Graph".to_string())
        .build()?;

    // Add nodes
    graph.add_node("increment1".to_string(), Box::new(IncrementNode::new("increment1".to_string(), 1)))?;
    graph.add_node("process1".to_string(), Box::new(DataProcessingNode::new("process1".to_string(), 10)))?;
    graph.add_node("increment2".to_string(), Box::new(IncrementNode::new("increment2".to_string(), 5)))?;

    // Add edges
    graph.add_edge("increment1".to_string(), "process1".to_string())?;
    graph.add_edge("process1".to_string(), "increment2".to_string())?;

    // Set entry and exit points
    graph.set_entry_point("increment1".to_string())?;
    graph.add_finish_point("increment2".to_string())?;

    Ok(graph)
}

fn create_complex_graph(num_nodes: usize) -> CoreResult<Graph<BenchmarkState>> {
    let mut graph = GraphBuilder::new()
        .with_name(format!("Complex Benchmark Graph ({})", num_nodes))
        .build()?;

    // Add nodes in a chain
    for i in 0..num_nodes {
        let node_id = format!("node_{}", i);
        if i % 2 == 0 {
            graph.add_node(node_id.clone(), Box::new(IncrementNode::new(node_id, 1)))?;
        } else {
            graph.add_node(node_id.clone(), Box::new(DataProcessingNode::new(node_id, 5)))?;
        }
    }

    // Add edges to create a chain
    for i in 0..num_nodes - 1 {
        let from = format!("node_{}", i);
        let to = format!("node_{}", i + 1);
        graph.add_edge(from, to)?;
    }

    // Set entry and exit points
    graph.set_entry_point("node_0".to_string())?;
    graph.add_finish_point(format!("node_{}", num_nodes - 1))?;

    Ok(graph)
}

fn bench_simple_graph_execution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("simple_graph_execution", |b| {
        b.to_async(&rt).iter(|| async {
            let graph = create_simple_graph().unwrap();
            let mut state = BenchmarkState::default();
            
            let result = graph.execute(black_box(&mut state)).await;
            black_box(result).unwrap();
        });
    });
}

fn bench_complex_graph_execution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("complex_graph_execution");
    
    for num_nodes in [5, 10, 20, 50].iter() {
        group.bench_with_input(
            BenchmarkId::new("nodes", num_nodes),
            num_nodes,
            |b, &num_nodes| {
                b.to_async(&rt).iter(|| async {
                    let graph = create_complex_graph(num_nodes).unwrap();
                    let mut state = BenchmarkState::default();
                    
                    let result = graph.execute(black_box(&mut state)).await;
                    black_box(result).unwrap();
                });
            },
        );
    }
    group.finish();
}

fn bench_state_management(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("state_snapshot_creation", |b| {
        b.to_async(&rt).iter(|| async {
            let state = BenchmarkState {
                counter: 1000,
                data: (0..100).map(|i| (format!("key_{}", i), format!("value_{}", i))).collect(),
                messages: (0..50).map(|i| format!("Message {}", i)).collect(),
            };
            
            let snapshot = StateSnapshot::new(black_box(state.clone()));
            black_box(snapshot);
        });
    });
}

fn bench_node_execution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("node_execution");
    
    group.bench_function("increment_node", |b| {
        b.to_async(&rt).iter(|| async {
            let node = IncrementNode::new("test".to_string(), 1);
            let mut state = BenchmarkState::default();
            
            let result = node.execute(black_box(&mut state)).await;
            black_box(result).unwrap();
        });
    });
    
    group.bench_function("data_processing_node", |b| {
        b.to_async(&rt).iter(|| async {
            let node = DataProcessingNode::new("test".to_string(), 20);
            let mut state = BenchmarkState::default();
            
            let result = node.execute(black_box(&mut state)).await;
            black_box(result).unwrap();
        });
    });
    
    group.finish();
}

fn bench_concurrent_execution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("concurrent_execution");
    
    for num_concurrent in [1, 2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_graphs", num_concurrent),
            num_concurrent,
            |b, &num_concurrent| {
                b.to_async(&rt).iter(|| async {
                    let mut handles = Vec::new();
                    
                    for _ in 0..num_concurrent {
                        let graph = create_simple_graph().unwrap();
                        let handle = tokio::spawn(async move {
                            let mut state = BenchmarkState::default();
                            graph.execute(&mut state).await
                        });
                        handles.push(handle);
                    }
                    
                    for handle in handles {
                        black_box(handle.await.unwrap()).unwrap();
                    }
                });
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_simple_graph_execution,
    bench_complex_graph_execution,
    bench_state_management,
    bench_node_execution,
    bench_concurrent_execution
);
criterion_main!(benches);