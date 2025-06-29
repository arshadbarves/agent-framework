//! # AgentGraph Visualization 📊
//!
//! Visualization, debugging, and monitoring interface for AgentGraph providing:
//! - Execution tracing and analysis
//! - Graph visualization and layouts
//! - Metrics collection and dashboards
//! - Web interface and data export

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

/// Execution tracing
pub mod tracer;

/// Graph visualization
pub mod visualizer;

/// Metrics collection
pub mod metrics;

/// Web interface
pub mod web;

/// Data export
pub mod export;

// Re-export core types
pub use agent_graph_core::{CoreError, CoreResult};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");