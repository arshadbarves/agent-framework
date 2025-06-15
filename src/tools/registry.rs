// Tool registry for managing and discovering tools

use super::traits::{Tool, ToolError, ToolResult};
use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

/// A registry for managing tools
#[derive(Debug)]
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
    categories: HashMap<String, Vec<String>>,
}

impl ToolRegistry {
    /// Create a new empty tool registry
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            categories: HashMap::new(),
        }
    }
    
    /// Register a tool in the registry
    pub fn register<T: Tool + 'static>(&mut self, tool: T) -> ToolResult<()> {
        let metadata = tool.metadata().clone();
        let tool_id = metadata.id.clone();

        // Check if tool already exists
        if self.tools.contains_key(&tool_id) {
            return Err(ToolError::ConfigurationError {
                message: format!("Tool with ID '{}' already registered", tool_id),
            });
        }

        // Register tool
        self.tools.insert(tool_id.clone(), Arc::new(tool));

        // Update categories
        for tag in &metadata.tags {
            self.categories
                .entry(tag.clone())
                .or_insert_with(Vec::new)
                .push(tool_id.clone());
        }

        Ok(())
    }
    
    /// Get a tool by ID
    pub fn get(&self, tool_id: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(tool_id).cloned()
    }
    
    /// Check if a tool exists
    pub fn contains(&self, tool_id: &str) -> bool {
        self.tools.contains_key(tool_id)
    }
    
    /// Get all tool IDs
    pub fn list_tools(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }
    
    /// Get tools by category/tag
    pub fn get_by_category(&self, category: &str) -> Vec<Arc<dyn Tool>> {
        self.categories
            .get(category)
            .map(|tool_ids| {
                tool_ids
                    .iter()
                    .filter_map(|id| self.tools.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Get all categories
    pub fn list_categories(&self) -> Vec<String> {
        self.categories.keys().cloned().collect()
    }
    
    /// Search tools by name or description
    pub fn search(&self, query: &str) -> Vec<Arc<dyn Tool>> {
        let query_lower = query.to_lowercase();
        self.tools
            .values()
            .filter(|tool| {
                let metadata = tool.metadata();
                metadata.name.to_lowercase().contains(&query_lower)
                    || metadata.description.to_lowercase().contains(&query_lower)
                    || metadata.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect()
    }
    
    /// Remove a tool from the registry
    pub fn unregister(&mut self, tool_id: &str) -> ToolResult<()> {
        if let Some(tool) = self.tools.remove(tool_id) {
            // Remove from categories
            let metadata = tool.metadata();
            for tag in &metadata.tags {
                if let Some(tool_ids) = self.categories.get_mut(tag) {
                    tool_ids.retain(|id| id != tool_id);
                    if tool_ids.is_empty() {
                        self.categories.remove(tag);
                    }
                }
            }
            Ok(())
        } else {
            Err(ToolError::NotFoundError {
                tool_id: tool_id.to_string(),
            })
        }
    }
    
    /// Get registry statistics
    pub fn stats(&self) -> RegistryStats {
        RegistryStats {
            total_tools: self.tools.len(),
            total_categories: self.categories.len(),
            tools_by_category: self.categories
                .iter()
                .map(|(k, v)| (k.clone(), v.len()))
                .collect(),
        }
    }
    
    /// Clear all tools from the registry
    pub fn clear(&mut self) {
        self.tools.clear();
        self.categories.clear();
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the tool registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats {
    /// Total number of tools registered
    pub total_tools: usize,
    /// Total number of categories
    pub total_categories: usize,
    /// Number of tools per category
    pub tools_by_category: HashMap<String, usize>,
}

/// Builder for creating and configuring a tool registry
#[derive(Debug)]
pub struct ToolRegistryBuilder {
    registry: ToolRegistry,
}

impl ToolRegistryBuilder {
    /// Create a new registry builder
    pub fn new() -> Self {
        Self {
            registry: ToolRegistry::new(),
        }
    }
    
    /// Add a tool to the registry
    pub fn with_tool<T: Tool + 'static>(mut self, tool: T) -> ToolResult<Self> {
        self.registry.register(tool)?;
        Ok(self)
    }
    
    /// Build the final registry
    pub fn build(self) -> ToolRegistry {
        self.registry
    }
}

impl Default for ToolRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::traits::{ToolInput, ToolMetadata, ToolOutput};
    use async_trait::async_trait;
    use serde_json::json;

    #[derive(Debug)]
    struct TestTool {
        metadata: ToolMetadata,
    }

    impl TestTool {
        fn new(id: &str, name: &str, tags: Vec<&str>) -> Self {
            let mut metadata = ToolMetadata::new(id, name, "Test tool");
            for tag in tags {
                metadata = metadata.with_tag(tag);
            }
            Self { metadata }
        }
    }

    #[async_trait]
    impl Tool for TestTool {
        fn metadata(&self) -> &ToolMetadata {
            &self.metadata
        }

        async fn execute(&self, _input: ToolInput) -> ToolResult<ToolOutput> {
            Ok(ToolOutput::new(json!({"result": "test"})))
        }
    }

    #[test]
    fn test_registry_creation() {
        let registry = ToolRegistry::new();
        assert_eq!(registry.list_tools().len(), 0);
        assert_eq!(registry.list_categories().len(), 0);
    }

    #[test]
    fn test_tool_registration() {
        let mut registry = ToolRegistry::new();
        let tool = TestTool::new("test1", "Test Tool 1", vec!["testing", "utility"]);
        
        assert!(registry.register(tool).is_ok());
        assert!(registry.contains("test1"));
        assert_eq!(registry.list_tools().len(), 1);
        assert_eq!(registry.list_categories().len(), 2);
    }

    #[test]
    fn test_duplicate_registration() {
        let mut registry = ToolRegistry::new();
        let tool1 = TestTool::new("test1", "Test Tool 1", vec!["testing"]);
        let tool2 = TestTool::new("test1", "Test Tool 1 Duplicate", vec!["testing"]);
        
        assert!(registry.register(tool1).is_ok());
        assert!(registry.register(tool2).is_err());
    }

    #[test]
    fn test_tool_retrieval() {
        let mut registry = ToolRegistry::new();
        let tool = TestTool::new("test1", "Test Tool 1", vec!["testing"]);
        
        registry.register(tool).unwrap();
        
        let retrieved = registry.get("test1");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().metadata().id, "test1");
        
        let not_found = registry.get("nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_category_filtering() {
        let mut registry = ToolRegistry::new();
        let tool1 = TestTool::new("test1", "Test Tool 1", vec!["testing", "utility"]);
        let tool2 = TestTool::new("test2", "Test Tool 2", vec!["testing"]);
        let tool3 = TestTool::new("test3", "Test Tool 3", vec!["utility"]);
        
        registry.register(tool1).unwrap();
        registry.register(tool2).unwrap();
        registry.register(tool3).unwrap();
        
        let testing_tools = registry.get_by_category("testing");
        assert_eq!(testing_tools.len(), 2);
        
        let utility_tools = registry.get_by_category("utility");
        assert_eq!(utility_tools.len(), 2);
        
        let nonexistent_tools = registry.get_by_category("nonexistent");
        assert_eq!(nonexistent_tools.len(), 0);
    }

    #[test]
    fn test_tool_search() {
        let mut registry = ToolRegistry::new();
        let tool1 = TestTool::new("test1", "HTTP Client", vec!["network", "http"]);
        let tool2 = TestTool::new("test2", "Database Query", vec!["database", "sql"]);
        let tool3 = TestTool::new("test3", "File Reader", vec!["file", "io"]);
        
        registry.register(tool1).unwrap();
        registry.register(tool2).unwrap();
        registry.register(tool3).unwrap();
        
        let http_tools = registry.search("http");
        assert_eq!(http_tools.len(), 1);
        assert_eq!(http_tools[0].metadata().id, "test1");
        
        let client_tools = registry.search("client");
        assert_eq!(client_tools.len(), 1);
        
        let no_match = registry.search("xyz");
        assert_eq!(no_match.len(), 0);
    }

    #[test]
    fn test_tool_unregistration() {
        let mut registry = ToolRegistry::new();
        let tool = TestTool::new("test1", "Test Tool 1", vec!["testing"]);
        
        registry.register(tool).unwrap();
        assert!(registry.contains("test1"));
        
        assert!(registry.unregister("test1").is_ok());
        assert!(!registry.contains("test1"));
        assert_eq!(registry.list_tools().len(), 0);
        
        // Try to unregister non-existent tool
        assert!(registry.unregister("nonexistent").is_err());
    }

    #[test]
    fn test_registry_builder() {
        let tool1 = TestTool::new("test1", "Test Tool 1", vec!["testing"]);
        let tool2 = TestTool::new("test2", "Test Tool 2", vec!["utility"]);
        
        let registry = ToolRegistryBuilder::new()
            .with_tool(tool1)
            .unwrap()
            .with_tool(tool2)
            .unwrap()
            .build();
        
        assert_eq!(registry.list_tools().len(), 2);
        assert!(registry.contains("test1"));
        assert!(registry.contains("test2"));
    }
}
