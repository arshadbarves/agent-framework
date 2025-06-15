// Tool execution engine with retry, timeout, and caching support

use super::traits::{Tool, ToolError, ToolInput, ToolOutput, ToolResult};
use super::{ToolConfig, ToolStats};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use serde::{Deserialize, Serialize};

/// Context for tool execution
#[derive(Debug, Clone)]
pub struct ToolExecutionContext {
    /// Execution ID for tracking
    pub execution_id: String,
    /// User ID for auditing
    pub user_id: Option<String>,
    /// Session ID for grouping
    pub session_id: Option<String>,
    /// Additional context data
    pub context_data: HashMap<String, String>,
}

impl ToolExecutionContext {
    /// Create a new execution context
    pub fn new(execution_id: String) -> Self {
        Self {
            execution_id,
            user_id: None,
            session_id: None,
            context_data: HashMap::new(),
        }
    }
    
    /// Set user ID
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }
    
    /// Set session ID
    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }
    
    /// Add context data
    pub fn with_context_data(mut self, key: String, value: String) -> Self {
        self.context_data.insert(key, value);
        self
    }
}

/// Result of tool execution with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionResult {
    /// The tool output
    pub output: ToolOutput,
    /// Execution metadata
    pub metadata: ExecutionMetadata,
}

/// Metadata about tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetadata {
    /// Tool ID that was executed
    pub tool_id: String,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    /// Number of retry attempts
    pub retry_attempts: u32,
    /// Whether result was cached
    pub from_cache: bool,
    /// Execution timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Success status
    pub success: bool,
    /// Error message if failed
    pub error_message: Option<String>,
}

/// Simple in-memory cache for tool results
#[derive(Debug)]
pub struct ToolCache {
    cache: HashMap<String, (ToolOutput, Instant)>,
    ttl: Duration,
}

impl ToolCache {
    /// Create a new cache with TTL
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: HashMap::new(),
            ttl,
        }
    }
    
    /// Get cached result if valid
    pub fn get(&self, key: &str) -> Option<ToolOutput> {
        if let Some((output, timestamp)) = self.cache.get(key) {
            if timestamp.elapsed() < self.ttl {
                return Some(output.clone());
            }
        }
        None
    }
    
    /// Store result in cache
    pub fn put(&mut self, key: String, output: ToolOutput) {
        self.cache.insert(key, (output, Instant::now()));
    }
    
    /// Clear expired entries
    pub fn cleanup(&mut self) {
        let now = Instant::now();
        self.cache.retain(|_, (_, timestamp)| now.duration_since(*timestamp) < self.ttl);
    }
    
    /// Clear all entries
    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

/// Tool executor with retry, timeout, and caching capabilities
#[derive(Debug)]
pub struct ToolExecutor {
    cache: Option<ToolCache>,
    stats: HashMap<String, ToolStats>,
}

impl ToolExecutor {
    /// Create a new tool executor
    pub fn new() -> Self {
        Self {
            cache: None,
            stats: HashMap::new(),
        }
    }
    
    /// Enable caching with TTL
    pub fn with_cache(mut self, ttl: Duration) -> Self {
        self.cache = Some(ToolCache::new(ttl));
        self
    }
    
    /// Execute a tool with configuration and context
    pub async fn execute(
        &mut self,
        tool: Arc<dyn Tool>,
        input: ToolInput,
        config: &ToolConfig,
        _context: &ToolExecutionContext,
    ) -> ToolResult<ToolExecutionResult> {
        let tool_id = tool.metadata().id.clone();
        let start_time = Instant::now();
        let mut retry_attempts;
        
        // Check cache first if enabled (simplified for now)
        // TODO: Implement proper caching with trait object downcasting
        if config.cache_results {
            // Cache implementation will be added in a future version
        }
        
        // Execute with retries
        let mut last_error = None;
        
        for attempt in 0..=config.max_retries {
            retry_attempts = attempt;
            
            // Validate input
            if let Err(e) = tool.validate_input(&input).await {
                last_error = Some(e);
                break;
            }
            
            // Execute with timeout
            let execution_future = tool.execute(input.clone());
            let result = if let Some(timeout_duration) = config.timeout {
                timeout(timeout_duration, execution_future).await
            } else {
                Ok(execution_future.await)
            };
            
            match result {
                Ok(Ok(output)) => {
                    let duration_ms = start_time.elapsed().as_millis() as u64;
                    
                    // Cache result if enabled (simplified for now)
                    // TODO: Implement proper caching with trait object downcasting
                    if config.cache_results {
                        // Cache implementation will be added in a future version
                    }
                    
                    // Update statistics
                    self.update_stats(&tool_id, duration_ms, true);
                    
                    return Ok(ToolExecutionResult {
                        output,
                        metadata: ExecutionMetadata {
                            tool_id,
                            duration_ms,
                            retry_attempts,
                            from_cache: false,
                            timestamp: chrono::Utc::now(),
                            success: true,
                            error_message: None,
                        },
                    });
                }
                Ok(Err(e)) => {
                    last_error = Some(e);
                    if attempt < config.max_retries {
                        tokio::time::sleep(config.retry_delay).await;
                    }
                }
                Err(_) => {
                    // Timeout occurred
                    let timeout_error = ToolError::TimeoutError {
                        timeout_ms: config.timeout.unwrap_or(Duration::from_secs(30)).as_millis() as u64,
                    };
                    last_error = Some(timeout_error);
                    if attempt < config.max_retries {
                        tokio::time::sleep(config.retry_delay).await;
                    }
                }
            }
        }
        
        // All retries failed
        let duration_ms = start_time.elapsed().as_millis() as u64;
        let error = last_error.unwrap_or(ToolError::ExecutionError {
            message: "Unknown execution error".to_string(),
        });
        
        // Update statistics
        self.update_stats(&tool_id, duration_ms, false);
        
        Err(error)
    }
    
    /// Get statistics for a tool
    pub fn get_stats(&self, tool_id: &str) -> Option<&ToolStats> {
        self.stats.get(tool_id)
    }
    
    /// Get all statistics
    pub fn get_all_stats(&self) -> &HashMap<String, ToolStats> {
        &self.stats
    }
    
    /// Clear cache if enabled
    pub fn clear_cache(&mut self) {
        if let Some(cache) = &mut self.cache {
            cache.clear();
        }
    }
    
    /// Cleanup expired cache entries
    pub fn cleanup_cache(&mut self) {
        if let Some(cache) = &mut self.cache {
            cache.cleanup();
        }
    }
    
    /// Update tool statistics
    fn update_stats(&mut self, tool_id: &str, duration_ms: u64, success: bool) {
        let stats = self.stats.entry(tool_id.to_string()).or_default();
        stats.update(duration_ms, success);
    }
}

impl Default for ToolExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::traits::{ToolMetadata};
    use async_trait::async_trait;
    use serde_json::json;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[derive(Debug)]
    struct TestTool {
        metadata: ToolMetadata,
        call_count: Arc<AtomicU32>,
        should_fail: bool,
    }

    impl TestTool {
        fn new(id: &str, should_fail: bool) -> Self {
            Self {
                metadata: ToolMetadata::new(id, "Test Tool", "A test tool"),
                call_count: Arc::new(AtomicU32::new(0)),
                should_fail,
            }
        }
        
        fn call_count(&self) -> u32 {
            self.call_count.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl Tool for TestTool {
        fn metadata(&self) -> &ToolMetadata {
            &self.metadata
        }

        async fn execute(&self, _input: ToolInput) -> ToolResult<ToolOutput> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            
            if self.should_fail {
                Err(ToolError::ExecutionError {
                    message: "Test failure".to_string(),
                })
            } else {
                Ok(ToolOutput::new(json!({"result": "success"})))
            }
        }
    }

    #[tokio::test]
    async fn test_successful_execution() {
        let mut executor = ToolExecutor::new();
        let tool = Arc::new(TestTool::new("test_tool", false));
        let input = ToolInput::new(json!({"test": "data"}));
        let config = ToolConfig::default();
        let context = ToolExecutionContext::new("exec_1".to_string());
        
        let result = executor.execute(tool.clone(), input, &config, &context).await;
        
        assert!(result.is_ok());
        let execution_result = result.unwrap();
        assert!(execution_result.metadata.success);
        assert_eq!(execution_result.metadata.retry_attempts, 0);
        assert!(!execution_result.metadata.from_cache);
        assert_eq!(tool.call_count(), 1);
    }

    #[tokio::test]
    async fn test_retry_on_failure() {
        let mut executor = ToolExecutor::new();
        let tool = Arc::new(TestTool::new("test_tool", true));
        let input = ToolInput::new(json!({"test": "data"}));
        let config = ToolConfig {
            max_retries: 2,
            retry_delay: Duration::from_millis(10),
            ..Default::default()
        };
        let context = ToolExecutionContext::new("exec_1".to_string());
        
        let result = executor.execute(tool.clone(), input, &config, &context).await;
        
        assert!(result.is_err());
        assert_eq!(tool.call_count(), 3); // Initial + 2 retries
    }

    #[tokio::test]
    async fn test_timeout() {
        #[derive(Debug)]
        struct SlowTool {
            metadata: ToolMetadata,
        }

        #[async_trait]
        impl Tool for SlowTool {
            fn metadata(&self) -> &ToolMetadata {
                &self.metadata
            }

            async fn execute(&self, _input: ToolInput) -> ToolResult<ToolOutput> {
                tokio::time::sleep(Duration::from_millis(200)).await;
                Ok(ToolOutput::new(json!({"result": "success"})))
            }
        }

        let mut executor = ToolExecutor::new();
        let tool = Arc::new(SlowTool {
            metadata: ToolMetadata::new("slow_tool", "Slow Tool", "A slow test tool"),
        });
        let input = ToolInput::new(json!({"test": "data"}));
        let config = ToolConfig {
            timeout: Some(Duration::from_millis(50)),
            max_retries: 0,
            ..Default::default()
        };
        let context = ToolExecutionContext::new("exec_1".to_string());
        
        let result = executor.execute(tool, input, &config, &context).await;
        
        assert!(result.is_err());
        if let Err(ToolError::TimeoutError { timeout_ms }) = result {
            assert_eq!(timeout_ms, 50);
        } else {
            panic!("Expected timeout error");
        }
    }

    #[test]
    fn test_cache_operations() {
        let mut cache = ToolCache::new(Duration::from_millis(100));
        let output = ToolOutput::new(json!({"result": "cached"}));
        
        // Test put and get
        cache.put("key1".to_string(), output.clone());
        let retrieved = cache.get("key1");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().data, output.data);
        
        // Test cache miss
        let missing = cache.get("nonexistent");
        assert!(missing.is_none());
    }

    #[test]
    fn test_execution_context() {
        let context = ToolExecutionContext::new("exec_1".to_string())
            .with_user_id("user_123".to_string())
            .with_session_id("session_456".to_string())
            .with_context_data("key1".to_string(), "value1".to_string());
        
        assert_eq!(context.execution_id, "exec_1");
        assert_eq!(context.user_id, Some("user_123".to_string()));
        assert_eq!(context.session_id, Some("session_456".to_string()));
        assert_eq!(context.context_data.get("key1"), Some(&"value1".to_string()));
    }
}
