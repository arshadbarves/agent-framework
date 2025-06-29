//! # AgentGraph 🤖
//!
//! A powerful, production-grade multi-agent framework for Rust.
//! 
//! This is the main crate that re-exports functionality from all other
//! AgentGraph crates, providing a unified API for users.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

// Re-export core functionality (always available)
pub use agent_graph_core::*;

// Re-export optional features
#[cfg(feature = "execution")]
#[cfg_attr(docsrs, doc(cfg(feature = "execution")))]
pub use agent_graph_execution as execution;

#[cfg(feature = "agents")]
#[cfg_attr(docsrs, doc(cfg(feature = "agents")))]
pub use agent_graph_agents as agents;

#[cfg(feature = "llm")]
#[cfg_attr(docsrs, doc(cfg(feature = "llm")))]
pub use agent_graph_llm as llm;

#[cfg(feature = "tools")]
#[cfg_attr(docsrs, doc(cfg(feature = "tools")))]
pub use agent_graph_tools as tools;

#[cfg(feature = "human")]
#[cfg_attr(docsrs, doc(cfg(feature = "human")))]
pub use agent_graph_human as human;

#[cfg(feature = "enterprise")]
#[cfg_attr(docsrs, doc(cfg(feature = "enterprise")))]
pub use agent_graph_enterprise as enterprise;

#[cfg(feature = "visualization")]
#[cfg_attr(docsrs, doc(cfg(feature = "visualization")))]
pub use agent_graph_visualization as visualization;

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the AgentGraph framework with default settings
pub fn init() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}

/// Initialize the AgentGraph framework with custom tracing
pub fn init_with_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}

/// Prelude module for convenient imports
pub mod prelude {
    // Core types
    pub use agent_graph_core::{
        CoreError, CoreResult, State, Node, NodeId, NodeMetadata, NodeOutput,
        Graph, GraphBuilder, Edge, EdgeCondition, EdgeType,
        StateManager, StateSnapshot, ExecutionContext, ExecutionConfig,
    };

    #[cfg(feature = "agents")]
    pub use agent_graph_agents::{
        Agent, AgentBuilder, AgentConfig, AgentRole, AgentRuntime,
    };

    #[cfg(feature = "llm")]
    pub use agent_graph_llm::{
        LLMClient, LLMClientBuilder, CompletionRequest, CompletionResponse,
        Message, MessageRole, ModelInfo,
    };

    #[cfg(feature = "tools")]
    pub use agent_graph_tools::{
        Tool, ToolRegistry, ToolInput, ToolOutput, ToolMetadata,
    };

    #[cfg(feature = "human")]
    pub use agent_graph_human::{
        HumanInputCollector, InputRequest, InputResponse, InputManager,
        ApprovalSystem, ApprovalRequest, ApprovalResponse, ApprovalManager,
    };

    #[cfg(feature = "enterprise")]
    pub use agent_graph_enterprise::{
        EnterpriseManager, EnterpriseConfig, TenantManager, SecurityManager,
    };
}