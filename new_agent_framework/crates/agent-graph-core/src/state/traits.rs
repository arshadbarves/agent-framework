//! Core state traits and abstractions.

use crate::error::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Core trait that all state objects must implement
pub trait State: Debug + Clone + Send + Sync + 'static {
    /// Get a value from the state by key (optional for advanced state access)
    fn get_value(&self, key: &str) -> Option<serde_json::Value> {
        // Default implementation returns None - states can override this
        let _ = key;
        None
    }

    /// Set a value in the state by key (optional for advanced state access)
    fn set_value(&mut self, key: &str, value: serde_json::Value) -> CoreResult<()> {
        // Default implementation does nothing - states can override this
        let _ = (key, value);
        Ok(())
    }

    /// Serialize the state to JSON (optional for advanced state access)
    fn to_json(&self) -> CoreResult<serde_json::Value> {
        // Default implementation tries to serialize using serde if possible
        // For now, return empty object
        Ok(serde_json::json!({}))
    }

    /// Get all keys in the state (optional for advanced state access)
    fn keys(&self) -> Vec<String> {
        // Default implementation returns empty vector
        Vec::new()
    }

    /// Validate the state (optional)
    fn validate(&self) -> CoreResult<()> {
        Ok(())
    }

    /// Get state size in bytes (optional for memory management)
    fn size_bytes(&self) -> usize {
        // Default implementation estimates based on JSON serialization
        self.to_json()
            .map(|json| json.to_string().len())
            .unwrap_or(0)
    }
}

/// Automatic implementation for types that meet the requirements
impl<T> State for T where T: Debug + Clone + Send + Sync + 'static {}

/// Trait for states that support versioning
pub trait VersionedState: State {
    /// Get the current version of the state
    fn version(&self) -> u64;
    
    /// Set the version of the state
    fn set_version(&mut self, version: u64);
    
    /// Increment the version
    fn increment_version(&mut self) {
        let current = self.version();
        self.set_version(current + 1);
    }
}