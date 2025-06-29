//! Advanced scheduling algorithms for optimal execution.

use crate::{CoreError, CoreResult, State, NodeId};
use agent_graph_core::{Graph, Node, NodeMetadata};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BinaryHeap, VecDeque};
use std::cmp::Ordering;
use tracing::{debug, info};

/// Advanced execution scheduler with multiple algorithms
#[derive(Debug)]
pub struct ExecutionScheduler<S>
where
    S: State,
{
    strategy: SchedulingStrategy,
    config: SchedulerConfig,
}

impl<S> ExecutionScheduler<S>
where
    S: State,
{
    /// Create a new execution scheduler
    pub fn new(strategy: SchedulingStrategy, config: SchedulerConfig) -> Self {
        Self { strategy, config }
    }

    /// Schedule nodes for execution based on the configured strategy
    pub async fn schedule_execution(
        &self,
        graph: &Graph<S>,
        available_nodes: &[NodeId],
        resource_manager: &ResourceManager,
    ) -> CoreResult<Vec<ScheduledTask>> {
        debug!("Scheduling {} nodes with strategy: {:?}", available_nodes.len(), self.strategy);

        match self.strategy {
            SchedulingStrategy::FIFO => self.schedule_fifo(graph, available_nodes).await,
            SchedulingStrategy::Priority => self.schedule_priority(graph, available_nodes).await,
            SchedulingStrategy::ShortestJobFirst => self.schedule_sjf(graph, available_nodes).await,
            SchedulingStrategy::ResourceAware => self.schedule_resource_aware(graph, available_nodes, resource_manager).await,
            SchedulingStrategy::CriticalPath => self.schedule_critical_path(graph, available_nodes).await,
            SchedulingStrategy::LoadBalanced => self.schedule_load_balanced(graph, available_nodes, resource_manager).await,
        }
    }

    /// FIFO (First In, First Out) scheduling
    async fn schedule_fifo(
        &self,
        graph: &Graph<S>,
        available_nodes: &[NodeId],
    ) -> CoreResult<Vec<ScheduledTask>> {
        let mut tasks = Vec::new();
        let mut priority = 0;

        for node_id in available_nodes {
            if let Some(node) = graph.nodes.get(node_id) {
                tasks.push(ScheduledTask {
                    node_id: node_id.clone(),
                    priority,
                    estimated_duration: node.estimated_duration_ms().unwrap_or(1000),
                    resource_requirements: node.metadata().resource_requirements.clone(),
                    dependencies: Vec::new(),
                    scheduled_at: chrono::Utc::now(),
                });
                priority += 1;
            }
        }

        Ok(tasks)
    }

    /// Priority-based scheduling
    async fn schedule_priority(
        &self,
        graph: &Graph<S>,
        available_nodes: &[NodeId],
    ) -> CoreResult<Vec<ScheduledTask>> {
        let mut tasks = Vec::new();

        for node_id in available_nodes {
            if let Some(node) = graph.nodes.get(node_id) {
                let metadata = node.metadata();
                let priority = self.calculate_node_priority(metadata);
                
                tasks.push(ScheduledTask {
                    node_id: node_id.clone(),
                    priority,
                    estimated_duration: node.estimated_duration_ms().unwrap_or(1000),
                    resource_requirements: metadata.resource_requirements.clone(),
                    dependencies: Vec::new(),
                    scheduled_at: chrono::Utc::now(),
                });
            }
        }

        // Sort by priority (higher priority first)
        tasks.sort_by(|a, b| b.priority.cmp(&a.priority));
        Ok(tasks)
    }

    /// Shortest Job First scheduling
    async fn schedule_sjf(
        &self,
        graph: &Graph<S>,
        available_nodes: &[NodeId],
    ) -> CoreResult<Vec<ScheduledTask>> {
        let mut tasks = Vec::new();

        for node_id in available_nodes {
            if let Some(node) = graph.nodes.get(node_id) {
                let metadata = node.metadata();
                let estimated_duration = node.estimated_duration_ms().unwrap_or(1000);
                
                tasks.push(ScheduledTask {
                    node_id: node_id.clone(),
                    priority: 0,
                    estimated_duration,
                    resource_requirements: metadata.resource_requirements.clone(),
                    dependencies: Vec::new(),
                    scheduled_at: chrono::Utc::now(),
                });
            }
        }

        // Sort by estimated duration (shortest first)
        tasks.sort_by(|a, b| a.estimated_duration.cmp(&b.estimated_duration));
        Ok(tasks)
    }

    /// Resource-aware scheduling
    async fn schedule_resource_aware(
        &self,
        graph: &Graph<S>,
        available_nodes: &[NodeId],
        resource_manager: &ResourceManager,
    ) -> CoreResult<Vec<ScheduledTask>> {
        let mut tasks = Vec::new();
        let available_resources = resource_manager.get_available_resources().await;

        for node_id in available_nodes {
            if let Some(node) = graph.nodes.get(node_id) {
                let metadata = node.metadata();
                let requirements = &metadata.resource_requirements;
                
                // Check if resources are available
                if requirements.is_compatible_with(&available_resources) {
                    let priority = self.calculate_resource_priority(requirements, &available_resources);
                    
                    tasks.push(ScheduledTask {
                        node_id: node_id.clone(),
                        priority,
                        estimated_duration: node.estimated_duration_ms().unwrap_or(1000),
                        resource_requirements: requirements.clone(),
                        dependencies: Vec::new(),
                        scheduled_at: chrono::Utc::now(),
                    });
                }
            }
        }

        // Sort by resource priority
        tasks.sort_by(|a, b| b.priority.cmp(&a.priority));
        Ok(tasks)
    }

    /// Critical path scheduling
    async fn schedule_critical_path(
        &self,
        graph: &Graph<S>,
        available_nodes: &[NodeId],
    ) -> CoreResult<Vec<ScheduledTask>> {
        let critical_path_lengths = self.calculate_critical_path_lengths(graph, available_nodes).await?;
        let mut tasks = Vec::new();

        for node_id in available_nodes {
            if let Some(node) = graph.nodes.get(node_id) {
                let metadata = node.metadata();
                let critical_path_length = critical_path_lengths.get(node_id).unwrap_or(&0);
                
                tasks.push(ScheduledTask {
                    node_id: node_id.clone(),
                    priority: *critical_path_length as i32,
                    estimated_duration: node.estimated_duration_ms().unwrap_or(1000),
                    resource_requirements: metadata.resource_requirements.clone(),
                    dependencies: Vec::new(),
                    scheduled_at: chrono::Utc::now(),
                });
            }
        }

        // Sort by critical path length (longest first)
        tasks.sort_by(|a, b| b.priority.cmp(&a.priority));
        Ok(tasks)
    }

    /// Load-balanced scheduling
    async fn schedule_load_balanced(
        &self,
        graph: &Graph<S>,
        available_nodes: &[NodeId],
        resource_manager: &ResourceManager,
    ) -> CoreResult<Vec<ScheduledTask>> {
        let mut tasks = Vec::new();
        let load_info = resource_manager.get_load_information().await;

        for node_id in available_nodes {
            if let Some(node) = graph.nodes.get(node_id) {
                let metadata = node.metadata();
                let load_score = self.calculate_load_score(&metadata.resource_requirements, &load_info);
                
                tasks.push(ScheduledTask {
                    node_id: node_id.clone(),
                    priority: load_score,
                    estimated_duration: node.estimated_duration_ms().unwrap_or(1000),
                    resource_requirements: metadata.resource_requirements.clone(),
                    dependencies: Vec::new(),
                    scheduled_at: chrono::Utc::now(),
                });
            }
        }

        // Sort by load score (lower load first)
        tasks.sort_by(|a, b| a.priority.cmp(&b.priority));
        Ok(tasks)
    }

    /// Calculate node priority based on metadata
    fn calculate_node_priority(&self, metadata: &NodeMetadata) -> i32 {
        match metadata.priority {
            agent_graph_core::node::NodePriority::Critical => 1000,
            agent_graph_core::node::NodePriority::High => 750,
            agent_graph_core::node::NodePriority::Normal => 500,
            agent_graph_core::node::NodePriority::Low => 250,
        }
    }

    /// Calculate resource-based priority
    fn calculate_resource_priority(
        &self,
        requirements: &agent_graph_core::node::ResourceRequirements,
        available: &agent_graph_core::node::AvailableResources,
    ) -> i32 {
        let mut score = 100;

        // Prefer nodes with lower resource requirements when resources are scarce
        if let (Some(req_mem), Some(avail_mem)) = (requirements.memory_mb, available.memory_mb) {
            let memory_ratio = req_mem as f64 / avail_mem as f64;
            score -= (memory_ratio * 50.0) as i32;
        }

        if let (Some(req_cpu), Some(avail_cpu)) = (requirements.cpu_cores, available.cpu_cores) {
            let cpu_ratio = req_cpu / avail_cpu;
            score -= (cpu_ratio * 30.0) as i32;
        }

        score.max(0)
    }

    /// Calculate critical path lengths for nodes
    async fn calculate_critical_path_lengths(
        &self,
        graph: &Graph<S>,
        available_nodes: &[NodeId],
    ) -> CoreResult<HashMap<NodeId, u64>> {
        let mut path_lengths = HashMap::new();
        
        // Simple implementation: use estimated duration as path length
        // In a more sophisticated implementation, this would traverse the graph
        for node_id in available_nodes {
            if let Some(node) = graph.nodes.get(node_id) {
                let duration = node.estimated_duration_ms().unwrap_or(1000);
                path_lengths.insert(node_id.clone(), duration);
            }
        }

        Ok(path_lengths)
    }

    /// Calculate load score for load balancing
    fn calculate_load_score(
        &self,
        requirements: &agent_graph_core::node::ResourceRequirements,
        load_info: &LoadInformation,
    ) -> i32 {
        let mut score = 0;

        // Higher score for higher resource usage (we want to balance load)
        if let Some(memory_mb) = requirements.memory_mb {
            score += (memory_mb as f64 * load_info.memory_load_factor) as i32;
        }

        if let Some(cpu_cores) = requirements.cpu_cores {
            score += (cpu_cores * load_info.cpu_load_factor) as i32;
        }

        score
    }
}

/// Scheduling strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchedulingStrategy {
    /// First In, First Out
    FIFO,
    /// Priority-based scheduling
    Priority,
    /// Shortest Job First
    ShortestJobFirst,
    /// Resource-aware scheduling
    ResourceAware,
    /// Critical path scheduling
    CriticalPath,
    /// Load-balanced scheduling
    LoadBalanced,
}

impl Default for SchedulingStrategy {
    fn default() -> Self {
        Self::Priority
    }
}

/// Scheduler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// Maximum number of tasks to schedule at once
    pub max_concurrent_tasks: usize,
    /// Enable adaptive scheduling
    pub adaptive_scheduling: bool,
    /// Resource utilization threshold
    pub resource_threshold: f64,
    /// Load balancing factor
    pub load_balance_factor: f64,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 10,
            adaptive_scheduling: true,
            resource_threshold: 0.8,
            load_balance_factor: 1.0,
        }
    }
}

/// Scheduled task information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    /// Node ID to execute
    pub node_id: NodeId,
    /// Execution priority
    pub priority: i32,
    /// Estimated execution duration in milliseconds
    pub estimated_duration: u64,
    /// Resource requirements
    pub resource_requirements: agent_graph_core::node::ResourceRequirements,
    /// Task dependencies
    pub dependencies: Vec<NodeId>,
    /// When the task was scheduled
    pub scheduled_at: chrono::DateTime<chrono::Utc>,
}

impl PartialEq for ScheduledTask {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl Eq for ScheduledTask {}

impl PartialOrd for ScheduledTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScheduledTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first
        other.priority.cmp(&self.priority)
    }
}

/// Resource manager for tracking available resources
#[derive(Debug)]
pub struct ResourceManager {
    available_resources: agent_graph_core::node::AvailableResources,
    load_info: LoadInformation,
}

impl ResourceManager {
    /// Create a new resource manager
    pub fn new(available_resources: agent_graph_core::node::AvailableResources) -> Self {
        Self {
            available_resources,
            load_info: LoadInformation::default(),
        }
    }

    /// Get available resources
    pub async fn get_available_resources(&self) -> agent_graph_core::node::AvailableResources {
        self.available_resources.clone()
    }

    /// Get load information
    pub async fn get_load_information(&self) -> LoadInformation {
        self.load_info.clone()
    }

    /// Update resource availability
    pub async fn update_resources(&mut self, resources: agent_graph_core::node::AvailableResources) {
        self.available_resources = resources;
    }

    /// Update load information
    pub async fn update_load_info(&mut self, load_info: LoadInformation) {
        self.load_info = load_info;
    }
}

/// Load information for scheduling decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadInformation {
    /// Current CPU load factor (0.0 to 1.0)
    pub cpu_load_factor: f32,
    /// Current memory load factor (0.0 to 1.0)
    pub memory_load_factor: f64,
    /// Number of active tasks
    pub active_tasks: usize,
    /// Average task duration
    pub average_task_duration: u64,
}

impl Default for LoadInformation {
    fn default() -> Self {
        Self {
            cpu_load_factor: 0.5,
            memory_load_factor: 0.5,
            active_tasks: 0,
            average_task_duration: 1000,
        }
    }
}

/// Task priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    /// Lowest priority
    Low = 1,
    /// Normal priority
    Normal = 2,
    /// High priority
    High = 3,
    /// Critical priority
    Critical = 4,
}

impl Default for TaskPriority {
    fn default() -> Self {
        Self::Normal
    }
}

impl From<agent_graph_core::node::NodePriority> for TaskPriority {
    fn from(priority: agent_graph_core::node::NodePriority) -> Self {
        match priority {
            agent_graph_core::node::NodePriority::Low => TaskPriority::Low,
            agent_graph_core::node::NodePriority::Normal => TaskPriority::Normal,
            agent_graph_core::node::NodePriority::High => TaskPriority::High,
            agent_graph_core::node::NodePriority::Critical => TaskPriority::Critical,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduling_strategy_default() {
        let strategy = SchedulingStrategy::default();
        assert!(matches!(strategy, SchedulingStrategy::Priority));
    }

    #[test]
    fn test_scheduler_config_default() {
        let config = SchedulerConfig::default();
        assert_eq!(config.max_concurrent_tasks, 10);
        assert!(config.adaptive_scheduling);
        assert_eq!(config.resource_threshold, 0.8);
    }

    #[test]
    fn test_scheduled_task_ordering() {
        let task1 = ScheduledTask {
            node_id: "task1".to_string(),
            priority: 100,
            estimated_duration: 1000,
            resource_requirements: agent_graph_core::node::ResourceRequirements::default(),
            dependencies: Vec::new(),
            scheduled_at: chrono::Utc::now(),
        };

        let task2 = ScheduledTask {
            node_id: "task2".to_string(),
            priority: 200,
            estimated_duration: 500,
            resource_requirements: agent_graph_core::node::ResourceRequirements::default(),
            dependencies: Vec::new(),
            scheduled_at: chrono::Utc::now(),
        };

        // Higher priority should come first
        assert!(task2 < task1);
    }

    #[test]
    fn test_task_priority_conversion() {
        let node_priority = agent_graph_core::node::NodePriority::High;
        let task_priority: TaskPriority = node_priority.into();
        assert_eq!(task_priority, TaskPriority::High);
    }

    #[tokio::test]
    async fn test_resource_manager() {
        let resources = agent_graph_core::node::AvailableResources::default();
        let manager = ResourceManager::new(resources.clone());
        
        let available = manager.get_available_resources().await;
        assert_eq!(available.memory_mb, resources.memory_mb);
        assert_eq!(available.cpu_cores, resources.cpu_cores);
    }
}