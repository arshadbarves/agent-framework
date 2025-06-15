// Checkpoint management for AgentGraph
// Provides state persistence and recovery capabilities for long-running executions

#![allow(missing_docs)]

use super::{ExecutionConfig, ExecutionContext, NodeExecution};
use crate::state::State;
use crate::node::NodeId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use thiserror::Error;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Checkpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointConfig {
    /// Enable checkpointing
    pub enabled: bool,
    /// Checkpoint directory
    pub checkpoint_dir: PathBuf,
    /// Checkpoint interval
    pub interval: Duration,
    /// Maximum number of checkpoints to keep
    pub max_checkpoints: usize,
    /// Compression enabled
    pub compression_enabled: bool,
    /// Encryption enabled
    pub encryption_enabled: bool,
    /// Checkpoint format
    pub format: CheckpointFormat,
}

impl Default for CheckpointConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            checkpoint_dir: PathBuf::from("./checkpoints"),
            interval: Duration::from_secs(60), // 1 minute
            max_checkpoints: 10,
            compression_enabled: false,
            encryption_enabled: false,
            format: CheckpointFormat::Json,
        }
    }
}

/// Checkpoint format
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckpointFormat {
    /// JSON format
    Json,
    /// Binary format
    Binary,
    /// MessagePack format
    MessagePack,
}

/// Checkpoint data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Checkpoint ID
    pub id: String,
    /// Execution ID
    pub execution_id: String,
    /// Checkpoint timestamp
    pub timestamp: SystemTime,
    /// Execution context at checkpoint
    pub execution_context: ExecutionContext,
    /// Current state
    pub current_state: State,
    /// Completed node executions
    pub completed_nodes: HashMap<NodeId, NodeExecution>,
    /// Failed node executions
    pub failed_nodes: HashMap<NodeId, String>,
    /// Checkpoint metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Checkpoint version
    pub version: u32,
}

impl Checkpoint {
    /// Create a new checkpoint
    pub fn new(
        execution_id: String,
        context: ExecutionContext,
        state: State,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            execution_id,
            timestamp: SystemTime::now(),
            execution_context: context,
            current_state: state,
            completed_nodes: HashMap::new(),
            failed_nodes: HashMap::new(),
            metadata: HashMap::new(),
            version: 1,
        }
    }
    
    /// Add completed node execution
    pub fn add_completed_node(&mut self, node_id: NodeId, execution: NodeExecution) {
        self.completed_nodes.insert(node_id, execution);
    }
    
    /// Add failed node execution
    pub fn add_failed_node(&mut self, node_id: NodeId, error: String) {
        self.failed_nodes.insert(node_id, error);
    }
    
    /// Add metadata
    pub fn add_metadata<T: Serialize>(&mut self, key: String, value: T) {
        self.metadata.insert(
            key,
            serde_json::to_value(value).unwrap_or(serde_json::Value::Null),
        );
    }
    
    /// Get checkpoint age
    pub fn age(&self) -> Duration {
        SystemTime::now()
            .duration_since(self.timestamp)
            .unwrap_or(Duration::ZERO)
    }
    
    /// Check if checkpoint is valid
    pub fn is_valid(&self) -> bool {
        !self.execution_id.is_empty() && self.version > 0
    }
}

/// Checkpoint manager
#[derive(Debug)]
pub struct CheckpointManager {
    /// Configuration
    config: CheckpointConfig,
    /// Active checkpoints
    active_checkpoints: HashMap<String, Vec<Checkpoint>>,
}

impl CheckpointManager {
    /// Create a new checkpoint manager
    pub fn new(execution_config: ExecutionConfig) -> Self {
        let config = CheckpointConfig {
            enabled: execution_config.checkpointing_enabled,
            interval: execution_config.checkpoint_interval,
            ..Default::default()
        };
        
        Self {
            config,
            active_checkpoints: HashMap::new(),
        }
    }
    
    /// Create a checkpoint
    pub async fn create_checkpoint(
        &mut self,
        execution_id: String,
        context: ExecutionContext,
        state: State,
    ) -> Result<String, CheckpointError> {
        if !self.config.enabled {
            return Err(CheckpointError::CheckpointingDisabled);
        }
        
        let checkpoint = Checkpoint::new(execution_id.clone(), context, state);
        let checkpoint_id = checkpoint.id.clone();
        
        // Save checkpoint to disk
        self.save_checkpoint(&checkpoint).await?;
        
        // Add to active checkpoints
        self.active_checkpoints
            .entry(execution_id)
            .or_insert_with(Vec::new)
            .push(checkpoint);
        
        // Clean up old checkpoints
        self.cleanup_old_checkpoints(&execution_id).await?;
        
        Ok(checkpoint_id)
    }
    
    /// Save checkpoint to disk
    async fn save_checkpoint(&self, checkpoint: &Checkpoint) -> Result<(), CheckpointError> {
        // Ensure checkpoint directory exists
        fs::create_dir_all(&self.config.checkpoint_dir).await
            .map_err(|e| CheckpointError::IoError {
                operation: "create_dir".to_string(),
                error: e.to_string(),
            })?;
        
        // Generate filename
        let filename = format!("{}_{}.checkpoint", checkpoint.execution_id, checkpoint.id);
        let filepath = self.config.checkpoint_dir.join(filename);
        
        // Serialize checkpoint
        let data = match self.config.format {
            CheckpointFormat::Json => {
                serde_json::to_vec_pretty(checkpoint)
                    .map_err(|e| CheckpointError::SerializationError {
                        format: "json".to_string(),
                        error: e.to_string(),
                    })?
            }
            CheckpointFormat::Binary => {
                bincode::serialize(checkpoint)
                    .map_err(|e| CheckpointError::SerializationError {
                        format: "binary".to_string(),
                        error: e.to_string(),
                    })?
            }
            CheckpointFormat::MessagePack => {
                rmp_serde::to_vec(checkpoint)
                    .map_err(|e| CheckpointError::SerializationError {
                        format: "messagepack".to_string(),
                        error: e.to_string(),
                    })?
            }
        };
        
        // Apply compression if enabled
        let final_data = if self.config.compression_enabled {
            self.compress_data(&data)?
        } else {
            data
        };
        
        // Apply encryption if enabled
        let encrypted_data = if self.config.encryption_enabled {
            self.encrypt_data(&final_data)?
        } else {
            final_data
        };
        
        // Write to file
        fs::write(&filepath, encrypted_data).await
            .map_err(|e| CheckpointError::IoError {
                operation: "write".to_string(),
                error: e.to_string(),
            })?;
        
        Ok(())
    }
    
    /// Load checkpoint from disk
    pub async fn load_checkpoint(&self, checkpoint_id: &str) -> Result<Checkpoint, CheckpointError> {
        // Find checkpoint file
        let mut checkpoint_file = None;
        let mut entries = fs::read_dir(&self.config.checkpoint_dir).await
            .map_err(|e| CheckpointError::IoError {
                operation: "read_dir".to_string(),
                error: e.to_string(),
            })?;
        
        while let Some(entry) = entries.next_entry().await
            .map_err(|e| CheckpointError::IoError {
                operation: "next_entry".to_string(),
                error: e.to_string(),
            })? {
            
            let filename = entry.file_name();
            let filename_str = filename.to_string_lossy();
            
            if filename_str.contains(checkpoint_id) && filename_str.ends_with(".checkpoint") {
                checkpoint_file = Some(entry.path());
                break;
            }
        }
        
        let filepath = checkpoint_file.ok_or_else(|| CheckpointError::CheckpointNotFound {
            checkpoint_id: checkpoint_id.to_string(),
        })?;
        
        // Read file
        let encrypted_data = fs::read(&filepath).await
            .map_err(|e| CheckpointError::IoError {
                operation: "read".to_string(),
                error: e.to_string(),
            })?;
        
        // Decrypt if needed
        let compressed_data = if self.config.encryption_enabled {
            self.decrypt_data(&encrypted_data)?
        } else {
            encrypted_data
        };
        
        // Decompress if needed
        let data = if self.config.compression_enabled {
            self.decompress_data(&compressed_data)?
        } else {
            compressed_data
        };
        
        // Deserialize checkpoint
        let checkpoint = match self.config.format {
            CheckpointFormat::Json => {
                serde_json::from_slice(&data)
                    .map_err(|e| CheckpointError::DeserializationError {
                        format: "json".to_string(),
                        error: e.to_string(),
                    })?
            }
            CheckpointFormat::Binary => {
                bincode::deserialize(&data)
                    .map_err(|e| CheckpointError::DeserializationError {
                        format: "binary".to_string(),
                        error: e.to_string(),
                    })?
            }
            CheckpointFormat::MessagePack => {
                rmp_serde::from_slice(&data)
                    .map_err(|e| CheckpointError::DeserializationError {
                        format: "messagepack".to_string(),
                        error: e.to_string(),
                    })?
            }
        };
        
        Ok(checkpoint)
    }
    
    /// Restore execution from checkpoint
    pub async fn restore_execution(&self, checkpoint_id: &str) -> Result<(ExecutionContext, State), CheckpointError> {
        let checkpoint = self.load_checkpoint(checkpoint_id).await?;
        
        if !checkpoint.is_valid() {
            return Err(CheckpointError::InvalidCheckpoint {
                checkpoint_id: checkpoint_id.to_string(),
                reason: "Checkpoint validation failed".to_string(),
            });
        }
        
        Ok((checkpoint.execution_context, checkpoint.current_state))
    }
    
    /// List checkpoints for execution
    pub async fn list_checkpoints(&self, execution_id: &str) -> Result<Vec<String>, CheckpointError> {
        let mut checkpoint_ids = Vec::new();
        
        if let Some(checkpoints) = self.active_checkpoints.get(execution_id) {
            for checkpoint in checkpoints {
                checkpoint_ids.push(checkpoint.id.clone());
            }
        }
        
        Ok(checkpoint_ids)
    }
    
    /// Delete checkpoint
    pub async fn delete_checkpoint(&mut self, checkpoint_id: &str) -> Result<(), CheckpointError> {
        // Remove from active checkpoints
        for (_, checkpoints) in self.active_checkpoints.iter_mut() {
            checkpoints.retain(|c| c.id != checkpoint_id);
        }
        
        // Delete file
        let mut entries = fs::read_dir(&self.config.checkpoint_dir).await
            .map_err(|e| CheckpointError::IoError {
                operation: "read_dir".to_string(),
                error: e.to_string(),
            })?;
        
        while let Some(entry) = entries.next_entry().await
            .map_err(|e| CheckpointError::IoError {
                operation: "next_entry".to_string(),
                error: e.to_string(),
            })? {
            
            let filename = entry.file_name();
            let filename_str = filename.to_string_lossy();
            
            if filename_str.contains(checkpoint_id) && filename_str.ends_with(".checkpoint") {
                fs::remove_file(entry.path()).await
                    .map_err(|e| CheckpointError::IoError {
                        operation: "remove_file".to_string(),
                        error: e.to_string(),
                    })?;
                break;
            }
        }
        
        Ok(())
    }
    
    /// Clean up old checkpoints
    async fn cleanup_old_checkpoints(&mut self, execution_id: &str) -> Result<(), CheckpointError> {
        if let Some(checkpoints) = self.active_checkpoints.get_mut(execution_id) {
            // Sort by timestamp (newest first)
            checkpoints.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            
            // Remove excess checkpoints
            while checkpoints.len() > self.config.max_checkpoints {
                if let Some(old_checkpoint) = checkpoints.pop() {
                    self.delete_checkpoint(&old_checkpoint.id).await?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Compress data (placeholder implementation)
    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>, CheckpointError> {
        // For now, just return the data as-is
        // In a real implementation, you would use a compression library like flate2
        Ok(data.to_vec())
    }
    
    /// Decompress data (placeholder implementation)
    fn decompress_data(&self, data: &[u8]) -> Result<Vec<u8>, CheckpointError> {
        // For now, just return the data as-is
        // In a real implementation, you would use a compression library like flate2
        Ok(data.to_vec())
    }
    
    /// Encrypt data (placeholder implementation)
    fn encrypt_data(&self, data: &[u8]) -> Result<Vec<u8>, CheckpointError> {
        // For now, just return the data as-is
        // In a real implementation, you would use an encryption library
        Ok(data.to_vec())
    }
    
    /// Decrypt data (placeholder implementation)
    fn decrypt_data(&self, data: &[u8]) -> Result<Vec<u8>, CheckpointError> {
        // For now, just return the data as-is
        // In a real implementation, you would use an encryption library
        Ok(data.to_vec())
    }
    
    /// Get checkpoint statistics
    pub fn get_stats(&self) -> CheckpointStats {
        let total_checkpoints = self.active_checkpoints.values()
            .map(|checkpoints| checkpoints.len())
            .sum();
        
        let total_executions = self.active_checkpoints.len();
        
        CheckpointStats {
            total_checkpoints,
            total_executions,
            checkpoint_directory: self.config.checkpoint_dir.clone(),
            compression_enabled: self.config.compression_enabled,
            encryption_enabled: self.config.encryption_enabled,
        }
    }
    
    /// Get configuration
    pub fn config(&self) -> &CheckpointConfig {
        &self.config
    }
}

/// Checkpoint statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointStats {
    /// Total number of checkpoints
    pub total_checkpoints: usize,
    /// Total number of executions with checkpoints
    pub total_executions: usize,
    /// Checkpoint directory
    pub checkpoint_directory: PathBuf,
    /// Compression enabled
    pub compression_enabled: bool,
    /// Encryption enabled
    pub encryption_enabled: bool,
}

/// Checkpoint errors
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum CheckpointError {
    /// Checkpointing is disabled
    #[error("Checkpointing is disabled")]
    CheckpointingDisabled,
    
    /// Checkpoint not found
    #[error("Checkpoint not found: {checkpoint_id}")]
    CheckpointNotFound { checkpoint_id: String },
    
    /// Invalid checkpoint
    #[error("Invalid checkpoint {checkpoint_id}: {reason}")]
    InvalidCheckpoint { checkpoint_id: String, reason: String },
    
    /// IO error
    #[error("IO error during {operation}: {error}")]
    IoError { operation: String, error: String },
    
    /// Serialization error
    #[error("Serialization error ({format}): {error}")]
    SerializationError { format: String, error: String },
    
    /// Deserialization error
    #[error("Deserialization error ({format}): {error}")]
    DeserializationError { format: String, error: String },
    
    /// Compression error
    #[error("Compression error: {error}")]
    CompressionError { error: String },
    
    /// Encryption error
    #[error("Encryption error: {error}")]
    EncryptionError { error: String },
    
    /// Configuration error
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkpoint_config_default() {
        let config = CheckpointConfig::default();
        assert!(config.enabled);
        assert_eq!(config.max_checkpoints, 10);
        assert_eq!(config.format, CheckpointFormat::Json);
    }

    #[test]
    fn test_checkpoint_creation() {
        let execution_id = "test_execution".to_string();
        let context = ExecutionContext::new(ExecutionConfig::default(), State::new());
        let state = State::new();
        
        let checkpoint = Checkpoint::new(execution_id.clone(), context, state);
        assert_eq!(checkpoint.execution_id, execution_id);
        assert!(checkpoint.is_valid());
        assert!(checkpoint.completed_nodes.is_empty());
    }

    #[test]
    fn test_checkpoint_metadata() {
        let execution_id = "test_execution".to_string();
        let context = ExecutionContext::new(ExecutionConfig::default(), State::new());
        let state = State::new();
        
        let mut checkpoint = Checkpoint::new(execution_id, context, state);
        checkpoint.add_metadata("test_key".to_string(), "test_value");
        
        assert!(checkpoint.metadata.contains_key("test_key"));
    }

    #[test]
    fn test_checkpoint_manager_creation() {
        let config = ExecutionConfig::default();
        let manager = CheckpointManager::new(config);
        
        assert_eq!(manager.config.enabled, true);
        assert!(manager.active_checkpoints.is_empty());
    }

    #[test]
    fn test_checkpoint_stats() {
        let config = ExecutionConfig::default();
        let manager = CheckpointManager::new(config);
        
        let stats = manager.get_stats();
        assert_eq!(stats.total_checkpoints, 0);
        assert_eq!(stats.total_executions, 0);
    }
}
