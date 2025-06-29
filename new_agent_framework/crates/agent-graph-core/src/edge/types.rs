//! Core edge types and definitions.

use crate::error::{CoreError, CoreResult};
use crate::node::NodeId;
use crate::state::State;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a connection between two nodes in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    /// Unique identifier for this edge
    pub id: String,
    /// Source node ID
    pub from: NodeId,
    /// Target node ID
    pub to: NodeId,
    /// Edge type
    pub edge_type: EdgeType,
    /// Optional condition for traversal
    pub condition: Option<String>,
    /// Edge metadata
    pub metadata: EdgeMetadata,
    /// Weight/priority of this edge (higher = preferred)
    pub weight: f64,
}

impl Edge {
    /// Create a new edge
    pub fn new(id: String, from: NodeId, to: NodeId) -> Self {
        Self {
            id,
            from,
            to,
            edge_type: EdgeType::Normal,
            condition: None,
            metadata: EdgeMetadata::default(),
            weight: 1.0,
        }
    }

    /// Create a conditional edge
    pub fn conditional(id: String, from: NodeId, to: NodeId, condition: String) -> Self {
        Self {
            id,
            from,
            to,
            edge_type: EdgeType::Conditional,
            condition: Some(condition),
            metadata: EdgeMetadata::default(),
            weight: 1.0,
        }
    }

    /// Set edge weight
    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight;
        self
    }

    /// Set edge metadata
    pub fn with_metadata(mut self, metadata: EdgeMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Check if this edge should be traversed given the current state
    pub async fn should_traverse<S>(&self, _state: &S) -> CoreResult<bool>
    where
        S: State,
    {
        // For now, simple implementation - always traverse unless there's a condition
        match &self.condition {
            Some(_condition_expr) => {
                // TODO: Implement condition evaluation
                Ok(true)
            }
            None => Ok(true),
        }
    }
}

/// Types of edges in the graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EdgeType {
    /// Normal execution flow
    Normal,
    /// Conditional execution based on state
    Conditional,
    /// Error handling flow
    ErrorHandler,
    /// Parallel execution branch
    Parallel,
    /// Loop back edge
    Loop,
    /// Interrupt/break edge
    Interrupt,
    /// Custom edge type
    Custom(String),
}

impl Default for EdgeType {
    fn default() -> Self {
        Self::Normal
    }
}

/// Metadata associated with an edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeMetadata {
    /// Human-readable name
    pub name: Option<String>,
    /// Description of the edge purpose
    pub description: Option<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Custom metadata fields
    pub custom: HashMap<String, serde_json::Value>,
    /// Whether this edge can be traversed in parallel
    pub parallel_safe: bool,
    /// Expected traversal frequency (for optimization)
    pub frequency_hint: EdgeFrequency,
}

impl Default for EdgeMetadata {
    fn default() -> Self {
        Self {
            name: None,
            description: None,
            tags: Vec::new(),
            custom: HashMap::new(),
            parallel_safe: true,
            frequency_hint: EdgeFrequency::Normal,
        }
    }
}

/// Hint about edge traversal frequency
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EdgeFrequency {
    /// Rarely traversed
    Rare,
    /// Normal frequency
    Normal,
    /// Frequently traversed
    Frequent,
    /// Very frequently traversed (hot path)
    VeryFrequent,
}

impl Default for EdgeFrequency {
    fn default() -> Self {
        Self::Normal
    }
}

