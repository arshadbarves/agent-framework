//! # AgentGraph Human-in-the-Loop 👥
//!
//! Human-in-the-loop workflows and interfaces for AgentGraph providing:
//! - Approval workflows and policies
//! - Human input collection and validation
//! - Execution interruption and recovery
//! - Web and CLI interfaces

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

/// Approval workflows
pub mod approval;

/// Human input collection
pub mod input;

/// Execution interruption
pub mod interrupt;

/// User interfaces
pub mod interface;

// Re-export core types
pub use agent_graph_core::{CoreError, CoreResult};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");