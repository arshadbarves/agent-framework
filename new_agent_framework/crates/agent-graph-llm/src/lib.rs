//! # AgentGraph LLM Integration 🧠
//!
//! Multi-provider LLM integration for the AgentGraph framework providing:
//! - Unified LLM client abstraction
//! - Multiple provider support (OpenAI, Anthropic, Google, etc.)
//! - Function calling and tool integration
//! - Rate limiting and retry mechanisms

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

/// LLM client abstraction
pub mod client;

/// LLM providers
pub mod providers;

/// Common types and messages
pub mod types;

/// Utilities for rate limiting, retry, etc.
pub mod utils;

// Re-export core types
pub use agent_graph_core::{CoreError, CoreResult};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");