//! Core error types for the AgentGraph framework.

use thiserror::Error;

/// Comprehensive error types for the AgentGraph core framework
#[derive(Error, Debug)]
pub enum CoreError {
    /// Node-related errors
    #[error("Node error in '{node_id}': {message}")]
    NodeError {
        /// The ID of the node that caused the error
        node_id: String,
        /// Error message
        message: String,
        /// Optional source error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Graph structure errors
    #[error("Graph structure error: {0}")]
    GraphStructure(String),

    /// State management errors
    #[error("State error: {0}")]
    StateError(String),

    /// Execution errors
    #[error("Execution error: {0}")]
    ExecutionError(String),

    /// Edge routing errors
    #[error("Edge routing error: {0}")]
    EdgeError(String),

    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// I/O errors
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Timeout errors
    #[error("Operation timed out after {seconds} seconds")]
    Timeout {
        /// Number of seconds before timeout
        seconds: u64
    },

    /// Concurrency errors
    #[error("Concurrency error: {0}")]
    ConcurrencyError(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// Resource errors (memory, file handles, etc.)
    #[error("Resource error: {0}")]
    ResourceError(String),

    /// Validation errors
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Generic internal errors
    #[error("Internal error: {0}")]
    Internal(String),
}

impl CoreError {
    /// Create a new node error
    pub fn node_error<S: Into<String>>(
        node_id: S,
        message: S,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self::NodeError {
            node_id: node_id.into(),
            message: message.into(),
            source,
        }
    }

    /// Create a new graph structure error
    pub fn graph_structure<S: Into<String>>(message: S) -> Self {
        Self::GraphStructure(message.into())
    }

    /// Create a new state error
    pub fn state_error<S: Into<String>>(message: S) -> Self {
        Self::StateError(message.into())
    }

    /// Create a new execution error
    pub fn execution_error<S: Into<String>>(message: S) -> Self {
        Self::ExecutionError(message.into())
    }

    /// Create a new edge error
    pub fn edge_error<S: Into<String>>(message: S) -> Self {
        Self::EdgeError(message.into())
    }

    /// Create a new timeout error
    pub fn timeout(seconds: u64) -> Self {
        Self::Timeout { seconds }
    }

    /// Create a new validation error
    pub fn validation_error<S: Into<String>>(message: S) -> Self {
        Self::ValidationError(message.into())
    }

    /// Create a new configuration error
    pub fn configuration_error<S: Into<String>>(message: S) -> Self {
        Self::ConfigurationError(message.into())
    }

    /// Create a new internal error
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::Internal(message.into())
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            CoreError::Timeout { .. }
                | CoreError::ResourceError(_)
                | CoreError::ConcurrencyError(_)
        )
    }

    /// Get the error category for metrics/logging
    pub fn category(&self) -> &'static str {
        match self {
            CoreError::NodeError { .. } => "node",
            CoreError::GraphStructure(_) => "graph_structure",
            CoreError::StateError(_) => "state",
            CoreError::ExecutionError(_) => "execution",
            CoreError::EdgeError(_) => "edge",
            CoreError::SerializationError(_) => "serialization",
            CoreError::IoError(_) => "io",
            CoreError::Timeout { .. } => "timeout",
            CoreError::ConcurrencyError(_) => "concurrency",
            CoreError::ConfigurationError(_) => "configuration",
            CoreError::ResourceError(_) => "resource",
            CoreError::ValidationError(_) => "validation",
            CoreError::Internal(_) => "internal",
        }
    }
}