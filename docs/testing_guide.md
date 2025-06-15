# AgentGraph Testing Guide

This document provides comprehensive manual test cases for validating all features of the AgentGraph framework. These tests are designed to be battle-tested scenarios that ensure production readiness.

## 🧪 Test Categories

### 1. Core Functionality Tests
### 2. State Management Tests
### 3. Parallel Execution Tests
### 4. Error Handling & Recovery Tests
### 5. Performance & Stress Tests
### 6. Edge Cases & Boundary Tests
### 7. Integration Tests
### 8. Production Scenario Tests

---

## 1. 🔧 Core Functionality Tests

### Test 1.1: Basic Graph Construction and Execution

**Objective**: Verify basic graph creation, node addition, and execution flow.

**Test Steps**:
1. Create a new graph using `GraphBuilder`
2. Add 3 sequential nodes (A → B → C)
3. Set entry point to node A
4. Set finish point to node C
5. Execute with initial state
6. Verify execution path and final state

**Expected Results**:
- Graph builds successfully
- All nodes execute in correct order
- Final state reflects all transformations
- Execution context shows correct path

**Code Example**:
```rust
// Create test state
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestState { value: i32, log: Vec<String> }

// Create nodes that increment value and log execution
struct IncrementNode { id: String, amount: i32 }

// Build and execute graph
let graph = GraphBuilder::new()
    .add_node("A", IncrementNode { id: "A", amount: 10 })
    .add_node("B", IncrementNode { id: "B", amount: 20 })
    .add_node("C", IncrementNode { id: "C", amount: 30 })
    .add_edge(Edge::simple("A", "B"))
    .add_edge(Edge::simple("B", "C"))
    .with_entry_point("A")
    .add_finish_point("C")
    .build()?;

let mut state = TestState { value: 0, log: vec![] };
let context = graph.run(&mut state).await?;

// Verify results
assert_eq!(state.value, 60); // 10 + 20 + 30
assert_eq!(context.execution_path, vec!["A", "B", "C"]);
assert_eq!(context.current_step, 3);
```

### Test 1.2: Conditional Edge Routing

**Objective**: Test conditional routing based on state values.

**Test Steps**:
1. Create graph with conditional edges
2. Test both true and false conditions
3. Verify correct routing in each case
4. Test edge cases (boundary values)

**Expected Results**:
- Correct routing based on conditions
- Both paths execute properly
- State changes affect routing decisions

### Test 1.3: Dynamic Edge Routing

**Objective**: Validate dynamic routing with custom routers.

**Test Steps**:
1. Implement custom router logic
2. Create graph with dynamic edges
3. Test multiple routing scenarios
4. Verify router state independence

**Expected Results**:
- Router logic executes correctly
- Multiple targets handled properly
- Routing decisions are consistent

---

## 2. 💾 State Management Tests

### Test 2.1: State Persistence and Checkpointing

**Objective**: Verify state checkpointing and recovery functionality.

**Test Steps**:
1. Enable checkpointing with FileCheckpointer
2. Execute graph with checkpoint intervals
3. Simulate failure mid-execution
4. Restore from checkpoint and continue
5. Verify state integrity

**Expected Results**:
- Checkpoints created at intervals
- State restored correctly after failure
- Execution continues from checkpoint
- No data loss or corruption

**Code Example**:
```rust
// Setup checkpointing
let checkpointer = FileCheckpointer::new("./test_checkpoints");
let mut graph = Graph::new();
graph.set_checkpointer(checkpointer);

// Configure checkpoint intervals
let config = ExecutionConfig {
    enable_checkpointing: true,
    checkpoint_interval: Some(2), // Every 2 steps
    ..Default::default()
};
graph.set_config(config);

// Execute and verify checkpoints
let context = graph.run(&mut state).await?;
let checkpoints = std::fs::read_dir("./test_checkpoints")?;
assert!(checkpoints.count() > 0);
```

### Test 2.2: State Versioning and Integrity

**Objective**: Test state versioning and integrity verification.

**Test Steps**:
1. Create VersionedState with initial data
2. Perform multiple state updates
3. Verify version increments
4. Test integrity verification
5. Simulate corruption and detect

**Expected Results**:
- Version numbers increment correctly
- Integrity checks pass for valid state
- Corruption detected and reported
- State history maintained

### Test 2.3: Concurrent State Access

**Objective**: Validate thread-safe state operations.

**Test Steps**:
1. Create SharedState instance
2. Spawn multiple concurrent readers/writers
3. Perform simultaneous operations
4. Verify data consistency
5. Test lock contention scenarios

**Expected Results**:
- No data races or corruption
- Consistent state across threads
- Proper lock behavior
- Performance within acceptable limits

---

## 3. ⚡ Parallel Execution Tests

### Test 3.1: Basic Parallel Node Execution

**Objective**: Verify parallel execution of independent nodes.

**Test Steps**:
1. Create graph with parallel edges
2. Add nodes with different execution times
3. Enable parallel execution
4. Measure execution time vs sequential
5. Verify all nodes complete successfully

**Expected Results**:
- Parallel execution faster than sequential
- All nodes execute concurrently
- Results properly aggregated
- No race conditions

**Code Example**:
```rust
// Create nodes with different delays
struct DelayNode { delay_ms: u64, id: String }

// Build parallel graph
let graph = GraphBuilder::new()
    .add_node("init", InitNode)
    .add_node("task1", DelayNode { delay_ms: 1000, id: "task1" })
    .add_node("task2", DelayNode { delay_ms: 800, id: "task2" })
    .add_node("task3", DelayNode { delay_ms: 1200, id: "task3" })
    .add_edge(Edge::simple("init", "task1"))
    .add_edge(Edge::parallel("task1", vec!["task2", "task3"]))
    .build()?;

let start = Instant::now();
let context = graph.run(&mut state).await?;
let duration = start.elapsed();

// Should be ~1200ms (longest task) not ~3000ms (sum of all)
assert!(duration.as_millis() < 2000);
```

### Test 3.2: Parallel State Merging

**Objective**: Test state merging strategies for parallel execution.

**Test Steps**:
1. Create parallel nodes that modify different state fields
2. Execute with various merging strategies
3. Verify state consistency after merge
4. Test conflict resolution

**Expected Results**:
- State changes properly merged
- No data loss during merge
- Conflicts resolved correctly
- Deterministic results

### Test 3.3: Parallel Error Handling

**Objective**: Validate error handling in parallel execution.

**Test Steps**:
1. Create parallel nodes where some fail
2. Test different error handling strategies
3. Verify partial success scenarios
4. Test error propagation

**Expected Results**:
- Failed nodes don't affect successful ones
- Errors properly propagated
- Partial results available
- Cleanup performed correctly

---

## 4. 🛡️ Error Handling & Recovery Tests

### Test 4.1: Node Failure and Retry Logic

**Objective**: Test retry mechanisms for failed nodes.

**Test Steps**:
1. Create node that fails intermittently
2. Configure retry parameters
3. Test retry exhaustion scenarios
4. Verify retry delays and backoff

**Expected Results**:
- Failed nodes retry as configured
- Exponential backoff works correctly
- Retry exhaustion handled properly
- Metrics track retry attempts

**Code Example**:
```rust
struct FlakeyNode { fail_count: Arc<AtomicU32> }

#[async_trait]
impl RetryableNode<TestState> for FlakeyNode {
    fn max_retries(&self) -> u32 { 3 }
    fn retry_delay(&self) -> Duration { Duration::from_millis(100) }
    
    fn is_retryable_error(&self, error: &GraphError) -> bool {
        matches!(error, GraphError::ExternalServiceError(_))
    }
}

// Test retry behavior
let node = FlakeyNode { fail_count: Arc::new(AtomicU32::new(0)) };
let result = node.invoke_with_retry(&mut state).await;
// Should succeed after retries
```

### Test 4.2: Timeout Handling

**Objective**: Verify timeout mechanisms for long-running nodes.

**Test Steps**:
1. Create nodes with configurable delays
2. Set various timeout values
3. Test timeout enforcement
4. Verify cleanup after timeout

**Expected Results**:
- Timeouts enforced correctly
- Long-running nodes terminated
- Resources cleaned up properly
- Timeout errors reported accurately

### Test 4.3: Graceful Degradation

**Objective**: Test system behavior under various failure conditions.

**Test Steps**:
1. Simulate network failures
2. Test memory pressure scenarios
3. Simulate disk space issues
4. Test partial system failures

**Expected Results**:
- System degrades gracefully
- Critical functions continue
- Errors logged appropriately
- Recovery possible when conditions improve

---

## 5. 📊 Performance & Stress Tests

### Test 5.1: Large Graph Execution

**Objective**: Test performance with large, complex graphs.

**Test Steps**:
1. Create graph with 1000+ nodes
2. Mix of sequential and parallel paths
3. Measure execution time and memory usage
4. Test with different graph topologies

**Expected Results**:
- Execution completes within reasonable time
- Memory usage remains bounded
- Performance scales appropriately
- No memory leaks detected

### Test 5.2: High-Frequency Execution

**Objective**: Test rapid, repeated graph executions.

**Test Steps**:
1. Execute same graph 10,000 times
2. Measure throughput and latency
3. Monitor resource usage
4. Test concurrent executions

**Expected Results**:
- Consistent performance over time
- No resource leaks
- Throughput meets requirements
- Latency remains stable

### Test 5.3: Memory Stress Testing

**Objective**: Validate behavior under memory pressure.

**Test Steps**:
1. Create graphs with large state objects
2. Execute with limited memory
3. Test garbage collection behavior
4. Verify memory cleanup

**Expected Results**:
- Graceful handling of memory pressure
- Proper cleanup of resources
- No memory leaks
- Performance degradation is predictable

---

## 6. 🎯 Edge Cases & Boundary Tests

### Test 6.1: Empty and Single-Node Graphs

**Objective**: Test edge cases with minimal graphs.

**Test Steps**:
1. Test empty graph execution
2. Test single-node graph
3. Test graph with no edges
4. Test circular dependencies

**Expected Results**:
- Appropriate error messages for invalid graphs
- Single-node graphs execute correctly
- Circular dependencies detected
- Validation catches issues early

### Test 6.2: Extreme Configuration Values

**Objective**: Test behavior with boundary configuration values.

**Test Steps**:
1. Test with zero timeouts
2. Test with maximum retry counts
3. Test with extreme checkpoint intervals
4. Test with very large/small state objects

**Expected Results**:
- Boundary values handled correctly
- No integer overflows or underflows
- Appropriate error messages
- System remains stable

### Test 6.3: Resource Exhaustion Scenarios

**Objective**: Test behavior when resources are exhausted.

**Test Steps**:
1. Exhaust file descriptors
2. Fill disk space during checkpointing
3. Exhaust thread pool
4. Test network connection limits

**Expected Results**:
- Graceful handling of resource exhaustion
- Appropriate error messages
- System recovery when resources available
- No crashes or data corruption

---

## 7. 🔗 Integration Tests

### Test 7.1: Streaming Integration

**Objective**: Test real-time event streaming functionality.

**Test Steps**:
1. Enable streaming for graph execution
2. Subscribe to event stream
3. Verify all events are received
4. Test event filtering
5. Test stream backpressure

**Expected Results**:
- All execution events captured
- Events received in correct order
- Filtering works as expected
- Backpressure handled gracefully

### Test 7.2: External Service Integration

**Objective**: Test integration with external services.

**Test Steps**:
1. Create nodes that call external APIs
2. Test with various response times
3. Simulate service failures
4. Test authentication and authorization

**Expected Results**:
- External calls succeed when services available
- Failures handled gracefully
- Retry logic works with external services
- Security credentials managed properly

### Test 7.3: Database Integration

**Objective**: Test integration with database systems.

**Test Steps**:
1. Create nodes that read/write to database
2. Test transaction handling
3. Test connection pooling
4. Simulate database failures

**Expected Results**:
- Database operations succeed
- Transactions handled correctly
- Connection pooling works efficiently
- Database failures handled gracefully

---

## 8. 🏭 Production Scenario Tests

### Test 8.1: Multi-Agent Research Pipeline

**Objective**: Simulate a real-world research automation pipeline.

**Test Scenario**:
- Web scraping nodes
- Data processing nodes
- Analysis nodes
- Report generation nodes
- Error recovery and retry logic

**Test Steps**:
1. Execute full research pipeline
2. Inject various failure scenarios
3. Test with different data volumes
4. Verify end-to-end functionality

### Test 8.2: Real-Time Data Processing

**Objective**: Test real-time data processing capabilities.

**Test Scenario**:
- Continuous data ingestion
- Real-time transformation
- Parallel processing streams
- Output generation

**Test Steps**:
1. Setup continuous data flow
2. Process data in real-time
3. Monitor latency and throughput
4. Test with varying data rates

### Test 8.3: Fault-Tolerant Workflow

**Objective**: Test enterprise-grade fault tolerance.

**Test Scenario**:
- Critical business workflow
- Multiple failure points
- Automatic recovery
- Data consistency requirements

**Test Steps**:
1. Execute critical workflow
2. Inject failures at various points
3. Verify automatic recovery
4. Ensure data consistency maintained

---

## 🔍 Test Execution Guidelines

### Prerequisites
- Rust 1.70+ installed
- All dependencies available
- Test data prepared
- Monitoring tools setup

### Test Environment Setup
```bash
# Clone repository
git clone <repository-url>
cd agent-framework

# Setup test environment
cargo build --all-features
mkdir -p test_data test_checkpoints test_logs

# Run basic validation
cargo test --all-features
```

### Manual Test Execution

1. **Prepare Test Data**: Create test datasets for each scenario
2. **Configure Logging**: Enable detailed logging for test analysis
3. **Execute Tests**: Run each test category systematically
4. **Monitor Resources**: Track CPU, memory, and I/O during tests
5. **Document Results**: Record all test outcomes and metrics
6. **Analyze Failures**: Investigate any test failures thoroughly

### Success Criteria

- ✅ All core functionality tests pass
- ✅ Performance meets requirements
- ✅ Error handling works correctly
- ✅ Resource usage within limits
- ✅ No memory leaks detected
- ✅ Production scenarios complete successfully

### Failure Investigation

When tests fail:
1. Check logs for error details
2. Verify test environment setup
3. Reproduce failure in isolation
4. Analyze resource usage patterns
5. Review code for potential issues
6. Document findings and fixes

This comprehensive testing guide ensures that AgentGraph is thoroughly validated for production use across all its features and capabilities.

---

## 🚀 Quick Test Execution

### Automated Test Suite

Run the complete test suite with our automated script:

```bash
# Make script executable
chmod +x scripts/run_tests.sh

# Run all tests
./scripts/run_tests.sh
```

This will execute:
- Unit tests
- Integration tests
- Stress tests
- Production scenario tests
- Example validations
- Code quality checks

### Manual Test Execution

For targeted testing, run specific test categories:

```bash
# Core functionality tests
cargo test --test integration_tests --all-features

# Stress tests (may take several minutes)
cargo test --test stress_tests --all-features

# Production scenarios
cargo test --test production_scenarios --all-features

# Run with detailed output
cargo test -- --nocapture
```

---

## 📋 Test Execution Checklist

### Pre-Test Setup
- [ ] Rust 1.70+ installed
- [ ] All dependencies resolved (`cargo build`)
- [ ] Test directories created
- [ ] Environment variables set
- [ ] Sufficient system resources available

### Core Tests
- [ ] Basic graph construction and execution
- [ ] Node registry and metadata
- [ ] Edge routing (simple, conditional, dynamic)
- [ ] State management and persistence
- [ ] Error handling and propagation
- [ ] Graph validation

### Advanced Features
- [ ] Parallel node execution
- [ ] State checkpointing (file and memory)
- [ ] Retry mechanisms with backoff
- [ ] Timeout handling
- [ ] Streaming execution events
- [ ] Complex routing scenarios

### Performance Tests
- [ ] High CPU load handling
- [ ] Memory pressure resistance
- [ ] I/O intensive operations
- [ ] Large graph scalability (1000+ nodes)
- [ ] Rapid sequential executions
- [ ] Massive parallel execution

### Production Scenarios
- [ ] Data processing pipelines
- [ ] Real-time event processing
- [ ] Fault-tolerant workflows
- [ ] Multi-stage analysis
- [ ] Error recovery and graceful degradation

### Battle-Tested Scenarios
- [ ] Network failures and timeouts
- [ ] Memory exhaustion
- [ ] Disk space limitations
- [ ] High concurrency loads
- [ ] Long-running executions
- [ ] Resource contention

---

## 🔍 Test Result Analysis

### Success Criteria

**Functional Requirements:**
- ✅ All unit tests pass
- ✅ Integration tests complete successfully
- ✅ Examples run without errors
- ✅ Documentation tests pass

**Performance Requirements:**
- ✅ Sequential execution: >100 operations/second
- ✅ Parallel execution: >1000 operations/second
- ✅ Memory usage: Bounded and predictable
- ✅ Large graphs: Support 1000+ nodes

**Reliability Requirements:**
- ✅ Error recovery: Automatic retry mechanisms
- ✅ Fault tolerance: Graceful degradation
- ✅ State persistence: Checkpoint/restore works
- ✅ Resource cleanup: No memory leaks

### Failure Investigation

When tests fail, follow this process:

1. **Check Environment**:
   ```bash
   # Verify Rust version
   rustc --version

   # Check available memory
   free -h

   # Verify disk space
   df -h
   ```

2. **Run with Verbose Output**:
   ```bash
   RUST_LOG=debug cargo test failing_test_name -- --nocapture
   ```

3. **Isolate the Issue**:
   ```bash
   # Run single test
   cargo test specific_test_name

   # Run test category
   cargo test --test integration_tests
   ```

4. **Analyze Logs**:
   - Check for error patterns
   - Look for resource exhaustion
   - Verify timing issues
   - Check for race conditions

---

## 📊 Performance Benchmarking

### Baseline Metrics

Establish baseline performance metrics:

```bash
# Run performance tests
cargo test --test stress_tests test_rapid_sequential_executions -- --nocapture

# Monitor resource usage
top -p $(pgrep cargo)
```

**Expected Baselines:**
- Simple node execution: <1ms
- Graph with 100 nodes: <100ms
- Parallel execution speedup: 2-4x
- Memory usage: <100MB for typical graphs

### Load Testing

Test under various load conditions:

```bash
# High CPU load
cargo test test_high_cpu_load -- --nocapture

# High memory usage
cargo test test_high_memory_usage -- --nocapture

# High I/O load
cargo test test_high_io_load -- --nocapture
```

---

## 🛠️ Debugging Failed Tests

### Common Issues and Solutions

**1. Timeout Errors**
```
Error: Operation timed out after 30 seconds
```
- Increase timeout values in test configuration
- Check for deadlocks or infinite loops
- Verify system performance

**2. Memory Issues**
```
Error: Cannot allocate memory
```
- Reduce test data sizes
- Check for memory leaks
- Increase available system memory

**3. Concurrency Issues**
```
Error: Thread panicked at 'already borrowed'
```
- Check for improper state sharing
- Verify lock ordering
- Review parallel execution logic

**4. File System Issues**
```
Error: Permission denied
```
- Check file permissions
- Verify disk space
- Ensure test directories exist

### Debug Tools

Use these tools for debugging:

```bash
# Memory profiling
valgrind --tool=memcheck cargo test

# CPU profiling
perf record cargo test
perf report

# Trace execution
RUST_LOG=trace cargo test

# Check for deadlocks
timeout 30s cargo test || echo "Potential deadlock"
```

---

## 📈 Continuous Testing

### Automated Testing Pipeline

Set up continuous testing:

```yaml
# .github/workflows/test.yml
name: Comprehensive Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
      - name: Run Tests
        run: ./scripts/run_tests.sh
```

### Performance Regression Testing

Monitor performance over time:

```bash
# Benchmark and store results
cargo test --test stress_tests -- --nocapture > performance_$(date +%Y%m%d).log

# Compare with previous results
diff performance_baseline.log performance_$(date +%Y%m%d).log
```

---

## 🎯 Production Readiness Checklist

Before deploying to production:

### Functional Validation
- [ ] All test categories pass
- [ ] Examples work correctly
- [ ] Documentation is accurate
- [ ] API is stable

### Performance Validation
- [ ] Meets throughput requirements
- [ ] Memory usage is acceptable
- [ ] Latency is within bounds
- [ ] Scales to required load

### Reliability Validation
- [ ] Error handling is comprehensive
- [ ] Recovery mechanisms work
- [ ] Graceful degradation occurs
- [ ] Monitoring is in place

### Security Validation
- [ ] No known vulnerabilities
- [ ] Input validation works
- [ ] Resource limits enforced
- [ ] Audit trail available

This comprehensive testing approach ensures AgentGraph is thoroughly validated and ready for production deployment.
