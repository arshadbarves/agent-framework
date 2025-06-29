//! State snapshot functionality for checkpointing and recovery.

use crate::error::{CoreError, CoreResult};
use crate::state::State;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// A snapshot of the graph state at a specific point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot<S> {
    /// Unique identifier for this snapshot
    pub id: Uuid,
    /// Timestamp when the snapshot was created
    pub timestamp: DateTime<Utc>,
    /// The actual state data
    pub state: S,
    /// Metadata associated with this snapshot
    pub metadata: HashMap<String, serde_json::Value>,
    /// Version of the snapshot format
    pub version: u32,
    /// Optional description of the snapshot
    pub description: Option<String>,
}

impl<S> StateSnapshot<S>
where
    S: State,
{
    /// Create a new state snapshot
    pub fn new(state: S) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            state,
            metadata: HashMap::new(),
            version: 1,
            description: None,
        }
    }

    /// Create a new state snapshot with description
    pub fn with_description(state: S, description: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            state,
            metadata: HashMap::new(),
            version: 1,
            description: Some(description),
        }
    }

    /// Get the state from the snapshot
    pub fn state(&self) -> &S {
        &self.state
    }

    /// Take ownership of the state from the snapshot
    pub fn into_state(self) -> S {
        self.state
    }

    /// Get metadata value by key
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }

    /// Set metadata value
    pub fn set_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
    }

    /// Get the age of this snapshot
    pub fn age(&self) -> chrono::Duration {
        Utc::now() - self.timestamp
    }

    /// Check if this snapshot is older than the given duration
    pub fn is_older_than(&self, duration: chrono::Duration) -> bool {
        self.age() > duration
    }
}