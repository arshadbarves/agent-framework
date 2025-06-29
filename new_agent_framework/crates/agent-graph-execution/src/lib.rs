//! # AgentGraph Execution Engine 🚀
//!
//! Advanced execution engine for the AgentGraph framework providing:
//! - Parallel node execution with dependency management
//! - Streaming execution with real-time updates
//! - State checkpointing and recovery
//! - Advanced scheduling algorithms

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

/// Parallel execution with dependency management
pub mod parallel;

/// Streaming execution with real-time updates
pub mod streaming;

/// State checkpointing and recovery
pub mod checkpoint;

/// Advanced scheduling algorithms
pub mod scheduler;

// Re-export core types
pub use agent_graph_core::{CoreError, CoreResult, State, Node, NodeId};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");