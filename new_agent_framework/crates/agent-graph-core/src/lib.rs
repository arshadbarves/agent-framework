//! # AgentGraph Core 🦀
//!
//! The core graph engine and execution logic for the AgentGraph framework.
//! This crate provides the fundamental building blocks for creating stateful,
//! multi-agent systems with graph-based execution.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

/// Error handling and result types
pub mod error;

/// Graph engine and execution logic
pub mod graph;

/// Node system with traits and registry
pub mod node;

/// Edge types, conditions, and routing
pub mod edge;

/// State management and snapshots
pub mod state;

/// Runtime configuration and context
pub mod runtime;

// Re-export core types for convenience
pub use error::{CoreError, CoreResult};
pub use graph::{Graph, GraphMetadata};
pub use node::{Node, NodeId, NodeMetadata, NodeRegistry};
pub use edge::{Edge, EdgeCondition, EdgeType, EdgeRegistry};
pub use state::{State, StateSnapshot, StateManager};
pub use runtime::{ExecutionContext, ExecutionConfig, RuntimeConfig};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");