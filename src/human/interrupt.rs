// Interrupt and resume system for human-in-the-loop workflows

use super::traits::{HumanResult, InteractionError};
use crate::state::State;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Token for resuming interrupted execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResumeToken {
    /// Unique identifier for the interrupt
    pub interrupt_id: String,
    /// Graph execution ID
    pub execution_id: String,
    /// Node where execution was interrupted
    pub node_id: String,
    /// Timestamp when interrupt occurred
    pub interrupted_at: SystemTime,
    /// Expiration time for the token
    pub expires_at: Option<SystemTime>,
}

impl ResumeToken {
    /// Create a new resume token
    pub fn new(execution_id: String, node_id: String) -> Self {
        Self {
            interrupt_id: Uuid::new_v4().to_string(),
            execution_id,
            node_id,
            interrupted_at: SystemTime::now(),
            expires_at: None,
        }
    }
    
    /// Set expiration time
    pub fn with_expiration(mut self, duration: Duration) -> Self {
        self.expires_at = Some(self.interrupted_at + duration);
        self
    }
    
    /// Check if the token has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            SystemTime::now() > expires_at
        } else {
            false
        }
    }
    
    /// Get time remaining before expiration
    pub fn time_remaining(&self) -> Option<Duration> {
        self.expires_at.and_then(|expires_at| {
            expires_at.duration_since(SystemTime::now()).ok()
        })
    }
}

/// State of an interrupted execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterruptState<S: State> {
    /// Resume token
    pub token: ResumeToken,
    /// Serialized state at interrupt point
    pub state: S,
    /// Context data at interrupt
    pub context: HashMap<String, serde_json::Value>,
    /// Reason for interrupt
    pub reason: String,
    /// Human interaction data if applicable
    pub interaction_data: Option<serde_json::Value>,
}

impl<S: State> InterruptState<S> {
    /// Create a new interrupt state
    pub fn new(
        token: ResumeToken,
        state: S,
        reason: String,
    ) -> Self {
        Self {
            token,
            state,
            context: HashMap::new(),
            reason,
            interaction_data: None,
        }
    }
    
    /// Add context data
    pub fn with_context(mut self, key: String, value: serde_json::Value) -> Self {
        self.context.insert(key, value);
        self
    }
    
    /// Add interaction data
    pub fn with_interaction_data(mut self, data: serde_json::Value) -> Self {
        self.interaction_data = Some(data);
        self
    }
}

/// Point in execution where interrupt can occur
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterruptPoint {
    /// Node ID where interrupt occurs
    pub node_id: String,
    /// Interrupt type
    pub interrupt_type: InterruptType,
    /// Human interaction required
    pub requires_human: bool,
    /// Timeout for the interrupt
    pub timeout: Option<Duration>,
    /// Custom data for the interrupt
    pub data: HashMap<String, serde_json::Value>,
}

impl InterruptPoint {
    /// Create a new interrupt point
    pub fn new(node_id: String, interrupt_type: InterruptType) -> Self {
        Self {
            node_id,
            interrupt_type,
            requires_human: true,
            timeout: None,
            data: HashMap::new(),
        }
    }
    
    /// Create a human approval interrupt
    pub fn approval(node_id: String) -> Self {
        Self::new(node_id, InterruptType::HumanApproval)
    }
    
    /// Create a human input interrupt
    pub fn input(node_id: String) -> Self {
        Self::new(node_id, InterruptType::HumanInput)
    }
    
    /// Create a checkpoint interrupt
    pub fn checkpoint(node_id: String) -> Self {
        let mut point = Self::new(node_id, InterruptType::Checkpoint);
        point.requires_human = false;
        point
    }
    
    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
    
    /// Add custom data
    pub fn with_data(mut self, key: String, value: serde_json::Value) -> Self {
        self.data.insert(key, value);
        self
    }
}

/// Types of interrupts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterruptType {
    /// Human approval required
    HumanApproval,
    /// Human input required
    HumanInput,
    /// Checkpoint for state saving
    Checkpoint,
    /// Error occurred, human intervention needed
    ErrorIntervention,
    /// Custom interrupt type
    Custom(String),
}

/// Manager for handling interrupts and resumes
#[derive(Debug)]
pub struct InterruptManager<S: State> {
    /// Active interrupts
    interrupts: Arc<Mutex<HashMap<String, InterruptState<S>>>>,
    /// Interrupt points configuration
    interrupt_points: Arc<Mutex<HashMap<String, InterruptPoint>>>,
    /// Statistics
    stats: Arc<Mutex<InterruptStats>>,
}

impl<S: State> InterruptManager<S> {
    /// Create a new interrupt manager
    pub fn new() -> Self {
        Self {
            interrupts: Arc::new(Mutex::new(HashMap::new())),
            interrupt_points: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(InterruptStats::default())),
        }
    }
    
    /// Register an interrupt point
    pub fn register_interrupt_point(&self, point: InterruptPoint) -> HumanResult<()> {
        let mut points = self.interrupt_points.lock().map_err(|_| {
            InteractionError::SystemError {
                message: "Failed to acquire interrupt points lock".to_string(),
            }
        })?;
        
        points.insert(point.node_id.clone(), point);
        Ok(())
    }
    
    /// Check if a node has an interrupt point
    pub fn has_interrupt_point(&self, node_id: &str) -> bool {
        self.interrupt_points
            .lock()
            .map(|points| points.contains_key(node_id))
            .unwrap_or(false)
    }
    
    /// Create an interrupt at the specified node
    pub fn create_interrupt(
        &self,
        execution_id: String,
        node_id: String,
        state: S,
        reason: String,
    ) -> HumanResult<ResumeToken> {
        let token = ResumeToken::new(execution_id, node_id.clone());
        
        // Check if there's an interrupt point configuration
        let interrupt_point = self.interrupt_points
            .lock()
            .map_err(|_| InteractionError::SystemError {
                message: "Failed to acquire interrupt points lock".to_string(),
            })?
            .get(&node_id)
            .cloned();
        
        // Set expiration if configured
        let token = if let Some(point) = &interrupt_point {
            if let Some(timeout) = point.timeout {
                token.with_expiration(timeout)
            } else {
                token
            }
        } else {
            token
        };
        
        let interrupt_state = InterruptState::new(token.clone(), state, reason);
        
        // Store the interrupt
        let mut interrupts = self.interrupts.lock().map_err(|_| {
            InteractionError::SystemError {
                message: "Failed to acquire interrupts lock".to_string(),
            }
        })?;
        
        interrupts.insert(token.interrupt_id.clone(), interrupt_state);
        
        // Update statistics
        let mut stats = self.stats.lock().map_err(|_| {
            InteractionError::SystemError {
                message: "Failed to acquire stats lock".to_string(),
            }
        })?;
        stats.total_interrupts += 1;
        stats.active_interrupts += 1;
        
        Ok(token)
    }
    
    /// Resume execution from an interrupt
    pub fn resume_execution(&self, token: &ResumeToken) -> HumanResult<InterruptState<S>> {
        // Check if token is expired
        if token.is_expired() {
            return Err(InteractionError::TimeoutError {
                timeout_ms: 0, // Token already expired
            });
        }
        
        let mut interrupts = self.interrupts.lock().map_err(|_| {
            InteractionError::SystemError {
                message: "Failed to acquire interrupts lock".to_string(),
            }
        })?;
        
        let interrupt_state = interrupts.remove(&token.interrupt_id)
            .ok_or_else(|| InteractionError::ValidationError {
                message: format!("Invalid or expired resume token: {}", token.interrupt_id),
            })?;
        
        // Update statistics
        let mut stats = self.stats.lock().map_err(|_| {
            InteractionError::SystemError {
                message: "Failed to acquire stats lock".to_string(),
            }
        })?;
        stats.active_interrupts = stats.active_interrupts.saturating_sub(1);
        stats.resumed_interrupts += 1;
        
        Ok(interrupt_state)
    }
    
    /// Cancel an interrupt
    pub fn cancel_interrupt(&self, interrupt_id: &str) -> HumanResult<()> {
        let mut interrupts = self.interrupts.lock().map_err(|_| {
            InteractionError::SystemError {
                message: "Failed to acquire interrupts lock".to_string(),
            }
        })?;
        
        if interrupts.remove(interrupt_id).is_some() {
            // Update statistics
            let mut stats = self.stats.lock().map_err(|_| {
                InteractionError::SystemError {
                    message: "Failed to acquire stats lock".to_string(),
                }
            })?;
            stats.active_interrupts = stats.active_interrupts.saturating_sub(1);
            stats.cancelled_interrupts += 1;
            
            Ok(())
        } else {
            Err(InteractionError::ValidationError {
                message: format!("Interrupt not found: {}", interrupt_id),
            })
        }
    }
    
    /// List active interrupts
    pub fn list_active_interrupts(&self) -> HumanResult<Vec<ResumeToken>> {
        let interrupts = self.interrupts.lock().map_err(|_| {
            InteractionError::SystemError {
                message: "Failed to acquire interrupts lock".to_string(),
            }
        })?;
        
        Ok(interrupts.values().map(|state| state.token.clone()).collect())
    }
    
    /// Clean up expired interrupts
    pub fn cleanup_expired(&self) -> HumanResult<u32> {
        let mut interrupts = self.interrupts.lock().map_err(|_| {
            InteractionError::SystemError {
                message: "Failed to acquire interrupts lock".to_string(),
            }
        })?;
        
        let initial_count = interrupts.len();
        interrupts.retain(|_, state| !state.token.is_expired());
        let removed_count = initial_count - interrupts.len();
        
        // Update statistics
        if removed_count > 0 {
            let mut stats = self.stats.lock().map_err(|_| {
                InteractionError::SystemError {
                    message: "Failed to acquire stats lock".to_string(),
                }
            })?;
            stats.active_interrupts = stats.active_interrupts.saturating_sub(removed_count as u64);
            stats.expired_interrupts += removed_count as u64;
        }
        
        Ok(removed_count as u32)
    }
    
    /// Get interrupt statistics
    pub fn get_stats(&self) -> HumanResult<InterruptStats> {
        let stats = self.stats.lock().map_err(|_| {
            InteractionError::SystemError {
                message: "Failed to acquire stats lock".to_string(),
            }
        })?;
        
        Ok(stats.clone())
    }
}

impl<S: State> Default for InterruptManager<S> {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for interrupt management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterruptStats {
    /// Total number of interrupts created
    pub total_interrupts: u64,
    /// Number of currently active interrupts
    pub active_interrupts: u64,
    /// Number of successfully resumed interrupts
    pub resumed_interrupts: u64,
    /// Number of cancelled interrupts
    pub cancelled_interrupts: u64,
    /// Number of expired interrupts
    pub expired_interrupts: u64,
}

impl Default for InterruptStats {
    fn default() -> Self {
        Self {
            total_interrupts: 0,
            active_interrupts: 0,
            resumed_interrupts: 0,
            cancelled_interrupts: 0,
            expired_interrupts: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestState {
        value: i32,
    }

    #[test]
    fn test_resume_token_creation() {
        let token = ResumeToken::new("exec_1".to_string(), "node_1".to_string())
            .with_expiration(Duration::from_secs(300));
        
        assert_eq!(token.execution_id, "exec_1");
        assert_eq!(token.node_id, "node_1");
        assert!(!token.is_expired());
        assert!(token.time_remaining().is_some());
    }

    #[test]
    fn test_interrupt_point_creation() {
        let point = InterruptPoint::approval("approval_node".to_string())
            .with_timeout(Duration::from_secs(600))
            .with_data("priority".to_string(), json!("high"));
        
        assert_eq!(point.node_id, "approval_node");
        assert_eq!(point.interrupt_type, InterruptType::HumanApproval);
        assert!(point.requires_human);
        assert_eq!(point.timeout, Some(Duration::from_secs(600)));
        assert_eq!(point.data.get("priority"), Some(&json!("high")));
    }

    #[test]
    fn test_interrupt_manager() {
        let manager = InterruptManager::<TestState>::new();
        let state = TestState { value: 42 };
        
        // Create interrupt
        let token = manager.create_interrupt(
            "exec_1".to_string(),
            "node_1".to_string(),
            state.clone(),
            "Human approval required".to_string(),
        ).unwrap();
        
        // Check active interrupts
        let active = manager.list_active_interrupts().unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].interrupt_id, token.interrupt_id);
        
        // Resume execution
        let interrupt_state = manager.resume_execution(&token).unwrap();
        assert_eq!(interrupt_state.state, state);
        assert_eq!(interrupt_state.reason, "Human approval required");
        
        // Check no active interrupts
        let active = manager.list_active_interrupts().unwrap();
        assert_eq!(active.len(), 0);
    }

    #[test]
    fn test_interrupt_cancellation() {
        let manager = InterruptManager::<TestState>::new();
        let state = TestState { value: 42 };
        
        let token = manager.create_interrupt(
            "exec_1".to_string(),
            "node_1".to_string(),
            state,
            "Test interrupt".to_string(),
        ).unwrap();
        
        // Cancel interrupt
        manager.cancel_interrupt(&token.interrupt_id).unwrap();
        
        // Try to resume - should fail
        let result = manager.resume_execution(&token);
        assert!(result.is_err());
    }
}
