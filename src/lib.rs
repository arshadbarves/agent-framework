//! # AgentGraph 🦀
//!
//! A powerful, production-grade multi-agent framework for Rust inspired by LangGraph.
//!
//! AgentGraph provides a robust foundation for building complex, stateful, multi-agent systems
//! with support for parallel execution, state checkpointing, streaming outputs, and comprehensive
//! observability.
//!
//! ## Features
//!
//! - **Stateful Execution**: Manage complex state through graph execution
//! - **Async by Design**: Built on tokio for high-performance async operations
//! - **Parallel Execution**: Run independent nodes concurrently for better performance
//! - **State Checkpointing**: Save and resume graph state for fault tolerance
//! - **Streaming Outputs**: Real-time streaming of execution results
//! - **Production-Grade Error Handling**: Comprehensive error types and handling
//! - **Observability**: Integrated tracing and metrics support
//!
//! ## Quick Start
//!
//! ```rust
//! use agent_graph::{Graph, Node, State, GraphResult};
//! use async_trait::async_trait;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Clone, Serialize, Deserialize)]
//! struct MyState {
//!     counter: i32,
//!     message: String,
//! }
//!
//! impl State for MyState {}
//!
//! struct IncrementNode;
//!
//! #[async_trait]
//! impl Node<MyState> for IncrementNode {
//!     async fn invoke(&self, state: &mut MyState) -> GraphResult<()> {
//!         state.counter += 1;
//!         state.message = format!("Counter is now: {}", state.counter);
//!         Ok(())
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> GraphResult<()> {
//!     let mut graph = Graph::new();
//!     graph.add_node("increment", IncrementNode)?;
//!     graph.set_entry_point("increment")?;
//!     graph.set_finish_point("increment")?;
//!
//!     let mut state = MyState {
//!         counter: 0,
//!         message: "Starting".to_string(),
//!     };
//!
//!     graph.run(&mut state).await?;
//!     println!("Final state: {:?}", state);
//!     Ok(())
//! }
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod error;
pub mod graph;
pub mod node;
pub mod state;
pub mod edge;

#[cfg(feature = "streaming")]
#[cfg_attr(docsrs, doc(cfg(feature = "streaming")))]
pub mod streaming;

/// Tools framework for integrating external capabilities
pub mod tools;

/// Human-in-the-loop interaction system
pub mod human;

// Re-export core types for convenience
pub use error::{GraphError, GraphResult};
pub use graph::{Graph, GraphBuilder, ExecutionContext, ExecutionConfig};
pub use node::{Node, NodeId, NodeMetadata};
pub use state::{State, StateSnapshot};
pub use edge::{Edge, EdgeCondition, EdgeType};

#[cfg(feature = "streaming")]
pub use streaming::{ExecutionEvent, ExecutionStream};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize tracing for the framework
pub fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
