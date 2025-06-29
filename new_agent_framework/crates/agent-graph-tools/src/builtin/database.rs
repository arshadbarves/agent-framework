//! Database tools for querying and data manipulation.

use crate::{CoreError, CoreResult};
use crate::core::{Tool, ToolMetadata, ToolInput, ToolOutput, ToolCategory, ToolError, ToolMetrics};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// SQL query tool for database operations
#[derive(Debug)]
pub struct SqlQueryTool {
    metadata: ToolMetadata,
}

impl SqlQueryTool {
    /// Create a new SQL query tool
    pub fn new() -> Self {
        let metadata = ToolMetadata {
            id: "sql_query".to_string(),
            name: "SQL Query".to_string(),
            description: "Execute SQL queries against databases".to_string(),
            version: "1.0.0".to_string(),
            category: ToolCategory::Database,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "SQL query to execute"
                    },
                    "connection_string": {
                        "type": "string",
                        "description": "Database connection string"
                    },
                    "parameters": {
                        "type": "object",
                        "description": "Query parameters"
                    },
                    "timeout_seconds": {
                        "type": "number",
                        "default": 30,
                        "description": "Query timeout in seconds"
                    }
                },
                "required": ["query"]
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "rows": {
                        "type": "array",
                        "description": "Query result rows"
                    },
                    "affected_rows": {
                        "type": "number",
                        "description": "Number of affected rows"
                    },
                    "execution_time_ms": {
                        "type": "number",
                        "description": "Query execution time in milliseconds"
                    },
                    "columns": {
                        "type": "array",
                        "description": "Column information"
                    }
                }
            }),
            parallel_safe: false, // Database operations may not be safe to run in parallel
            estimated_duration: Some(Duration::from_millis(500)),
            required_permissions: vec!["database.query".to_string()],
            tags: vec!["database".to_string(), "sql".to_string(), "query".to_string()],
            properties: HashMap::new(),
        };

        Self { metadata }
    }
}

impl Default for SqlQueryTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for SqlQueryTool {
    async fn execute(&self, input: ToolInput) -> CoreResult<ToolOutput> {
        let start_time = Instant::now();

        // Extract parameters
        let query: String = input.get_param("query")?;
        let connection_string: Option<String> = input.get_optional_param("connection_string")?;
        let parameters: Option<HashMap<String, serde_json::Value>> = input.get_optional_param("parameters")?;
        let timeout_seconds: Option<f64> = input.get_optional_param("timeout_seconds")?;

        // Check permissions
        if !input.context.security_context.permissions.contains(&"database.query".to_string()) {
            return Ok(ToolOutput::failure(ToolError::new(
                "PERMISSION_DENIED".to_string(),
                "Database query permission not granted".to_string(),
            )));
        }

        // Validate query (basic SQL injection prevention)
        if query.to_lowercase().contains("drop ") || 
           query.to_lowercase().contains("delete ") ||
           query.to_lowercase().contains("truncate ") {
            return Ok(ToolOutput::failure(ToolError::new(
                "UNSAFE_QUERY".to_string(),
                "Potentially unsafe SQL query detected".to_string(),
            )));
        }

        // Simulate database execution (in real implementation, would connect to actual database)
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Mock result based on query type
        let (rows, affected_rows) = if query.to_lowercase().starts_with("select") {
            // Mock SELECT result
            let mock_rows = vec![
                serde_json::json!({"id": 1, "name": "John Doe", "email": "john@example.com"}),
                serde_json::json!({"id": 2, "name": "Jane Smith", "email": "jane@example.com"}),
            ];
            (mock_rows, 0)
        } else if query.to_lowercase().starts_with("insert") ||
                  query.to_lowercase().starts_with("update") ||
                  query.to_lowercase().starts_with("delete") {
            // Mock DML result
            (vec![], 2)
        } else {
            // Other queries
            (vec![], 0)
        };

        let execution_time = start_time.elapsed().as_millis() as u64;
        let metrics = ToolMetrics {
            execution_time_ms: execution_time,
            ..Default::default()
        };

        Ok(ToolOutput::success(Some(serde_json::json!({
            "rows": rows,
            "affected_rows": affected_rows,
            "execution_time_ms": execution_time,
            "columns": ["id", "name", "email"],
            "query": query
        }))).with_metrics(metrics))
    }

    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }
}

/// JSON query tool for filtering and extracting data from JSON
#[derive(Debug)]
pub struct JsonQueryTool {
    metadata: ToolMetadata,
}

impl JsonQueryTool {
    /// Create a new JSON query tool
    pub fn new() -> Self {
        let metadata = ToolMetadata {
            id: "json_query".to_string(),
            name: "JSON Query".to_string(),
            description: "Query and filter JSON data using JSONPath expressions".to_string(),
            version: "1.0.0".to_string(),
            category: ToolCategory::DataAnalysis,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "data": {
                        "description": "JSON data to query"
                    },
                    "query": {
                        "type": "string",
                        "description": "JSONPath query expression"
                    },
                    "multiple": {
                        "type": "boolean",
                        "default": true,
                        "description": "Return multiple matches"
                    }
                },
                "required": ["data", "query"]
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "results": {
                        "type": "array",
                        "description": "Query results"
                    },
                    "match_count": {
                        "type": "number",
                        "description": "Number of matches found"
                    },
                    "query": {
                        "type": "string",
                        "description": "Original query"
                    }
                }
            }),
            parallel_safe: true,
            estimated_duration: Some(Duration::from_millis(50)),
            required_permissions: vec![],
            tags: vec!["json".to_string(), "query".to_string(), "data".to_string()],
            properties: HashMap::new(),
        };

        Self { metadata }
    }

    /// Simple JSONPath-like query implementation
    fn query_json(&self, data: &serde_json::Value, query: &str) -> Vec<serde_json::Value> {
        // This is a simplified JSONPath implementation
        // In a real implementation, you'd use a proper JSONPath library
        
        if query == "$" {
            return vec![data.clone()];
        }

        if query.starts_with("$.") {
            let path = &query[2..];
            if let Some(value) = data.get(path) {
                return vec![value.clone()];
            }
        }

        // Handle array access like $[*] or $[0]
        if query.starts_with("$[") && query.ends_with("]") {
            let index_str = &query[2..query.len()-1];
            if index_str == "*" {
                if let Some(array) = data.as_array() {
                    return array.clone();
                }
            } else if let Ok(index) = index_str.parse::<usize>() {
                if let Some(array) = data.as_array() {
                    if let Some(value) = array.get(index) {
                        return vec![value.clone()];
                    }
                }
            }
        }

        vec![]
    }
}

impl Default for JsonQueryTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for JsonQueryTool {
    async fn execute(&self, input: ToolInput) -> CoreResult<ToolOutput> {
        let start_time = Instant::now();

        // Extract parameters
        let data: serde_json::Value = input.get_param("data")?;
        let query: String = input.get_param("query")?;
        let multiple: bool = input.get_optional_param("multiple")?.unwrap_or(true);

        // Execute query
        let results = self.query_json(&data, &query);
        let match_count = results.len();

        let final_results = if multiple {
            results
        } else {
            results.into_iter().take(1).collect()
        };

        let execution_time = start_time.elapsed().as_millis() as u64;
        let metrics = ToolMetrics {
            execution_time_ms: execution_time,
            ..Default::default()
        };

        Ok(ToolOutput::success(Some(serde_json::json!({
            "results": final_results,
            "match_count": match_count,
            "query": query
        }))).with_metrics(metrics))
    }

    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }
}

/// CSV query tool for processing CSV data
#[derive(Debug)]
pub struct CsvQueryTool {
    metadata: ToolMetadata,
}

impl CsvQueryTool {
    /// Create a new CSV query tool
    pub fn new() -> Self {
        let metadata = ToolMetadata {
            id: "csv_query".to_string(),
            name: "CSV Query".to_string(),
            description: "Query and filter CSV data".to_string(),
            version: "1.0.0".to_string(),
            category: ToolCategory::DataAnalysis,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "csv_data": {
                        "type": "string",
                        "description": "CSV data as string"
                    },
                    "operation": {
                        "type": "string",
                        "enum": ["filter", "select", "count", "group_by"],
                        "description": "Operation to perform"
                    },
                    "columns": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Columns to select or group by"
                    },
                    "filter_condition": {
                        "type": "string",
                        "description": "Filter condition (simple column=value format)"
                    }
                },
                "required": ["csv_data", "operation"]
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "result": {
                        "description": "Query result"
                    },
                    "row_count": {
                        "type": "number",
                        "description": "Number of rows in result"
                    }
                }
            }),
            parallel_safe: true,
            estimated_duration: Some(Duration::from_millis(200)),
            required_permissions: vec![],
            tags: vec!["csv".to_string(), "data".to_string(), "query".to_string()],
            properties: HashMap::new(),
        };

        Self { metadata }
    }

    /// Parse CSV data into structured format
    fn parse_csv(&self, csv_data: &str) -> CoreResult<Vec<HashMap<String, String>>> {
        let lines: Vec<&str> = csv_data.lines().collect();
        if lines.is_empty() {
            return Ok(vec![]);
        }

        // Parse header
        let headers: Vec<&str> = lines[0].split(',').map(|h| h.trim()).collect();
        let mut rows = Vec::new();

        // Parse data rows
        for line in lines.iter().skip(1) {
            let values: Vec<&str> = line.split(',').map(|v| v.trim()).collect();
            if values.len() == headers.len() {
                let mut row = HashMap::new();
                for (header, value) in headers.iter().zip(values.iter()) {
                    row.insert(header.to_string(), value.to_string());
                }
                rows.push(row);
            }
        }

        Ok(rows)
    }
}

impl Default for CsvQueryTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for CsvQueryTool {
    async fn execute(&self, input: ToolInput) -> CoreResult<ToolOutput> {
        let start_time = Instant::now();

        // Extract parameters
        let csv_data: String = input.get_param("csv_data")?;
        let operation: String = input.get_param("operation")?;
        let columns: Option<Vec<String>> = input.get_optional_param("columns")?;
        let filter_condition: Option<String> = input.get_optional_param("filter_condition")?;

        // Parse CSV
        let rows = self.parse_csv(&csv_data)?;

        // Execute operation
        let result = match operation.as_str() {
            "count" => {
                serde_json::json!({
                    "count": rows.len()
                })
            }
            "select" => {
                if let Some(cols) = columns {
                    let selected_rows: Vec<HashMap<String, String>> = rows
                        .into_iter()
                        .map(|row| {
                            let mut selected_row = HashMap::new();
                            for col in &cols {
                                if let Some(value) = row.get(col) {
                                    selected_row.insert(col.clone(), value.clone());
                                }
                            }
                            selected_row
                        })
                        .collect();
                    serde_json::to_value(selected_rows).unwrap_or_default()
                } else {
                    serde_json::to_value(rows).unwrap_or_default()
                }
            }
            "filter" => {
                if let Some(condition) = filter_condition {
                    // Simple filter: column=value
                    if let Some((col, val)) = condition.split_once('=') {
                        let filtered_rows: Vec<_> = rows
                            .into_iter()
                            .filter(|row| {
                                row.get(col.trim()).map_or(false, |v| v == val.trim())
                            })
                            .collect();
                        serde_json::to_value(filtered_rows).unwrap_or_default()
                    } else {
                        return Ok(ToolOutput::failure(ToolError::new(
                            "INVALID_FILTER".to_string(),
                            "Filter condition must be in format 'column=value'".to_string(),
                        )));
                    }
                } else {
                    serde_json::to_value(rows).unwrap_or_default()
                }
            }
            _ => {
                return Ok(ToolOutput::failure(ToolError::new(
                    "INVALID_OPERATION".to_string(),
                    format!("Unsupported operation: {}", operation),
                )));
            }
        };

        let execution_time = start_time.elapsed().as_millis() as u64;
        let metrics = ToolMetrics {
            execution_time_ms: execution_time,
            ..Default::default()
        };

        Ok(ToolOutput::success(Some(serde_json::json!({
            "result": result,
            "row_count": if result.is_array() { result.as_array().unwrap().len() } else { 1 }
        }))).with_metrics(metrics))
    }

    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }
}