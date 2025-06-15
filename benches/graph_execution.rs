//! Performance benchmarks for AgentGraph framework

use agent_graph::{
    Graph, GraphBuilder, Node, State, GraphResult, Edge, ExecutionConfig,
};
use async_trait::async_trait;
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::runtime::Runtime;

/// Benchmark state for performance testing
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BenchState {
    value: i64,
    data: Vec<u8>,
    counter: u32,
}

impl State for BenchState {}

impl Default for BenchState {
    fn default() -> Self {
        Self {
            value: 0,
            data: Vec::new(),
            counter: 0,
        }
    }
}

/// Lightweight computation node for benchmarking
#[derive(Debug)]
struct LightComputeNode {
    id: String,
    work_amount: u32,
}

#[async_trait]
impl Node<BenchState> for LightComputeNode {
    async fn invoke(&self, state: &mut BenchState) -> GraphResult<()> {
        // Perform lightweight computation
        for i in 0..self.work_amount {
            state.value = state.value.wrapping_add(i as i64);
        }
        state.counter += 1;
        Ok(())
    }

    fn metadata(&self) -> agent_graph::NodeMetadata {
        agent_graph::NodeMetadata::new(&self.id)
            .with_description("Lightweight computation node")
            .with_tag("benchmark")
    }
}

/// Memory allocation node for benchmarking
#[derive(Debug)]
struct MemoryNode {
    id: String,
    allocation_size: usize,
}

#[async_trait]
impl Node<BenchState> for MemoryNode {
    async fn invoke(&self, state: &mut BenchState) -> GraphResult<()> {
        // Allocate and fill memory
        let mut data = vec![0u8; self.allocation_size];
        for (i, byte) in data.iter_mut().enumerate() {
            *byte = (i % 256) as u8;
        }
        
        // Store some data in state
        state.data.extend_from_slice(&data[..100.min(self.allocation_size)]);
        state.counter += 1;
        Ok(())
    }

    fn metadata(&self) -> agent_graph::NodeMetadata {
        agent_graph::NodeMetadata::new(&self.id)
            .with_description("Memory allocation node")
            .with_tag("benchmark")
    }
}

/// Async delay node for benchmarking
#[derive(Debug)]
struct DelayNode {
    id: String,
    delay_ms: u64,
}

#[async_trait]
impl Node<BenchState> for DelayNode {
    async fn invoke(&self, state: &mut BenchState) -> GraphResult<()> {
        tokio::time::sleep(Duration::from_millis(self.delay_ms)).await;
        state.counter += 1;
        Ok(())
    }

    fn metadata(&self) -> agent_graph::NodeMetadata {
        agent_graph::NodeMetadata::new(&self.id)
            .with_description("Async delay node")
            .with_tag("benchmark")
    }
}

/// Benchmark single node execution
fn bench_single_node_execution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("single_node_execution");
    
    for work_amount in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("light_compute", work_amount),
            work_amount,
            |b, &work_amount| {
                b.to_async(&rt).iter(|| async {
                    let graph = GraphBuilder::new()
                        .add_node("compute".to_string(), LightComputeNode {
                            id: "compute".to_string(),
                            work_amount,
                        }).unwrap()
                        .with_entry_point("compute".to_string()).unwrap()
                        .add_finish_point("compute".to_string()).unwrap()
                        .build().unwrap();
                    
                    let mut state = BenchState::default();
                    black_box(graph.run(&mut state).await.unwrap());
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark sequential graph execution
fn bench_sequential_execution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("sequential_execution");
    
    for node_count in [5, 10, 25, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("sequential_nodes", node_count),
            node_count,
            |b, &node_count| {
                b.to_async(&rt).iter(|| async {
                    let mut builder = GraphBuilder::new();
                    
                    // Create sequential chain
                    for i in 0..node_count {
                        let node = LightComputeNode {
                            id: format!("node_{}", i),
                            work_amount: 100,
                        };
                        builder = builder.add_node(format!("node_{}", i), node).unwrap();
                        
                        if i > 0 {
                            builder = builder.add_edge(Edge::simple(
                                format!("node_{}", i - 1),
                                format!("node_{}", i)
                            )).unwrap();
                        }
                    }
                    
                    let graph = builder
                        .with_entry_point("node_0".to_string()).unwrap()
                        .add_finish_point(format!("node_{}", node_count - 1)).unwrap()
                        .build().unwrap();
                    
                    let mut state = BenchState::default();
                    black_box(graph.run(&mut state).await.unwrap());
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark parallel graph execution
fn bench_parallel_execution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("parallel_execution");
    
    for node_count in [2, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::new("parallel_nodes", node_count),
            node_count,
            |b, &node_count| {
                b.to_async(&rt).iter(|| async {
                    let config = ExecutionConfig {
                        enable_parallel: true,
                        ..Default::default()
                    };
                    
                    let mut builder = GraphBuilder::new().with_config(config);
                    
                    // Create parallel structure
                    builder = builder.add_node("init".to_string(), LightComputeNode {
                        id: "init".to_string(),
                        work_amount: 50,
                    }).unwrap();
                    
                    let mut parallel_nodes = Vec::new();
                    for i in 0..node_count {
                        let node_id = format!("parallel_{}", i);
                        let node = LightComputeNode {
                            id: node_id.clone(),
                            work_amount: 100,
                        };
                        builder = builder.add_node(node_id.clone(), node).unwrap();
                        parallel_nodes.push(node_id);
                    }
                    
                    let graph = builder
                        .add_edge(Edge::parallel("init", parallel_nodes)).unwrap()
                        .with_entry_point("init".to_string()).unwrap()
                        .add_finish_point("parallel_0".to_string()).unwrap()
                        .build().unwrap();
                    
                    let mut state = BenchState::default();
                    black_box(graph.run(&mut state).await.unwrap());
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark memory allocation performance
fn bench_memory_allocation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("memory_allocation");
    
    for size_kb in [1, 10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("memory_alloc", size_kb),
            size_kb,
            |b, &size_kb| {
                b.to_async(&rt).iter(|| async {
                    let graph = GraphBuilder::new()
                        .add_node("memory".to_string(), MemoryNode {
                            id: "memory".to_string(),
                            allocation_size: size_kb * 1024,
                        }).unwrap()
                        .with_entry_point("memory".to_string()).unwrap()
                        .add_finish_point("memory".to_string()).unwrap()
                        .build().unwrap();
                    
                    let mut state = BenchState::default();
                    black_box(graph.run(&mut state).await.unwrap());
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark graph construction overhead
fn bench_graph_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph_construction");
    
    for node_count in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("construction", node_count),
            node_count,
            |b, &node_count| {
                b.iter(|| {
                    let mut builder = GraphBuilder::new();
                    
                    // Create nodes
                    for i in 0..node_count {
                        let node = LightComputeNode {
                            id: format!("node_{}", i),
                            work_amount: 10,
                        };
                        builder = builder.add_node(format!("node_{}", i), node).unwrap();
                        
                        if i > 0 {
                            builder = builder.add_edge(Edge::simple(
                                format!("node_{}", i - 1),
                                format!("node_{}", i)
                            )).unwrap();
                        }
                    }
                    
                    let graph = builder
                        .with_entry_point("node_0".to_string()).unwrap()
                        .add_finish_point(format!("node_{}", node_count - 1)).unwrap()
                        .build().unwrap();
                    
                    black_box(graph);
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark state serialization/deserialization
fn bench_state_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_serialization");
    
    for data_size in [100, 1000, 10000, 100000].iter() {
        group.bench_with_input(
            BenchmarkId::new("serialize", data_size),
            data_size,
            |b, &data_size| {
                let state = BenchState {
                    value: 12345,
                    data: vec![42u8; data_size],
                    counter: 100,
                };
                
                b.iter(|| {
                    let serialized = black_box(serde_json::to_string(&state).unwrap());
                    let _deserialized: BenchState = black_box(serde_json::from_str(&serialized).unwrap());
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_single_node_execution,
    bench_sequential_execution,
    bench_parallel_execution,
    bench_memory_allocation,
    bench_graph_construction,
    bench_state_serialization
);

criterion_main!(benches);
