//! Result types and utilities for error handling.

use super::types::CoreError;

/// Result type alias for core operations
pub type CoreResult<T> = Result<T, CoreError>;