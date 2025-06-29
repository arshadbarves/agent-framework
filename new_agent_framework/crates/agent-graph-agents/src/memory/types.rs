//! Memory types and configurations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Memory configuration for agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Enable short-term memory
    pub enable_short_term: bool,
    /// Enable long-term memory
    pub enable_long_term: bool,
    /// Enable episodic memory
    pub enable_episodic: bool,
    /// Maximum short-term memory entries
    pub max_short_term_entries: usize,
    /// Maximum long-term memory entries
    pub max_long_term_entries: usize,
    /// Memory retention policy
    pub retention_policy: RetentionPolicy,
    /// Custom memory parameters
    pub custom_params: HashMap<String, serde_json::Value>,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            enable_short_term: true,
            enable_long_term: true,
            enable_episodic: true,
            max_short_term_entries: 100,
            max_long_term_entries: 1000,
            retention_policy: RetentionPolicy::default(),
            custom_params: HashMap::new(),
        }
    }
}

/// Memory retention policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Short-term memory retention duration (in hours)
    pub short_term_retention_hours: u64,
    /// Long-term memory retention duration (in days)
    pub long_term_retention_days: u64,
    /// Episodic memory retention duration (in days)
    pub episodic_retention_days: u64,
    /// Auto-cleanup enabled
    pub auto_cleanup: bool,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            short_term_retention_hours: 24,    // 1 day
            long_term_retention_days: 30,      // 30 days
            episodic_retention_days: 7,        // 7 days
            auto_cleanup: true,
        }
    }
}

/// Types of memory
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MemoryType {
    /// Short-term working memory
    ShortTerm,
    /// Long-term persistent memory
    LongTerm,
    /// Episodic memory for experiences
    Episodic,
    /// Custom memory type
    Custom(String),
}

/// Memory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// Unique identifier
    pub id: String,
    /// Memory type
    pub memory_type: MemoryType,
    /// Memory content
    pub content: serde_json::Value,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last accessed timestamp
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    /// Access count
    pub access_count: u64,
    /// Memory importance score (0.0 to 1.0)
    pub importance: f64,
    /// Associated tags
    pub tags: Vec<String>,
    /// Memory metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl MemoryEntry {
    /// Create a new memory entry
    pub fn new(
        memory_type: MemoryType,
        content: serde_json::Value,
        importance: f64,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            memory_type,
            content,
            created_at: now,
            last_accessed: now,
            access_count: 0,
            importance: importance.clamp(0.0, 1.0),
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Mark memory as accessed
    pub fn mark_accessed(&mut self) {
        self.last_accessed = chrono::Utc::now();
        self.access_count += 1;
    }

    /// Add a tag to the memory
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// Check if memory has a specific tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(&tag.to_string())
    }

    /// Get memory age in hours
    pub fn age_hours(&self) -> f64 {
        let now = chrono::Utc::now();
        let duration = now - self.created_at;
        duration.num_seconds() as f64 / 3600.0
    }

    /// Check if memory should be retained based on policy
    pub fn should_retain(&self, policy: &RetentionPolicy) -> bool {
        let age_hours = self.age_hours();
        
        match self.memory_type {
            MemoryType::ShortTerm => age_hours < policy.short_term_retention_hours as f64,
            MemoryType::LongTerm => age_hours < (policy.long_term_retention_days * 24) as f64,
            MemoryType::Episodic => age_hours < (policy.episodic_retention_days * 24) as f64,
            MemoryType::Custom(_) => true, // Custom memories are retained by default
        }
    }
}

/// Memory retrieval query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryQuery {
    /// Memory types to search
    pub memory_types: Vec<MemoryType>,
    /// Tags to filter by
    pub tags: Vec<String>,
    /// Minimum importance threshold
    pub min_importance: Option<f64>,
    /// Maximum age in hours
    pub max_age_hours: Option<f64>,
    /// Maximum number of results
    pub limit: Option<usize>,
    /// Sort order
    pub sort_by: MemorySortOrder,
}

impl Default for MemoryQuery {
    fn default() -> Self {
        Self {
            memory_types: vec![MemoryType::ShortTerm, MemoryType::LongTerm, MemoryType::Episodic],
            tags: Vec::new(),
            min_importance: None,
            max_age_hours: None,
            limit: Some(10),
            sort_by: MemorySortOrder::Relevance,
        }
    }
}

/// Memory sort order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemorySortOrder {
    /// Sort by relevance/importance
    Relevance,
    /// Sort by creation time (newest first)
    Newest,
    /// Sort by creation time (oldest first)
    Oldest,
    /// Sort by access count
    MostAccessed,
    /// Sort by last access time
    RecentlyAccessed,
}