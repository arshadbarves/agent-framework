//! Comprehensive integration tests for AgentGraph framework
//! 
//! These tests validate all major features and battle-tested scenarios
//! to ensure production readiness.

use agent_graph::{
    GraphBuilder, Node, State, GraphResult, Edge, ExecutionConfig,
    node::traits::{RetryableNode, TimeoutNode as TimeoutNodeTrait},
    edge::conditions::FunctionCondition,
    state::checkpointing::{FileCheckpointer, MemoryCheckpointer},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, atomic::{AtomicU32, AtomicBool, Ordering}};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tempfile::TempDir;

/// Test state for integration tests
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestState {
    value: i32,
    log: Vec<String>,
    metadata: std::collections::HashMap<String, String>,
    step_count: u32,
    error_count: u32,
}



impl Default for TestState {
    fn default() -> Self {
        Self {
            value: 0,
            log: Vec::new(),
            metadata: std::collections::HashMap::new(),
            step_count: 0,
            error_count: 0,
        }
    }
}

/// Basic increment node for testing
#[derive(Debug)]
struct IncrementNode {
    id: String,
    amount: i32,
}

#[async_trait]
impl Node<TestState> for IncrementNode {
    async fn invoke(&self, state: &mut TestState) -> GraphResult<()> {
        state.value += self.amount;
        state.log.push(format!("Node {} incremented by {}", self.id, self.amount));
        state.step_count += 1;
        Ok(())
    }

    fn metadata(&self) -> agent_graph::NodeMetadata {
        agent_graph::NodeMetadata::new(&self.id)
            .with_description(&format!("Increments value by {}", self.amount))
            .with_tag("increment")
    }
}

/// Node that simulates processing delay
#[derive(Debug)]
struct DelayNode {
    id: String,
    delay_ms: u64,
    work_amount: i32,
}

#[async_trait]
impl Node<TestState> for DelayNode {
    async fn invoke(&self, state: &mut TestState) -> GraphResult<()> {
        sleep(Duration::from_millis(self.delay_ms)).await;
        state.value += self.work_amount;
        state.log.push(format!("Node {} completed after {}ms", self.id, self.delay_ms));
        state.step_count += 1;
        Ok(())
    }

    fn metadata(&self) -> agent_graph::NodeMetadata {
        agent_graph::NodeMetadata::new(&self.id)
            .with_description(&format!("Simulates {}ms processing delay", self.delay_ms))
            .with_tag("delay")
            .with_expected_duration(self.delay_ms)
    }
}

/// Flaky node that fails intermittently for testing retry logic
#[derive(Debug)]
struct FlakeyNode {
    id: String,
    fail_count: Arc<AtomicU32>,
    max_failures: u32,
}

impl FlakeyNode {
    fn new(id: String, max_failures: u32) -> Self {
        Self {
            id,
            fail_count: Arc::new(AtomicU32::new(0)),
            max_failures,
        }
    }
}

#[async_trait]
impl Node<TestState> for FlakeyNode {
    async fn invoke(&self, state: &mut TestState) -> GraphResult<()> {
        let current_failures = self.fail_count.fetch_add(1, Ordering::SeqCst);
        
        if current_failures < self.max_failures {
            state.error_count += 1;
            return Err(agent_graph::GraphError::ExternalServiceError(
                format!("Simulated failure {} from node {}", current_failures + 1, self.id)
            ));
        }
        
        state.value += 100;
        state.log.push(format!("Node {} succeeded after {} failures", self.id, current_failures));
        state.step_count += 1;
        Ok(())
    }
}

#[async_trait]
impl RetryableNode<TestState> for FlakeyNode {
    fn max_retries(&self) -> u32 {
        self.max_failures + 1
    }

    fn retry_delay(&self) -> Duration {
        Duration::from_millis(50)
    }

    fn is_retryable_error(&self, error: &agent_graph::GraphError) -> bool {
        matches!(error, agent_graph::GraphError::ExternalServiceError(_))
    }
}

/// Node that can timeout for testing timeout handling
#[derive(Debug)]
struct TimeoutTestNode {
    id: String,
    should_timeout: Arc<AtomicBool>,
}

impl TimeoutTestNode {
    fn new(id: String) -> Self {
        Self {
            id,
            should_timeout: Arc::new(AtomicBool::new(false)),
        }
    }

    fn set_timeout(&self, should_timeout: bool) {
        self.should_timeout.store(should_timeout, Ordering::SeqCst);
    }
}

#[async_trait]
impl Node<TestState> for TimeoutTestNode {
    async fn invoke(&self, state: &mut TestState) -> GraphResult<()> {
        if self.should_timeout.load(Ordering::SeqCst) {
            // Sleep longer than timeout to trigger timeout
            sleep(Duration::from_secs(2)).await;
        } else {
            sleep(Duration::from_millis(100)).await;
        }
        
        state.value += 50;
        state.log.push(format!("Node {} completed", self.id));
        state.step_count += 1;
        Ok(())
    }
}

#[async_trait]
impl TimeoutNodeTrait<TestState> for TimeoutTestNode {
    fn timeout(&self) -> Duration {
        Duration::from_secs(1)
    }
}

/// Conditional node for testing conditional routing
#[derive(Debug)]
struct ConditionalNode {
    id: String,
    threshold: i32,
}

#[async_trait]
impl Node<TestState> for ConditionalNode {
    async fn invoke(&self, state: &mut TestState) -> GraphResult<()> {
        state.log.push(format!("Node {} checking condition (value: {} vs threshold: {})", 
                              self.id, state.value, self.threshold));
        state.step_count += 1;
        Ok(())
    }
}

// Test 1: Basic Graph Construction and Execution
#[tokio::test]
async fn test_basic_graph_execution() {
    let graph = GraphBuilder::new()
        .add_node("A".to_string(), IncrementNode { id: "A".to_string(), amount: 10 }).unwrap()
        .add_node("B".to_string(), IncrementNode { id: "B".to_string(), amount: 20 }).unwrap()
        .add_node("C".to_string(), IncrementNode { id: "C".to_string(), amount: 30 }).unwrap()
        .add_edge(Edge::simple("A", "B")).unwrap()
        .add_edge(Edge::simple("B", "C")).unwrap()
        .with_entry_point("A".to_string()).unwrap()
        .add_finish_point("C".to_string()).unwrap()
        .build().unwrap();

    let mut state = TestState::default();
    let context = graph.run(&mut state).await.unwrap();

    // Verify results
    assert_eq!(state.value, 60); // 10 + 20 + 30
    assert_eq!(context.execution_path, vec!["A", "B", "C"]);
    assert_eq!(context.current_step, 3);
    assert_eq!(state.step_count, 3);
    assert_eq!(state.log.len(), 3);
}

// Test 2: Parallel Execution Performance
#[tokio::test]
async fn test_parallel_execution_performance() {
    let config = ExecutionConfig {
        enable_parallel: true,
        ..Default::default()
    };

    let graph = GraphBuilder::new()
        .with_config(config)
        .add_node("init".to_string(), IncrementNode { id: "init".to_string(), amount: 1 }).unwrap()
        .add_node("task1".to_string(), DelayNode { id: "task1".to_string(), delay_ms: 200, work_amount: 10 }).unwrap()
        .add_node("task2".to_string(), DelayNode { id: "task2".to_string(), delay_ms: 200, work_amount: 20 }).unwrap()
        .add_node("task3".to_string(), DelayNode { id: "task3".to_string(), delay_ms: 200, work_amount: 30 }).unwrap()
        .add_node("final".to_string(), IncrementNode { id: "final".to_string(), amount: 1 }).unwrap()
        .add_edge(Edge::simple("init", "task1")).unwrap()
        .add_edge(Edge::parallel("task1", vec!["task2".to_string(), "task3".to_string()])).unwrap()
        .add_edge(Edge::simple("task2", "final")).unwrap()
        .add_edge(Edge::simple("task3", "final")).unwrap()
        .with_entry_point("init".to_string()).unwrap()
        .add_finish_point("final".to_string()).unwrap()
        .build().unwrap();

    let mut state = TestState::default();
    let start = Instant::now();
    let context = graph.run(&mut state).await.unwrap();
    let duration = start.elapsed();

    // Should complete in ~400ms (2 sequential + 1 parallel) not ~800ms (all sequential)
    assert!(duration.as_millis() < 600, "Parallel execution took too long: {:?}", duration);
    assert!(state.value > 50); // All tasks should complete
    assert!(context.current_step >= 4);
}

// Test 3: Retry Logic with Flaky Nodes
#[tokio::test]
async fn test_retry_logic() {
    let flaky_node = FlakeyNode::new("flaky".to_string(), 2); // Fail 2 times, then succeed

    let graph = GraphBuilder::new()
        .add_node("start".to_string(), IncrementNode { id: "start".to_string(), amount: 5 }).unwrap()
        .add_node("flaky".to_string(), flaky_node).unwrap()
        .add_node("end".to_string(), IncrementNode { id: "end".to_string(), amount: 5 }).unwrap()
        .add_edge(Edge::simple("start", "flaky")).unwrap()
        .add_edge(Edge::simple("flaky", "end")).unwrap()
        .with_entry_point("start".to_string()).unwrap()
        .add_finish_point("end".to_string()).unwrap()
        .build().unwrap();

    let mut state = TestState::default();
    let context = graph.run(&mut state).await.unwrap();

    // Should succeed after retries
    assert_eq!(state.value, 110); // 5 + 100 + 5
    assert_eq!(state.error_count, 2); // 2 failures before success
    assert!(context.current_step >= 3);
}

// Test 4: Timeout Handling
#[tokio::test]
async fn test_timeout_handling() {
    let timeout_node = TimeoutTestNode::new("timeout".to_string());
    timeout_node.set_timeout(true); // This will cause timeout

    let graph = GraphBuilder::new()
        .add_node("start".to_string(), IncrementNode { id: "start".to_string(), amount: 10 }).unwrap()
        .add_node("timeout".to_string(), timeout_node).unwrap()
        .add_edge(Edge::simple("start", "timeout")).unwrap()
        .with_entry_point("start".to_string()).unwrap()
        .add_finish_point("timeout".to_string()).unwrap()
        .build().unwrap();

    let mut state = TestState::default();
    let result = graph.run(&mut state).await;

    // Should fail with timeout error
    assert!(result.is_err());
    assert_eq!(state.value, 10); // Only start node should complete
    
    match result.unwrap_err() {
        agent_graph::GraphError::Timeout { seconds } => {
            assert_eq!(seconds, 1);
        }
        _ => panic!("Expected timeout error"),
    }
}

// Test 5: State Checkpointing
#[tokio::test]
async fn test_state_checkpointing() {
    let temp_dir = TempDir::new().unwrap();
    let checkpointer = FileCheckpointer::new(temp_dir.path());
    
    let mut graph = GraphBuilder::new()
        .add_node("step1".to_string(), IncrementNode { id: "step1".to_string(), amount: 10 }).unwrap()
        .add_node("step2".to_string(), IncrementNode { id: "step2".to_string(), amount: 20 }).unwrap()
        .add_node("step3".to_string(), IncrementNode { id: "step3".to_string(), amount: 30 }).unwrap()
        .add_edge(Edge::simple("step1", "step2")).unwrap()
        .add_edge(Edge::simple("step2", "step3")).unwrap()
        .with_entry_point("step1".to_string()).unwrap()
        .add_finish_point("step3".to_string()).unwrap()
        .build().unwrap();

    // Enable checkpointing
    graph.set_checkpointer(checkpointer);

    let mut state = TestState::default();
    let context = graph.run(&mut state).await.unwrap();

    // Verify execution completed
    assert_eq!(state.value, 60);
    assert_eq!(context.current_step, 3);

    // Verify checkpoints were created
    let checkpoint_files: Vec<_> = std::fs::read_dir(temp_dir.path())
        .unwrap()
        .collect();
    
    // Should have at least one checkpoint file
    assert!(!checkpoint_files.is_empty());
}

// Test 6: Memory Checkpointing
#[tokio::test]
async fn test_memory_checkpointing() {
    let checkpointer = MemoryCheckpointer::new();
    
    let mut graph = GraphBuilder::new()
        .add_node("node1".to_string(), IncrementNode { id: "node1".to_string(), amount: 15 }).unwrap()
        .add_node("node2".to_string(), IncrementNode { id: "node2".to_string(), amount: 25 }).unwrap()
        .add_edge(Edge::simple("node1", "node2")).unwrap()
        .with_entry_point("node1".to_string()).unwrap()
        .add_finish_point("node2".to_string()).unwrap()
        .build().unwrap();

    graph.set_checkpointer(checkpointer);

    let mut state = TestState::default();
    let context = graph.run(&mut state).await.unwrap();

    assert_eq!(state.value, 40);
    assert_eq!(context.current_step, 2);
}

// Test 7: Conditional Routing
#[tokio::test]
async fn test_conditional_routing() {
    let condition = FunctionCondition::new("value_check", |state: &TestState| state.value > 50);
    
    let mut graph = GraphBuilder::new()
        .add_node("start".to_string(), IncrementNode { id: "start".to_string(), amount: 60 }).unwrap()
        .add_node("high_path".to_string(), IncrementNode { id: "high_path".to_string(), amount: 100 }).unwrap()
        .add_node("low_path".to_string(), IncrementNode { id: "low_path".to_string(), amount: 10 }).unwrap()
        .add_edge(Edge::conditional("start", "value_check".to_string(), "high_path", "low_path")).unwrap()
        .with_entry_point("start".to_string()).unwrap()
        .add_finish_point("high_path".to_string()).unwrap()
        .add_finish_point("low_path".to_string()).unwrap()
        .build().unwrap();

    graph.edge_registry_mut().register_condition(condition);

    let mut state = TestState::default();
    let context = graph.run(&mut state).await.unwrap();

    // Should take high path since 60 > 50
    assert_eq!(state.value, 160); // 60 + 100
    assert!(context.execution_path.contains(&"high_path".to_string()));
    assert!(!context.execution_path.contains(&"low_path".to_string()));
}

// Test 8: Large Graph Stress Test
#[tokio::test]
async fn test_large_graph_execution() {
    let mut builder = GraphBuilder::new();
    
    // Create a large graph with 100 nodes
    for i in 0..100 {
        let node = IncrementNode { 
            id: format!("node_{}", i), 
            amount: 1 
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
        .add_finish_point("node_99".to_string()).unwrap()
        .build().unwrap();

    let mut state = TestState::default();
    let start = Instant::now();
    let context = graph.run(&mut state).await.unwrap();
    let duration = start.elapsed();

    // Verify all nodes executed
    assert_eq!(state.value, 100); // 100 nodes * 1 increment each
    assert_eq!(context.current_step, 100);
    assert_eq!(context.execution_path.len(), 100);
    
    // Should complete in reasonable time (less than 5 seconds)
    assert!(duration.as_secs() < 5, "Large graph took too long: {:?}", duration);
}

// Test 9: Error Recovery and Graceful Degradation
#[tokio::test]
async fn test_error_recovery() {
    let config = ExecutionConfig {
        stop_on_error: false, // Continue on errors
        max_retries: 1,
        ..Default::default()
    };

    let flaky_node = FlakeyNode::new("flaky".to_string(), 10); // Will always fail

    let graph = GraphBuilder::new()
        .with_config(config)
        .add_node("start".to_string(), IncrementNode { id: "start".to_string(), amount: 10 }).unwrap()
        .add_node("flaky".to_string(), flaky_node).unwrap()
        .add_node("recovery".to_string(), IncrementNode { id: "recovery".to_string(), amount: 5 }).unwrap()
        .add_edge(Edge::simple("start", "flaky")).unwrap()
        .add_edge(Edge::simple("flaky", "recovery")).unwrap()
        .with_entry_point("start".to_string()).unwrap()
        .add_finish_point("recovery".to_string()).unwrap()
        .build().unwrap();

    let mut state = TestState::default();
    let result = graph.run(&mut state).await;

    // Should fail but start node should have executed
    assert!(result.is_err());
    assert_eq!(state.value, 10); // Only start node completed
    assert!(state.error_count > 0); // Errors were recorded
}

// Test 10: Graph Validation
#[tokio::test]
async fn test_graph_validation() {
    // Test invalid graph (no entry point)
    let result = GraphBuilder::new()
        .add_node("node1".to_string(), IncrementNode { id: "node1".to_string(), amount: 1 }).unwrap()
        .add_finish_point("node1".to_string()).unwrap()
        .build();
    
    assert!(result.is_err()); // Should fail validation

    // Test invalid graph (no finish point)
    let result = GraphBuilder::new()
        .add_node("node1".to_string(), IncrementNode { id: "node1".to_string(), amount: 1 }).unwrap()
        .with_entry_point("node1".to_string()).unwrap()
        .build();
    
    assert!(result.is_err()); // Should fail validation

    // Test invalid graph (edge to non-existent node)
    let result = GraphBuilder::new()
        .add_node("node1".to_string(), IncrementNode { id: "node1".to_string(), amount: 1 }).unwrap()
        .add_edge(Edge::simple("node1", "nonexistent")).unwrap()
        .with_entry_point("node1".to_string()).unwrap()
        .add_finish_point("node1".to_string()).unwrap()
        .build();
    
    assert!(result.is_err()); // Should fail validation
}
