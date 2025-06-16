# Performance Optimization Guide

This guide covers best practices for optimizing AgentGraph performance in production environments.

## Overview

AgentGraph is designed for high-performance multi-agent systems, but proper configuration and optimization are essential for production deployments.

## Key Performance Areas

### 1. LLM Provider Optimization

#### Connection Pooling
```rust
use agent_graph::llm::LLMConfig;
use std::time::Duration;

let llm_config = LLMConfig {
    default_provider: "openai".to_string(),
    timeout: Duration::from_secs(30),
    max_retries: 3,
    retry_delay: Duration::from_millis(1000),
};
```

#### Request Batching
- Batch multiple requests when possible
- Use streaming for long-running tasks
- Implement request queuing for high-throughput scenarios

#### Provider Selection
```rust
// Choose providers based on your needs:
// - OpenAI: Best general performance, higher cost
// - Anthropic: Good for reasoning tasks, moderate cost
// - Google: Good for specific domains, variable cost
// - OpenRouter: Access to multiple models, flexible pricing
```

### 2. Memory System Optimization

#### Memory Configuration
```rust
use agent_graph::agents::memory::MemoryConfig;

let memory_config = MemoryConfig {
    max_entries: 1000,           // Limit memory size
    embedding_model: "text-embedding-ada-002".to_string(),
    similarity_threshold: 0.7,   // Adjust for relevance
    cleanup_interval: Duration::from_secs(3600), // Periodic cleanup
};
```

#### Memory Management Strategies
- **Sliding Window**: Keep only recent interactions
- **Importance-based**: Retain high-importance memories
- **Hierarchical**: Use different retention policies by memory type

#### Embedding Optimization
```rust
// Use efficient embedding models
// - Smaller models for speed
// - Larger models for accuracy
// - Local models for privacy
```

### 3. Agent Performance Tuning

#### Temperature Settings
```rust
// Optimize temperature by use case:
let configs = vec![
    ("coding", 0.1),      // Deterministic for code
    ("analysis", 0.3),    // Focused for analysis
    ("creative", 0.8),    // Creative for writing
    ("general", 0.5),     // Balanced for general tasks
];
```

#### Token Management
```rust
// Optimize token usage
let config = AgentConfig {
    max_tokens: Some(1000),  // Limit response length
    // ... other config
};

// Monitor token usage
let state = agent.state();
println!("Tokens used: {}", state.total_tokens_used);
```

#### Concurrent Execution
```rust
use tokio::task::JoinSet;

// Execute multiple agents concurrently
let mut join_set = JoinSet::new();

for agent in agents {
    join_set.spawn(async move {
        agent.execute_task(task.clone()).await
    });
}

// Collect results
while let Some(result) = join_set.join_next().await {
    match result {
        Ok(response) => println!("Response: {:?}", response),
        Err(e) => eprintln!("Error: {:?}", e),
    }
}
```

### 4. Tool System Optimization

#### Tool Selection
```rust
// Register only necessary tools
let mut tool_registry = ToolRegistry::new();

// Essential tools only
tool_registry.register(FileReadTool::new());
tool_registry.register(HttpGetTool::new());

// Avoid registering unused tools
```

#### Tool Caching
```rust
// Implement caching for expensive tools
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

struct CachedTool {
    cache: Arc<RwLock<HashMap<String, String>>>,
    inner_tool: Box<dyn Tool>,
}

impl CachedTool {
    async fn execute_cached(&self, input: &str) -> Result<String, ToolError> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(result) = cache.get(input) {
                return Ok(result.clone());
            }
        }
        
        // Execute and cache result
        let result = self.inner_tool.execute(input).await?;
        {
            let mut cache = self.cache.write().await;
            cache.insert(input.to_string(), result.clone());
        }
        
        Ok(result)
    }
}
```

### 5. Collaboration Optimization

#### Message Routing
```rust
use agent_graph::agents::collaboration::CollaborationConfig;

let collab_config = CollaborationConfig {
    max_agents: 50,                              // Limit concurrent agents
    message_timeout: Duration::from_secs(30),    // Prevent hanging
    heartbeat_interval: Duration::from_secs(10), // Health monitoring
};
```

#### Load Balancing
```rust
// Distribute work across agents
async fn distribute_tasks(
    tasks: Vec<String>,
    agents: Vec<&mut Agent>,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();
    let chunk_size = tasks.len() / agents.len();
    
    let mut join_set = JoinSet::new();
    
    for (i, agent) in agents.into_iter().enumerate() {
        let start = i * chunk_size;
        let end = if i == agents.len() - 1 { tasks.len() } else { (i + 1) * chunk_size };
        let agent_tasks = tasks[start..end].to_vec();
        
        join_set.spawn(async move {
            let mut agent_results = Vec::new();
            for task in agent_tasks {
                let result = agent.execute_task(task).await?;
                agent_results.push(result);
            }
            Ok::<Vec<String>, Box<dyn std::error::Error>>(agent_results)
        });
    }
    
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Ok(agent_results)) => results.extend(agent_results),
            Ok(Err(e)) => eprintln!("Agent error: {:?}", e),
            Err(e) => eprintln!("Join error: {:?}", e),
        }
    }
    
    Ok(results)
}
```

## Monitoring and Metrics

### 1. Performance Metrics
```rust
// Track key metrics
struct PerformanceMetrics {
    pub requests_per_second: f64,
    pub average_response_time: Duration,
    pub token_usage_rate: f64,
    pub error_rate: f64,
    pub memory_usage: usize,
}

// Implement metrics collection
impl Agent {
    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        let state = self.state();
        PerformanceMetrics {
            requests_per_second: state.tasks_completed as f64 / state.uptime_seconds,
            average_response_time: Duration::from_millis(state.average_response_time_ms as u64),
            token_usage_rate: state.total_tokens_used as f64 / state.uptime_seconds,
            error_rate: state.error_count as f64 / state.tasks_completed as f64,
            memory_usage: state.memory_usage_bytes,
        }
    }
}
```

### 2. Health Checks
```rust
// Implement health monitoring
async fn health_check(agent: &Agent) -> bool {
    // Check if agent is responsive
    let start = std::time::Instant::now();
    let result = agent.execute_task("ping".to_string()).await;
    let duration = start.elapsed();
    
    match result {
        Ok(_) => duration < Duration::from_secs(5), // Responsive within 5 seconds
        Err(_) => false,
    }
}
```

### 3. Resource Monitoring
```rust
use agent_graph::enterprise::monitoring::{MonitoringManager, MetricType};

// Record custom metrics
async fn record_performance_metrics(
    monitoring: &MonitoringManager,
    agent_name: &str,
    metrics: &PerformanceMetrics,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut labels = std::collections::HashMap::new();
    labels.insert("agent".to_string(), agent_name.to_string());
    
    monitoring.record_metric(
        "agent_requests_per_second".to_string(),
        MetricType::Gauge,
        metrics.requests_per_second,
        labels.clone(),
    ).await?;
    
    monitoring.record_metric(
        "agent_response_time_ms".to_string(),
        MetricType::Histogram,
        metrics.average_response_time.as_millis() as f64,
        labels.clone(),
    ).await?;
    
    monitoring.record_metric(
        "agent_token_usage".to_string(),
        MetricType::Counter,
        metrics.token_usage_rate,
        labels,
    ).await?;
    
    Ok(())
}
```

## Scaling Strategies

### 1. Horizontal Scaling
- Deploy multiple agent instances
- Use load balancers for request distribution
- Implement service discovery for agent coordination

### 2. Vertical Scaling
- Increase memory for larger context windows
- Use faster CPUs for better processing
- Optimize for specific workloads

### 3. Caching Strategies
```rust
// Multi-level caching
struct CacheHierarchy {
    l1_cache: HashMap<String, String>,      // In-memory, fast
    l2_cache: Arc<RwLock<HashMap<String, String>>>, // Shared, medium
    l3_cache: Box<dyn ExternalCache>,       // Redis/DB, slow but persistent
}
```

## Production Deployment

### 1. Configuration Management
```rust
// Use environment-based configuration
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct ProductionConfig {
    llm_provider: String,
    max_concurrent_agents: usize,
    memory_limit_mb: usize,
    cache_ttl_seconds: u64,
    monitoring_enabled: bool,
}

// Load from environment or config file
let config: ProductionConfig = envy::from_env()?;
```

### 2. Error Recovery
```rust
// Implement circuit breaker pattern
struct CircuitBreaker {
    failure_count: AtomicUsize,
    last_failure: AtomicU64,
    threshold: usize,
    timeout: Duration,
}

impl CircuitBreaker {
    async fn call<F, T, E>(&self, f: F) -> Result<T, E>
    where
        F: Future<Output = Result<T, E>>,
    {
        if self.is_open() {
            return Err(/* circuit open error */);
        }
        
        match f.await {
            Ok(result) => {
                self.reset();
                Ok(result)
            }
            Err(e) => {
                self.record_failure();
                Err(e)
            }
        }
    }
}
```

### 3. Graceful Shutdown
```rust
// Implement graceful shutdown
use tokio::signal;

async fn graceful_shutdown(agents: Vec<Agent>) {
    // Wait for shutdown signal
    signal::ctrl_c().await.expect("Failed to listen for ctrl-c");
    
    println!("Shutting down gracefully...");
    
    // Stop accepting new requests
    // Complete ongoing tasks
    // Save state
    // Clean up resources
    
    println!("Shutdown complete");
}
```

## Benchmarking

### 1. Load Testing
```rust
// Simple load test
async fn load_test(agent: &mut Agent, concurrent_requests: usize) {
    let mut join_set = JoinSet::new();
    let start = std::time::Instant::now();
    
    for i in 0..concurrent_requests {
        let task = format!("Load test task {}", i);
        join_set.spawn(agent.execute_task(task));
    }
    
    let mut completed = 0;
    while let Some(result) = join_set.join_next().await {
        if result.is_ok() {
            completed += 1;
        }
    }
    
    let duration = start.elapsed();
    let rps = completed as f64 / duration.as_secs_f64();
    
    println!("Completed {} requests in {:?} ({:.2} RPS)", completed, duration, rps);
}
```

### 2. Memory Profiling
```rust
// Track memory usage
fn profile_memory_usage(agent: &Agent) {
    let state = agent.state();
    println!("Memory usage: {} bytes", state.memory_usage_bytes);
    
    let memory_stats = agent.memory().get_stats();
    println!("Memory entries: {}", memory_stats.total_entries);
    println!("Memory efficiency: {:.2} bytes/entry", 
             memory_stats.memory_usage_bytes as f64 / memory_stats.total_entries as f64);
}
```

This performance optimization guide provides comprehensive strategies for maximizing AgentGraph performance in production environments. Regular monitoring and profiling are essential for maintaining optimal performance as your system scales.
