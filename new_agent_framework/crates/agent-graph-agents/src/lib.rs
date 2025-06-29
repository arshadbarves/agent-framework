//! # AgentGraph Agents 🤖
//!
//! Advanced agent system for the AgentGraph framework providing:
//! - Role-based agent templates and behaviors
//! - Advanced memory systems with retrieval and storage
//! - Multi-agent collaboration and communication
//! - Agent lifecycle management

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

/// Core agent system
pub mod agent;

/// Agent roles and templates
pub mod roles;

/// Agent memory systems
pub mod memory;

/// Multi-agent collaboration
pub mod collaboration;

// Re-export core types
pub use agent_graph_core::{CoreError, CoreResult, State, Node, NodeId};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");