//! Stress tests for AgentGraph framework
//! 
//! These tests validate system behavior under extreme conditions
//! and high load scenarios to ensure production reliability.

use agent_graph::{
    Graph, GraphBuilder, Node, State, GraphResult, Edge, ExecutionConfig,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Heavy computation state for stress testing
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StressTestState {
    computation_result: u64,
    memory_data: Vec<u8>,
    iteration_count: u64,
    total_operations: u64,
}



impl Default for StressTestState {
    fn default() -> Self {
        Self {
            computation_result: 0,
            memory_data: Vec::new(),
            iteration_count: 0,
            total_operations: 0,
        }
    }
}

/// CPU-intensive computation node
#[derive(Debug)]
struct CpuIntensiveNode {
    id: String,
    iterations: u64,
}

#[async_trait]
impl Node<StressTestState> for CpuIntensiveNode {
    async fn invoke(&self, state: &mut StressTestState) -> GraphResult<()> {
        // Perform CPU-intensive computation
        let mut result = 0u64;
        for i in 0..self.iterations {
            result = result.wrapping_add(i * i);
            
            // Yield occasionally to prevent blocking
            if i % 10000 == 0 {
                tokio::task::yield_now().await;
            }
        }
        
        state.computation_result = state.computation_result.wrapping_add(result);
        state.iteration_count += self.iterations;
        state.total_operations += 1;
        
        Ok(())
    }

    fn metadata(&self) -> agent_graph::NodeMetadata {
        agent_graph::NodeMetadata::new(&self.id)
            .with_description(&format!("CPU-intensive computation with {} iterations", self.iterations))
            .with_tag("cpu_intensive")
            .with_expected_duration((self.iterations / 1000) as u64) // Rough estimate
    }
}

/// Memory-intensive node that allocates large amounts of data
#[derive(Debug)]
struct MemoryIntensiveNode {
    id: String,
    memory_size_mb: usize,
}

#[async_trait]
impl Node<StressTestState> for MemoryIntensiveNode {
    async fn invoke(&self, state: &mut StressTestState) -> GraphResult<()> {
        // Allocate large amount of memory
        let size_bytes = self.memory_size_mb * 1024 * 1024;
        let mut data = vec![0u8; size_bytes];
        
        // Fill with some pattern to ensure allocation
        for (i, byte) in data.iter_mut().enumerate() {
            *byte = (i % 256) as u8;
        }
        
        // Store some of the data in state
        state.memory_data.extend_from_slice(&data[..1024.min(size_bytes)]);
        state.total_operations += 1;
        
        // Simulate some work with the data
        let checksum: u64 = data.iter().map(|&b| b as u64).sum();
        state.computation_result = state.computation_result.wrapping_add(checksum);
        
        Ok(())
    }

    fn metadata(&self) -> agent_graph::NodeMetadata {
        agent_graph::NodeMetadata::new(&self.id)
            .with_description(&format!("Memory-intensive operation allocating {}MB", self.memory_size_mb))
            .with_tag("memory_intensive")
    }
}

/// I/O intensive node that simulates file operations
#[derive(Debug)]
struct IoIntensiveNode {
    id: String,
    io_operations: u32,
}

#[async_trait]
impl Node<StressTestState> for IoIntensiveNode {
    async fn invoke(&self, state: &mut StressTestState) -> GraphResult<()> {
        // Simulate I/O operations with delays
        for i in 0..self.io_operations {
            // Simulate file read/write delay
            sleep(Duration::from_millis(1)).await;
            
            // Simulate some data processing
            state.computation_result = state.computation_result.wrapping_add(i as u64);
        }
        
        state.total_operations += 1;
        Ok(())
    }

    fn metadata(&self) -> agent_graph::NodeMetadata {
        agent_graph::NodeMetadata::new(&self.id)
            .with_description(&format!("I/O-intensive operation with {} operations", self.io_operations))
            .with_tag("io_intensive")
            .with_expected_duration(self.io_operations as u64)
    }
}

/// Concurrent execution counter for tracking parallel operations
static CONCURRENT_EXECUTIONS: AtomicU64 = AtomicU64::new(0);

/// Node that tracks concurrent executions
#[derive(Debug)]
struct ConcurrencyTestNode {
    id: String,
    hold_duration_ms: u64,
}

#[async_trait]
impl Node<StressTestState> for ConcurrencyTestNode {
    async fn invoke(&self, state: &mut StressTestState) -> GraphResult<()> {
        let current_count = CONCURRENT_EXECUTIONS.fetch_add(1, Ordering::SeqCst);
        
        // Hold the execution for specified duration
        sleep(Duration::from_millis(self.hold_duration_ms)).await;
        
        state.computation_result = state.computation_result.wrapping_add(current_count);
        state.total_operations += 1;
        
        CONCURRENT_EXECUTIONS.fetch_sub(1, Ordering::SeqCst);
        Ok(())
    }

    fn metadata(&self) -> agent_graph::NodeMetadata {
        agent_graph::NodeMetadata::new(&self.id)
            .with_description("Tests concurrent execution tracking")
            .with_tag("concurrency_test")
    }
}

// Stress Test 1: High CPU Load
#[tokio::test]
async fn test_high_cpu_load() {
    let graph = GraphBuilder::new()
        .add_node("cpu1".to_string(), CpuIntensiveNode { id: "cpu1".to_string(), iterations: 1_000_000 }).unwrap()
        .add_node("cpu2".to_string(), CpuIntensiveNode { id: "cpu2".to_string(), iterations: 1_000_000 }).unwrap()
        .add_node("cpu3".to_string(), CpuIntensiveNode { id: "cpu3".to_string(), iterations: 1_000_000 }).unwrap()
        .add_edge(Edge::simple("cpu1", "cpu2")).unwrap()
        .add_edge(Edge::simple("cpu2", "cpu3")).unwrap()
        .with_entry_point("cpu1".to_string()).unwrap()
        .add_finish_point("cpu3".to_string()).unwrap()
        .build().unwrap();

    let mut state = StressTestState::default();
    let start = Instant::now();
    let context = graph.run(&mut state).await.unwrap();
    let duration = start.elapsed();

    // Verify all computations completed
    assert_eq!(state.total_operations, 3);
    assert_eq!(state.iteration_count, 3_000_000);
    assert!(state.computation_result > 0);
    assert_eq!(context.current_step, 3);
    
    println!("High CPU load test completed in {:?}", duration);
    println!("Total iterations: {}", state.iteration_count);
    println!("Computation result: {}", state.computation_result);
}

// Stress Test 2: High Memory Usage
#[tokio::test]
async fn test_high_memory_usage() {
    let graph = GraphBuilder::new()
        .add_node("mem1".to_string(), MemoryIntensiveNode { id: "mem1".to_string(), memory_size_mb: 50 }).unwrap()
        .add_node("mem2".to_string(), MemoryIntensiveNode { id: "mem2".to_string(), memory_size_mb: 50 }).unwrap()
        .add_node("mem3".to_string(), MemoryIntensiveNode { id: "mem3".to_string(), memory_size_mb: 50 }).unwrap()
        .add_edge(Edge::simple("mem1", "mem2")).unwrap()
        .add_edge(Edge::simple("mem2", "mem3")).unwrap()
        .with_entry_point("mem1".to_string()).unwrap()
        .add_finish_point("mem3".to_string()).unwrap()
        .build().unwrap();

    let mut state = StressTestState::default();
    let start = Instant::now();
    let context = graph.run(&mut state).await.unwrap();
    let duration = start.elapsed();

    // Verify all memory operations completed
    assert_eq!(state.total_operations, 3);
    assert!(state.memory_data.len() >= 3072); // At least 3KB from 3 nodes
    assert!(state.computation_result > 0);
    assert_eq!(context.current_step, 3);
    
    println!("High memory usage test completed in {:?}", duration);
    println!("Memory data size: {} bytes", state.memory_data.len());
    println!("Computation result: {}", state.computation_result);
}

// Stress Test 3: High I/O Load
#[tokio::test]
async fn test_high_io_load() {
    let config = ExecutionConfig {
        enable_parallel: true,
        ..Default::default()
    };

    let graph = GraphBuilder::new()
        .with_config(config)
        .add_node("init".to_string(), CpuIntensiveNode { id: "init".to_string(), iterations: 1000 }).unwrap()
        .add_node("io1".to_string(), IoIntensiveNode { id: "io1".to_string(), io_operations: 100 }).unwrap()
        .add_node("io2".to_string(), IoIntensiveNode { id: "io2".to_string(), io_operations: 100 }).unwrap()
        .add_node("io3".to_string(), IoIntensiveNode { id: "io3".to_string(), io_operations: 100 }).unwrap()
        .add_node("io4".to_string(), IoIntensiveNode { id: "io4".to_string(), io_operations: 100 }).unwrap()
        .add_edge(Edge::simple("init", "io1")).unwrap()
        .add_edge(Edge::parallel("io1", vec!["io2".to_string(), "io3".to_string(), "io4".to_string()])).unwrap()
        .with_entry_point("init".to_string()).unwrap()
        .add_finish_point("io2".to_string()).unwrap()
        .add_finish_point("io3".to_string()).unwrap()
        .add_finish_point("io4".to_string()).unwrap()
        .build().unwrap();

    let mut state = StressTestState::default();
    let start = Instant::now();
    let context = graph.run(&mut state).await.unwrap();
    let duration = start.elapsed();

    // Verify all I/O operations completed
    assert!(state.total_operations >= 4); // At least init + 3 I/O nodes
    assert!(state.computation_result > 0);
    assert!(context.current_step >= 4);
    
    // Parallel I/O should be faster than sequential
    assert!(duration.as_millis() < 500, "Parallel I/O took too long: {:?}", duration);
    
    println!("High I/O load test completed in {:?}", duration);
    println!("Total operations: {}", state.total_operations);
}

// Stress Test 4: Massive Parallel Execution
#[tokio::test]
async fn test_massive_parallel_execution() {
    CONCURRENT_EXECUTIONS.store(0, Ordering::SeqCst);
    
    let config = ExecutionConfig {
        enable_parallel: true,
        max_execution_time_seconds: Some(30),
        ..Default::default()
    };

    let mut builder = GraphBuilder::new().with_config(config);
    
    // Create 50 parallel nodes
    let parallel_nodes: Vec<String> = (0..50)
        .map(|i| {
            let node_id = format!("parallel_{}", i);
            let node = ConcurrencyTestNode { 
                id: node_id.clone(), 
                hold_duration_ms: 100 
            };
            builder = builder.add_node(node_id.clone(), node).unwrap();
            node_id
        })
        .collect();

    let graph = builder
        .add_node("init".to_string(), CpuIntensiveNode { id: "init".to_string(), iterations: 1000 }).unwrap()
        .add_edge(Edge::parallel("init", parallel_nodes)).unwrap()
        .with_entry_point("init".to_string()).unwrap()
        .add_finish_point("parallel_0".to_string()).unwrap() // Just need one finish point
        .build().unwrap();

    let mut state = StressTestState::default();
    let start = Instant::now();
    let context = graph.run(&mut state).await.unwrap();
    let duration = start.elapsed();

    // Should complete much faster than sequential execution
    assert!(duration.as_millis() < 5000, "Massive parallel execution took too long: {:?}", duration);
    assert!(state.total_operations >= 50); // At least 50 parallel operations
    
    println!("Massive parallel execution completed in {:?}", duration);
    println!("Total operations: {}", state.total_operations);
    println!("Final concurrent count: {}", CONCURRENT_EXECUTIONS.load(Ordering::SeqCst));
}

// Stress Test 5: Rapid Sequential Executions
#[tokio::test]
async fn test_rapid_sequential_executions() {
    let graph = GraphBuilder::new()
        .add_node("quick".to_string(), CpuIntensiveNode { id: "quick".to_string(), iterations: 1000 }).unwrap()
        .with_entry_point("quick".to_string()).unwrap()
        .add_finish_point("quick".to_string()).unwrap()
        .build().unwrap();

    let execution_count = 1000;
    let start = Instant::now();
    
    for i in 0..execution_count {
        let mut state = StressTestState::default();
        let context = graph.run(&mut state).await.unwrap();
        
        assert_eq!(context.current_step, 1);
        assert_eq!(state.total_operations, 1);
        
        // Log progress every 100 executions
        if i % 100 == 0 {
            println!("Completed {} executions", i);
        }
    }
    
    let duration = start.elapsed();
    let executions_per_second = execution_count as f64 / duration.as_secs_f64();
    
    println!("Rapid sequential executions completed in {:?}", duration);
    println!("Executions per second: {:.2}", executions_per_second);
    
    // Should achieve reasonable throughput
    assert!(executions_per_second > 100.0, "Throughput too low: {:.2} exec/sec", executions_per_second);
}

// Stress Test 6: Mixed Workload Stress Test
#[tokio::test]
async fn test_mixed_workload_stress() {
    let config = ExecutionConfig {
        enable_parallel: true,
        max_execution_time_seconds: Some(60),
        ..Default::default()
    };

    let graph = GraphBuilder::new()
        .with_config(config)
        .add_node("cpu_heavy".to_string(), CpuIntensiveNode { id: "cpu_heavy".to_string(), iterations: 500_000 }).unwrap()
        .add_node("mem_heavy".to_string(), MemoryIntensiveNode { id: "mem_heavy".to_string(), memory_size_mb: 25 }).unwrap()
        .add_node("io_heavy".to_string(), IoIntensiveNode { id: "io_heavy".to_string(), io_operations: 50 }).unwrap()
        .add_node("cpu_light1".to_string(), CpuIntensiveNode { id: "cpu_light1".to_string(), iterations: 10_000 }).unwrap()
        .add_node("cpu_light2".to_string(), CpuIntensiveNode { id: "cpu_light2".to_string(), iterations: 10_000 }).unwrap()
        .add_node("cpu_light3".to_string(), CpuIntensiveNode { id: "cpu_light3".to_string(), iterations: 10_000 }).unwrap()
        .add_node("final".to_string(), CpuIntensiveNode { id: "final".to_string(), iterations: 1000 }).unwrap()
        // Sequential heavy operations
        .add_edge(Edge::simple("cpu_heavy", "mem_heavy")).unwrap()
        .add_edge(Edge::simple("mem_heavy", "io_heavy")).unwrap()
        // Parallel light operations
        .add_edge(Edge::parallel("io_heavy", vec![
            "cpu_light1".to_string(), 
            "cpu_light2".to_string(), 
            "cpu_light3".to_string()
        ])).unwrap()
        // Convergence
        .add_edge(Edge::simple("cpu_light1", "final")).unwrap()
        .add_edge(Edge::simple("cpu_light2", "final")).unwrap()
        .add_edge(Edge::simple("cpu_light3", "final")).unwrap()
        .with_entry_point("cpu_heavy".to_string()).unwrap()
        .add_finish_point("final".to_string()).unwrap()
        .build().unwrap();

    let mut state = StressTestState::default();
    let start = Instant::now();
    let context = graph.run(&mut state).await.unwrap();
    let duration = start.elapsed();

    // Verify all operations completed
    assert!(state.total_operations >= 6); // At least 6 different operations
    assert!(state.iteration_count > 500_000); // Heavy CPU work completed
    assert!(state.memory_data.len() >= 1024); // Memory work completed
    assert!(state.computation_result > 0);
    assert!(context.current_step >= 6);
    
    println!("Mixed workload stress test completed in {:?}", duration);
    println!("Total operations: {}", state.total_operations);
    println!("Total iterations: {}", state.iteration_count);
    println!("Memory data size: {} bytes", state.memory_data.len());
    println!("Computation result: {}", state.computation_result);
}

// Stress Test 7: Long-Running Graph Execution
#[tokio::test]
async fn test_long_running_execution() {
    let config = ExecutionConfig {
        max_execution_time_seconds: Some(120), // 2 minutes max
        ..Default::default()
    };

    let mut builder = GraphBuilder::new().with_config(config);
    
    // Create a long chain of lightweight operations
    for i in 0..200 {
        let node = CpuIntensiveNode { 
            id: format!("step_{}", i), 
            iterations: 5000 
        };
        builder = builder.add_node(format!("step_{}", i), node).unwrap();
        
        if i > 0 {
            builder = builder.add_edge(Edge::simple(
                format!("step_{}", i - 1), 
                format!("step_{}", i)
            )).unwrap();
        }
    }
    
    let graph = builder
        .with_entry_point("step_0".to_string()).unwrap()
        .add_finish_point("step_199".to_string()).unwrap()
        .build().unwrap();

    let mut state = StressTestState::default();
    let start = Instant::now();
    let context = graph.run(&mut state).await.unwrap();
    let duration = start.elapsed();

    // Verify long execution completed
    assert_eq!(state.total_operations, 200);
    assert_eq!(state.iteration_count, 1_000_000); // 200 * 5000
    assert_eq!(context.current_step, 200);
    assert_eq!(context.execution_path.len(), 200);
    
    println!("Long-running execution completed in {:?}", duration);
    println!("Average time per step: {:?}", duration / 200);
    println!("Total iterations: {}", state.iteration_count);
}

// Stress Test 8: Memory Pressure Test
#[tokio::test]
async fn test_memory_pressure() {
    let config = ExecutionConfig {
        enable_parallel: true,
        max_execution_time_seconds: Some(60),
        ..Default::default()
    };

    let graph = GraphBuilder::new()
        .with_config(config)
        .add_node("init".to_string(), CpuIntensiveNode { id: "init".to_string(), iterations: 1000 }).unwrap()
        .add_node("mem1".to_string(), MemoryIntensiveNode { id: "mem1".to_string(), memory_size_mb: 100 }).unwrap()
        .add_node("mem2".to_string(), MemoryIntensiveNode { id: "mem2".to_string(), memory_size_mb: 100 }).unwrap()
        .add_node("mem3".to_string(), MemoryIntensiveNode { id: "mem3".to_string(), memory_size_mb: 100 }).unwrap()
        .add_node("mem4".to_string(), MemoryIntensiveNode { id: "mem4".to_string(), memory_size_mb: 100 }).unwrap()
        .add_edge(Edge::simple("init", "mem1")).unwrap()
        .add_edge(Edge::parallel("mem1", vec![
            "mem2".to_string(), 
            "mem3".to_string(), 
            "mem4".to_string()
        ])).unwrap()
        .with_entry_point("init".to_string()).unwrap()
        .add_finish_point("mem2".to_string()).unwrap()
        .add_finish_point("mem3".to_string()).unwrap()
        .add_finish_point("mem4".to_string()).unwrap()
        .build().unwrap();

    let mut state = StressTestState::default();
    let start = Instant::now();
    let context = graph.run(&mut state).await.unwrap();
    let duration = start.elapsed();

    // Verify memory operations completed
    assert!(state.total_operations >= 4);
    assert!(state.memory_data.len() >= 4096); // At least 4KB from 4 nodes
    assert!(context.current_step >= 4);
    
    println!("Memory pressure test completed in {:?}", duration);
    println!("Total operations: {}", state.total_operations);
    println!("Memory data size: {} bytes", state.memory_data.len());
}
