//! State management for the AgentGraph framework.

pub mod checkpointing;
pub mod management;

use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Core trait that all state objects must implement
pub trait State: Debug + Clone + Send + Sync + 'static {
    /// Get a value from the state by key (optional for advanced state access)
    fn get_value(&self, key: &str) -> Option<serde_json::Value> {
        // Default implementation returns None - states can override this
        let _ = key;
        None
    }

    /// Set a value in the state by key (optional for advanced state access)
    fn set_value(&mut self, key: &str, value: serde_json::Value) -> crate::error::GraphResult<()> {
        // Default implementation does nothing - states can override this
        let _ = (key, value);
        Ok(())
    }

    /// Serialize the state to JSON (optional for advanced state access)
    fn to_json(&self) -> crate::error::GraphResult<serde_json::Value> {
        // Default implementation tries to serialize using serde
        serde_json::to_value(self).map_err(|e| {
            crate::error::GraphError::state_error(format!("Failed to serialize state: {}", e))
        })
    }

    /// Get all keys in the state (optional for advanced state access)
    fn keys(&self) -> Vec<String> {
        // Default implementation returns empty vector
        Vec::new()
    }
}

/// Automatic implementation for types that meet the requirements
impl<T> State for T where T: Debug + Clone + Send + Sync + 'static {}

/// A snapshot of the graph state at a specific point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot<S> {
    /// Unique identifier for this snapshot
    pub id: Uuid,
    /// Timestamp when the snapshot was created
    pub timestamp: DateTime<Utc>,
    /// The actual state data
    pub state: S,
    /// Optional metadata about the snapshot
    pub metadata: SnapshotMetadata,
}

/// Metadata associated with a state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    /// The node that was executing when this snapshot was created
    pub current_node: Option<String>,
    /// The execution step number
    pub step: u64,
    /// Optional tags for categorizing snapshots
    pub tags: Vec<String>,
    /// Custom metadata fields
    pub custom: std::collections::HashMap<String, serde_json::Value>,
}

impl Default for SnapshotMetadata {
    fn default() -> Self {
        Self {
            current_node: None,
            step: 0,
            tags: Vec::new(),
            custom: std::collections::HashMap::new(),
        }
    }
}

impl<S> StateSnapshot<S>
where
    S: Clone + Serialize + for<'de> Deserialize<'de>,
{
    /// Create a new state snapshot
    pub fn new(state: S) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            state,
            metadata: SnapshotMetadata::default(),
        }
    }

    /// Create a new state snapshot with metadata
    pub fn with_metadata(state: S, metadata: SnapshotMetadata) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            state,
            metadata,
        }
    }

    /// Add a tag to this snapshot
    pub fn add_tag<T: Into<String>>(&mut self, tag: T) {
        self.metadata.tags.push(tag.into());
    }

    /// Set custom metadata
    pub fn set_custom_metadata<K, V>(&mut self, key: K, value: V)
    where
        K: Into<String>,
        V: Serialize,
    {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.metadata.custom.insert(key.into(), json_value);
        }
    }

    /// Get custom metadata
    pub fn get_custom_metadata<T>(&self, key: &str) -> Option<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.metadata
            .custom
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }
}

/// State manager for handling state operations
#[derive(Debug)]
pub struct StateManager<S> {
    /// Current state
    current_state: S,
    /// History of state snapshots
    snapshots: Vec<StateSnapshot<S>>,
    /// Maximum number of snapshots to keep
    max_snapshots: usize,
}

impl<S> StateManager<S>
where
    S: State + Serialize + for<'de> Deserialize<'de>,
{
    /// Create a new state manager
    pub fn new(initial_state: S) -> Self {
        Self {
            current_state: initial_state,
            snapshots: Vec::new(),
            max_snapshots: 100, // Default limit
        }
    }

    /// Create a new state manager with custom snapshot limit
    pub fn with_snapshot_limit(initial_state: S, max_snapshots: usize) -> Self {
        Self {
            current_state: initial_state,
            snapshots: Vec::new(),
            max_snapshots,
        }
    }

    /// Get a reference to the current state
    pub fn current_state(&self) -> &S {
        &self.current_state
    }

    /// Get a mutable reference to the current state
    pub fn current_state_mut(&mut self) -> &mut S {
        &mut self.current_state
    }

    /// Create a snapshot of the current state
    pub fn create_snapshot(&mut self) -> Uuid {
        self.create_snapshot_with_metadata(SnapshotMetadata::default())
    }

    /// Create a snapshot with custom metadata
    pub fn create_snapshot_with_metadata(&mut self, metadata: SnapshotMetadata) -> Uuid {
        let snapshot = StateSnapshot::with_metadata(self.current_state.clone(), metadata);
        let id = snapshot.id;
        
        self.snapshots.push(snapshot);
        
        // Maintain snapshot limit
        if self.snapshots.len() > self.max_snapshots {
            self.snapshots.remove(0);
        }
        
        id
    }

    /// Restore state from a snapshot
    pub fn restore_from_snapshot(&mut self, snapshot_id: Uuid) -> crate::error::GraphResult<()> {
        if let Some(snapshot) = self.snapshots.iter().find(|s| s.id == snapshot_id) {
            self.current_state = snapshot.state.clone();
            Ok(())
        } else {
            Err(crate::error::GraphError::state_error(format!(
                "Snapshot with ID {} not found",
                snapshot_id
            )))
        }
    }

    /// Get all snapshots
    pub fn snapshots(&self) -> &[StateSnapshot<S>] {
        &self.snapshots
    }

    /// Get a specific snapshot by ID
    pub fn get_snapshot(&self, snapshot_id: Uuid) -> Option<&StateSnapshot<S>> {
        self.snapshots.iter().find(|s| s.id == snapshot_id)
    }

    /// Clear all snapshots
    pub fn clear_snapshots(&mut self) {
        self.snapshots.clear();
    }

    /// Get the number of snapshots
    pub fn snapshot_count(&self) -> usize {
        self.snapshots.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestState {
        value: i32,
        message: String,
    }

    #[test]
    fn test_state_manager() {
        let initial_state = TestState {
            value: 0,
            message: "initial".to_string(),
        };

        let mut manager = StateManager::new(initial_state.clone());
        assert_eq!(manager.current_state(), &initial_state);

        // Create a snapshot
        let snapshot_id = manager.create_snapshot();
        assert_eq!(manager.snapshot_count(), 1);

        // Modify state
        manager.current_state_mut().value = 42;
        manager.current_state_mut().message = "modified".to_string();

        // Restore from snapshot
        manager.restore_from_snapshot(snapshot_id).unwrap();
        assert_eq!(manager.current_state(), &initial_state);
    }

    #[test]
    fn test_snapshot_metadata() {
        let state = TestState {
            value: 1,
            message: "test".to_string(),
        };

        let mut snapshot = StateSnapshot::new(state);
        snapshot.add_tag("test");
        snapshot.set_custom_metadata("step", 5);

        assert!(snapshot.metadata.tags.contains(&"test".to_string()));
        assert_eq!(snapshot.get_custom_metadata::<i32>("step"), Some(5));
    }
}
