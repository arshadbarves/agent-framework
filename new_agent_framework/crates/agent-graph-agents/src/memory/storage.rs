//! Memory storage implementation.

use crate::{CoreError, CoreResult};
use crate::memory::{MemoryConfig, MemoryType, MemoryEntry, MemoryQuery, MemorySortOrder};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, warn};

/// In-memory storage for agent memories
#[derive(Debug)]
pub struct MemoryStorage {
    /// Configuration
    config: MemoryConfig,
    /// Memory entries by type
    memories: Arc<RwLock<HashMap<MemoryType, Vec<MemoryEntry>>>>,
    /// Index by tags for fast retrieval
    tag_index: Arc<RwLock<HashMap<String, Vec<String>>>>, // tag -> memory_ids
    /// Memory access statistics
    stats: Arc<RwLock<MemoryStats>>,
}

impl MemoryStorage {
    /// Create a new memory storage
    pub fn new(config: MemoryConfig) -> Self {
        Self {
            config,
            memories: Arc::new(RwLock::new(HashMap::new())),
            tag_index: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(MemoryStats::default())),
        }
    }

    /// Store a memory entry
    pub async fn store(&self, mut entry: MemoryEntry) -> CoreResult<String> {
        debug!("Storing memory entry: {}", entry.id);

        // Update access time
        entry.last_accessed = chrono::Utc::now();

        let memory_id = entry.id.clone();
        let memory_type = entry.memory_type.clone();
        let tags = entry.tags.clone();

        // Store the memory
        {
            let mut memories = self.memories.write();
            let type_memories = memories.entry(memory_type.clone()).or_insert_with(Vec::new);
            
            // Check capacity limits
            match memory_type {
                MemoryType::ShortTerm => {
                    if type_memories.len() >= self.config.max_short_term_entries {
                        // Remove oldest entry
                        if let Some(oldest_idx) = self.find_oldest_entry_index(type_memories) {
                            let removed = type_memories.remove(oldest_idx);
                            self.remove_from_tag_index(&removed);
                            debug!("Removed oldest short-term memory: {}", removed.id);
                        }
                    }
                }
                MemoryType::LongTerm => {
                    if type_memories.len() >= self.config.max_long_term_entries {
                        // Remove least important entry
                        if let Some(least_important_idx) = self.find_least_important_entry_index(type_memories) {
                            let removed = type_memories.remove(least_important_idx);
                            self.remove_from_tag_index(&removed);
                            debug!("Removed least important long-term memory: {}", removed.id);
                        }
                    }
                }
                _ => {} // No limits for other types
            }
            
            type_memories.push(entry);
        }

        // Update tag index
        {
            let mut tag_index = self.tag_index.write();
            for tag in tags {
                tag_index.entry(tag).or_insert_with(Vec::new).push(memory_id.clone());
            }
        }

        // Update statistics
        {
            let mut stats = self.stats.write();
            stats.total_memories += 1;
            match memory_type {
                MemoryType::ShortTerm => stats.short_term_count += 1,
                MemoryType::LongTerm => stats.long_term_count += 1,
                MemoryType::Episodic => stats.episodic_count += 1,
                MemoryType::Custom(_) => stats.custom_count += 1,
            }
        }

        Ok(memory_id)
    }

    /// Retrieve memories based on a query
    pub async fn retrieve(&self, query: &MemoryQuery) -> CoreResult<Vec<MemoryEntry>> {
        debug!("Retrieving memories with query: {:?}", query);

        let memories = self.memories.read();
        let mut results = Vec::new();

        // Collect memories from requested types
        for memory_type in &query.memory_types {
            if let Some(type_memories) = memories.get(memory_type) {
                for memory in type_memories {
                    if self.matches_query(memory, query) {
                        let mut memory_clone = memory.clone();
                        memory_clone.mark_accessed();
                        results.push(memory_clone);
                    }
                }
            }
        }

        // Sort results
        self.sort_memories(&mut results, &query.sort_by);

        // Apply limit
        if let Some(limit) = query.limit {
            results.truncate(limit);
        }

        // Update access statistics
        {
            let mut stats = self.stats.write();
            stats.total_retrievals += 1;
            stats.total_accessed_memories += results.len();
        }

        debug!("Retrieved {} memories", results.len());
        Ok(results)
    }

    /// Get memory by ID
    pub async fn get_by_id(&self, memory_id: &str) -> CoreResult<Option<MemoryEntry>> {
        let memories = self.memories.read();
        
        for type_memories in memories.values() {
            if let Some(memory) = type_memories.iter().find(|m| m.id == memory_id) {
                let mut memory_clone = memory.clone();
                memory_clone.mark_accessed();
                return Ok(Some(memory_clone));
            }
        }
        
        Ok(None)
    }

    /// Remove memory by ID
    pub async fn remove(&self, memory_id: &str) -> CoreResult<bool> {
        debug!("Removing memory: {}", memory_id);

        let mut memories = self.memories.write();
        
        for type_memories in memories.values_mut() {
            if let Some(pos) = type_memories.iter().position(|m| m.id == memory_id) {
                let removed = type_memories.remove(pos);
                self.remove_from_tag_index(&removed);
                
                // Update statistics
                {
                    let mut stats = self.stats.write();
                    stats.total_memories = stats.total_memories.saturating_sub(1);
                    match removed.memory_type {
                        MemoryType::ShortTerm => stats.short_term_count = stats.short_term_count.saturating_sub(1),
                        MemoryType::LongTerm => stats.long_term_count = stats.long_term_count.saturating_sub(1),
                        MemoryType::Episodic => stats.episodic_count = stats.episodic_count.saturating_sub(1),
                        MemoryType::Custom(_) => stats.custom_count = stats.custom_count.saturating_sub(1),
                    }
                }
                
                return Ok(true);
            }
        }
        
        Ok(false)
    }

    /// Clean up expired memories
    pub async fn cleanup_expired(&self) -> CoreResult<usize> {
        debug!("Cleaning up expired memories");

        let mut removed_count = 0;
        let mut memories = self.memories.write();
        
        for type_memories in memories.values_mut() {
            let initial_len = type_memories.len();
            type_memories.retain(|memory| {
                if memory.should_retain(&self.config.retention_policy) {
                    true
                } else {
                    self.remove_from_tag_index(memory);
                    false
                }
            });
            removed_count += initial_len - type_memories.len();
        }

        if removed_count > 0 {
            debug!("Removed {} expired memories", removed_count);
            
            // Update statistics
            let mut stats = self.stats.write();
            stats.total_memories = stats.total_memories.saturating_sub(removed_count);
            stats.cleanup_operations += 1;
        }

        Ok(removed_count)
    }

    /// Get memory statistics
    pub async fn get_stats(&self) -> MemoryStats {
        self.stats.read().clone()
    }

    /// Clear all memories
    pub async fn clear(&self) -> CoreResult<()> {
        debug!("Clearing all memories");

        {
            let mut memories = self.memories.write();
            memories.clear();
        }

        {
            let mut tag_index = self.tag_index.write();
            tag_index.clear();
        }

        {
            let mut stats = self.stats.write();
            *stats = MemoryStats::default();
        }

        Ok(())
    }

    /// Check if a memory matches the query criteria
    fn matches_query(&self, memory: &MemoryEntry, query: &MemoryQuery) -> bool {
        // Check tags
        if !query.tags.is_empty() {
            let has_matching_tag = query.tags.iter().any(|tag| memory.has_tag(tag));
            if !has_matching_tag {
                return false;
            }
        }

        // Check minimum importance
        if let Some(min_importance) = query.min_importance {
            if memory.importance < min_importance {
                return false;
            }
        }

        // Check maximum age
        if let Some(max_age_hours) = query.max_age_hours {
            if memory.age_hours() > max_age_hours {
                return false;
            }
        }

        true
    }

    /// Sort memories based on the sort order
    fn sort_memories(&self, memories: &mut [MemoryEntry], sort_by: &MemorySortOrder) {
        match sort_by {
            MemorySortOrder::Relevance => {
                // Sort by importance (descending) then by access count
                memories.sort_by(|a, b| {
                    b.importance.partial_cmp(&a.importance)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| b.access_count.cmp(&a.access_count))
                });
            }
            MemorySortOrder::Newest => {
                memories.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            }
            MemorySortOrder::Oldest => {
                memories.sort_by(|a, b| a.created_at.cmp(&b.created_at));
            }
            MemorySortOrder::MostAccessed => {
                memories.sort_by(|a, b| b.access_count.cmp(&a.access_count));
            }
            MemorySortOrder::RecentlyAccessed => {
                memories.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));
            }
        }
    }

    /// Find the index of the oldest entry
    fn find_oldest_entry_index(&self, memories: &[MemoryEntry]) -> Option<usize> {
        memories.iter()
            .enumerate()
            .min_by_key(|(_, memory)| memory.created_at)
            .map(|(idx, _)| idx)
    }

    /// Find the index of the least important entry
    fn find_least_important_entry_index(&self, memories: &[MemoryEntry]) -> Option<usize> {
        memories.iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                a.importance.partial_cmp(&b.importance)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.access_count.cmp(&b.access_count))
            })
            .map(|(idx, _)| idx)
    }

    /// Remove memory from tag index
    fn remove_from_tag_index(&self, memory: &MemoryEntry) {
        let mut tag_index = self.tag_index.write();
        for tag in &memory.tags {
            if let Some(memory_ids) = tag_index.get_mut(tag) {
                memory_ids.retain(|id| id != &memory.id);
                if memory_ids.is_empty() {
                    tag_index.remove(tag);
                }
            }
        }
    }
}

/// Memory storage statistics
#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    /// Total number of memories stored
    pub total_memories: usize,
    /// Number of short-term memories
    pub short_term_count: usize,
    /// Number of long-term memories
    pub long_term_count: usize,
    /// Number of episodic memories
    pub episodic_count: usize,
    /// Number of custom memories
    pub custom_count: usize,
    /// Total number of retrieval operations
    pub total_retrievals: usize,
    /// Total number of memories accessed
    pub total_accessed_memories: usize,
    /// Number of cleanup operations performed
    pub cleanup_operations: usize,
}

impl MemoryStats {
    /// Get average memories per retrieval
    pub fn avg_memories_per_retrieval(&self) -> f64 {
        if self.total_retrievals == 0 {
            0.0
        } else {
            self.total_accessed_memories as f64 / self.total_retrievals as f64
        }
    }

    /// Get memory distribution
    pub fn memory_distribution(&self) -> HashMap<String, usize> {
        let mut distribution = HashMap::new();
        distribution.insert("short_term".to_string(), self.short_term_count);
        distribution.insert("long_term".to_string(), self.long_term_count);
        distribution.insert("episodic".to_string(), self.episodic_count);
        distribution.insert("custom".to_string(), self.custom_count);
        distribution
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_storage_creation() {
        let config = MemoryConfig::default();
        let storage = MemoryStorage::new(config);
        
        let stats = storage.get_stats().await;
        assert_eq!(stats.total_memories, 0);
    }

    #[tokio::test]
    async fn test_store_and_retrieve_memory() {
        let config = MemoryConfig::default();
        let storage = MemoryStorage::new(config);
        
        let memory = MemoryEntry::new(
            MemoryType::ShortTerm,
            serde_json::json!({"test": "data"}),
            0.8,
        );
        
        let memory_id = storage.store(memory.clone()).await.unwrap();
        assert_eq!(memory_id, memory.id);
        
        let retrieved = storage.get_by_id(&memory_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().content, memory.content);
    }

    #[tokio::test]
    async fn test_memory_query() {
        let config = MemoryConfig::default();
        let storage = MemoryStorage::new(config);
        
        let mut memory1 = MemoryEntry::new(
            MemoryType::ShortTerm,
            serde_json::json!({"test": "data1"}),
            0.9,
        );
        memory1.add_tag("important".to_string());
        
        let memory2 = MemoryEntry::new(
            MemoryType::LongTerm,
            serde_json::json!({"test": "data2"}),
            0.5,
        );
        
        storage.store(memory1).await.unwrap();
        storage.store(memory2).await.unwrap();
        
        let query = MemoryQuery {
            memory_types: vec![MemoryType::ShortTerm, MemoryType::LongTerm],
            min_importance: Some(0.8),
            ..Default::default()
        };
        
        let results = storage.retrieve(&query).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].importance, 0.9);
    }

    #[tokio::test]
    async fn test_memory_cleanup() {
        let mut config = MemoryConfig::default();
        config.retention_policy.short_term_retention_hours = 0; // Immediate expiration
        
        let storage = MemoryStorage::new(config);
        
        let memory = MemoryEntry::new(
            MemoryType::ShortTerm,
            serde_json::json!({"test": "data"}),
            0.8,
        );
        
        storage.store(memory).await.unwrap();
        
        let removed_count = storage.cleanup_expired().await.unwrap();
        assert_eq!(removed_count, 1);
        
        let stats = storage.get_stats().await;
        assert_eq!(stats.total_memories, 0);
    }
}