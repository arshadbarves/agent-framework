//! Task priority system for scheduling.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Task priority levels for scheduling
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskPriority {
    /// Critical priority - must execute immediately
    Critical,
    /// High priority - execute before normal tasks
    High,
    /// Normal priority - default priority level
    Normal,
    /// Low priority - execute when resources available
    Low,
}

impl TaskPriority {
    /// Get numeric value for priority comparison (higher = more important)
    pub fn numeric_value(&self) -> u8 {
        match self {
            TaskPriority::Critical => 4,
            TaskPriority::High => 3,
            TaskPriority::Normal => 2,
            TaskPriority::Low => 1,
        }
    }

    /// Get priority name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskPriority::Critical => "critical",
            TaskPriority::High => "high",
            TaskPriority::Normal => "normal",
            TaskPriority::Low => "low",
        }
    }

    /// Create priority from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "critical" => Some(TaskPriority::Critical),
            "high" => Some(TaskPriority::High),
            "normal" => Some(TaskPriority::Normal),
            "low" => Some(TaskPriority::Low),
            _ => None,
        }
    }

    /// Check if this priority is higher than another
    pub fn is_higher_than(&self, other: &TaskPriority) -> bool {
        self.numeric_value() > other.numeric_value()
    }

    /// Get all priority levels in order (highest to lowest)
    pub fn all_levels() -> Vec<TaskPriority> {
        vec![
            TaskPriority::Critical,
            TaskPriority::High,
            TaskPriority::Normal,
            TaskPriority::Low,
        ]
    }
}

impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::Normal
    }
}

impl Ord for TaskPriority {
    fn cmp(&self, other: &Self) -> Ordering {
        self.numeric_value().cmp(&other.numeric_value())
    }
}

impl PartialOrd for TaskPriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl std::fmt::Display for TaskPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Priority queue for managing tasks by priority
#[derive(Debug)]
pub struct PriorityQueue<T> {
    queues: std::collections::HashMap<TaskPriority, std::collections::VecDeque<T>>,
}

impl<T> PriorityQueue<T> {
    /// Create a new priority queue
    pub fn new() -> Self {
        let mut queues = std::collections::HashMap::new();
        for priority in TaskPriority::all_levels() {
            queues.insert(priority, std::collections::VecDeque::new());
        }
        
        Self { queues }
    }

    /// Add an item with the given priority
    pub fn push(&mut self, item: T, priority: TaskPriority) {
        if let Some(queue) = self.queues.get_mut(&priority) {
            queue.push_back(item);
        }
    }

    /// Remove and return the highest priority item
    pub fn pop(&mut self) -> Option<T> {
        // Check queues in priority order (highest first)
        for priority in TaskPriority::all_levels() {
            if let Some(queue) = self.queues.get_mut(&priority) {
                if let Some(item) = queue.pop_front() {
                    return Some(item);
                }
            }
        }
        None
    }

    /// Peek at the highest priority item without removing it
    pub fn peek(&self) -> Option<&T> {
        for priority in TaskPriority::all_levels() {
            if let Some(queue) = self.queues.get(&priority) {
                if let Some(item) = queue.front() {
                    return Some(item);
                }
            }
        }
        None
    }

    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.queues.values().all(|queue| queue.is_empty())
    }

    /// Get the total number of items in all queues
    pub fn len(&self) -> usize {
        self.queues.values().map(|queue| queue.len()).sum()
    }

    /// Get the number of items at each priority level
    pub fn len_by_priority(&self) -> std::collections::HashMap<TaskPriority, usize> {
        self.queues.iter()
            .map(|(priority, queue)| (*priority, queue.len()))
            .collect()
    }

    /// Clear all items from the queue
    pub fn clear(&mut self) {
        for queue in self.queues.values_mut() {
            queue.clear();
        }
    }

    /// Get items at a specific priority level
    pub fn items_at_priority(&self, priority: TaskPriority) -> Option<&std::collections::VecDeque<T>> {
        self.queues.get(&priority)
    }

    /// Remove all items at a specific priority level
    pub fn clear_priority(&mut self, priority: TaskPriority) {
        if let Some(queue) = self.queues.get_mut(&priority) {
            queue.clear();
        }
    }
}

impl<T> Default for PriorityQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Priority-based task scheduler
#[derive(Debug)]
pub struct PriorityScheduler<T> {
    queue: PriorityQueue<T>,
    stats: SchedulerStats,
}

impl<T> PriorityScheduler<T> {
    /// Create a new priority scheduler
    pub fn new() -> Self {
        Self {
            queue: PriorityQueue::new(),
            stats: SchedulerStats::default(),
        }
    }

    /// Schedule a task with the given priority
    pub fn schedule(&mut self, task: T, priority: TaskPriority) {
        self.queue.push(task, priority);
        self.stats.tasks_scheduled += 1;
        self.stats.increment_priority_count(priority);
    }

    /// Get the next task to execute
    pub fn next_task(&mut self) -> Option<T> {
        if let Some(task) = self.queue.pop() {
            self.stats.tasks_executed += 1;
            Some(task)
        } else {
            None
        }
    }

    /// Check if there are any tasks to execute
    pub fn has_tasks(&self) -> bool {
        !self.queue.is_empty()
    }

    /// Get the number of pending tasks
    pub fn pending_tasks(&self) -> usize {
        self.queue.len()
    }

    /// Get scheduler statistics
    pub fn stats(&self) -> &SchedulerStats {
        &self.stats
    }

    /// Get tasks by priority level
    pub fn tasks_by_priority(&self) -> std::collections::HashMap<TaskPriority, usize> {
        self.queue.len_by_priority()
    }

    /// Clear all scheduled tasks
    pub fn clear(&mut self) {
        self.queue.clear();
        self.stats = SchedulerStats::default();
    }
}

impl<T> Default for PriorityScheduler<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Scheduler statistics
#[derive(Debug, Clone, Default)]
pub struct SchedulerStats {
    /// Total tasks scheduled
    pub tasks_scheduled: usize,
    /// Total tasks executed
    pub tasks_executed: usize,
    /// Tasks by priority level
    pub priority_counts: std::collections::HashMap<TaskPriority, usize>,
}

impl SchedulerStats {
    /// Increment count for a priority level
    pub fn increment_priority_count(&mut self, priority: TaskPriority) {
        *self.priority_counts.entry(priority).or_insert(0) += 1;
    }

    /// Get pending tasks (scheduled but not executed)
    pub fn pending_tasks(&self) -> usize {
        self.tasks_scheduled.saturating_sub(self.tasks_executed)
    }

    /// Get execution rate (0.0 to 1.0)
    pub fn execution_rate(&self) -> f64 {
        if self.tasks_scheduled == 0 {
            0.0
        } else {
            self.tasks_executed as f64 / self.tasks_scheduled as f64
        }
    }

    /// Get most common priority level
    pub fn most_common_priority(&self) -> Option<TaskPriority> {
        self.priority_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(priority, _)| *priority)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_priority_ordering() {
        assert!(TaskPriority::Critical > TaskPriority::High);
        assert!(TaskPriority::High > TaskPriority::Normal);
        assert!(TaskPriority::Normal > TaskPriority::Low);
        
        assert_eq!(TaskPriority::Critical.numeric_value(), 4);
        assert_eq!(TaskPriority::Low.numeric_value(), 1);
    }

    #[test]
    fn test_task_priority_from_string() {
        assert_eq!(TaskPriority::from_str("critical"), Some(TaskPriority::Critical));
        assert_eq!(TaskPriority::from_str("HIGH"), Some(TaskPriority::High));
        assert_eq!(TaskPriority::from_str("normal"), Some(TaskPriority::Normal));
        assert_eq!(TaskPriority::from_str("low"), Some(TaskPriority::Low));
        assert_eq!(TaskPriority::from_str("invalid"), None);
    }

    #[test]
    fn test_priority_queue() {
        let mut queue = PriorityQueue::new();
        
        queue.push("low task", TaskPriority::Low);
        queue.push("high task", TaskPriority::High);
        queue.push("normal task", TaskPriority::Normal);
        queue.push("critical task", TaskPriority::Critical);
        
        assert_eq!(queue.len(), 4);
        assert!(!queue.is_empty());
        
        // Should return tasks in priority order
        assert_eq!(queue.pop(), Some("critical task"));
        assert_eq!(queue.pop(), Some("high task"));
        assert_eq!(queue.pop(), Some("normal task"));
        assert_eq!(queue.pop(), Some("low task"));
        assert_eq!(queue.pop(), None);
        
        assert!(queue.is_empty());
    }

    #[test]
    fn test_priority_scheduler() {
        let mut scheduler = PriorityScheduler::new();
        
        scheduler.schedule("task1", TaskPriority::Low);
        scheduler.schedule("task2", TaskPriority::High);
        scheduler.schedule("task3", TaskPriority::Normal);
        
        assert_eq!(scheduler.pending_tasks(), 3);
        assert!(scheduler.has_tasks());
        
        // Should execute in priority order
        assert_eq!(scheduler.next_task(), Some("task2")); // High priority
        assert_eq!(scheduler.next_task(), Some("task3")); // Normal priority
        assert_eq!(scheduler.next_task(), Some("task1")); // Low priority
        assert_eq!(scheduler.next_task(), None);
        
        assert!(!scheduler.has_tasks());
        assert_eq!(scheduler.stats().tasks_executed, 3);
        assert_eq!(scheduler.stats().execution_rate(), 1.0);
    }

    #[test]
    fn test_scheduler_stats() {
        let mut stats = SchedulerStats::default();
        
        stats.tasks_scheduled = 10;
        stats.tasks_executed = 7;
        stats.increment_priority_count(TaskPriority::High);
        stats.increment_priority_count(TaskPriority::High);
        stats.increment_priority_count(TaskPriority::Normal);
        
        assert_eq!(stats.pending_tasks(), 3);
        assert_eq!(stats.execution_rate(), 0.7);
        assert_eq!(stats.most_common_priority(), Some(TaskPriority::High));
    }
}