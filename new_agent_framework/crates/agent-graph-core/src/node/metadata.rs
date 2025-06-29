//! Node metadata and resource requirements.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadata associated with a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetadata {
    /// Human-readable name of the node
    pub name: String,
    /// Description of what the node does
    pub description: Option<String>,
    /// Tags for categorizing nodes
    pub tags: Vec<String>,
    /// Custom metadata fields
    pub custom: HashMap<String, serde_json::Value>,
    /// Node version for compatibility tracking
    pub version: String,
    /// Whether this node can be executed in parallel with others
    pub parallel_safe: bool,
    /// Expected execution time in milliseconds (for scheduling)
    pub expected_duration_ms: Option<u64>,
    /// Resource requirements
    pub resource_requirements: ResourceRequirements,
    /// Node category for organization
    pub category: NodeCategory,
    /// Priority level for execution scheduling
    pub priority: NodePriority,
}

impl Default for NodeMetadata {
    fn default() -> Self {
        Self {
            name: "Unnamed Node".to_string(),
            description: None,
            tags: Vec::new(),
            custom: HashMap::new(),
            version: "1.0.0".to_string(),
            parallel_safe: true,
            expected_duration_ms: None,
            resource_requirements: ResourceRequirements::default(),
            category: NodeCategory::Processing,
            priority: NodePriority::Normal,
        }
    }
}

impl NodeMetadata {
    /// Create new metadata with name
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }

    /// Create metadata with name and description
    pub fn with_description(name: String, description: String) -> Self {
        Self {
            name,
            description: Some(description),
            ..Default::default()
        }
    }

    /// Add a tag to the metadata
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// Set custom metadata field
    pub fn set_custom(&mut self, key: String, value: serde_json::Value) {
        self.custom.insert(key, value);
    }

    /// Get custom metadata field
    pub fn get_custom(&self, key: &str) -> Option<&serde_json::Value> {
        self.custom.get(key)
    }

    /// Check if node has a specific tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(&tag.to_string())
    }

    /// Set resource requirements
    pub fn with_resources(mut self, requirements: ResourceRequirements) -> Self {
        self.resource_requirements = requirements;
        self
    }

    /// Set category
    pub fn with_category(mut self, category: NodeCategory) -> Self {
        self.category = category;
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: NodePriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set expected duration
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.expected_duration_ms = Some(duration_ms);
        self
    }

    /// Mark as not parallel safe
    pub fn not_parallel_safe(mut self) -> Self {
        self.parallel_safe = false;
        self
    }
}

/// Resource requirements for a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// Memory requirement in MB
    pub memory_mb: Option<u64>,
    /// CPU cores required (can be fractional)
    pub cpu_cores: Option<f32>,
    /// Whether the node requires network access
    pub network_access: bool,
    /// Whether the node requires file system access
    pub filesystem_access: bool,
    /// Whether the node requires GPU access
    pub gpu_access: bool,
    /// Custom resource requirements
    pub custom_resources: HashMap<String, serde_json::Value>,
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            memory_mb: None,
            cpu_cores: None,
            network_access: false,
            filesystem_access: false,
            gpu_access: false,
            custom_resources: HashMap::new(),
        }
    }
}

/// Node category for organization and filtering
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeCategory {
    /// Input/output operations
    IO,
    /// Data processing and transformation
    Processing,
    /// Control flow and routing
    Control,
    /// External service integration
    Integration,
    /// Validation and verification
    Validation,
    /// Monitoring and observability
    Monitoring,
    /// Custom category
    Custom(String),
}

impl Default for NodeCategory {
    fn default() -> Self {
        Self::Processing
    }
}

/// Node execution priority
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum NodePriority {
    /// Lowest priority
    Low,
    /// Normal priority (default)
    Normal,
    /// High priority
    High,
    /// Critical priority
    Critical,
}

impl Default for NodePriority {
    fn default() -> Self {
        Self::Normal
    }
}