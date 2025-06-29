//! Checkpoint manager for state persistence and recovery.

use crate::{CoreError, CoreResult, State};
use agent_graph_core::{Graph, StateSnapshot, ExecutionContext};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug, warn, error};

/// Checkpoint manager for handling state persistence and recovery
#[derive(Debug)]
pub struct CheckpointManager<S>
where
    S: State,
{
    config: CheckpointConfig,
    storage: Arc<dyn CheckpointStorage<S>>,
    active_checkpoints: Arc<RwLock<HashMap<String, CheckpointMetadata>>>,
}

impl<S> CheckpointManager<S>
where
    S: State + Send + Sync + 'static,
{
    /// Create a new checkpoint manager
    pub fn new(config: CheckpointConfig, storage: Arc<dyn CheckpointStorage<S>>) -> Self {
        Self {
            config,
            storage,
            active_checkpoints: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a checkpoint of the current graph state
    pub async fn create_checkpoint(
        &self,
        graph: &Graph<S>,
        context: &ExecutionContext,
        checkpoint_type: CheckpointType,
    ) -> CoreResult<String> {
        let checkpoint_id = uuid::Uuid::new_v4().to_string();
        info!("Creating checkpoint: {} (type: {:?})", checkpoint_id, checkpoint_type);

        // Create state snapshot
        let state_snapshot = graph.state_manager.create_snapshot()?;
        
        // Create execution context snapshot
        let context_snapshot = self.create_context_snapshot(context);
        
        // Create checkpoint
        let checkpoint = Checkpoint {
            id: checkpoint_id.clone(),
            execution_id: context.execution_id.to_string(),
            checkpoint_type,
            state_snapshot,
            context_snapshot,
            graph_metadata: graph.metadata.clone(),
            created_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        // Store checkpoint
        self.storage.store_checkpoint(checkpoint).await?;

        // Update active checkpoints
        let checkpoint_metadata = CheckpointMetadata {
            id: checkpoint_id.clone(),
            execution_id: context.execution_id.to_string(),
            checkpoint_type,
            created_at: chrono::Utc::now(),
            size_bytes: 0, // TODO: Calculate actual size
        };

        self.active_checkpoints.write().await.insert(checkpoint_id.clone(), checkpoint_metadata);

        // Cleanup old checkpoints if needed
        if self.config.auto_cleanup {
            self.cleanup_old_checkpoints().await?;
        }

        info!("Checkpoint created successfully: {}", checkpoint_id);
        Ok(checkpoint_id)
    }

    /// Restore graph state from a checkpoint
    pub async fn restore_checkpoint(
        &self,
        graph: &Graph<S>,
        checkpoint_id: &str,
    ) -> CoreResult<ExecutionContext> {
        info!("Restoring from checkpoint: {}", checkpoint_id);

        // Load checkpoint
        let checkpoint = self.storage.load_checkpoint(checkpoint_id).await?
            .ok_or_else(|| CoreError::validation_error(format!("Checkpoint '{}' not found", checkpoint_id)))?;

        // Restore state
        graph.state_manager.update_state(checkpoint.state_snapshot.into_state())?;

        // Restore execution context
        let restored_context = self.restore_context_from_snapshot(&checkpoint.context_snapshot)?;

        info!("Checkpoint restored successfully: {}", checkpoint_id);
        Ok(restored_context)
    }

    /// List available checkpoints
    pub async fn list_checkpoints(&self) -> CoreResult<Vec<CheckpointMetadata>> {
        let checkpoints = self.active_checkpoints.read().await;
        Ok(checkpoints.values().cloned().collect())
    }

    /// List checkpoints for a specific execution
    pub async fn list_checkpoints_for_execution(&self, execution_id: &str) -> CoreResult<Vec<CheckpointMetadata>> {
        let checkpoints = self.active_checkpoints.read().await;
        Ok(checkpoints
            .values()
            .filter(|cp| cp.execution_id == execution_id)
            .cloned()
            .collect())
    }

    /// Delete a checkpoint
    pub async fn delete_checkpoint(&self, checkpoint_id: &str) -> CoreResult<bool> {
        info!("Deleting checkpoint: {}", checkpoint_id);

        let deleted = self.storage.delete_checkpoint(checkpoint_id).await?;
        
        if deleted {
            self.active_checkpoints.write().await.remove(checkpoint_id);
            info!("Checkpoint deleted: {}", checkpoint_id);
        }

        Ok(deleted)
    }

    /// Create automatic checkpoint based on configuration
    pub async fn auto_checkpoint(
        &self,
        graph: &Graph<S>,
        context: &ExecutionContext,
        trigger: AutoCheckpointTrigger,
    ) -> CoreResult<Option<String>> {
        if !self.should_create_auto_checkpoint(&trigger) {
            return Ok(None);
        }

        debug!("Creating automatic checkpoint (trigger: {:?})", trigger);
        
        let checkpoint_id = self.create_checkpoint(
            graph,
            context,
            CheckpointType::Automatic,
        ).await?;

        Ok(Some(checkpoint_id))
    }

    /// Cleanup old checkpoints based on retention policy
    pub async fn cleanup_old_checkpoints(&self) -> CoreResult<usize> {
        debug!("Cleaning up old checkpoints");

        let mut removed_count = 0;
        let mut checkpoints = self.active_checkpoints.write().await;
        let cutoff_time = chrono::Utc::now() - chrono::Duration::hours(self.config.retention_hours as i64);

        let to_remove: Vec<_> = checkpoints
            .iter()
            .filter(|(_, metadata)| {
                metadata.created_at < cutoff_time && 
                metadata.checkpoint_type == CheckpointType::Automatic
            })
            .map(|(id, _)| id.clone())
            .collect();

        for checkpoint_id in to_remove {
            if self.storage.delete_checkpoint(&checkpoint_id).await? {
                checkpoints.remove(&checkpoint_id);
                removed_count += 1;
            }
        }

        if removed_count > 0 {
            info!("Cleaned up {} old checkpoints", removed_count);
        }

        Ok(removed_count)
    }

    /// Get checkpoint statistics
    pub async fn get_statistics(&self) -> CheckpointStatistics {
        let checkpoints = self.active_checkpoints.read().await;
        
        let mut stats = CheckpointStatistics {
            total_checkpoints: checkpoints.len(),
            automatic_checkpoints: 0,
            manual_checkpoints: 0,
            total_size_bytes: 0,
            oldest_checkpoint: None,
            newest_checkpoint: None,
        };

        for metadata in checkpoints.values() {
            match metadata.checkpoint_type {
                CheckpointType::Automatic => stats.automatic_checkpoints += 1,
                CheckpointType::Manual => stats.manual_checkpoints += 1,
            }
            
            stats.total_size_bytes += metadata.size_bytes;
            
            if stats.oldest_checkpoint.is_none() || metadata.created_at < stats.oldest_checkpoint.unwrap() {
                stats.oldest_checkpoint = Some(metadata.created_at);
            }
            
            if stats.newest_checkpoint.is_none() || metadata.created_at > stats.newest_checkpoint.unwrap() {
                stats.newest_checkpoint = Some(metadata.created_at);
            }
        }

        stats
    }

    /// Create execution context snapshot
    fn create_context_snapshot(&self, context: &ExecutionContext) -> ExecutionContextSnapshot {
        ExecutionContextSnapshot {
            execution_id: context.execution_id,
            config: context.config.clone(),
            start_time: context.start_time,
            depth: context.depth,
            metadata: context.metadata.read().clone(),
            metrics: context.get_metrics(),
        }
    }

    /// Restore execution context from snapshot
    fn restore_context_from_snapshot(&self, snapshot: &ExecutionContextSnapshot) -> CoreResult<ExecutionContext> {
        let mut context = ExecutionContext::new(snapshot.config.clone());
        context.execution_id = snapshot.execution_id;
        context.start_time = snapshot.start_time;
        context.depth = snapshot.depth;
        
        // Restore metadata
        {
            let mut metadata = context.metadata.write();
            *metadata = snapshot.metadata.clone();
        }
        
        // Restore metrics
        context.update_metrics(|metrics| {
            *metrics = snapshot.metrics.clone();
        });

        Ok(context)
    }

    /// Check if automatic checkpoint should be created
    fn should_create_auto_checkpoint(&self, trigger: &AutoCheckpointTrigger) -> bool {
        match trigger {
            AutoCheckpointTrigger::NodeCount(count) => {
                *count % self.config.auto_checkpoint_interval == 0
            }
            AutoCheckpointTrigger::TimeInterval => true,
            AutoCheckpointTrigger::BeforeRiskyOperation => true,
            AutoCheckpointTrigger::OnError => self.config.checkpoint_on_error,
        }
    }
}

/// Checkpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointConfig {
    /// Enable automatic checkpointing
    pub auto_checkpoint: bool,
    /// Interval for automatic checkpoints (in nodes executed)
    pub auto_checkpoint_interval: usize,
    /// Checkpoint on errors
    pub checkpoint_on_error: bool,
    /// Maximum number of checkpoints to keep
    pub max_checkpoints: usize,
    /// Retention time in hours
    pub retention_hours: u64,
    /// Enable automatic cleanup
    pub auto_cleanup: bool,
    /// Compression level (0-9)
    pub compression_level: u8,
}

impl Default for CheckpointConfig {
    fn default() -> Self {
        Self {
            auto_checkpoint: true,
            auto_checkpoint_interval: 10,
            checkpoint_on_error: true,
            max_checkpoints: 50,
            retention_hours: 24,
            auto_cleanup: true,
            compression_level: 6,
        }
    }
}

/// Checkpoint data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint<S>
where
    S: State,
{
    /// Unique checkpoint identifier
    pub id: String,
    /// Associated execution ID
    pub execution_id: String,
    /// Type of checkpoint
    pub checkpoint_type: CheckpointType,
    /// State snapshot
    pub state_snapshot: StateSnapshot<S>,
    /// Execution context snapshot
    pub context_snapshot: ExecutionContextSnapshot,
    /// Graph metadata at checkpoint time
    pub graph_metadata: agent_graph_core::GraphMetadata,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Execution context snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContextSnapshot {
    /// Execution ID
    pub execution_id: uuid::Uuid,
    /// Execution configuration
    pub config: agent_graph_core::ExecutionConfig,
    /// Start time
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// Execution depth
    pub depth: usize,
    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Execution metrics
    pub metrics: agent_graph_core::runtime::ExecutionMetrics,
}

/// Checkpoint metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointMetadata {
    /// Checkpoint ID
    pub id: String,
    /// Associated execution ID
    pub execution_id: String,
    /// Checkpoint type
    pub checkpoint_type: CheckpointType,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Size in bytes
    pub size_bytes: u64,
}

/// Type of checkpoint
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CheckpointType {
    /// Manually created checkpoint
    Manual,
    /// Automatically created checkpoint
    Automatic,
}

/// Triggers for automatic checkpointing
#[derive(Debug, Clone)]
pub enum AutoCheckpointTrigger {
    /// After executing N nodes
    NodeCount(usize),
    /// Time-based interval
    TimeInterval,
    /// Before risky operations
    BeforeRiskyOperation,
    /// On error occurrence
    OnError,
}

/// Checkpoint statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointStatistics {
    /// Total number of checkpoints
    pub total_checkpoints: usize,
    /// Number of automatic checkpoints
    pub automatic_checkpoints: usize,
    /// Number of manual checkpoints
    pub manual_checkpoints: usize,
    /// Total size in bytes
    pub total_size_bytes: u64,
    /// Oldest checkpoint timestamp
    pub oldest_checkpoint: Option<chrono::DateTime<chrono::Utc>>,
    /// Newest checkpoint timestamp
    pub newest_checkpoint: Option<chrono::DateTime<chrono::Utc>>,
}

/// Trait for checkpoint storage backends
#[async_trait::async_trait]
pub trait CheckpointStorage<S>: Send + Sync + std::fmt::Debug
where
    S: State,
{
    /// Store a checkpoint
    async fn store_checkpoint(&self, checkpoint: Checkpoint<S>) -> CoreResult<()>;
    
    /// Load a checkpoint by ID
    async fn load_checkpoint(&self, checkpoint_id: &str) -> CoreResult<Option<Checkpoint<S>>>;
    
    /// Delete a checkpoint by ID
    async fn delete_checkpoint(&self, checkpoint_id: &str) -> CoreResult<bool>;
    
    /// List all checkpoint IDs
    async fn list_checkpoint_ids(&self) -> CoreResult<Vec<String>>;
    
    /// Get storage statistics
    async fn get_storage_stats(&self) -> CoreResult<StorageStatistics>;
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStatistics {
    /// Total number of stored checkpoints
    pub total_checkpoints: usize,
    /// Total storage size in bytes
    pub total_size_bytes: u64,
    /// Average checkpoint size
    pub average_size_bytes: u64,
    /// Storage backend type
    pub backend_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_graph_core::{GraphBuilder, ExecutionConfig};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestState {
        value: i32,
    }

    #[tokio::test]
    async fn test_checkpoint_config_default() {
        let config = CheckpointConfig::default();
        assert!(config.auto_checkpoint);
        assert_eq!(config.auto_checkpoint_interval, 10);
        assert!(config.checkpoint_on_error);
    }

    #[test]
    fn test_checkpoint_metadata_creation() {
        let metadata = CheckpointMetadata {
            id: "test-checkpoint".to_string(),
            execution_id: "test-execution".to_string(),
            checkpoint_type: CheckpointType::Manual,
            created_at: chrono::Utc::now(),
            size_bytes: 1024,
        };

        assert_eq!(metadata.id, "test-checkpoint");
        assert_eq!(metadata.checkpoint_type, CheckpointType::Manual);
        assert_eq!(metadata.size_bytes, 1024);
    }

    #[test]
    fn test_auto_checkpoint_trigger() {
        let trigger = AutoCheckpointTrigger::NodeCount(10);
        
        match trigger {
            AutoCheckpointTrigger::NodeCount(count) => assert_eq!(count, 10),
            _ => panic!("Wrong trigger type"),
        }
    }
}