//! # AgentGraph Tools Framework 🔧
//!
//! Comprehensive tools framework for the AgentGraph providing:
//! - Core tool system with traits and registry
//! - Built-in tools for common operations
//! - Secure tool execution runtime
//! - Tool metadata and discovery

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

/// Core tool system
pub mod core;

/// Built-in tools
pub mod builtin;

/// Tool execution runtime
pub mod runtime;

// Re-export core types
pub use agent_graph_core::{CoreError, CoreResult};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");