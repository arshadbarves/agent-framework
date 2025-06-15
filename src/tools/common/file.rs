// File system tools for reading, writing, and managing files

use crate::tools::traits::{Tool, ToolError, ToolInput, ToolMetadata, ToolOutput, ToolResult};
use async_trait::async_trait;
use serde_json::json;
use std::path::Path;
use tokio::fs;

/// Tool for reading files
#[derive(Debug)]
pub struct FileReadTool {
    metadata: ToolMetadata,
}

impl FileReadTool {
    /// Create a new file read tool
    pub fn new() -> Self {
        let metadata = ToolMetadata::new(
            "file_read",
            "File Reader",
            "Read contents from files on the filesystem"
        )
        .with_tag("file")
        .with_tag("io")
        .with_tag("utility")
        .with_deterministic(true)
        .with_side_effects(false)
        .with_estimated_duration_ms(100);
        
        Self { metadata }
    }
}

#[async_trait]
impl Tool for FileReadTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(&self, input: ToolInput) -> ToolResult<ToolOutput> {
        let path = input.data.as_str()
            .ok_or_else(|| ToolError::ValidationError {
                message: "File path is required".to_string(),
            })?;

        let contents = fs::read_to_string(path).await
            .map_err(|e| ToolError::IoError {
                message: format!("Failed to read file '{}': {}", path, e),
            })?;

        let output = ToolOutput::new(json!({
            "path": path,
            "contents": contents,
            "size": contents.len()
        }))
        .with_metadata("file_path", path)
        .with_metric("file_size_bytes", contents.len() as f64);

        Ok(output)
    }

    async fn validate_input(&self, input: &ToolInput) -> ToolResult<()> {
        let path = input.data.as_str()
            .ok_or_else(|| ToolError::ValidationError {
                message: "File path is required".to_string(),
            })?;

        if !Path::new(path).exists() {
            return Err(ToolError::ValidationError {
                message: format!("File '{}' does not exist", path),
            });
        }

        Ok(())
    }
}

/// Tool for writing files
#[derive(Debug)]
pub struct FileWriteTool {
    metadata: ToolMetadata,
}

impl FileWriteTool {
    /// Create a new file write tool
    pub fn new() -> Self {
        let metadata = ToolMetadata::new(
            "file_write",
            "File Writer",
            "Write contents to files on the filesystem"
        )
        .with_tag("file")
        .with_tag("io")
        .with_tag("utility")
        .with_deterministic(true)
        .with_side_effects(true)
        .with_estimated_duration_ms(200);
        
        Self { metadata }
    }
}

#[async_trait]
impl Tool for FileWriteTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(&self, input: ToolInput) -> ToolResult<ToolOutput> {
        let path = input.get_parameter::<String>("path")
            .ok_or_else(|| ToolError::ValidationError {
                message: "File path parameter is required".to_string(),
            })?;

        let contents = input.data.as_str()
            .ok_or_else(|| ToolError::ValidationError {
                message: "File contents are required in data field".to_string(),
            })?;

        fs::write(&path, contents).await
            .map_err(|e| ToolError::IoError {
                message: format!("Failed to write file '{}': {}", path, e),
            })?;

        let output = ToolOutput::new(json!({
            "path": path,
            "bytes_written": contents.len(),
            "success": true
        }))
        .with_metadata("file_path", &path)
        .with_metric("bytes_written", contents.len() as f64);

        Ok(output)
    }

    async fn validate_input(&self, input: &ToolInput) -> ToolResult<()> {
        let path = input.get_parameter::<String>("path")
            .ok_or_else(|| ToolError::ValidationError {
                message: "File path parameter is required".to_string(),
            })?;

        if input.data.as_str().is_none() {
            return Err(ToolError::ValidationError {
                message: "File contents are required in data field".to_string(),
            });
        }

        // Check if parent directory exists
        if let Some(parent) = Path::new(&path).parent() {
            if !parent.exists() {
                return Err(ToolError::ValidationError {
                    message: format!("Parent directory '{}' does not exist", parent.display()),
                });
            }
        }

        Ok(())
    }
}

/// Tool for listing directory contents
#[derive(Debug)]
pub struct DirectoryListTool {
    metadata: ToolMetadata,
}

impl DirectoryListTool {
    /// Create a new directory list tool
    pub fn new() -> Self {
        let metadata = ToolMetadata::new(
            "directory_list",
            "Directory Lister",
            "List contents of directories on the filesystem"
        )
        .with_tag("file")
        .with_tag("io")
        .with_tag("utility")
        .with_deterministic(true)
        .with_side_effects(false)
        .with_estimated_duration_ms(50);
        
        Self { metadata }
    }
}

#[async_trait]
impl Tool for DirectoryListTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(&self, input: ToolInput) -> ToolResult<ToolOutput> {
        let path = input.data.as_str()
            .ok_or_else(|| ToolError::ValidationError {
                message: "Directory path is required".to_string(),
            })?;

        let mut entries = fs::read_dir(path).await
            .map_err(|e| ToolError::IoError {
                message: format!("Failed to read directory '{}': {}", path, e),
            })?;

        let mut files = Vec::new();
        let mut directories = Vec::new();

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| ToolError::IoError {
                message: format!("Failed to read directory entry: {}", e),
            })? {
            
            let file_name = entry.file_name().to_string_lossy().to_string();
            let metadata = entry.metadata().await
                .map_err(|e| ToolError::IoError {
                    message: format!("Failed to read metadata for '{}': {}", file_name, e),
                })?;

            if metadata.is_dir() {
                directories.push(file_name);
            } else {
                files.push(json!({
                    "name": file_name,
                    "size": metadata.len(),
                    "modified": metadata.modified()
                        .ok()
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs())
                }));
            }
        }

        let output = ToolOutput::new(json!({
            "path": path,
            "files": files,
            "directories": directories,
            "total_files": files.len(),
            "total_directories": directories.len()
        }))
        .with_metadata("directory_path", path)
        .with_metric("file_count", files.len() as f64)
        .with_metric("directory_count", directories.len() as f64);

        Ok(output)
    }

    async fn validate_input(&self, input: &ToolInput) -> ToolResult<()> {
        let path = input.data.as_str()
            .ok_or_else(|| ToolError::ValidationError {
                message: "Directory path is required".to_string(),
            })?;

        let path_obj = Path::new(path);
        if !path_obj.exists() {
            return Err(ToolError::ValidationError {
                message: format!("Directory '{}' does not exist", path),
            });
        }

        if !path_obj.is_dir() {
            return Err(ToolError::ValidationError {
                message: format!("Path '{}' is not a directory", path),
            });
        }

        Ok(())
    }
}
