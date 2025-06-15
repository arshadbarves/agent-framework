//! State checkpointing functionality for persistence and recovery.

use crate::error::{GraphError, GraphResult};
use crate::state::{StateSnapshot, SnapshotMetadata};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;
use uuid::Uuid;

/// Trait for implementing state checkpointing backends
#[async_trait]
pub trait Checkpointer<S>: Send + Sync
where
    S: Serialize + for<'de> Deserialize<'de> + Send + Sync,
{
    /// Save a state snapshot
    async fn save(&self, snapshot: &StateSnapshot<S>) -> GraphResult<()>;

    /// Load a state snapshot by ID
    async fn load(&self, snapshot_id: Uuid) -> GraphResult<StateSnapshot<S>>;

    /// List all available snapshots
    async fn list_snapshots(&self) -> GraphResult<Vec<Uuid>>;

    /// Delete a snapshot
    async fn delete(&self, snapshot_id: Uuid) -> GraphResult<()>;

    /// Check if a snapshot exists
    async fn exists(&self, snapshot_id: Uuid) -> GraphResult<bool>;

    /// Get metadata for a snapshot without loading the full state
    async fn get_metadata(&self, snapshot_id: Uuid) -> GraphResult<SnapshotMetadata>;
}

/// File-based checkpointer implementation
#[derive(Debug, Clone)]
pub struct FileCheckpointer {
    /// Directory to store checkpoint files
    checkpoint_dir: PathBuf,
    /// Whether to compress checkpoint files
    compress: bool,
}

impl FileCheckpointer {
    /// Create a new file checkpointer
    pub fn new<P: AsRef<Path>>(checkpoint_dir: P) -> Self {
        Self {
            checkpoint_dir: checkpoint_dir.as_ref().to_path_buf(),
            compress: false,
        }
    }

    /// Create a new file checkpointer with compression
    pub fn with_compression<P: AsRef<Path>>(checkpoint_dir: P) -> Self {
        Self {
            checkpoint_dir: checkpoint_dir.as_ref().to_path_buf(),
            compress: true,
        }
    }

    /// Get the file path for a snapshot
    fn snapshot_path(&self, snapshot_id: Uuid) -> PathBuf {
        let filename = if self.compress {
            format!("{}.json.gz", snapshot_id)
        } else {
            format!("{}.json", snapshot_id)
        };
        self.checkpoint_dir.join(filename)
    }

    /// Ensure the checkpoint directory exists
    async fn ensure_directory(&self) -> GraphResult<()> {
        if !self.checkpoint_dir.exists() {
            fs::create_dir_all(&self.checkpoint_dir).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl<S> Checkpointer<S> for FileCheckpointer
where
    S: Serialize + for<'de> Deserialize<'de> + Send + Sync,
{
    async fn save(&self, snapshot: &StateSnapshot<S>) -> GraphResult<()> {
        self.ensure_directory().await?;
        
        let path = self.snapshot_path(snapshot.id);
        let json_data = serde_json::to_string_pretty(snapshot)?;
        
        if self.compress {
            // For now, just save as regular JSON
            // In a production implementation, you might use flate2 or similar
            fs::write(&path, json_data).await?;
        } else {
            fs::write(&path, json_data).await?;
        }
        
        tracing::info!(
            snapshot_id = %snapshot.id,
            path = %path.display(),
            "Saved checkpoint"
        );
        
        Ok(())
    }

    async fn load(&self, snapshot_id: Uuid) -> GraphResult<StateSnapshot<S>> {
        let path = self.snapshot_path(snapshot_id);
        
        if !path.exists() {
            return Err(GraphError::CheckpointError(format!(
                "Checkpoint file not found: {}",
                path.display()
            )));
        }
        
        let json_data = fs::read_to_string(&path).await?;
        let snapshot: StateSnapshot<S> = serde_json::from_str(&json_data)?;
        
        tracing::info!(
            snapshot_id = %snapshot_id,
            path = %path.display(),
            "Loaded checkpoint"
        );
        
        Ok(snapshot)
    }

    async fn list_snapshots(&self) -> GraphResult<Vec<Uuid>> {
        if !self.checkpoint_dir.exists() {
            return Ok(Vec::new());
        }
        
        let mut snapshots = Vec::new();
        let mut entries = fs::read_dir(&self.checkpoint_dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.ends_with(".json") || file_name.ends_with(".json.gz") {
                    let uuid_str = file_name
                        .replace(".json.gz", "")
                        .replace(".json", "");
                    
                    if let Ok(uuid) = Uuid::parse_str(&uuid_str) {
                        snapshots.push(uuid);
                    }
                }
            }
        }
        
        Ok(snapshots)
    }

    async fn delete(&self, snapshot_id: Uuid) -> GraphResult<()> {
        let path = self.snapshot_path(snapshot_id);
        
        if path.exists() {
            fs::remove_file(&path).await?;
            tracing::info!(
                snapshot_id = %snapshot_id,
                path = %path.display(),
                "Deleted checkpoint"
            );
        }
        
        Ok(())
    }

    async fn exists(&self, snapshot_id: Uuid) -> GraphResult<bool> {
        let path = self.snapshot_path(snapshot_id);
        Ok(path.exists())
    }

    async fn get_metadata(&self, snapshot_id: Uuid) -> GraphResult<SnapshotMetadata> {
        // For file-based checkpointer, we need to load the full snapshot
        // In a more sophisticated implementation, metadata might be stored separately
        let snapshot: StateSnapshot<S> = self.load(snapshot_id).await?;
        Ok(snapshot.metadata)
    }
}

/// In-memory checkpointer for testing and development
#[derive(Debug, Default)]
pub struct MemoryCheckpointer<S> {
    snapshots: std::sync::Arc<parking_lot::RwLock<std::collections::HashMap<Uuid, StateSnapshot<S>>>>,
}

impl<S> MemoryCheckpointer<S> {
    /// Create a new memory checkpointer
    pub fn new() -> Self {
        Self {
            snapshots: std::sync::Arc::new(parking_lot::RwLock::new(std::collections::HashMap::new())),
        }
    }
}

#[async_trait]
impl<S> Checkpointer<S> for MemoryCheckpointer<S>
where
    S: Serialize + for<'de> Deserialize<'de> + Send + Sync + Clone,
{
    async fn save(&self, snapshot: &StateSnapshot<S>) -> GraphResult<()> {
        let mut snapshots = self.snapshots.write();
        snapshots.insert(snapshot.id, snapshot.clone());
        
        tracing::debug!(
            snapshot_id = %snapshot.id,
            "Saved checkpoint to memory"
        );
        
        Ok(())
    }

    async fn load(&self, snapshot_id: Uuid) -> GraphResult<StateSnapshot<S>> {
        let snapshots = self.snapshots.read();
        snapshots
            .get(&snapshot_id)
            .cloned()
            .ok_or_else(|| {
                GraphError::CheckpointError(format!(
                    "Snapshot {} not found in memory",
                    snapshot_id
                ))
            })
    }

    async fn list_snapshots(&self) -> GraphResult<Vec<Uuid>> {
        let snapshots = self.snapshots.read();
        Ok(snapshots.keys().copied().collect())
    }

    async fn delete(&self, snapshot_id: Uuid) -> GraphResult<()> {
        let mut snapshots = self.snapshots.write();
        snapshots.remove(&snapshot_id);
        Ok(())
    }

    async fn exists(&self, snapshot_id: Uuid) -> GraphResult<bool> {
        let snapshots = self.snapshots.read();
        Ok(snapshots.contains_key(&snapshot_id))
    }

    async fn get_metadata(&self, snapshot_id: Uuid) -> GraphResult<SnapshotMetadata> {
        let snapshots = self.snapshots.read();
        snapshots
            .get(&snapshot_id)
            .map(|s| s.metadata.clone())
            .ok_or_else(|| {
                GraphError::CheckpointError(format!(
                    "Snapshot {} not found in memory",
                    snapshot_id
                ))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestState {
        value: i32,
    }

    #[tokio::test]
    async fn test_file_checkpointer() {
        let temp_dir = TempDir::new().unwrap();
        let checkpointer = FileCheckpointer::new(temp_dir.path());
        
        let state = TestState { value: 42 };
        let snapshot = StateSnapshot::new(state);
        let snapshot_id = snapshot.id;
        
        // Save snapshot
        checkpointer.save(&snapshot).await.unwrap();
        
        // Check if exists
        assert!(<FileCheckpointer as Checkpointer<TestState>>::exists(&checkpointer, snapshot_id).await.unwrap());

        // Load snapshot
        let loaded: StateSnapshot<TestState> = checkpointer.load(snapshot_id).await.unwrap();
        assert_eq!(loaded.state, snapshot.state);
        
        // List snapshots
        let snapshots = <FileCheckpointer as Checkpointer<TestState>>::list_snapshots(&checkpointer).await.unwrap();
        assert!(snapshots.contains(&snapshot_id));
        
        // Delete snapshot
        <FileCheckpointer as Checkpointer<TestState>>::delete(&checkpointer, snapshot_id).await.unwrap();
        assert!(!<FileCheckpointer as Checkpointer<TestState>>::exists(&checkpointer, snapshot_id).await.unwrap());
    }

    #[tokio::test]
    async fn test_memory_checkpointer() {
        let checkpointer = MemoryCheckpointer::new();
        
        let state = TestState { value: 42 };
        let snapshot = StateSnapshot::new(state);
        let snapshot_id = snapshot.id;
        
        // Save snapshot
        checkpointer.save(&snapshot).await.unwrap();
        
        // Load snapshot
        let loaded = checkpointer.load(snapshot_id).await.unwrap();
        assert_eq!(loaded.state, snapshot.state);
        
        // Check metadata
        let metadata = checkpointer.get_metadata(snapshot_id).await.unwrap();
        assert_eq!(metadata.step, 0);
    }
}
