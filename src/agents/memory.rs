// Agent memory system for AgentGraph
// Provides short-term and long-term memory capabilities for agents

#![allow(missing_docs)]

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, SystemTime};
use thiserror::Error;

/// Memory configuration for agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Maximum short-term memory entries
    pub max_short_term_entries: usize,
    /// Maximum long-term memory entries
    pub max_long_term_entries: usize,
    /// Memory retention period
    pub retention_period: Duration,
    /// Enable semantic search in memory
    pub semantic_search: bool,
    /// Memory compression threshold
    pub compression_threshold: usize,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            max_short_term_entries: 50,
            max_long_term_entries: 1000,
            retention_period: Duration::from_secs(86400 * 30), // 30 days
            semantic_search: false, // Disabled for now, would require embeddings
            compression_threshold: 100,
        }
    }
}

/// Memory entry storing interaction data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// Unique entry ID
    pub id: String,
    /// Entry type
    pub entry_type: MemoryEntryType,
    /// Entry content
    pub content: String,
    /// Associated metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Creation timestamp
    pub created_at: SystemTime,
    /// Last accessed timestamp
    pub last_accessed: SystemTime,
    /// Access count
    pub access_count: u32,
    /// Importance score (0.0 - 1.0)
    pub importance: f32,
}

impl MemoryEntry {
    /// Create a new memory entry
    pub fn new(entry_type: MemoryEntryType, content: String) -> Self {
        let now = SystemTime::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            entry_type,
            content,
            metadata: HashMap::new(),
            created_at: now,
            last_accessed: now,
            access_count: 0,
            importance: 0.5, // Default importance
        }
    }
    
    /// Update access information
    pub fn access(&mut self) {
        self.last_accessed = SystemTime::now();
        self.access_count += 1;
    }
    
    /// Set importance score
    pub fn with_importance(mut self, importance: f32) -> Self {
        self.importance = importance.clamp(0.0, 1.0);
        self
    }
    
    /// Add metadata
    pub fn with_metadata<T: Serialize>(mut self, key: String, value: T) -> Self {
        self.metadata.insert(
            key,
            serde_json::to_value(value).unwrap_or(serde_json::Value::Null),
        );
        self
    }
    
    /// Check if entry has expired
    pub fn is_expired(&self, retention_period: Duration) -> bool {
        SystemTime::now()
            .duration_since(self.created_at)
            .unwrap_or(Duration::ZERO) > retention_period
    }
}

/// Types of memory entries
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryEntryType {
    /// User interaction
    Interaction,
    /// Task execution
    Task,
    /// Tool usage
    Tool,
    /// Learning/insight
    Learning,
    /// Error/failure
    Error,
    /// Success/achievement
    Success,
    /// Context/background
    Context,
}

/// Agent memory system
#[derive(Debug)]
pub struct AgentMemory {
    /// Configuration
    config: MemoryConfig,
    /// Short-term memory (recent interactions)
    short_term: VecDeque<MemoryEntry>,
    /// Long-term memory (important/frequent interactions)
    long_term: Vec<MemoryEntry>,
    /// Working memory (current session)
    working_memory: HashMap<String, serde_json::Value>,
}

impl AgentMemory {
    /// Create a new agent memory system
    pub fn new(config: MemoryConfig) -> Result<Self, MemoryError> {
        Ok(Self {
            config,
            short_term: VecDeque::new(),
            long_term: Vec::new(),
            working_memory: HashMap::new(),
        })
    }
    
    /// Store an interaction in memory
    pub async fn store_interaction(&mut self, input: &str, output: &str) -> Result<(), MemoryError> {
        let content = format!("Input: {}\nOutput: {}", input, output);
        let entry = MemoryEntry::new(MemoryEntryType::Interaction, content)
            .with_importance(self.calculate_importance(input, output))
            .with_metadata("input_length".to_string(), input.len())
            .with_metadata("output_length".to_string(), output.len());
        
        self.add_to_short_term(entry);
        self.manage_memory_limits();
        
        Ok(())
    }
    
    /// Store a task execution in memory
    pub async fn store_task(&mut self, task: &str, result: &str, success: bool) -> Result<(), MemoryError> {
        let content = format!("Task: {}\nResult: {}\nSuccess: {}", task, result, success);
        let entry_type = if success { MemoryEntryType::Success } else { MemoryEntryType::Error };
        let importance = if success { 0.7 } else { 0.8 }; // Errors are slightly more important for learning
        
        let entry = MemoryEntry::new(entry_type, content)
            .with_importance(importance)
            .with_metadata("task_length".to_string(), task.len())
            .with_metadata("success".to_string(), success);
        
        self.add_to_short_term(entry);
        self.manage_memory_limits();
        
        Ok(())
    }
    
    /// Store tool usage in memory
    pub async fn store_tool_usage(&mut self, tool_name: &str, args: &str, result: &str) -> Result<(), MemoryError> {
        let content = format!("Tool: {}\nArgs: {}\nResult: {}", tool_name, args, result);
        let entry = MemoryEntry::new(MemoryEntryType::Tool, content)
            .with_importance(0.6)
            .with_metadata("tool_name".to_string(), tool_name)
            .with_metadata("args_length".to_string(), args.len());
        
        self.add_to_short_term(entry);
        self.manage_memory_limits();
        
        Ok(())
    }
    
    /// Get relevant context for a query
    pub async fn get_relevant_context(&mut self, query: &str) -> Result<String, MemoryError> {
        let mut relevant_entries = Vec::new();

        // Search short-term memory
        let mut indices_to_update = Vec::new();
        for (i, entry) in self.short_term.iter().enumerate() {
            if self.is_relevant(query, &entry.content) {
                indices_to_update.push(i);
                relevant_entries.push(entry.clone());
            }
        }

        // Update access information for short-term entries
        for i in indices_to_update {
            if let Some(entry) = self.short_term.get_mut(i) {
                entry.access();
            }
        }

        // Search long-term memory
        let mut indices_to_update = Vec::new();
        for (i, entry) in self.long_term.iter().enumerate() {
            if self.is_relevant(query, &entry.content) {
                indices_to_update.push(i);
                relevant_entries.push(entry.clone());
            }
        }

        // Update access information for long-term entries
        for i in indices_to_update {
            if let Some(entry) = self.long_term.get_mut(i) {
                entry.access();
            }
        }
        
        // Sort by relevance and importance
        relevant_entries.sort_by(|a, b| {
            let score_a = a.importance + (a.access_count as f32 * 0.1);
            let score_b = b.importance + (b.access_count as f32 * 0.1);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Take top 5 most relevant entries
        let context = relevant_entries
            .into_iter()
            .take(5)
            .map(|entry| entry.content)
            .collect::<Vec<_>>()
            .join("\n---\n");
        
        Ok(context)
    }
    
    /// Add entry to short-term memory
    fn add_to_short_term(&mut self, entry: MemoryEntry) {
        self.short_term.push_back(entry);
        
        // Remove oldest if exceeding limit
        while self.short_term.len() > self.config.max_short_term_entries {
            if let Some(old_entry) = self.short_term.pop_front() {
                // Consider moving to long-term if important
                if old_entry.importance > 0.7 || old_entry.access_count > 3 {
                    self.add_to_long_term(old_entry);
                }
            }
        }
    }
    
    /// Add entry to long-term memory
    fn add_to_long_term(&mut self, entry: MemoryEntry) {
        self.long_term.push(entry);
        
        // Sort by importance and access count
        self.long_term.sort_by(|a, b| {
            let score_a = a.importance + (a.access_count as f32 * 0.1);
            let score_b = b.importance + (b.access_count as f32 * 0.1);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Remove least important if exceeding limit
        while self.long_term.len() > self.config.max_long_term_entries {
            self.long_term.pop();
        }
    }
    
    /// Manage memory limits and cleanup
    fn manage_memory_limits(&mut self) {
        let now = SystemTime::now();
        
        // Remove expired entries from short-term memory
        self.short_term.retain(|entry| !entry.is_expired(self.config.retention_period));
        
        // Remove expired entries from long-term memory
        self.long_term.retain(|entry| !entry.is_expired(self.config.retention_period));
        
        // Compress memory if needed
        if self.short_term.len() + self.long_term.len() > self.config.compression_threshold {
            self.compress_memory();
        }
    }
    
    /// Compress memory by removing less important entries
    fn compress_memory(&mut self) {
        // Remove entries with low importance and access count from short-term
        self.short_term.retain(|entry| {
            entry.importance > 0.3 || entry.access_count > 1
        });
        
        // Keep only top 80% of long-term memory
        let keep_count = (self.long_term.len() as f32 * 0.8) as usize;
        self.long_term.truncate(keep_count);
    }
    
    /// Check if content is relevant to query (simple keyword matching)
    fn is_relevant(&self, query: &str, content: &str) -> bool {
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        let content_lower = content.to_lowercase();

        // Check if any query words appear in content
        query_words.iter().any(|word| content_lower.contains(word))
    }
    
    /// Calculate importance score for an interaction
    fn calculate_importance(&self, input: &str, output: &str) -> f32 {
        let mut importance: f32 = 0.5; // Base importance
        
        // Longer interactions might be more important
        let total_length = input.len() + output.len();
        if total_length > 500 {
            importance += 0.1;
        }
        if total_length > 1000 {
            importance += 0.1;
        }
        
        // Check for important keywords
        let important_keywords = ["error", "problem", "solution", "important", "critical", "urgent"];
        let combined_text = format!("{} {}", input, output).to_lowercase();
        
        for keyword in &important_keywords {
            if combined_text.contains(keyword) {
                importance += 0.1;
            }
        }
        
        importance.clamp(0.0, 1.0)
    }
    
    /// Set working memory value
    pub fn set_working_memory<T: Serialize>(&mut self, key: String, value: T) {
        self.working_memory.insert(
            key,
            serde_json::to_value(value).unwrap_or(serde_json::Value::Null),
        );
    }
    
    /// Get working memory value
    pub fn get_working_memory(&self, key: &str) -> Option<&serde_json::Value> {
        self.working_memory.get(key)
    }
    
    /// Clear working memory
    pub fn clear_working_memory(&mut self) {
        self.working_memory.clear();
    }
    
    /// Clear all memory
    pub fn clear(&mut self) {
        self.short_term.clear();
        self.long_term.clear();
        self.working_memory.clear();
    }
    
    /// Get memory statistics
    pub fn get_stats(&self) -> MemoryStats {
        MemoryStats {
            short_term_entries: self.short_term.len(),
            long_term_entries: self.long_term.len(),
            working_memory_entries: self.working_memory.len(),
            total_entries: self.short_term.len() + self.long_term.len(),
            average_importance: self.calculate_average_importance(),
        }
    }
    
    /// Calculate average importance across all entries
    fn calculate_average_importance(&self) -> f32 {
        let all_entries: Vec<&MemoryEntry> = self.short_term.iter()
            .chain(self.long_term.iter())
            .collect();
        
        if all_entries.is_empty() {
            return 0.0;
        }
        
        let total_importance: f32 = all_entries.iter()
            .map(|entry| entry.importance)
            .sum();
        
        total_importance / all_entries.len() as f32
    }
    
    /// Get configuration
    pub fn config(&self) -> &MemoryConfig {
        &self.config
    }
}

/// Memory statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Number of short-term memory entries
    pub short_term_entries: usize,
    /// Number of long-term memory entries
    pub long_term_entries: usize,
    /// Number of working memory entries
    pub working_memory_entries: usize,
    /// Total memory entries
    pub total_entries: usize,
    /// Average importance score
    pub average_importance: f32,
}

/// Errors that can occur in memory operations
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum MemoryError {
    /// Storage error
    #[error("Memory storage error: {message}")]
    StorageError { message: String },
    
    /// Retrieval error
    #[error("Memory retrieval error: {message}")]
    RetrievalError { message: String },
    
    /// Configuration error
    #[error("Memory configuration error: {message}")]
    ConfigurationError { message: String },
    
    /// System error
    #[error("Memory system error: {message}")]
    SystemError { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_config_default() {
        let config = MemoryConfig::default();
        assert_eq!(config.max_short_term_entries, 50);
        assert_eq!(config.max_long_term_entries, 1000);
        assert!(!config.semantic_search);
    }

    #[test]
    fn test_memory_entry_creation() {
        let entry = MemoryEntry::new(
            MemoryEntryType::Interaction,
            "Test content".to_string(),
        )
        .with_importance(0.8)
        .with_metadata("test_key".to_string(), "test_value");
        
        assert_eq!(entry.entry_type, MemoryEntryType::Interaction);
        assert_eq!(entry.content, "Test content");
        assert_eq!(entry.importance, 0.8);
        assert_eq!(entry.metadata.get("test_key"), Some(&serde_json::json!("test_value")));
    }

    #[tokio::test]
    async fn test_agent_memory_basic_operations() {
        let config = MemoryConfig::default();
        let mut memory = AgentMemory::new(config).unwrap();
        
        // Store interaction
        memory.store_interaction("Hello", "Hi there!").await.unwrap();
        
        // Get relevant context
        let context = memory.get_relevant_context("Hello").await.unwrap();
        assert!(context.contains("Hello"));
        assert!(context.contains("Hi there!"));
        
        // Check stats
        let stats = memory.get_stats();
        assert_eq!(stats.short_term_entries, 1);
        assert_eq!(stats.total_entries, 1);
    }

    #[tokio::test]
    async fn test_memory_working_memory() {
        let config = MemoryConfig::default();
        let mut memory = AgentMemory::new(config).unwrap();
        
        // Set working memory
        memory.set_working_memory("current_task".to_string(), "Testing");
        
        // Get working memory
        let value = memory.get_working_memory("current_task");
        assert_eq!(value, Some(&serde_json::json!("Testing")));
        
        // Clear working memory
        memory.clear_working_memory();
        assert!(memory.get_working_memory("current_task").is_none());
    }

    #[test]
    fn test_memory_entry_access() {
        let mut entry = MemoryEntry::new(
            MemoryEntryType::Task,
            "Test task".to_string(),
        );
        
        assert_eq!(entry.access_count, 0);
        
        entry.access();
        assert_eq!(entry.access_count, 1);
        
        entry.access();
        assert_eq!(entry.access_count, 2);
    }

    #[test]
    fn test_importance_calculation() {
        let config = MemoryConfig::default();
        let memory = AgentMemory::new(config).unwrap();
        
        // Short interaction
        let importance1 = memory.calculate_importance("Hi", "Hello");
        assert!(importance1 >= 0.5 && importance1 <= 0.6);
        
        // Long interaction with important keyword
        let long_input = "This is a very long input that contains an error message".repeat(10);
        let long_output = "This is a detailed response explaining the solution".repeat(10);
        let importance2 = memory.calculate_importance(&long_input, &long_output);
        assert!(importance2 > importance1);
    }
}
