//! # AgentGraph Enterprise Features 🏢
//!
//! Enterprise-grade features for AgentGraph providing:
//! - Multi-tenancy and isolation
//! - Security, RBAC, and audit logging
//! - Monitoring and observability
//! - Resource management and quotas

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

/// Multi-tenancy support
pub mod tenancy;

/// Security and RBAC
pub mod security;

/// Monitoring and observability
pub mod monitoring;

/// Resource management
pub mod resources;

// Re-export core types
pub use agent_graph_core::{CoreError, CoreResult};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");