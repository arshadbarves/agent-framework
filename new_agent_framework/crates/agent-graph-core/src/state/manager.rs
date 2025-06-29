//! State management and coordination.

use crate::error::{CoreError, CoreResult};
use crate::state::{State, StateSnapshot};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// State manager for coordinating state access and snapshots
#[derive(Debug)]
pub struct StateManager<S> {
    /// Current state
    current_state: Arc<RwLock<S>>,
    /// State metadata
    metadata: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    /// State change listeners
    listeners: Arc<RwLock<Vec<Box<dyn StateChangeListener<S>>>>>,
}

impl<S> StateManager<S>
where
    S: State,
{
    /// Create a new state manager
    pub fn new(initial_state: S) -> Self {
        Self {
            current_state: Arc::new(RwLock::new(initial_state)),
            metadata: Arc::new(RwLock::new(HashMap::new())),
            listeners: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get a read-only reference to the current state
    pub fn read_state<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&S) -> R,
    {
        let state = self.current_state.read();
        f(&*state)
    }

    /// Get a mutable reference to the current state
    pub async fn write_state<F, R>(&self, f: F) -> CoreResult<R>
    where
        F: FnOnce(&mut S) -> CoreResult<R>,
    {
        let mut state = self.current_state.write();
        let old_state = state.clone();
        
        match f(&mut *state) {
            Ok(result) => {
                // Notify listeners of state change
                self.notify_listeners(&old_state, &*state);
                Ok(result)
            }
            Err(e) => {
                // Restore old state on error
                *state = old_state;
                Err(e)
            }
        }
    }

    /// Update the state with a new value
    pub fn update_state(&self, new_state: S) -> CoreResult<()> {
        let old_state = {
            let mut state = self.current_state.write();
            let old = state.clone();
            *state = new_state;
            old
        };

        // Notify listeners
        let current = self.current_state.read();
        self.notify_listeners(&old_state, &*current);
        
        Ok(())
    }

    /// Create a snapshot of the current state
    pub fn create_snapshot(&self) -> CoreResult<StateSnapshot<S>> {
        let state = self.current_state.read().clone();
        Ok(StateSnapshot::new(state))
    }

    /// Set metadata
    pub fn set_metadata(&self, key: String, value: serde_json::Value) {
        let mut metadata = self.metadata.write();
        metadata.insert(key, value);
    }

    /// Get metadata
    pub fn get_metadata(&self, key: &str) -> Option<serde_json::Value> {
        let metadata = self.metadata.read();
        metadata.get(key).cloned()
    }

    /// Add a state change listener
    pub fn add_listener(&self, listener: Box<dyn StateChangeListener<S>>) {
        let mut listeners = self.listeners.write();
        listeners.push(listener);
    }

    /// Notify all listeners of a state change
    fn notify_listeners(&self, old_state: &S, new_state: &S) {
        let listeners = self.listeners.read();
        for listener in listeners.iter() {
            listener.on_state_changed(old_state, new_state);
        }
    }

    /// Get current state size in bytes
    pub fn state_size(&self) -> usize {
        let state = self.current_state.read();
        state.size_bytes()
    }

    /// Validate current state
    pub fn validate_state(&self) -> CoreResult<()> {
        let state = self.current_state.read();
        state.validate()
    }
}

impl<S> Clone for StateManager<S>
where
    S: State,
{
    fn clone(&self) -> Self {
        Self {
            current_state: Arc::clone(&self.current_state),
            metadata: Arc::clone(&self.metadata),
            listeners: Arc::clone(&self.listeners),
        }
    }
}

/// Trait for listening to state changes
pub trait StateChangeListener<S>: Send + Sync {
    /// Called when the state changes
    fn on_state_changed(&self, old_state: &S, new_state: &S);
}