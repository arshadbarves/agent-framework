// Execution scheduler for AgentGraph
// Provides intelligent scheduling and resource management for graph execution

#![allow(missing_docs)]

use super::{ExecutionConfig, ExecutionContext, ExecutionStatus};
use crate::graph::Graph;
use crate::node::NodeId;
use crate::state::State;
use serde::{Deserialize, Serialize};
use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::cmp::Ordering;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use thiserror::Error;

/// Scheduling strategy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchedulingStrategy {
    /// First-In-First-Out
    FIFO,
    /// Priority-based scheduling
    Priority,
    /// Shortest Job First
    SJF,
    /// Round Robin
    RoundRobin,
    /// Fair Share
    FairShare,
    /// Resource-aware scheduling
    ResourceAware,
}

/// Scheduled execution request
#[derive(Debug, Clone)]
pub struct ScheduledExecution {
    /// Execution ID
    pub execution_id: String,
    /// Graph to execute
    pub graph: Arc<Graph>,
    /// Input state
    pub input_state: State,
    /// Execution configuration
    pub config: ExecutionConfig,
    /// Priority (higher = more important)
    pub priority: u32,
    /// Estimated execution time
    pub estimated_duration: Option<Duration>,
    /// Resource requirements
    pub resource_requirements: ResourceRequirements,
    /// Scheduled time
    pub scheduled_at: SystemTime,
    /// Deadline (optional)
    pub deadline: Option<SystemTime>,
    /// User/tenant ID
    pub user_id: Option<String>,
}

impl ScheduledExecution {
    /// Create a new scheduled execution
    pub fn new(
        graph: Arc<Graph>,
        input_state: State,
        config: ExecutionConfig,
    ) -> Self {
        Self {
            execution_id: uuid::Uuid::new_v4().to_string(),
            graph,
            input_state,
            config,
            priority: 50, // Default priority
            estimated_duration: None,
            resource_requirements: ResourceRequirements::default(),
            scheduled_at: SystemTime::now(),
            deadline: None,
            user_id: None,
        }
    }
    
    /// Set priority
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }
    
    /// Set estimated duration
    pub fn with_estimated_duration(mut self, duration: Duration) -> Self {
        self.estimated_duration = Some(duration);
        self
    }
    
    /// Set resource requirements
    pub fn with_resource_requirements(mut self, requirements: ResourceRequirements) -> Self {
        self.resource_requirements = requirements;
        self
    }
    
    /// Set deadline
    pub fn with_deadline(mut self, deadline: SystemTime) -> Self {
        self.deadline = Some(deadline);
        self
    }
    
    /// Set user ID
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }
    
    /// Check if execution is overdue
    pub fn is_overdue(&self) -> bool {
        if let Some(deadline) = self.deadline {
            SystemTime::now() > deadline
        } else {
            false
        }
    }
    
    /// Get time until deadline
    pub fn time_until_deadline(&self) -> Option<Duration> {
        self.deadline.and_then(|deadline| {
            deadline.duration_since(SystemTime::now()).ok()
        })
    }
}

impl PartialEq for ScheduledExecution {
    fn eq(&self, other: &Self) -> bool {
        self.execution_id == other.execution_id
    }
}

impl Eq for ScheduledExecution {}

impl PartialOrd for ScheduledExecution {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScheduledExecution {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first, then earlier scheduled time
        other.priority.cmp(&self.priority)
            .then_with(|| self.scheduled_at.cmp(&other.scheduled_at))
    }
}

/// Resource requirements for execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// Required CPU cores
    pub cpu_cores: u32,
    /// Required memory (bytes)
    pub memory: u64,
    /// Required disk space (bytes)
    pub disk_space: u64,
    /// Required network bandwidth (bytes/sec)
    pub network_bandwidth: u64,
    /// Custom resource requirements
    pub custom_resources: HashMap<String, u64>,
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            cpu_cores: 1,
            memory: 512 * 1024 * 1024, // 512MB
            disk_space: 100 * 1024 * 1024, // 100MB
            network_bandwidth: 1024 * 1024, // 1MB/s
            custom_resources: HashMap::new(),
        }
    }
}

/// Current resource usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// Used CPU cores
    pub cpu_cores: u32,
    /// Used memory (bytes)
    pub memory: u64,
    /// Used disk space (bytes)
    pub disk_space: u64,
    /// Used network bandwidth (bytes/sec)
    pub network_bandwidth: u64,
    /// Custom resource usage
    pub custom_resources: HashMap<String, u64>,
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            cpu_cores: 0,
            memory: 0,
            disk_space: 0,
            network_bandwidth: 0,
            custom_resources: HashMap::new(),
        }
    }
}

impl ResourceUsage {
    /// Check if resources are available for requirements
    pub fn can_accommodate(&self, requirements: &ResourceRequirements, limits: &ResourceRequirements) -> bool {
        self.cpu_cores + requirements.cpu_cores <= limits.cpu_cores
            && self.memory + requirements.memory <= limits.memory
            && self.disk_space + requirements.disk_space <= limits.disk_space
            && self.network_bandwidth + requirements.network_bandwidth <= limits.network_bandwidth
    }
    
    /// Add resource usage
    pub fn add(&mut self, requirements: &ResourceRequirements) {
        self.cpu_cores += requirements.cpu_cores;
        self.memory += requirements.memory;
        self.disk_space += requirements.disk_space;
        self.network_bandwidth += requirements.network_bandwidth;
        
        for (key, value) in &requirements.custom_resources {
            *self.custom_resources.entry(key.clone()).or_insert(0) += value;
        }
    }
    
    /// Remove resource usage
    pub fn remove(&mut self, requirements: &ResourceRequirements) {
        self.cpu_cores = self.cpu_cores.saturating_sub(requirements.cpu_cores);
        self.memory = self.memory.saturating_sub(requirements.memory);
        self.disk_space = self.disk_space.saturating_sub(requirements.disk_space);
        self.network_bandwidth = self.network_bandwidth.saturating_sub(requirements.network_bandwidth);
        
        for (key, value) in &requirements.custom_resources {
            if let Some(current) = self.custom_resources.get_mut(key) {
                *current = current.saturating_sub(*value);
            }
        }
    }
}

/// Execution scheduler
#[derive(Debug)]
pub struct ExecutionScheduler {
    /// Configuration
    config: ExecutionConfig,
    /// Scheduling strategy
    strategy: SchedulingStrategy,
    /// Pending executions queue
    pending_queue: Arc<RwLock<BinaryHeap<ScheduledExecution>>>,
    /// Running executions
    running_executions: Arc<RwLock<HashMap<String, ExecutionContext>>>,
    /// Completed executions
    completed_executions: Arc<RwLock<HashMap<String, ExecutionContext>>>,
    /// Current resource usage
    current_usage: Arc<RwLock<ResourceUsage>>,
    /// Resource limits
    resource_limits: ResourceRequirements,
    /// User quotas
    user_quotas: Arc<RwLock<HashMap<String, UserQuota>>>,
}

impl ExecutionScheduler {
    /// Create a new execution scheduler
    pub fn new(config: ExecutionConfig) -> Self {
        let resource_limits = ResourceRequirements {
            cpu_cores: config.max_concurrency as u32,
            memory: config.resource_limits.max_memory,
            disk_space: config.resource_limits.max_node_time.as_secs() * 1024 * 1024, // Rough estimate
            network_bandwidth: 10 * 1024 * 1024, // 10MB/s
            custom_resources: HashMap::new(),
        };
        
        Self {
            config,
            strategy: SchedulingStrategy::Priority,
            pending_queue: Arc::new(RwLock::new(BinaryHeap::new())),
            running_executions: Arc::new(RwLock::new(HashMap::new())),
            completed_executions: Arc::new(RwLock::new(HashMap::new())),
            current_usage: Arc::new(RwLock::new(ResourceUsage::default())),
            resource_limits,
            user_quotas: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Schedule an execution
    pub async fn schedule_execution(&self, execution: ScheduledExecution) -> Result<(), SchedulerError> {
        // Check user quota
        if let Some(user_id) = &execution.user_id {
            let quotas = self.user_quotas.read().await;
            if let Some(quota) = quotas.get(user_id) {
                if !quota.can_schedule(&execution) {
                    return Err(SchedulerError::QuotaExceeded {
                        user_id: user_id.clone(),
                        quota_type: "execution_count".to_string(),
                    });
                }
            }
        }
        
        // Add to pending queue
        let mut queue = self.pending_queue.write().await;
        queue.push(execution);
        
        Ok(())
    }
    
    /// Get next execution to run
    pub async fn get_next_execution(&self) -> Option<ScheduledExecution> {
        let mut queue = self.pending_queue.write().await;
        let current_usage = self.current_usage.read().await;
        
        // Find an execution that can run with current resources
        let mut temp_queue = BinaryHeap::new();
        let mut selected = None;
        
        while let Some(execution) = queue.pop() {
            if current_usage.can_accommodate(&execution.resource_requirements, &self.resource_limits) {
                selected = Some(execution);
                break;
            } else {
                temp_queue.push(execution);
            }
        }
        
        // Put back the executions we couldn't run
        while let Some(execution) = temp_queue.pop() {
            queue.push(execution);
        }
        
        selected
    }
    
    /// Start execution
    pub async fn start_execution(&self, execution: ScheduledExecution) -> Result<String, SchedulerError> {
        // Reserve resources
        {
            let mut usage = self.current_usage.write().await;
            usage.add(&execution.resource_requirements);
        }
        
        // Create execution context
        let context = ExecutionContext::new(execution.config.clone(), execution.input_state.clone());
        let execution_id = context.execution_id.clone();
        
        // Add to running executions
        {
            let mut running = self.running_executions.write().await;
            running.insert(execution_id.clone(), context);
        }
        
        Ok(execution_id)
    }
    
    /// Complete execution
    pub async fn complete_execution(
        &self,
        execution_id: &str,
        context: ExecutionContext,
    ) -> Result<(), SchedulerError> {
        // Remove from running executions
        let execution = {
            let mut running = self.running_executions.write().await;
            running.remove(execution_id)
        };
        
        if let Some(exec) = execution {
            // Release resources
            {
                let mut usage = self.current_usage.write().await;
                // We need the original resource requirements to release properly
                // For now, we'll use default requirements
                let requirements = ResourceRequirements::default();
                usage.remove(&requirements);
            }
            
            // Add to completed executions
            {
                let mut completed = self.completed_executions.write().await;
                completed.insert(execution_id.to_string(), context);
            }
            
            Ok(())
        } else {
            Err(SchedulerError::ExecutionNotFound {
                execution_id: execution_id.to_string(),
            })
        }
    }
    
    /// Cancel execution
    pub async fn cancel_execution(&self, execution_id: &str) -> Result<(), SchedulerError> {
        // Try to remove from pending queue first
        {
            let mut queue = self.pending_queue.write().await;
            let mut temp_queue = BinaryHeap::new();
            let mut found = false;
            
            while let Some(execution) = queue.pop() {
                if execution.execution_id == execution_id {
                    found = true;
                    break;
                } else {
                    temp_queue.push(execution);
                }
            }
            
            // Put back other executions
            while let Some(execution) = temp_queue.pop() {
                queue.push(execution);
            }
            
            if found {
                return Ok(());
            }
        }
        
        // Try to cancel running execution
        {
            let mut running = self.running_executions.write().await;
            if let Some(mut context) = running.remove(execution_id) {
                context.status = ExecutionStatus::Cancelled;
                
                // Move to completed
                let mut completed = self.completed_executions.write().await;
                completed.insert(execution_id.to_string(), context);
                
                return Ok(());
            }
        }
        
        Err(SchedulerError::ExecutionNotFound {
            execution_id: execution_id.to_string(),
        })
    }
    
    /// Get scheduler statistics
    pub async fn get_stats(&self) -> SchedulerStats {
        let pending = self.pending_queue.read().await;
        let running = self.running_executions.read().await;
        let completed = self.completed_executions.read().await;
        let usage = self.current_usage.read().await;
        
        SchedulerStats {
            pending_executions: pending.len(),
            running_executions: running.len(),
            completed_executions: completed.len(),
            current_resource_usage: usage.clone(),
            resource_limits: self.resource_limits.clone(),
        }
    }
    
    /// Set user quota
    pub async fn set_user_quota(&self, user_id: String, quota: UserQuota) {
        let mut quotas = self.user_quotas.write().await;
        quotas.insert(user_id, quota);
    }
    
    /// Get pending executions for user
    pub async fn get_user_executions(&self, user_id: &str) -> Vec<ScheduledExecution> {
        let queue = self.pending_queue.read().await;
        queue.iter()
            .filter(|exec| exec.user_id.as_ref() == Some(user_id))
            .cloned()
            .collect()
    }
}

/// User quota configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserQuota {
    /// Maximum concurrent executions
    pub max_concurrent_executions: u32,
    /// Maximum executions per hour
    pub max_executions_per_hour: u32,
    /// Maximum resource usage
    pub max_resource_usage: ResourceRequirements,
    /// Current usage
    pub current_usage: ResourceUsage,
    /// Current execution count
    pub current_executions: u32,
}

impl UserQuota {
    /// Create a new user quota
    pub fn new(
        max_concurrent: u32,
        max_per_hour: u32,
        max_resources: ResourceRequirements,
    ) -> Self {
        Self {
            max_concurrent_executions: max_concurrent,
            max_executions_per_hour: max_per_hour,
            max_resource_usage: max_resources,
            current_usage: ResourceUsage::default(),
            current_executions: 0,
        }
    }
    
    /// Check if user can schedule execution
    pub fn can_schedule(&self, _execution: &ScheduledExecution) -> bool {
        self.current_executions < self.max_concurrent_executions
    }
}

/// Scheduler statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerStats {
    /// Number of pending executions
    pub pending_executions: usize,
    /// Number of running executions
    pub running_executions: usize,
    /// Number of completed executions
    pub completed_executions: usize,
    /// Current resource usage
    pub current_resource_usage: ResourceUsage,
    /// Resource limits
    pub resource_limits: ResourceRequirements,
}

/// Scheduler errors
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum SchedulerError {
    /// Execution not found
    #[error("Execution not found: {execution_id}")]
    ExecutionNotFound { execution_id: String },
    
    /// Quota exceeded
    #[error("Quota exceeded for user {user_id}: {quota_type}")]
    QuotaExceeded { user_id: String, quota_type: String },
    
    /// Resource unavailable
    #[error("Resource unavailable: {resource}")]
    ResourceUnavailable { resource: String },
    
    /// Configuration error
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },
    
    /// System error
    #[error("System error: {message}")]
    SystemError { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Graph;

    #[test]
    fn test_scheduled_execution_creation() {
        let graph = Arc::new(Graph::new());
        let state = State::new();
        let config = ExecutionConfig::default();
        
        let execution = ScheduledExecution::new(graph, state, config);
        assert_eq!(execution.priority, 50);
        assert!(execution.estimated_duration.is_none());
        assert!(!execution.is_overdue());
    }

    #[test]
    fn test_resource_requirements_default() {
        let requirements = ResourceRequirements::default();
        assert_eq!(requirements.cpu_cores, 1);
        assert_eq!(requirements.memory, 512 * 1024 * 1024);
    }

    #[test]
    fn test_resource_usage_operations() {
        let mut usage = ResourceUsage::default();
        let requirements = ResourceRequirements::default();
        let limits = ResourceRequirements {
            cpu_cores: 4,
            memory: 2048 * 1024 * 1024,
            ..Default::default()
        };
        
        assert!(usage.can_accommodate(&requirements, &limits));
        
        usage.add(&requirements);
        assert_eq!(usage.cpu_cores, 1);
        assert_eq!(usage.memory, 512 * 1024 * 1024);
        
        usage.remove(&requirements);
        assert_eq!(usage.cpu_cores, 0);
        assert_eq!(usage.memory, 0);
    }

    #[tokio::test]
    async fn test_scheduler_creation() {
        let config = ExecutionConfig::default();
        let scheduler = ExecutionScheduler::new(config);
        
        let stats = scheduler.get_stats().await;
        assert_eq!(stats.pending_executions, 0);
        assert_eq!(stats.running_executions, 0);
        assert_eq!(stats.completed_executions, 0);
    }

    #[test]
    fn test_user_quota() {
        let quota = UserQuota::new(
            5,
            100,
            ResourceRequirements::default(),
        );
        
        assert_eq!(quota.max_concurrent_executions, 5);
        assert_eq!(quota.max_executions_per_hour, 100);
        assert_eq!(quota.current_executions, 0);
    }
}
