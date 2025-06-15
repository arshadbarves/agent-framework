# AgentGraph 🦀

[![Crates.io](https://img.shields.io/crates/v/agent_graph.svg)](https://crates.io/crates/agent_graph)
[![Documentation](https://docs.rs/agent_graph/badge.svg)](https://docs.rs/agent_graph)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Build Status](https://github.com/agent-graph/agent-graph/workflows/CI/badge.svg)](https://github.com/agent-graph/agent-graph/actions)

A powerful, production-grade multi-agent framework for Rust inspired by LangGraph. Build complex, stateful, multi-agent systems with support for parallel execution, state checkpointing, streaming outputs, and comprehensive observability.

## ✨ Features

- **🔄 Stateful Execution**: Manage complex state through graph execution with automatic state management
- **⚡ Async by Design**: Built on tokio for high-performance async operations
- **🚀 Parallel Execution**: Run independent nodes concurrently for better performance
- **💾 State Checkpointing**: Save and resume graph state for fault tolerance and debugging
- **📡 Streaming Outputs**: Real-time streaming of execution results and events
- **🛡️ Production-Grade Error Handling**: Comprehensive error types and recovery mechanisms
- **📊 Observability**: Integrated tracing, metrics, and execution monitoring
- **🔧 Flexible Node System**: Powerful node traits with retry, timeout, and validation capabilities
- **🌐 Dynamic Routing**: Conditional and dynamic edge routing based on state
- **📈 Performance Monitoring**: Built-in execution statistics and performance analysis

## 🚀 Quick Start

Add AgentGraph to your `Cargo.toml`:

```toml
[dependencies]
agent_graph = "0.3.0"
```

### Basic Example

```rust
use agent_graph::{Graph, Node, State, GraphResult, GraphBuilder, Edge};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MyState {
    counter: i32,
    message: String,
}

impl State for MyState {}

struct IncrementNode;

#[async_trait]
impl Node<MyState> for IncrementNode {
    async fn invoke(&self, state: &mut MyState) -> GraphResult<()> {
        state.counter += 1;
        state.message = format!("Counter is now: {}", state.counter);
        println!("Incremented counter to: {}", state.counter);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> GraphResult<()> {
    let graph = GraphBuilder::new()
        .add_node("increment".to_string(), IncrementNode)?
        .with_entry_point("increment".to_string())?
        .add_finish_point("increment".to_string())?
        .build()?;

    let mut state = MyState {
        counter: 0,
        message: "Starting".to_string(),
    };

    let context = graph.run(&mut state).await?;
    println!("Final state: {:?}", state);
    println!("Execution took {} steps", context.current_step);
    
    Ok(())
}
```

## 📚 Core Concepts

### States

States represent the data that flows through your graph. They must implement the `State` trait and be serializable:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ResearchState {
    query: String,
    results: Vec<String>,
    metadata: HashMap<String, String>,
}

impl State for ResearchState {}
```

### Nodes

Nodes are the units of work in your graph. They implement the `Node` trait:

```rust
#[derive(Debug)]
struct WebSearchNode;

#[async_trait]
impl Node<ResearchState> for WebSearchNode {
    async fn invoke(&self, state: &mut ResearchState) -> GraphResult<()> {
        // Perform web search logic
        state.results.push("Search result".to_string());
        Ok(())
    }

    fn metadata(&self) -> NodeMetadata {
        NodeMetadata::new("WebSearch")
            .with_description("Performs web search")
            .with_tag("search")
            .with_expected_duration(1000)
    }
}
```

### Edges

Edges define how nodes are connected and how execution flows:

```rust
// Simple edge
let edge = Edge::simple("node1", "node2");

// Conditional edge
let edge = Edge::conditional("node1", "condition_id", "node2", "node3");

// Parallel edge
let edge = Edge::parallel("node1", vec!["node2", "node3", "node4"]);

// Weighted edge for probabilistic routing
let edge = Edge::weighted("node1", vec![
    ("node2".to_string(), 0.7),
    ("node3".to_string(), 0.3),
]);
```

## 🔧 Advanced Features

### Parallel Execution

Enable parallel execution for independent nodes:

```rust
let config = ExecutionConfig {
    enable_parallel: true,
    max_execution_time_seconds: Some(300),
    ..Default::default()
};

let graph = GraphBuilder::new()
    .with_config(config)
    // ... add nodes and edges
    .build()?;
```

### State Checkpointing

Save and restore graph state for fault tolerance:

```rust
use agent_graph::state::checkpointing::FileCheckpointer;

let checkpointer = FileCheckpointer::new("./checkpoints");
let mut graph = Graph::new();
graph.set_checkpointer(checkpointer);
```

### Streaming Execution

Stream execution events in real-time:

```rust
let (context, stream) = graph.run_streaming(&mut state).await?;

// Process events as they arrive
while let Some(event) = stream.next().await {
    match event {
        ExecutionEvent::NodeStarted { node_id, .. } => {
            println!("Node {} started", node_id);
        }
        ExecutionEvent::NodeCompleted { node_id, success, .. } => {
            println!("Node {} completed: {}", node_id, success);
        }
        _ => {}
    }
}
```

### Node Traits

Enhance nodes with additional capabilities:

```rust
use agent_graph::node::traits::{RetryableNode, TimeoutNode, ValidatingNode};

#[async_trait]
impl RetryableNode<MyState> for MyNode {
    fn max_retries(&self) -> u32 { 3 }
    fn retry_delay(&self) -> Duration { Duration::from_secs(1) }
}

#[async_trait]
impl TimeoutNode<MyState> for MyNode {
    fn timeout(&self) -> Duration { Duration::from_secs(30) }
}
```

## 📊 Monitoring and Observability

### Tracing

AgentGraph integrates with the `tracing` ecosystem:

```rust
// Initialize tracing
agent_graph::init_tracing();

// Tracing is automatically added to all graph operations
let context = graph.run(&mut state).await?;
```

### Execution Statistics

Get detailed execution statistics:

```rust
let stats = graph.execution_stats();
println!("Graph has {} nodes and {} edges", stats.node_count, stats.edge_count);
println!("Estimated complexity: {:?}", stats.estimated_complexity);

let summary = graph.summary();
println!("Graph summary: {}", summary);
```

## 🎯 Examples

The repository includes comprehensive examples:

- **[Simple Researcher](examples/simple_researcher.rs)**: Basic graph execution with sequential nodes
- **[Parallel Processing](examples/parallel_processing.rs)**: Demonstrates parallel node execution
- **[Streaming Chat](examples/streaming_chat.rs)**: Real-time streaming execution

Run examples with:

```bash
cargo run --example simple_researcher
cargo run --example parallel_processing
```

## 🏗️ Architecture

AgentGraph is built with a modular architecture:

- **Core Engine**: Graph execution and state management
- **Node System**: Flexible node traits and implementations
- **Edge Routing**: Dynamic and conditional routing logic
- **State Management**: Checkpointing, versioning, and persistence
- **Streaming**: Real-time event emission and processing
- **Error Handling**: Comprehensive error types and recovery

## 🔮 Roadmap

See our [detailed roadmap](ROADMAP.md) for upcoming features:

- **v0.4.0**: Ecosystem expansion with common node library
- **v1.0.0**: Graph visualization, dynamic modification, and WASM support
- **Future**: Human-in-the-loop nodes, advanced AI integrations

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

## 📄 License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 🙏 Acknowledgments

- Inspired by [LangGraph](https://github.com/langchain-ai/langgraph) from LangChain
- Built with the amazing Rust async ecosystem
- Thanks to all contributors and the Rust community

---

**AgentGraph** - Build the future of multi-agent systems in Rust 🦀✨
