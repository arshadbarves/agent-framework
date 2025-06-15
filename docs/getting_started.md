# Getting Started with AgentGraph

Welcome to AgentGraph! This guide will help you get up and running with the powerful multi-agent framework for Rust.

## 🚀 Quick Start

### Installation

Add AgentGraph to your `Cargo.toml`:

```toml
[dependencies]
agent_graph = "0.3.0"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
```

### Your First Graph

Let's create a simple graph that processes data through multiple stages:

```rust
use agent_graph::{Graph, GraphBuilder, Node, State, GraphResult, Edge};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// Define your state
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MyState {
    value: i32,
    message: String,
}

impl State for MyState {}

// Create a simple node
struct ProcessingNode {
    name: String,
    multiplier: i32,
}

#[async_trait]
impl Node<MyState> for ProcessingNode {
    async fn invoke(&self, state: &mut MyState) -> GraphResult<()> {
        state.value *= self.multiplier;
        state.message = format!("{} processed by {}", state.message, self.name);
        println!("Node '{}' processed: value={}, message='{}'", 
                self.name, state.value, state.message);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> GraphResult<()> {
    // Build the graph
    let graph = GraphBuilder::new()
        .add_node("double".to_string(), ProcessingNode {
            name: "Doubler".to_string(),
            multiplier: 2,
        })?
        .add_node("triple".to_string(), ProcessingNode {
            name: "Tripler".to_string(),
            multiplier: 3,
        })?
        .add_edge(Edge::simple("double", "triple"))?
        .with_entry_point("double".to_string())?
        .add_finish_point("triple".to_string())?
        .build()?;

    // Execute the graph
    let mut state = MyState {
        value: 5,
        message: "Initial".to_string(),
    };

    let context = graph.run(&mut state).await?;
    
    println!("Final result: value={}, message='{}'", state.value, state.message);
    println!("Execution took {} steps", context.current_step);
    
    Ok(())
}
```

## 🏗️ Core Concepts

### 1. States

States represent the data that flows through your graph. They must implement the `State` trait:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DataProcessingState {
    input_data: Vec<String>,
    processed_results: Vec<ProcessedItem>,
    metadata: HashMap<String, String>,
    progress: f32,
}

impl State for DataProcessingState {}
```

**Key Requirements:**
- Must be `Clone` for parallel execution
- Must be `Serialize + Deserialize` for checkpointing
- Should be `Debug` for logging and debugging

### 2. Nodes

Nodes are the processing units of your graph:

```rust
struct DataValidationNode {
    validation_rules: Vec<String>,
}

#[async_trait]
impl Node<DataProcessingState> for DataValidationNode {
    async fn invoke(&self, state: &mut DataProcessingState) -> GraphResult<()> {
        // Validate input data
        for item in &state.input_data {
            if item.is_empty() {
                return Err(GraphError::validation_error("Empty data item found"));
            }
        }
        
        state.metadata.insert("validation_status".to_string(), "passed".to_string());
        Ok(())
    }

    fn metadata(&self) -> NodeMetadata {
        NodeMetadata::new("DataValidation")
            .with_description("Validates input data according to rules")
            .with_tag("validation")
            .with_expected_duration(100)
    }
}
```

### 3. Edges

Edges define how execution flows between nodes:

```rust
// Simple edge: A → B
Edge::simple("node_a", "node_b")

// Conditional edge: A → B (if condition) or A → C (if not)
Edge::conditional("node_a", "my_condition", "node_b", "node_c")

// Parallel edge: A → [B, C, D] (all execute concurrently)
Edge::parallel("node_a", vec!["node_b", "node_c", "node_d"])

// Weighted edge: A → B (70%) or A → C (30%)
Edge::weighted("node_a", vec![
    ("node_b".to_string(), 0.7),
    ("node_c".to_string(), 0.3),
])
```

## 🔧 Building Complex Graphs

### Sequential Processing

```rust
let graph = GraphBuilder::new()
    .add_node("ingest", DataIngestionNode::new())
    .add_node("validate", DataValidationNode::new())
    .add_node("process", DataProcessingNode::new())
    .add_node("output", DataOutputNode::new())
    .add_edge(Edge::simple("ingest", "validate"))
    .add_edge(Edge::simple("validate", "process"))
    .add_edge(Edge::simple("process", "output"))
    .with_entry_point("ingest")
    .add_finish_point("output")
    .build()?;
```

### Parallel Processing

```rust
let config = ExecutionConfig {
    enable_parallel: true,
    ..Default::default()
};

let graph = GraphBuilder::new()
    .with_config(config)
    .add_node("prepare", PrepareDataNode::new())
    .add_node("analyze_sentiment", SentimentAnalysisNode::new())
    .add_node("analyze_keywords", KeywordAnalysisNode::new())
    .add_node("analyze_topics", TopicAnalysisNode::new())
    .add_node("combine", CombineResultsNode::new())
    .add_edge(Edge::simple("prepare", "analyze_sentiment"))
    .add_edge(Edge::parallel("analyze_sentiment", vec![
        "analyze_keywords", "analyze_topics"
    ]))
    .add_edge(Edge::simple("analyze_keywords", "combine"))
    .add_edge(Edge::simple("analyze_topics", "combine"))
    .with_entry_point("prepare")
    .add_finish_point("combine")
    .build()?;
```

### Conditional Routing

```rust
use agent_graph::edge::conditions::FunctionCondition;

// Create a condition
let quality_check = FunctionCondition::new("quality_check", |state: &MyState| {
    state.quality_score > 0.8
});

let graph = GraphBuilder::new()
    .add_node("analyze", AnalysisNode::new())
    .add_node("high_quality_path", HighQualityProcessingNode::new())
    .add_node("low_quality_path", LowQualityProcessingNode::new())
    .add_edge(Edge::conditional(
        "analyze", 
        "quality_check", 
        "high_quality_path", 
        "low_quality_path"
    ))
    .with_entry_point("analyze")
    .add_finish_point("high_quality_path")
    .add_finish_point("low_quality_path")
    .build()?;

// Register the condition
graph.edge_registry_mut().register_condition(quality_check);
```

## 🛡️ Error Handling

### Retry Logic

```rust
use agent_graph::node::traits::RetryableNode;

struct UnreliableServiceNode;

#[async_trait]
impl RetryableNode<MyState> for UnreliableServiceNode {
    fn max_retries(&self) -> u32 { 3 }
    fn retry_delay(&self) -> Duration { Duration::from_millis(1000) }
    
    fn is_retryable_error(&self, error: &GraphError) -> bool {
        matches!(error, GraphError::ExternalServiceError(_))
    }
}

// Use with retry
let result = node.invoke_with_retry(&mut state).await?;
```

### Timeout Handling

```rust
use agent_graph::node::traits::TimeoutNode;

struct LongRunningNode;

#[async_trait]
impl TimeoutNode<MyState> for LongRunningNode {
    fn timeout(&self) -> Duration { Duration::from_secs(30) }
}

// Use with timeout
let result = node.invoke_with_timeout(&mut state).await?;
```

## 💾 State Management

### Checkpointing

```rust
use agent_graph::state::checkpointing::FileCheckpointer;

let checkpointer = FileCheckpointer::new("./checkpoints");
let mut graph = Graph::new();
graph.set_checkpointer(checkpointer);

// Checkpoints will be created automatically during execution
let context = graph.run(&mut state).await?;
```

### State Validation

```rust
use agent_graph::node::traits::ValidatingNode;

#[async_trait]
impl ValidatingNode<MyState> for MyNode {
    async fn validate_input(&self, state: &MyState) -> GraphResult<()> {
        if state.value < 0 {
            return Err(GraphError::validation_error("Value must be non-negative"));
        }
        Ok(())
    }

    async fn validate_output(&self, state: &MyState) -> GraphResult<()> {
        if state.value > 1000 {
            return Err(GraphError::validation_error("Value too large"));
        }
        Ok(())
    }
}
```

## 📡 Streaming and Monitoring

### Real-time Events

```rust
let (context, stream) = graph.run_streaming(&mut state).await?;

// Process events as they arrive
while let Some(event) = stream.next().await {
    match event {
        ExecutionEvent::NodeStarted { node_id, .. } => {
            println!("Started: {}", node_id);
        }
        ExecutionEvent::NodeCompleted { node_id, success, .. } => {
            println!("Completed: {} (success: {})", node_id, success);
        }
        ExecutionEvent::Error { error, .. } => {
            eprintln!("Error: {}", error);
        }
        _ => {}
    }
}
```

### Performance Monitoring

```rust
let stats = graph.execution_stats();
println!("Graph complexity: {:?}", stats.estimated_complexity);
println!("Node count: {}", stats.node_count);
println!("Edge count: {}", stats.edge_count);

let summary = graph.summary();
println!("Graph summary: {}", summary);
```

## 🧪 Testing Your Graphs

```rust
#[tokio::test]
async fn test_my_graph() {
    let graph = create_test_graph();
    let mut state = create_test_state();
    
    let context = graph.run(&mut state).await.unwrap();
    
    assert_eq!(state.expected_value, 42);
    assert_eq!(context.current_step, 3);
    assert!(context.execution_path.contains(&"my_node".to_string()));
}
```

## 📚 Next Steps

1. **Explore Examples**: Check out the `examples/` directory for more complex scenarios
2. **Read the Architecture Guide**: Understand the framework's design in `docs/architecture.md`
3. **Performance Tuning**: Learn optimization techniques in the advanced guide
4. **Production Deployment**: Follow the production checklist for deployment

## 🆘 Getting Help

- **Documentation**: Full API docs at [docs.rs/agent_graph](https://docs.rs/agent_graph)
- **Examples**: Comprehensive examples in the repository
- **Issues**: Report bugs and request features on GitHub
- **Community**: Join our Discord for discussions and support

Happy building with AgentGraph! 🚀
