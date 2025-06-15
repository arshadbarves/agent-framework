// Common tools for AgentGraph
// Provides a collection of commonly used tools for various tasks

/// HTTP tools for making web requests
pub mod http;
/// File system tools for reading, writing, and managing files
pub mod file;
/// Database tools for querying and manipulating data
pub mod database;
/// Text processing tools
pub mod text;
/// Mathematical computation tools
pub mod math;

pub use http::{HttpGetTool, HttpPostTool, HttpPutTool, HttpDeleteTool};
pub use file::{FileReadTool, FileWriteTool, DirectoryListTool};
pub use database::{SqlQueryTool, JsonQueryTool};
pub use text::{TextProcessorTool, RegexTool, TemplateRenderTool};
pub use math::{CalculatorTool, StatisticsTool};

use crate::tools::registry::{ToolRegistry, ToolRegistryBuilder};
use crate::tools::traits::ToolResult;

/// Create a registry with all common tools
pub fn create_common_tools_registry() -> ToolResult<ToolRegistry> {
    let registry = ToolRegistryBuilder::new()
        // HTTP tools
        .with_tool(HttpGetTool::new())?
        .with_tool(HttpPostTool::new())?
        .with_tool(HttpPutTool::new())?
        .with_tool(HttpDeleteTool::new())?

        // File tools
        .with_tool(FileReadTool::new())?
        .with_tool(FileWriteTool::new())?
        .with_tool(DirectoryListTool::new())?

        // Database tools
        .with_tool(SqlQueryTool::new())?
        .with_tool(JsonQueryTool::new())?

        // Text tools
        .with_tool(TextProcessorTool::new())?
        .with_tool(RegexTool::new())?
        .with_tool(TemplateRenderTool::new())?

        // Math tools
        .with_tool(CalculatorTool::new())?
        .with_tool(StatisticsTool::new())?

        .build();

    Ok(registry)
}

/// Tool categories for organization
pub mod categories {
    /// HTTP and web-related tools
    pub const HTTP: &str = "http";
    /// File system tools
    pub const FILE: &str = "file";
    /// Database and data query tools
    pub const DATABASE: &str = "database";
    /// Text processing tools
    pub const TEXT: &str = "text";
    /// Mathematical computation tools
    pub const MATH: &str = "math";
    /// General utility tools
    pub const UTILITY: &str = "utility";
    /// Network-related tools
    pub const NETWORK: &str = "network";
    /// Input/output tools
    pub const IO: &str = "io";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_common_tools_registry() {
        let registry = create_common_tools_registry().unwrap();
        let stats = registry.stats();
        
        // Should have all the common tools
        assert!(stats.total_tools >= 11); // At least 11 tools
        assert!(stats.total_categories >= 5); // At least 5 categories
        
        // Check specific tools exist
        assert!(registry.contains("http_get"));
        assert!(registry.contains("file_read"));
        assert!(registry.contains("sql_query"));
        assert!(registry.contains("text_processor"));
        assert!(registry.contains("calculator"));
    }

    #[test]
    fn test_tool_categories() {
        let registry = create_common_tools_registry().unwrap();
        
        // Test HTTP category
        let http_tools = registry.get_by_category(categories::HTTP);
        assert!(!http_tools.is_empty());
        
        // Test File category
        let file_tools = registry.get_by_category(categories::FILE);
        assert!(!file_tools.is_empty());
        
        // Test Database category
        let db_tools = registry.get_by_category(categories::DATABASE);
        assert!(!db_tools.is_empty());
    }
}
