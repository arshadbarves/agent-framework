//! Error types and handling for the AgentGraph framework.

use thiserror::Error;

/// Result type alias for graph operations
pub type GraphResult<T> = Result<T, GraphError>;

/// Comprehensive error types for the AgentGraph framework
#[derive(Error, Debug)]
pub enum GraphError {
    /// Node-related errors
    #[error("Node error: {message}")]
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

    /// Checkpointing errors
    #[error("Checkpointing error: {0}")]
    CheckpointError(String),

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

    /// Network/external service errors
    #[error("External service error: {0}")]
    ExternalServiceError(String),

    /// Validation errors
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Generic internal errors
    #[error("Internal error: {0}")]
    Internal(String),
}

impl GraphError {
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

    /// Create a new timeout error
    pub fn timeout(seconds: u64) -> Self {
        Self::Timeout { seconds }
    }

    /// Create a new validation error
    pub fn validation_error<S: Into<String>>(message: S) -> Self {
        Self::ValidationError(message.into())
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            GraphError::Timeout { .. }
                | GraphError::ExternalServiceError(_)
                | GraphError::ResourceError(_)
                | GraphError::ConcurrencyError(_)
        )
    }

    /// Get the error category for metrics/logging
    pub fn category(&self) -> &'static str {
        match self {
            GraphError::NodeError { .. } => "node",
            GraphError::GraphStructure(_) => "graph_structure",
            GraphError::StateError(_) => "state",
            GraphError::ExecutionError(_) => "execution",
            GraphError::CheckpointError(_) => "checkpoint",
            GraphError::SerializationError(_) => "serialization",
            GraphError::IoError(_) => "io",
            GraphError::Timeout { .. } => "timeout",
            GraphError::ConcurrencyError(_) => "concurrency",
            GraphError::ConfigurationError(_) => "configuration",
            GraphError::ResourceError(_) => "resource",
            GraphError::ExternalServiceError(_) => "external_service",
            GraphError::ValidationError(_) => "validation",
            GraphError::Internal(_) => "internal",
        }
    }
}

/// Extension trait for converting errors to GraphError
pub trait IntoGraphError<T> {
    /// Convert the result into a GraphResult
    fn into_graph_error(self) -> GraphResult<T>;
}

impl<T, E> IntoGraphError<T> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn into_graph_error(self) -> GraphResult<T> {
        self.map_err(|e| GraphError::Internal(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_categories() {
        let errors = vec![
            GraphError::node_error("test", "message", None),
            GraphError::graph_structure("test"),
            GraphError::state_error("test"),
            GraphError::timeout(30),
        ];

        for error in errors {
            assert!(!error.category().is_empty());
        }
    }

    #[test]
    fn test_recoverable_errors() {
        assert!(GraphError::timeout(30).is_recoverable());
        assert!(!GraphError::graph_structure("test").is_recoverable());
    }
}
