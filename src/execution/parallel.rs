// Parallel execution engine for AgentGraph
// Provides advanced parallel processing capabilities with work stealing and load balancing

#![allow(missing_docs)]

use super::{ExecutionConfig, NodeExecution, ExecutionError};
use crate::node::{Node, NodeId};
use crate::state::State;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock, Semaphore};
use tokio::task::JoinHandle;
use thiserror::Error;

/// Parallel execution strategy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParallelStrategy {
    /// Fixed thread pool
    FixedPool,
    /// Dynamic thread pool with work stealing
    WorkStealing,
    /// Actor-based parallel execution
    ActorBased,
    /// Pipeline parallel execution
    Pipeline,
}

/// Work item for parallel execution
#[derive(Debug, Clone)]
pub struct WorkItem<S: State> {
    /// Node to execute
    pub node: Arc<dyn Node<S>>,
    /// Input state
    pub input_state: S,
    /// Priority (higher = more important)
    pub priority: u32,
    /// Dependencies that must complete first
    pub dependencies: Vec<NodeId>,
    /// Estimated execution time
    pub estimated_duration: Option<Duration>,
}

impl<S: State> WorkItem<S> {
    /// Create a new work item
    pub fn new(node: Arc<dyn Node<S>>, input_state: S) -> Self {
        Self {
            node,
            input_state,
            priority: 0,
            dependencies: Vec::new(),
            estimated_duration: None,
        }
    }
    
    /// Set priority
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }
    
    /// Set dependencies
    pub fn with_dependencies(mut self, dependencies: Vec<NodeId>) -> Self {
        self.dependencies = dependencies;
        self
    }
    
    /// Set estimated duration
    pub fn with_estimated_duration(mut self, duration: Duration) -> Self {
        self.estimated_duration = Some(duration);
        self
    }
}

/// Work queue for parallel execution
#[derive(Debug)]
pub struct WorkQueue {
    /// High priority queue
    high_priority: VecDeque<WorkItem>,
    /// Normal priority queue
    normal_priority: VecDeque<WorkItem>,
    /// Low priority queue
    low_priority: VecDeque<WorkItem>,
    /// Completed work items
    completed: HashMap<NodeId, NodeExecution>,
    /// Failed work items
    failed: HashMap<NodeId, ExecutionError>,
}

impl WorkQueue {
    /// Create a new work queue
    pub fn new() -> Self {
        Self {
            high_priority: VecDeque::new(),
            normal_priority: VecDeque::new(),
            low_priority: VecDeque::new(),
            completed: HashMap::new(),
            failed: HashMap::new(),
        }
    }
    
    /// Add work item to queue
    pub fn push(&mut self, item: WorkItem) {
        match item.priority {
            0..=33 => self.low_priority.push_back(item),
            34..=66 => self.normal_priority.push_back(item),
            _ => self.high_priority.push_back(item),
        }
    }
    
    /// Get next work item (priority-based)
    pub fn pop(&mut self) -> Option<WorkItem> {
        // Check dependencies first
        let ready_item = self.find_ready_item();
        if let Some(item) = ready_item {
            return Some(item);
        }
        
        // Fallback to priority-based selection
        self.high_priority.pop_front()
            .or_else(|| self.normal_priority.pop_front())
            .or_else(|| self.low_priority.pop_front())
    }
    
    /// Find work item with satisfied dependencies
    fn find_ready_item(&mut self) -> Option<WorkItem> {
        // Check high priority first
        if let Some(pos) = self.find_ready_in_queue(&self.high_priority) {
            return self.high_priority.remove(pos);
        }
        
        // Check normal priority
        if let Some(pos) = self.find_ready_in_queue(&self.normal_priority) {
            return self.normal_priority.remove(pos);
        }
        
        // Check low priority
        if let Some(pos) = self.find_ready_in_queue(&self.low_priority) {
            return self.low_priority.remove(pos);
        }
        
        None
    }
    
    /// Find ready item in specific queue
    fn find_ready_in_queue(&self, queue: &VecDeque<WorkItem>) -> Option<usize> {
        queue.iter().position(|item| {
            item.dependencies.iter().all(|dep| self.completed.contains_key(dep))
        })
    }
    
    /// Mark work item as completed
    pub fn mark_completed(&mut self, node_id: NodeId, execution: NodeExecution) {
        self.completed.insert(node_id, execution);
    }
    
    /// Mark work item as failed
    pub fn mark_failed(&mut self, node_id: NodeId, error: ExecutionError) {
        self.failed.insert(node_id, error);
    }
    
    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.high_priority.is_empty() 
            && self.normal_priority.is_empty() 
            && self.low_priority.is_empty()
    }
    
    /// Get queue size
    pub fn size(&self) -> usize {
        self.high_priority.len() + self.normal_priority.len() + self.low_priority.len()
    }
    
    /// Get completed executions
    pub fn completed_executions(&self) -> &HashMap<NodeId, NodeExecution> {
        &self.completed
    }
    
    /// Get failed executions
    pub fn failed_executions(&self) -> &HashMap<NodeId, ExecutionError> {
        &self.failed
    }
}

/// Parallel execution worker
#[derive(Debug)]
#[derive(Clone)]
pub struct Worker {
    /// Worker ID
    pub id: usize,
    /// Worker status
    pub status: WorkerStatus,
    /// Current work item
    pub current_work: Option<NodeId>,
    /// Completed work count
    pub completed_count: u64,
    /// Failed work count
    pub failed_count: u64,
    /// Total execution time
    pub total_execution_time: Duration,
}

impl Worker {
    /// Create a new worker
    pub fn new(id: usize) -> Self {
        Self {
            id,
            status: WorkerStatus::Idle,
            current_work: None,
            completed_count: 0,
            failed_count: 0,
            total_execution_time: Duration::ZERO,
        }
    }
    
    /// Start working on an item
    pub fn start_work(&mut self, node_id: NodeId) {
        self.status = WorkerStatus::Working;
        self.current_work = Some(node_id);
    }
    
    /// Complete work
    pub fn complete_work(&mut self, execution_time: Duration) {
        self.status = WorkerStatus::Idle;
        self.current_work = None;
        self.completed_count += 1;
        self.total_execution_time += execution_time;
    }
    
    /// Fail work
    pub fn fail_work(&mut self, execution_time: Duration) {
        self.status = WorkerStatus::Idle;
        self.current_work = None;
        self.failed_count += 1;
        self.total_execution_time += execution_time;
    }
    
    /// Get worker efficiency (completed / total)
    pub fn efficiency(&self) -> f64 {
        let total = self.completed_count + self.failed_count;
        if total == 0 {
            0.0
        } else {
            self.completed_count as f64 / total as f64
        }
    }
}

/// Worker status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkerStatus {
    /// Worker is idle
    Idle,
    /// Worker is working
    Working,
    /// Worker is paused
    Paused,
    /// Worker has failed
    Failed,
}

/// Parallel execution engine
#[derive(Debug)]
pub struct ParallelExecutor {
    /// Configuration
    config: ExecutionConfig,
    /// Execution strategy
    strategy: ParallelStrategy,
    /// Work queue
    work_queue: Arc<RwLock<WorkQueue>>,
    /// Workers
    workers: Arc<RwLock<Vec<Worker>>>,
    /// Semaphore for concurrency control
    semaphore: Arc<Semaphore>,
    /// Active tasks
    active_tasks: Arc<RwLock<HashMap<usize, JoinHandle<()>>>>,
}

impl ParallelExecutor {
    /// Create a new parallel executor
    pub fn new(config: ExecutionConfig, strategy: ParallelStrategy) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrency));
        let work_queue = Arc::new(RwLock::new(WorkQueue::new()));
        let workers = Arc::new(RwLock::new(Vec::<Worker>::new()));
        
        // Initialize workers
        let mut worker_vec = Vec::new();
        for i in 0..config.max_concurrency {
            worker_vec.push(Worker::new(i));
        }
        
        Self {
            config,
            strategy,
            work_queue,
            workers: Arc::new(RwLock::new(worker_vec)),
            semaphore,
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Execute work items in parallel
    pub async fn execute_parallel<S: State>(
        &self,
        work_items: Vec<WorkItem<S>>,
    ) -> Result<HashMap<NodeId, NodeExecution>, ParallelExecutionError> {
        // Add work items to queue
        {
            let mut queue = self.work_queue.write().await;
            for item in work_items {
                queue.push(item);
            }
        }
        
        // Start workers based on strategy
        match self.strategy {
            ParallelStrategy::FixedPool => self.execute_fixed_pool().await,
            ParallelStrategy::WorkStealing => self.execute_work_stealing().await,
            ParallelStrategy::ActorBased => self.execute_actor_based().await,
            ParallelStrategy::Pipeline => self.execute_pipeline().await,
        }
    }
    
    /// Execute with fixed thread pool
    async fn execute_fixed_pool(&self) -> Result<HashMap<NodeId, NodeExecution>, ParallelExecutionError> {
        let mut tasks = Vec::new();
        
        // Spawn worker tasks
        for worker_id in 0..self.config.max_concurrency {
            let queue = Arc::clone(&self.work_queue);
            let workers = Arc::clone(&self.workers);
            let semaphore = Arc::clone(&self.semaphore);
            let config = self.config.clone();
            
            let task = tokio::spawn(async move {
                loop {
                    // Get work item
                    let work_item = {
                        let mut queue_guard = queue.write().await;
                        queue_guard.pop()
                    };
                    
                    if let Some(item) = work_item {
                        // Acquire semaphore permit
                        let _permit = semaphore.acquire().await.unwrap();
                        
                        // Update worker status
                        {
                            let mut workers_guard = workers.write().await;
                            if let Some(worker) = workers_guard.get_mut(worker_id) {
                                worker.start_work(item.node.id().clone());
                            }
                        }
                        
                        // Execute work item
                        let start_time = Instant::now();
                        let result = Self::execute_work_item(&item, &config).await;
                        let execution_time = start_time.elapsed();
                        
                        // Update queue and worker
                        {
                            let mut queue_guard = queue.write().await;
                            let mut workers_guard = workers.write().await;
                            
                            match result {
                                Ok(execution) => {
                                    queue_guard.mark_completed(item.node.id().clone(), execution);
                                    if let Some(worker) = workers_guard.get_mut(worker_id) {
                                        worker.complete_work(execution_time);
                                    }
                                }
                                Err(error) => {
                                    queue_guard.mark_failed(item.node.id().clone(), error);
                                    if let Some(worker) = workers_guard.get_mut(worker_id) {
                                        worker.fail_work(execution_time);
                                    }
                                }
                            }
                        }
                    } else {
                        // No work available, check if we should exit
                        let queue_guard = queue.read().await;
                        if queue_guard.is_empty() {
                            break;
                        }
                        drop(queue_guard);
                        
                        // Wait a bit before checking again
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                }
            });
            
            tasks.push(task);
        }
        
        // Wait for all tasks to complete
        for task in tasks {
            let _ = task.await;
        }
        
        // Return completed executions
        let queue = self.work_queue.read().await;
        Ok(queue.completed_executions().clone())
    }
    
    /// Execute with work stealing
    async fn execute_work_stealing(&self) -> Result<HashMap<NodeId, NodeExecution>, ParallelExecutionError> {
        // For now, use fixed pool implementation
        // TODO: Implement actual work stealing algorithm
        self.execute_fixed_pool().await
    }
    
    /// Execute with actor-based approach
    async fn execute_actor_based(&self) -> Result<HashMap<NodeId, NodeExecution>, ParallelExecutionError> {
        // For now, use fixed pool implementation
        // TODO: Implement actor-based execution
        self.execute_fixed_pool().await
    }
    
    /// Execute with pipeline approach
    async fn execute_pipeline(&self) -> Result<HashMap<NodeId, NodeExecution>, ParallelExecutionError> {
        // For now, use fixed pool implementation
        // TODO: Implement pipeline execution
        self.execute_fixed_pool().await
    }
    
    /// Execute a single work item
    async fn execute_work_item<S: State>(
        item: &WorkItem<S>,
        config: &ExecutionConfig,
    ) -> Result<NodeExecution, ExecutionError> {
        let mut execution = NodeExecution::new(item.node.id().clone(), item.input_state.clone());
        execution.start();
        
        // Execute with timeout
        let result = tokio::time::timeout(
            config.node_timeout,
            item.node.execute(item.input_state.clone()),
        )
        .await;
        
        match result {
            Ok(Ok(output_state)) => {
                execution.complete(output_state);
                Ok(execution)
            }
            Ok(Err(error)) => {
                execution.fail(error.to_string());
                Ok(execution)
            }
            Err(_) => {
                execution.fail("Node execution timed out".to_string());
                Ok(execution)
            }
        }
    }
    
    /// Get worker statistics
    pub async fn get_worker_stats(&self) -> Vec<Worker> {
        let workers = self.workers.read().await;
        workers.clone()
    }
    
    /// Get queue statistics
    pub async fn get_queue_stats(&self) -> QueueStats {
        let queue = self.work_queue.read().await;
        QueueStats {
            pending_items: queue.size(),
            completed_items: queue.completed_executions().len(),
            failed_items: queue.failed_executions().len(),
        }
    }
}

/// Queue statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStats {
    /// Number of pending items
    pub pending_items: usize,
    /// Number of completed items
    pub completed_items: usize,
    /// Number of failed items
    pub failed_items: usize,
}

/// Parallel execution errors
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum ParallelExecutionError {
    /// Worker error
    #[error("Worker {worker_id} error: {error}")]
    WorkerError { worker_id: usize, error: String },
    
    /// Queue error
    #[error("Queue error: {error}")]
    QueueError { error: String },
    
    /// Coordination error
    #[error("Coordination error: {error}")]
    CoordinationError { error: String },
    
    /// Resource exhaustion
    #[error("Resource exhaustion: {resource}")]
    ResourceExhaustion { resource: String },
    
    /// Configuration error
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Node;

    #[test]
    fn test_work_item_creation() {
        let node = Arc::new(Node::new("test".to_string(), "test_type".to_string()));
        let state = State::new();
        let item = WorkItem::new(node, state);
        
        assert_eq!(item.priority, 0);
        assert!(item.dependencies.is_empty());
        assert!(item.estimated_duration.is_none());
    }

    #[test]
    fn test_work_queue_priority() {
        let mut queue = WorkQueue::new();
        
        let node = Arc::new(Node::new("test".to_string(), "test_type".to_string()));
        let state = State::new();
        
        // Add items with different priorities
        let low_item = WorkItem::new(node.clone(), state.clone()).with_priority(10);
        let high_item = WorkItem::new(node.clone(), state.clone()).with_priority(90);
        let normal_item = WorkItem::new(node, state).with_priority(50);
        
        queue.push(low_item);
        queue.push(high_item);
        queue.push(normal_item);
        
        // Should get high priority first
        let first = queue.pop().unwrap();
        assert_eq!(first.priority, 90);
        
        // Then normal priority
        let second = queue.pop().unwrap();
        assert_eq!(second.priority, 50);
        
        // Finally low priority
        let third = queue.pop().unwrap();
        assert_eq!(third.priority, 10);
    }

    #[test]
    fn test_worker_lifecycle() {
        let mut worker = Worker::new(0);
        assert_eq!(worker.status, WorkerStatus::Idle);
        assert_eq!(worker.completed_count, 0);
        
        let node_id = NodeId::new("test".to_string());
        worker.start_work(node_id);
        assert_eq!(worker.status, WorkerStatus::Working);
        
        worker.complete_work(Duration::from_millis(100));
        assert_eq!(worker.status, WorkerStatus::Idle);
        assert_eq!(worker.completed_count, 1);
    }

    #[tokio::test]
    async fn test_parallel_executor_creation() {
        let config = ExecutionConfig::default();
        let executor = ParallelExecutor::new(config, ParallelStrategy::FixedPool);
        
        let stats = executor.get_queue_stats().await;
        assert_eq!(stats.pending_items, 0);
        assert_eq!(stats.completed_items, 0);
        assert_eq!(stats.failed_items, 0);
    }
}
