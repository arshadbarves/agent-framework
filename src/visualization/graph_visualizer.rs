//! Graph visualization for AgentGraph workflows
//! Provides LangGraph Studio-style visual workflow representation

use crate::error::GraphResult;
use crate::graph::Graph;
use crate::node::Node;
use crate::state::State;
use crate::visualization::{VisualWorkflow, VisualNode, VisualEdge, NodeStatus, NodeStats};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Graph visualizer for creating visual representations of workflows
#[derive(Debug)]
pub struct GraphVisualizer {
    /// Layout algorithm to use
    layout_algorithm: LayoutAlgorithm,
    /// Node positioning cache
    node_positions: HashMap<String, (f64, f64)>,
    /// Visual styling configuration
    styling: VisualizationStyling,
}

/// Layout algorithms for graph visualization
#[derive(Debug, Clone)]
pub enum LayoutAlgorithm {
    /// Force-directed layout (good for general graphs)
    ForceDirected,
    /// Hierarchical layout (good for DAGs)
    Hierarchical,
    /// Circular layout
    Circular,
    /// Grid layout
    Grid,
    /// Custom manual positioning
    Manual,
}

/// Visual styling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationStyling {
    /// Node styling
    pub node_style: NodeStyling,
    /// Edge styling
    pub edge_style: EdgeStyling,
    /// Layout configuration
    pub layout_config: LayoutConfig,
}

/// Node visual styling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStyling {
    /// Default node width
    pub default_width: f64,
    /// Default node height
    pub default_height: f64,
    /// Node colors by type
    pub colors: HashMap<String, String>,
    /// Node shapes by type
    pub shapes: HashMap<String, String>,
    /// Font configuration
    pub font: FontConfig,
}

/// Edge visual styling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeStyling {
    /// Default edge color
    pub default_color: String,
    /// Edge colors by type
    pub colors: HashMap<String, String>,
    /// Edge thickness
    pub thickness: f64,
    /// Arrow style
    pub arrow_style: String,
}

/// Layout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    /// Spacing between nodes
    pub node_spacing: f64,
    /// Canvas width
    pub canvas_width: f64,
    /// Canvas height
    pub canvas_height: f64,
    /// Padding around the graph
    pub padding: f64,
}

/// Font configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    /// Font family
    pub family: String,
    /// Font size
    pub size: f64,
    /// Font color
    pub color: String,
}

impl Default for VisualizationStyling {
    fn default() -> Self {
        let mut node_colors = HashMap::new();
        node_colors.insert("agent".to_string(), "#4CAF50".to_string()); // Green
        node_colors.insert("tool".to_string(), "#2196F3".to_string());  // Blue
        node_colors.insert("routing".to_string(), "#FF9800".to_string()); // Orange
        node_colors.insert("quality_gate".to_string(), "#9C27B0".to_string()); // Purple
        node_colors.insert("start".to_string(), "#4CAF50".to_string()); // Green
        node_colors.insert("end".to_string(), "#F44336".to_string());   // Red

        let mut node_shapes = HashMap::new();
        node_shapes.insert("agent".to_string(), "rectangle".to_string());
        node_shapes.insert("tool".to_string(), "diamond".to_string());
        node_shapes.insert("routing".to_string(), "hexagon".to_string());
        node_shapes.insert("quality_gate".to_string(), "octagon".to_string());
        node_shapes.insert("start".to_string(), "circle".to_string());
        node_shapes.insert("end".to_string(), "circle".to_string());

        let mut edge_colors = HashMap::new();
        edge_colors.insert("default".to_string(), "#666666".to_string());
        edge_colors.insert("conditional".to_string(), "#FF9800".to_string());
        edge_colors.insert("parallel".to_string(), "#2196F3".to_string());

        Self {
            node_style: NodeStyling {
                default_width: 120.0,
                default_height: 60.0,
                colors: node_colors,
                shapes: node_shapes,
                font: FontConfig {
                    family: "Arial, sans-serif".to_string(),
                    size: 12.0,
                    color: "#333333".to_string(),
                },
            },
            edge_style: EdgeStyling {
                default_color: "#666666".to_string(),
                colors: edge_colors,
                thickness: 2.0,
                arrow_style: "arrow".to_string(),
            },
            layout_config: LayoutConfig {
                node_spacing: 150.0,
                canvas_width: 1200.0,
                canvas_height: 800.0,
                padding: 50.0,
            },
        }
    }
}

impl GraphVisualizer {
    /// Create a new graph visualizer
    pub fn new() -> Self {
        Self {
            layout_algorithm: LayoutAlgorithm::Hierarchical,
            node_positions: HashMap::new(),
            styling: VisualizationStyling::default(),
        }
    }

    /// Create visual representation of a graph
    pub fn visualize_graph<S: State>(&mut self, graph: &Graph<S>, workflow_id: String, workflow_name: String) -> GraphResult<VisualWorkflow> {
        let mut visual_nodes = Vec::new();
        let mut visual_edges = Vec::new();

        // Convert nodes to visual representation
        for node_id in graph.node_ids() {
            let visual_node = self.create_visual_node(node_id, graph)?;
            visual_nodes.push(visual_node);
        }

        // Convert edges to visual representation
        for edge in graph.edges() {
            let visual_edge = self.create_visual_edge(edge)?;
            visual_edges.push(visual_edge);
        }

        // Apply layout algorithm
        self.apply_layout(&mut visual_nodes)?;

        Ok(VisualWorkflow {
            id: workflow_id,
            name: workflow_name,
            nodes: visual_nodes,
            edges: visual_edges,
            current_execution: None,
            metadata: HashMap::new(),
        })
    }

    /// Create visual node from graph node
    fn create_visual_node<S: State>(&self, node_id: &str, graph: &Graph<S>) -> GraphResult<VisualNode> {
        // Get node from graph (simplified - would need actual node access)
        let node_type = self.determine_node_type(node_id);
        let display_name = self.format_node_name(node_id);
        
        // Get or generate position
        let position = self.node_positions.get(node_id)
            .copied()
            .unwrap_or((0.0, 0.0));

        Ok(VisualNode {
            id: node_id.to_string(),
            node_type: node_type.clone(),
            name: display_name,
            position,
            status: NodeStatus::Pending,
            metadata: HashMap::new(),
            stats: NodeStats::default(),
        })
    }

    /// Create visual edge from graph edge
    fn create_visual_edge(&self, edge: &crate::edge::Edge) -> GraphResult<VisualEdge> {
        Ok(VisualEdge {
            id: format!("{}_{}", edge.source(), edge.target()),
            source: edge.source().to_string(),
            target: edge.target().to_string(),
            edge_type: "default".to_string(),
            condition: None,
            execution_count: 0,
        })
    }

    /// Apply layout algorithm to position nodes
    fn apply_layout(&mut self, nodes: &mut [VisualNode]) -> GraphResult<()> {
        match self.layout_algorithm {
            LayoutAlgorithm::Hierarchical => self.apply_hierarchical_layout(nodes),
            LayoutAlgorithm::ForceDirected => self.apply_force_directed_layout(nodes),
            LayoutAlgorithm::Circular => self.apply_circular_layout(nodes),
            LayoutAlgorithm::Grid => self.apply_grid_layout(nodes),
            LayoutAlgorithm::Manual => Ok(()), // Use existing positions
        }
    }

    /// Apply hierarchical layout (top-down)
    fn apply_hierarchical_layout(&mut self, nodes: &mut [VisualNode]) -> GraphResult<()> {
        let spacing = self.styling.layout_config.node_spacing;
        let start_x = self.styling.layout_config.padding;
        let start_y = self.styling.layout_config.padding;

        for (i, node) in nodes.iter_mut().enumerate() {
            let x = start_x + (i as f64 % 4.0) * spacing;
            let y = start_y + (i as f64 / 4.0).floor() * spacing;
            
            node.position = (x, y);
            self.node_positions.insert(node.id.clone(), (x, y));
        }

        Ok(())
    }

    /// Apply force-directed layout
    fn apply_force_directed_layout(&mut self, nodes: &mut [VisualNode]) -> GraphResult<()> {
        // Simplified force-directed algorithm
        let center_x = self.styling.layout_config.canvas_width / 2.0;
        let center_y = self.styling.layout_config.canvas_height / 2.0;
        let radius = 200.0;

        for (i, node) in nodes.iter_mut().enumerate() {
            let angle = (i as f64 / nodes.len() as f64) * 2.0 * std::f64::consts::PI;
            let x = center_x + radius * angle.cos();
            let y = center_y + radius * angle.sin();
            
            node.position = (x, y);
            self.node_positions.insert(node.id.clone(), (x, y));
        }

        Ok(())
    }

    /// Apply circular layout
    fn apply_circular_layout(&mut self, nodes: &mut [VisualNode]) -> GraphResult<()> {
        let center_x = self.styling.layout_config.canvas_width / 2.0;
        let center_y = self.styling.layout_config.canvas_height / 2.0;
        let radius = 250.0;

        for (i, node) in nodes.iter_mut().enumerate() {
            let angle = (i as f64 / nodes.len() as f64) * 2.0 * std::f64::consts::PI;
            let x = center_x + radius * angle.cos();
            let y = center_y + radius * angle.sin();
            
            node.position = (x, y);
            self.node_positions.insert(node.id.clone(), (x, y));
        }

        Ok(())
    }

    /// Apply grid layout
    fn apply_grid_layout(&mut self, nodes: &mut [VisualNode]) -> GraphResult<()> {
        let spacing = self.styling.layout_config.node_spacing;
        let start_x = self.styling.layout_config.padding;
        let start_y = self.styling.layout_config.padding;
        let cols = 3;

        for (i, node) in nodes.iter_mut().enumerate() {
            let col = i % cols;
            let row = i / cols;
            let x = start_x + col as f64 * spacing;
            let y = start_y + row as f64 * spacing;
            
            node.position = (x, y);
            self.node_positions.insert(node.id.clone(), (x, y));
        }

        Ok(())
    }

    /// Determine node type from node ID or metadata
    fn determine_node_type(&self, node_id: &str) -> String {
        // Simple heuristic based on node ID
        if node_id.contains("agent") {
            "agent".to_string()
        } else if node_id.contains("tool") {
            "tool".to_string()
        } else if node_id.contains("routing") {
            "routing".to_string()
        } else if node_id.contains("quality") {
            "quality_gate".to_string()
        } else if node_id == "start" {
            "start".to_string()
        } else if node_id == "end" {
            "end".to_string()
        } else {
            "default".to_string()
        }
    }

    /// Format node name for display
    fn format_node_name(&self, node_id: &str) -> String {
        // Convert snake_case to Title Case
        node_id
            .split('_')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Update node status
    pub fn update_node_status(&mut self, workflow: &mut VisualWorkflow, node_id: &str, status: NodeStatus) {
        if let Some(node) = workflow.nodes.iter_mut().find(|n| n.id == node_id) {
            node.status = status;
        }
    }

    /// Update node statistics
    pub fn update_node_stats(&mut self, workflow: &mut VisualWorkflow, node_id: &str, stats: NodeStats) {
        if let Some(node) = workflow.nodes.iter_mut().find(|n| n.id == node_id) {
            node.stats = stats;
        }
    }

    /// Set layout algorithm
    pub fn set_layout_algorithm(&mut self, algorithm: LayoutAlgorithm) {
        self.layout_algorithm = algorithm;
    }

    /// Get styling configuration
    pub fn styling(&self) -> &VisualizationStyling {
        &self.styling
    }

    /// Update styling configuration
    pub fn set_styling(&mut self, styling: VisualizationStyling) {
        self.styling = styling;
    }

    /// Export workflow as SVG
    pub fn export_svg(&self, workflow: &VisualWorkflow) -> GraphResult<String> {
        let mut svg = String::new();
        
        // SVG header
        svg.push_str(&format!(
            r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">"#,
            self.styling.layout_config.canvas_width,
            self.styling.layout_config.canvas_height
        ));

        // Draw edges first (so they appear behind nodes)
        for edge in &workflow.edges {
            if let (Some(source), Some(target)) = (
                workflow.nodes.iter().find(|n| n.id == edge.source),
                workflow.nodes.iter().find(|n| n.id == edge.target)
            ) {
                svg.push_str(&format!(
                    r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="{}" marker-end="url(#arrowhead)" />"#,
                    source.position.0 + self.styling.node_style.default_width / 2.0,
                    source.position.1 + self.styling.node_style.default_height / 2.0,
                    target.position.0 + self.styling.node_style.default_width / 2.0,
                    target.position.1 + self.styling.node_style.default_height / 2.0,
                    self.styling.edge_style.default_color,
                    self.styling.edge_style.thickness
                ));
            }
        }

        // Draw nodes
        for node in &workflow.nodes {
            let color = self.styling.node_style.colors.get(&node.node_type)
                .unwrap_or(&"#CCCCCC".to_string());
            
            svg.push_str(&format!(
                r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" stroke="#333" stroke-width="1" rx="5" />"#,
                node.position.0,
                node.position.1,
                self.styling.node_style.default_width,
                self.styling.node_style.default_height,
                color
            ));

            // Add node label
            svg.push_str(&format!(
                r#"<text x="{}" y="{}" text-anchor="middle" font-family="{}" font-size="{}" fill="{}">{}</text>"#,
                node.position.0 + self.styling.node_style.default_width / 2.0,
                node.position.1 + self.styling.node_style.default_height / 2.0 + 4.0,
                self.styling.node_style.font.family,
                self.styling.node_style.font.size,
                self.styling.node_style.font.color,
                node.name
            ));
        }

        // Arrow marker definition
        svg.push_str(r#"<defs><marker id="arrowhead" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto"><polygon points="0 0, 10 3.5, 0 7" fill="#666" /></marker></defs>"#);

        svg.push_str("</svg>");
        Ok(svg)
    }
}

impl Default for GraphVisualizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_visualizer_creation() {
        let visualizer = GraphVisualizer::new();
        assert!(matches!(visualizer.layout_algorithm, LayoutAlgorithm::Hierarchical));
    }

    #[test]
    fn test_node_name_formatting() {
        let visualizer = GraphVisualizer::new();
        assert_eq!(visualizer.format_node_name("agent_node"), "Agent Node");
        assert_eq!(visualizer.format_node_name("quality_gate"), "Quality Gate");
    }

    #[test]
    fn test_node_type_determination() {
        let visualizer = GraphVisualizer::new();
        assert_eq!(visualizer.determine_node_type("agent_researcher"), "agent");
        assert_eq!(visualizer.determine_node_type("tool_web_search"), "tool");
        assert_eq!(visualizer.determine_node_type("routing_coordinator"), "routing");
    }
}
