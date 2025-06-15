//! Advanced state management utilities and patterns.

use crate::error::GraphResult;
use crate::state::State;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;

/// A thread-safe, shared state container
#[derive(Debug)]
pub struct SharedState<S> {
    inner: Arc<RwLock<S>>,
}

impl<S> Clone for SharedState<S> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<S> SharedState<S>
where
    S: State,
{
    /// Create a new shared state
    pub fn new(state: S) -> Self {
        Self {
            inner: Arc::new(RwLock::new(state)),
        }
    }

    /// Read the state
    pub fn read<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&S) -> R,
    {
        let guard = self.inner.read();
        f(&*guard)
    }

    /// Write to the state
    pub fn write<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut S) -> R,
    {
        let mut guard = self.inner.write();
        f(&mut *guard)
    }

    /// Try to read the state (non-blocking)
    pub fn try_read<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&S) -> R,
    {
        self.inner.try_read().map(|guard| f(&*guard))
    }

    /// Try to write to the state (non-blocking)
    pub fn try_write<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&mut S) -> R,
    {
        self.inner.try_write().map(|mut guard| f(&mut *guard))
    }
}

/// State versioning for tracking changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedState<S> {
    /// The actual state
    pub state: S,
    /// Version number
    pub version: u64,
    /// Hash of the state for integrity checking
    pub hash: String,
}

impl<S> VersionedState<S>
where
    S: Serialize,
{
    /// Create a new versioned state
    pub fn new(state: S) -> GraphResult<Self> {
        let serialized = serde_json::to_string(&state)?;
        let hash = format!("{:x}", md5::compute(serialized.as_bytes()));
        
        Ok(Self {
            state,
            version: 1,
            hash,
        })
    }

    /// Update the state and increment version
    pub fn update(&mut self, new_state: S) -> GraphResult<()> {
        let serialized = serde_json::to_string(&new_state)?;
        let new_hash = format!("{:x}", md5::compute(serialized.as_bytes()));
        
        self.state = new_state;
        self.version += 1;
        self.hash = new_hash;
        
        Ok(())
    }

    /// Verify the integrity of the state
    pub fn verify_integrity(&self) -> GraphResult<bool> {
        let serialized = serde_json::to_string(&self.state)?;
        let computed_hash = format!("{:x}", md5::compute(serialized.as_bytes()));
        Ok(computed_hash == self.hash)
    }
}

/// State diff for tracking changes between versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDiff {
    /// From version
    pub from_version: u64,
    /// To version
    pub to_version: u64,
    /// JSON patch representing the changes
    pub patch: serde_json::Value,
    /// Timestamp of the change
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Advanced state manager with versioning and diffing
#[derive(Debug)]
pub struct AdvancedStateManager<S> {
    /// Current versioned state
    current: VersionedState<S>,
    /// History of state diffs
    diffs: Vec<StateDiff>,
    /// Maximum number of diffs to keep
    max_diffs: usize,
}

impl<S> AdvancedStateManager<S>
where
    S: State + Serialize + for<'de> Deserialize<'de>,
{
    /// Create a new advanced state manager
    pub fn new(initial_state: S) -> GraphResult<Self> {
        Ok(Self {
            current: VersionedState::new(initial_state)?,
            diffs: Vec::new(),
            max_diffs: 1000,
        })
    }

    /// Get the current state
    pub fn current_state(&self) -> &S {
        &self.current.state
    }

    /// Get the current version
    pub fn current_version(&self) -> u64 {
        self.current.version
    }

    /// Update the state
    pub fn update_state(&mut self, new_state: S) -> GraphResult<()> {
        let old_version = self.current.version;
        
        // Create diff (simplified - in production you'd use a proper JSON diff library)
        let diff = StateDiff {
            from_version: old_version,
            to_version: old_version + 1,
            patch: serde_json::json!({}), // Placeholder
            timestamp: chrono::Utc::now(),
        };
        
        self.current.update(new_state)?;
        self.diffs.push(diff);
        
        // Maintain diff history limit
        if self.diffs.len() > self.max_diffs {
            self.diffs.remove(0);
        }
        
        Ok(())
    }

    /// Get all diffs
    pub fn diffs(&self) -> &[StateDiff] {
        &self.diffs
    }

    /// Verify state integrity
    pub fn verify_integrity(&self) -> GraphResult<bool> {
        self.current.verify_integrity()
    }
}

/// State middleware for intercepting state changes
pub trait StateMiddleware<S>: Send + Sync {
    /// Called before state is modified
    fn before_update(&self, current: &S, new: &S) -> GraphResult<()>;
    
    /// Called after state is modified
    fn after_update(&self, old: &S, current: &S) -> GraphResult<()>;
}

/// Logging middleware for state changes
#[derive(Debug)]
pub struct LoggingMiddleware;

impl<S> StateMiddleware<S> for LoggingMiddleware
where
    S: std::fmt::Debug,
{
    fn before_update(&self, _current: &S, _new: &S) -> GraphResult<()> {
        tracing::debug!("State update starting");
        Ok(())
    }
    
    fn after_update(&self, _old: &S, current: &S) -> GraphResult<()> {
        tracing::debug!("State updated: {:?}", current);
        Ok(())
    }
}

/// Validation middleware for state changes
#[derive(Debug)]
pub struct ValidationMiddleware<F> {
    validator: F,
}

impl<F> ValidationMiddleware<F> {
    /// Create a new validation middleware
    pub fn new(validator: F) -> Self {
        Self { validator }
    }
}

impl<S, F> StateMiddleware<S> for ValidationMiddleware<F>
where
    F: Fn(&S) -> GraphResult<()> + Send + Sync,
{
    fn before_update(&self, _current: &S, new: &S) -> GraphResult<()> {
        (self.validator)(new)
    }
    
    fn after_update(&self, _old: &S, _current: &S) -> GraphResult<()> {
        Ok(())
    }
}

/// State manager with middleware support
pub struct MiddlewareStateManager<S> {
    state: S,
    middleware: Vec<Box<dyn StateMiddleware<S>>>,
}

impl<S> MiddlewareStateManager<S>
where
    S: State,
{
    /// Create a new middleware state manager
    pub fn new(state: S) -> Self {
        Self {
            state,
            middleware: Vec::new(),
        }
    }

    /// Add middleware
    pub fn add_middleware<M>(&mut self, middleware: M)
    where
        M: StateMiddleware<S> + 'static,
    {
        self.middleware.push(Box::new(middleware));
    }

    /// Get the current state
    pub fn state(&self) -> &S {
        &self.state
    }

    /// Update the state with middleware
    pub fn update_state(&mut self, new_state: S) -> GraphResult<()> {
        // Run before_update middleware
        for middleware in &self.middleware {
            middleware.before_update(&self.state, &new_state)?;
        }

        let old_state = std::mem::replace(&mut self.state, new_state);

        // Run after_update middleware
        for middleware in &self.middleware {
            middleware.after_update(&old_state, &self.state)?;
        }

        Ok(())
    }
}

impl<S> std::fmt::Debug for MiddlewareStateManager<S>
where
    S: State,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MiddlewareStateManager")
            .field("state", &self.state)
            .field("middleware_count", &self.middleware.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestState {
        value: i32,
        name: String,
    }

    #[test]
    fn test_shared_state() {
        let state = TestState {
            value: 42,
            name: "test".to_string(),
        };
        
        let shared = SharedState::new(state.clone());
        
        // Test read
        let value = shared.read(|s| s.value);
        assert_eq!(value, 42);
        
        // Test write
        shared.write(|s| s.value = 100);
        let new_value = shared.read(|s| s.value);
        assert_eq!(new_value, 100);
    }

    #[test]
    fn test_versioned_state() {
        let state = TestState {
            value: 1,
            name: "initial".to_string(),
        };
        
        let mut versioned = VersionedState::new(state).unwrap();
        assert_eq!(versioned.version, 1);
        assert!(versioned.verify_integrity().unwrap());
        
        let new_state = TestState {
            value: 2,
            name: "updated".to_string(),
        };
        
        versioned.update(new_state).unwrap();
        assert_eq!(versioned.version, 2);
        assert!(versioned.verify_integrity().unwrap());
    }

    #[test]
    fn test_middleware_state_manager() {
        let state = TestState {
            value: 1,
            name: "test".to_string(),
        };
        
        let mut manager = MiddlewareStateManager::new(state);
        manager.add_middleware(LoggingMiddleware);
        
        let new_state = TestState {
            value: 2,
            name: "updated".to_string(),
        };
        
        manager.update_state(new_state).unwrap();
        assert_eq!(manager.state().value, 2);
    }
}
