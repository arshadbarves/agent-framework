//! File system tools for reading, writing, and managing files.

use crate::{CoreError, CoreResult};
use crate::core::{Tool, ToolMetadata, ToolInput, ToolOutput, ToolCategory, ToolError, ToolMetrics};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::fs;

/// File read tool for reading file contents
#[derive(Debug)]
pub struct FileReadTool {
    metadata: ToolMetadata,
}

impl FileReadTool {
    /// Create a new file read tool
    pub fn new() -> Self {
        let metadata = ToolMetadata {
            id: "file_read".to_string(),
            name: "File Reader".to_string(),
            description: "Read contents from files on the filesystem".to_string(),
            version: "1.0.0".to_string(),
            category: ToolCategory::FileSystem,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to read"
                    },
                    "encoding": {
                        "type": "string",
                        "enum": ["utf8", "binary"],
                        "default": "utf8",
                        "description": "File encoding"
                    },
                    "max_size_bytes": {
                        "type": "number",
                        "default": 10485760,
                        "description": "Maximum file size to read (10MB default)"
                    }
                },
                "required": ["path"]
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "File contents"
                    },
                    "size_bytes": {
                        "type": "number",
                        "description": "File size in bytes"
                    },
                    "encoding": {
                        "type": "string",
                        "description": "Encoding used"
                    },
                    "path": {
                        "type": "string",
                        "description": "File path"
                    }
                }
            }),
            parallel_safe: true,
            estimated_duration: Some(Duration::from_millis(100)),
            required_permissions: vec!["filesystem.read".to_string()],
            tags: vec!["file".to_string(), "io".to_string(), "read".to_string()],
            properties: HashMap::new(),
        };

        Self { metadata }
    }

    /// Check if path is safe (prevent directory traversal)
    fn is_safe_path(&self, path: &str) -> bool {
        let path = Path::new(path);
        
        // Prevent absolute paths and directory traversal
        if path.is_absolute() {
            return false;
        }
        
        for component in path.components() {
            if let std::path::Component::ParentDir = component {
                return false;
            }
        }
        
        true
    }
}

impl Default for FileReadTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for FileReadTool {
    async fn execute(&self, input: ToolInput) -> CoreResult<ToolOutput> {
        let start_time = Instant::now();

        // Extract parameters
        let path: String = input.get_param("path")?;
        let encoding: String = input.get_optional_param("encoding")?.unwrap_or_else(|| "utf8".to_string());
        let max_size_bytes: u64 = input.get_optional_param("max_size_bytes")?.unwrap_or(10 * 1024 * 1024);

        // Check permissions
        if !input.context.security_context.permissions.contains(&"filesystem.read".to_string()) {
            return Ok(ToolOutput::failure(ToolError::new(
                "PERMISSION_DENIED".to_string(),
                "File read permission not granted".to_string(),
            )));
        }

        // Validate path safety
        if !self.is_safe_path(&path) {
            return Ok(ToolOutput::failure(ToolError::new(
                "UNSAFE_PATH".to_string(),
                "Path contains unsafe components".to_string(),
            )));
        }

        // Check if file exists
        let file_path = Path::new(&path);
        if !file_path.exists() {
            return Ok(ToolOutput::failure(ToolError::new(
                "FILE_NOT_FOUND".to_string(),
                format!("File not found: {}", path),
            )));
        }

        // Check file size
        let metadata = fs::metadata(&file_path).await.map_err(|e| {
            CoreError::execution_error(format!("Failed to read file metadata: {}", e))
        })?;

        if metadata.len() > max_size_bytes {
            return Ok(ToolOutput::failure(ToolError::new(
                "FILE_TOO_LARGE".to_string(),
                format!("File size {} exceeds maximum {}", metadata.len(), max_size_bytes),
            )));
        }

        // Read file content
        let content = match encoding.as_str() {
            "utf8" => {
                match fs::read_to_string(&file_path).await {
                    Ok(content) => content,
                    Err(e) => {
                        return Ok(ToolOutput::failure(ToolError::new(
                            "READ_ERROR".to_string(),
                            format!("Failed to read file: {}", e),
                        )));
                    }
                }
            }
            "binary" => {
                match fs::read(&file_path).await {
                    Ok(bytes) => base64::encode(bytes),
                    Err(e) => {
                        return Ok(ToolOutput::failure(ToolError::new(
                            "READ_ERROR".to_string(),
                            format!("Failed to read file: {}", e),
                        )));
                    }
                }
            }
            _ => {
                return Ok(ToolOutput::failure(ToolError::new(
                    "INVALID_ENCODING".to_string(),
                    format!("Unsupported encoding: {}", encoding),
                )));
            }
        };

        let execution_time = start_time.elapsed().as_millis() as u64;
        let metrics = ToolMetrics {
            execution_time_ms: execution_time,
            files_accessed: Some(1),
            ..Default::default()
        };

        Ok(ToolOutput::success(Some(serde_json::json!({
            "content": content,
            "size_bytes": metadata.len(),
            "encoding": encoding,
            "path": path
        }))).with_metrics(metrics))
    }

    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }
}

/// File write tool for writing content to files
#[derive(Debug)]
pub struct FileWriteTool {
    metadata: ToolMetadata,
}

impl FileWriteTool {
    /// Create a new file write tool
    pub fn new() -> Self {
        let metadata = ToolMetadata {
            id: "file_write".to_string(),
            name: "File Writer".to_string(),
            description: "Write content to files on the filesystem".to_string(),
            version: "1.0.0".to_string(),
            category: ToolCategory::FileSystem,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to write"
                    },
                    "content": {
                        "type": "string",
                        "description": "Content to write to the file"
                    },
                    "encoding": {
                        "type": "string",
                        "enum": ["utf8", "binary"],
                        "default": "utf8",
                        "description": "Content encoding"
                    },
                    "create_dirs": {
                        "type": "boolean",
                        "default": false,
                        "description": "Create parent directories if they don't exist"
                    },
                    "overwrite": {
                        "type": "boolean",
                        "default": false,
                        "description": "Overwrite file if it exists"
                    }
                },
                "required": ["path", "content"]
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "File path"
                    },
                    "bytes_written": {
                        "type": "number",
                        "description": "Number of bytes written"
                    },
                    "created": {
                        "type": "boolean",
                        "description": "Whether the file was created (vs overwritten)"
                    }
                }
            }),
            parallel_safe: false, // File writes should be serialized
            estimated_duration: Some(Duration::from_millis(200)),
            required_permissions: vec!["filesystem.write".to_string()],
            tags: vec!["file".to_string(), "io".to_string(), "write".to_string()],
            properties: HashMap::new(),
        };

        Self { metadata }
    }

    /// Check if path is safe (prevent directory traversal)
    fn is_safe_path(&self, path: &str) -> bool {
        let path = Path::new(path);
        
        // Prevent absolute paths and directory traversal
        if path.is_absolute() {
            return false;
        }
        
        for component in path.components() {
            if let std::path::Component::ParentDir = component {
                return false;
            }
        }
        
        true
    }
}

impl Default for FileWriteTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for FileWriteTool {
    async fn execute(&self, input: ToolInput) -> CoreResult<ToolOutput> {
        let start_time = Instant::now();

        // Extract parameters
        let path: String = input.get_param("path")?;
        let content: String = input.get_param("content")?;
        let encoding: String = input.get_optional_param("encoding")?.unwrap_or_else(|| "utf8".to_string());
        let create_dirs: bool = input.get_optional_param("create_dirs")?.unwrap_or(false);
        let overwrite: bool = input.get_optional_param("overwrite")?.unwrap_or(false);

        // Check permissions
        if !input.context.security_context.permissions.contains(&"filesystem.write".to_string()) {
            return Ok(ToolOutput::failure(ToolError::new(
                "PERMISSION_DENIED".to_string(),
                "File write permission not granted".to_string(),
            )));
        }

        // Validate path safety
        if !self.is_safe_path(&path) {
            return Ok(ToolOutput::failure(ToolError::new(
                "UNSAFE_PATH".to_string(),
                "Path contains unsafe components".to_string(),
            )));
        }

        let file_path = Path::new(&path);
        let file_exists = file_path.exists();

        // Check if file exists and overwrite is not allowed
        if file_exists && !overwrite {
            return Ok(ToolOutput::failure(ToolError::new(
                "FILE_EXISTS".to_string(),
                format!("File already exists and overwrite is disabled: {}", path),
            )));
        }

        // Create parent directories if requested
        if create_dirs {
            if let Some(parent) = file_path.parent() {
                if let Err(e) = fs::create_dir_all(parent).await {
                    return Ok(ToolOutput::failure(ToolError::new(
                        "CREATE_DIR_ERROR".to_string(),
                        format!("Failed to create parent directories: {}", e),
                    )));
                }
            }
        }

        // Write content
        let bytes_written = match encoding.as_str() {
            "utf8" => {
                match fs::write(&file_path, &content).await {
                    Ok(_) => content.len() as u64,
                    Err(e) => {
                        return Ok(ToolOutput::failure(ToolError::new(
                            "WRITE_ERROR".to_string(),
                            format!("Failed to write file: {}", e),
                        )));
                    }
                }
            }
            "binary" => {
                match base64::decode(&content) {
                    Ok(bytes) => {
                        let byte_count = bytes.len() as u64;
                        match fs::write(&file_path, &bytes).await {
                            Ok(_) => byte_count,
                            Err(e) => {
                                return Ok(ToolOutput::failure(ToolError::new(
                                    "WRITE_ERROR".to_string(),
                                    format!("Failed to write file: {}", e),
                                )));
                            }
                        }
                    }
                    Err(e) => {
                        return Ok(ToolOutput::failure(ToolError::new(
                            "DECODE_ERROR".to_string(),
                            format!("Failed to decode base64 content: {}", e),
                        )));
                    }
                }
            }
            _ => {
                return Ok(ToolOutput::failure(ToolError::new(
                    "INVALID_ENCODING".to_string(),
                    format!("Unsupported encoding: {}", encoding),
                )));
            }
        };

        let execution_time = start_time.elapsed().as_millis() as u64;
        let metrics = ToolMetrics {
            execution_time_ms: execution_time,
            files_accessed: Some(1),
            ..Default::default()
        };

        Ok(ToolOutput::success(Some(serde_json::json!({
            "path": path,
            "bytes_written": bytes_written,
            "created": !file_exists
        }))).with_metrics(metrics))
    }

    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }
}

/// Directory listing tool
#[derive(Debug)]
pub struct DirectoryListTool {
    metadata: ToolMetadata,
}

impl DirectoryListTool {
    /// Create a new directory list tool
    pub fn new() -> Self {
        let metadata = ToolMetadata {
            id: "directory_list".to_string(),
            name: "Directory Lister".to_string(),
            description: "List contents of directories".to_string(),
            version: "1.0.0".to_string(),
            category: ToolCategory::FileSystem,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the directory to list"
                    },
                    "recursive": {
                        "type": "boolean",
                        "default": false,
                        "description": "List recursively"
                    },
                    "include_hidden": {
                        "type": "boolean",
                        "default": false,
                        "description": "Include hidden files"
                    },
                    "max_depth": {
                        "type": "number",
                        "default": 10,
                        "description": "Maximum recursion depth"
                    }
                },
                "required": ["path"]
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "entries": {
                        "type": "array",
                        "description": "Directory entries"
                    },
                    "total_count": {
                        "type": "number",
                        "description": "Total number of entries"
                    },
                    "path": {
                        "type": "string",
                        "description": "Directory path"
                    }
                }
            }),
            parallel_safe: true,
            estimated_duration: Some(Duration::from_millis(300)),
            required_permissions: vec!["filesystem.read".to_string()],
            tags: vec!["file".to_string(), "directory".to_string(), "list".to_string()],
            properties: HashMap::new(),
        };

        Self { metadata }
    }

    /// Check if path is safe (prevent directory traversal)
    fn is_safe_path(&self, path: &str) -> bool {
        let path = Path::new(path);
        
        // Prevent absolute paths and directory traversal
        if path.is_absolute() {
            return false;
        }
        
        for component in path.components() {
            if let std::path::Component::ParentDir = component {
                return false;
            }
        }
        
        true
    }

    /// List directory contents recursively
    async fn list_directory(
        &self,
        path: &Path,
        recursive: bool,
        include_hidden: bool,
        max_depth: usize,
        current_depth: usize,
    ) -> CoreResult<Vec<serde_json::Value>> {
        if current_depth > max_depth {
            return Ok(vec![]);
        }

        let mut entries = Vec::new();
        let mut dir_entries = fs::read_dir(path).await.map_err(|e| {
            CoreError::execution_error(format!("Failed to read directory: {}", e))
        })?;

        while let Some(entry) = dir_entries.next_entry().await.map_err(|e| {
            CoreError::execution_error(format!("Failed to read directory entry: {}", e))
        })? {
            let entry_path = entry.path();
            let file_name = entry_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            // Skip hidden files if not requested
            if !include_hidden && file_name.starts_with('.') {
                continue;
            }

            let metadata = entry.metadata().await.map_err(|e| {
                CoreError::execution_error(format!("Failed to read entry metadata: {}", e))
            })?;

            let entry_info = serde_json::json!({
                "name": file_name,
                "path": entry_path.to_string_lossy(),
                "is_file": metadata.is_file(),
                "is_dir": metadata.is_dir(),
                "size_bytes": metadata.len(),
                "modified": metadata.modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
            });

            entries.push(entry_info);

            // Recurse into subdirectories if requested
            if recursive && metadata.is_dir() {
                let sub_entries = self.list_directory(
                    &entry_path,
                    recursive,
                    include_hidden,
                    max_depth,
                    current_depth + 1,
                ).await?;
                entries.extend(sub_entries);
            }
        }

        Ok(entries)
    }
}

impl Default for DirectoryListTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for DirectoryListTool {
    async fn execute(&self, input: ToolInput) -> CoreResult<ToolOutput> {
        let start_time = Instant::now();

        // Extract parameters
        let path: String = input.get_param("path")?;
        let recursive: bool = input.get_optional_param("recursive")?.unwrap_or(false);
        let include_hidden: bool = input.get_optional_param("include_hidden")?.unwrap_or(false);
        let max_depth: usize = input.get_optional_param("max_depth")?.unwrap_or(10);

        // Check permissions
        if !input.context.security_context.permissions.contains(&"filesystem.read".to_string()) {
            return Ok(ToolOutput::failure(ToolError::new(
                "PERMISSION_DENIED".to_string(),
                "File read permission not granted".to_string(),
            )));
        }

        // Validate path safety
        if !self.is_safe_path(&path) {
            return Ok(ToolOutput::failure(ToolError::new(
                "UNSAFE_PATH".to_string(),
                "Path contains unsafe components".to_string(),
            )));
        }

        let dir_path = Path::new(&path);
        if !dir_path.exists() {
            return Ok(ToolOutput::failure(ToolError::new(
                "DIRECTORY_NOT_FOUND".to_string(),
                format!("Directory not found: {}", path),
            )));
        }

        if !dir_path.is_dir() {
            return Ok(ToolOutput::failure(ToolError::new(
                "NOT_A_DIRECTORY".to_string(),
                format!("Path is not a directory: {}", path),
            )));
        }

        // List directory contents
        let entries = self.list_directory(dir_path, recursive, include_hidden, max_depth, 0).await?;
        let total_count = entries.len();

        let execution_time = start_time.elapsed().as_millis() as u64;
        let metrics = ToolMetrics {
            execution_time_ms: execution_time,
            files_accessed: Some(total_count as u32),
            ..Default::default()
        };

        Ok(ToolOutput::success(Some(serde_json::json!({
            "entries": entries,
            "total_count": total_count,
            "path": path
        }))).with_metrics(metrics))
    }

    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }
}