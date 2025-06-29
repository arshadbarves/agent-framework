//! Interrupt and resume system for human-in-the-loop workflows.

use crate::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
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
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
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
            metadata: HashMap::new(),
        }
    }

    /// Set expiration time
    pub fn with_expiration(mut self, duration: Duration) -> Self {
        self.expires_at = Some(self.interrupted_at + duration);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
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

    /// Get time until expiration
    pub fn time_until_expiration(&self) -> Option<Duration> {
        self.expires_at.and_then(|expires_at| {
            expires_at.duration_since(SystemTime::now()).ok()
        })
    }
}

/// Interrupt point configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterruptPoint {
    /// Unique identifier for the interrupt point
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description of what happens at this point
    pub description: String,
    /// Whether this interrupt is required or optional
    pub required: bool,
    /// Default timeout for this interrupt
    pub default_timeout: Option<Duration>,
    /// Priority level
    pub priority: InterruptPriority,
    /// Conditions for triggering this interrupt
    pub conditions: Vec<InterruptCondition>,
}

/// Priority level for interrupts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum InterruptPriority {
    /// Low priority - can be deferred
    Low,
    /// Normal priority
    Normal,
    /// High priority - needs attention soon
    High,
    /// Critical priority - immediate attention required
    Critical,
}

/// Condition for triggering an interrupt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterruptCondition {
    /// Condition type
    pub condition_type: InterruptConditionType,
    /// Parameters for the condition
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Type of interrupt condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterruptConditionType {
    /// Always trigger
    Always,
    /// Trigger based on state value
    StateValue,
    /// Trigger based on execution time
    ExecutionTime,
    /// Trigger based on error occurrence
    OnError,
    /// Custom condition
    Custom(String),
}

/// Interrupted execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterruptedExecution {
    /// Resume token
    pub token: ResumeToken,
    /// Interrupt point that triggered this
    pub interrupt_point: InterruptPoint,
    /// Serialized state at interrupt
    pub state_snapshot: serde_json::Value,
    /// Execution context
    pub execution_context: HashMap<String, serde_json::Value>,
    /// Status of the interrupt
    pub status: InterruptStatus,
    /// When the interrupt was created
    pub created_at: SystemTime,
    /// When the interrupt was last updated
    pub updated_at: SystemTime,
}

/// Status of an interrupt
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InterruptStatus {
    /// Waiting for human intervention
    Pending,
    /// Human is currently reviewing
    InProgress,
    /// Approved to continue
    Approved,
    /// Rejected - execution should stop
    Rejected,
    /// Expired without response
    Expired,
    /// Cancelled
    Cancelled,
}

/// Manager for handling execution interrupts
#[derive(Debug)]
pub struct InterruptManager {
    /// Active interrupted executions
    interrupted_executions: Arc<RwLock<HashMap<String, InterruptedExecution>>>,
    /// Registered interrupt points
    interrupt_points: Arc<RwLock<HashMap<String, InterruptPoint>>>,
    /// Configuration
    config: InterruptConfig,
}

/// Configuration for interrupt manager
#[derive(Debug, Clone)]
pub struct InterruptConfig {
    /// Maximum number of concurrent interrupts
    pub max_concurrent_interrupts: usize,
    /// Default timeout for interrupts
    pub default_timeout: Duration,
    /// Enable automatic cleanup of expired interrupts
    pub auto_cleanup: bool,
    /// Cleanup interval
    pub cleanup_interval: Duration,
}

impl Default for InterruptConfig {
    fn default() -> Self {
        Self {
            max_concurrent_interrupts: 100,
            default_timeout: Duration::from_secs(3600), // 1 hour
            auto_cleanup: true,
            cleanup_interval: Duration::from_secs(300), // 5 minutes
        }
    }
}

impl InterruptManager {
    /// Create a new interrupt manager
    pub fn new(config: InterruptConfig) -> Self {
        Self {
            interrupted_executions: Arc::new(RwLock::new(HashMap::new())),
            interrupt_points: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Register an interrupt point
    pub async fn register_interrupt_point(&self, interrupt_point: InterruptPoint) -> CoreResult<()> {
        let mut points = self.interrupt_points.write().await;
        points.insert(interrupt_point.id.clone(), interrupt_point);
        Ok(())
    }

    /// Create an interrupt
    pub async fn create_interrupt(
        &self,
        execution_id: String,
        node_id: String,
        interrupt_point_id: String,
        state_snapshot: serde_json::Value,
        execution_context: HashMap<String, serde_json::Value>,
    ) -> CoreResult<ResumeToken> {
        // Check if we're at capacity
        let current_count = self.interrupted_executions.read().await.len();
        if current_count >= self.config.max_concurrent_interrupts {
            return Err(CoreError::resource_error(
                format!("Maximum concurrent interrupts ({}) reached", self.config.max_concurrent_interrupts)
            ));
        }

        // Get interrupt point
        let interrupt_point = {
            let points = self.interrupt_points.read().await;
            points.get(&interrupt_point_id)
                .ok_or_else(|| CoreError::configuration_error(format!("Interrupt point not found: {}", interrupt_point_id)))?
                .clone()
        };

        // Create resume token
        let token = ResumeToken::new(execution_id, node_id)
            .with_expiration(
                interrupt_point.default_timeout
                    .unwrap_or(self.config.default_timeout)
            );

        // Create interrupted execution
        let interrupted_execution = InterruptedExecution {
            token: token.clone(),
            interrupt_point,
            state_snapshot,
            execution_context,
            status: InterruptStatus::Pending,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        };

        // Store the interrupt
        let mut executions = self.interrupted_executions.write().await;
        executions.insert(token.interrupt_id.clone(), interrupted_execution);

        Ok(token)
    }

    /// Resume an interrupted execution
    pub async fn resume_execution(&self, token: &ResumeToken) -> CoreResult<InterruptedExecution> {
        // Check if token is expired
        if token.is_expired() {
            return Err(CoreError::execution_error("Resume token has expired"));
        }

        let mut executions = self.interrupted_executions.write().await;
        let execution = executions.get_mut(&token.interrupt_id)
            .ok_or_else(|| CoreError::execution_error("Interrupt not found"))?;

        // Check if already processed
        match execution.status {
            InterruptStatus::Approved => {
                // Remove from active interrupts
                let execution = executions.remove(&token.interrupt_id).unwrap();
                Ok(execution)
            }
            InterruptStatus::Rejected => {
                Err(CoreError::execution_error("Execution was rejected"))
            }
            InterruptStatus::Expired => {
                Err(CoreError::execution_error("Interrupt has expired"))
            }
            InterruptStatus::Cancelled => {
                Err(CoreError::execution_error("Interrupt was cancelled"))
            }
            _ => {
                Err(CoreError::execution_error("Interrupt is still pending"))
            }
        }
    }

    /// Approve an interrupt
    pub async fn approve_interrupt(&self, interrupt_id: &str, approver_info: Option<HashMap<String, serde_json::Value>>) -> CoreResult<()> {
        let mut executions = self.interrupted_executions.write().await;
        let execution = executions.get_mut(interrupt_id)
            .ok_or_else(|| CoreError::execution_error("Interrupt not found"))?;

        execution.status = InterruptStatus::Approved;
        execution.updated_at = SystemTime::now();

        if let Some(info) = approver_info {
            execution.execution_context.extend(info);
        }

        Ok(())
    }

    /// Reject an interrupt
    pub async fn reject_interrupt(&self, interrupt_id: &str, reason: Option<String>) -> CoreResult<()> {
        let mut executions = self.interrupted_executions.write().await;
        let execution = executions.get_mut(interrupt_id)
            .ok_or_else(|| CoreError::execution_error("Interrupt not found"))?;

        execution.status = InterruptStatus::Rejected;
        execution.updated_at = SystemTime::now();

        if let Some(reason) = reason {
            execution.execution_context.insert("rejection_reason".to_string(), serde_json::Value::String(reason));
        }

        Ok(())
    }

    /// Cancel an interrupt
    pub async fn cancel_interrupt(&self, interrupt_id: &str) -> CoreResult<()> {
        let mut executions = self.interrupted_executions.write().await;
        let execution = executions.get_mut(interrupt_id)
            .ok_or_else(|| CoreError::execution_error("Interrupt not found"))?;

        execution.status = InterruptStatus::Cancelled;
        execution.updated_at = SystemTime::now();

        Ok(())
    }

    /// Get interrupt by ID
    pub async fn get_interrupt(&self, interrupt_id: &str) -> Option<InterruptedExecution> {
        let executions = self.interrupted_executions.read().await;
        executions.get(interrupt_id).cloned()
    }

    /// List all pending interrupts
    pub async fn list_pending_interrupts(&self) -> Vec<InterruptedExecution> {
        let executions = self.interrupted_executions.read().await;
        executions.values()
            .filter(|exec| exec.status == InterruptStatus::Pending)
            .cloned()
            .collect()
    }

    /// List interrupts by status
    pub async fn list_interrupts_by_status(&self, status: InterruptStatus) -> Vec<InterruptedExecution> {
        let executions = self.interrupted_executions.read().await;
        executions.values()
            .filter(|exec| exec.status == status)
            .cloned()
            .collect()
    }

    /// Clean up expired interrupts
    pub async fn cleanup_expired_interrupts(&self) -> CoreResult<usize> {
        let mut executions = self.interrupted_executions.write().await;
        let mut expired_count = 0;

        executions.retain(|_, execution| {
            if execution.token.is_expired() {
                expired_count += 1;
                false
            } else {
                true
            }
        });

        Ok(expired_count)
    }

    /// Get interrupt statistics
    pub async fn get_statistics(&self) -> InterruptStatistics {
        let executions = self.interrupted_executions.read().await;
        let mut stats = InterruptStatistics::default();

        stats.total_interrupts = executions.len();

        for execution in executions.values() {
            match execution.status {
                InterruptStatus::Pending => stats.pending_interrupts += 1,
                InterruptStatus::InProgress => stats.in_progress_interrupts += 1,
                InterruptStatus::Approved => stats.approved_interrupts += 1,
                InterruptStatus::Rejected => stats.rejected_interrupts += 1,
                InterruptStatus::Expired => stats.expired_interrupts += 1,
                InterruptStatus::Cancelled => stats.cancelled_interrupts += 1,
            }
        }

        stats
    }
}

/// Statistics about interrupts
#[derive(Debug, Clone, Default)]
pub struct InterruptStatistics {
    /// Total number of interrupts
    pub total_interrupts: usize,
    /// Number of pending interrupts
    pub pending_interrupts: usize,
    /// Number of in-progress interrupts
    pub in_progress_interrupts: usize,
    /// Number of approved interrupts
    pub approved_interrupts: usize,
    /// Number of rejected interrupts
    pub rejected_interrupts: usize,
    /// Number of expired interrupts
    pub expired_interrupts: usize,
    /// Number of cancelled interrupts
    pub cancelled_interrupts: usize,
}

impl Default for InterruptManager {
    fn default() -> Self {
        Self::new(InterruptConfig::default())
    }
}