//! Command-based routing system for dynamic workflow control
//! This module implements LangGraph-style Command routing for AgentGraph

use crate::error::GraphResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Command for controlling workflow execution (similar to LangGraph's Command)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    /// Continue to the next node in the workflow
    Continue,
    
    /// Go to a specific node with optional state updates
    Goto {
        /// Target node ID to route to
        node: String,
        /// State updates to apply before routing
        update: HashMap<String, serde_json::Value>,
    },
    
    /// End the workflow with optional final state updates
    End {
        /// Final state updates before ending
        update: HashMap<String, serde_json::Value>,
    },
    
    /// Conditional routing based on state
    Conditional {
        /// Condition to evaluate
        condition: String,
        /// Node to route to if condition is true
        if_true: String,
        /// Node to route to if condition is false
        if_false: String,
        /// State updates to apply
        update: HashMap<String, serde_json::Value>,
    },
    
    /// Parallel execution of multiple nodes
    Parallel {
        /// List of nodes to execute in parallel
        nodes: Vec<String>,
        /// State updates to apply
        update: HashMap<String, serde_json::Value>,
    },
}

impl Command {
    /// Create a Continue command
    pub fn continue_() -> Self {
        Self::Continue
    }
    
    /// Create a Goto command
    pub fn goto<S: Into<String>>(node: S) -> Self {
        Self::Goto {
            node: node.into(),
            update: HashMap::new(),
        }
    }
    
    /// Create a Goto command with state updates
    pub fn goto_with_update<S: Into<String>>(
        node: S, 
        update: HashMap<String, serde_json::Value>
    ) -> Self {
        Self::Goto {
            node: node.into(),
            update,
        }
    }
    
    /// Create an End command
    pub fn end() -> Self {
        Self::End {
            update: HashMap::new(),
        }
    }
    
    /// Create an End command with state updates
    pub fn end_with_update(update: HashMap<String, serde_json::Value>) -> Self {
        Self::End { update }
    }
    
    /// Create a Conditional command
    pub fn conditional<S: Into<String>>(
        condition: S,
        if_true: S,
        if_false: S,
    ) -> Self {
        Self::Conditional {
            condition: condition.into(),
            if_true: if_true.into(),
            if_false: if_false.into(),
            update: HashMap::new(),
        }
    }
    
    /// Create a Parallel command
    pub fn parallel(nodes: Vec<String>) -> Self {
        Self::Parallel {
            nodes,
            update: HashMap::new(),
        }
    }
    
    /// Add state update to any command
    pub fn with_update(mut self, key: String, value: serde_json::Value) -> Self {
        match &mut self {
            Command::Goto { update, .. } |
            Command::End { update } |
            Command::Conditional { update, .. } |
            Command::Parallel { update, .. } => {
                update.insert(key, value);
            }
            Command::Continue => {
                // Convert to Goto with current node (requires context)
                // This is handled by the execution engine
            }
        }
        self
    }
    
    /// Check if this command ends the workflow
    pub fn is_end(&self) -> bool {
        matches!(self, Command::End { .. })
    }
    
    /// Check if this command continues normal execution
    pub fn is_continue(&self) -> bool {
        matches!(self, Command::Continue)
    }
    
    /// Get the target node for routing commands
    pub fn target_node(&self) -> Option<&str> {
        match self {
            Command::Goto { node, .. } => Some(node),
            Command::Conditional { if_true, .. } => Some(if_true), // Default to if_true
            _ => None,
        }
    }
    
    /// Get state updates from the command
    pub fn state_updates(&self) -> &HashMap<String, serde_json::Value> {
        match self {
            Command::Goto { update, .. } |
            Command::End { update } |
            Command::Conditional { update, .. } |
            Command::Parallel { update, .. } => update,
            Command::Continue => &EMPTY_UPDATES,
        }
    }
}

/// Empty updates for Continue command
static EMPTY_UPDATES: HashMap<String, serde_json::Value> = HashMap::new();

/// Command parser for extracting commands from agent responses
pub struct CommandParser {
    /// Patterns for detecting commands in text
    patterns: HashMap<String, CommandPattern>,
}

/// Pattern for matching commands in text
#[derive(Debug, Clone)]
pub struct CommandPattern {
    /// Regex pattern to match
    pub pattern: String,
    /// Command type to create
    pub command_type: String,
    /// Extraction rules for parameters
    pub extractors: HashMap<String, String>,
}

impl CommandParser {
    /// Create a new command parser with default patterns
    pub fn new() -> Self {
        let mut patterns = HashMap::new();
        
        // GOTO command pattern
        patterns.insert("goto".to_string(), CommandPattern {
            pattern: r"GOTO:\s*(\w+)".to_string(),
            command_type: "goto".to_string(),
            extractors: HashMap::from([
                ("node".to_string(), r"GOTO:\s*(\w+)".to_string()),
            ]),
        });
        
        // END command pattern
        patterns.insert("end".to_string(), CommandPattern {
            pattern: r"END\b".to_string(),
            command_type: "end".to_string(),
            extractors: HashMap::new(),
        });
        
        // CONDITIONAL command pattern
        patterns.insert("conditional".to_string(), CommandPattern {
            pattern: r"IF\s+(.+?)\s+THEN\s+(\w+)\s+ELSE\s+(\w+)".to_string(),
            command_type: "conditional".to_string(),
            extractors: HashMap::from([
                ("condition".to_string(), r"IF\s+(.+?)\s+THEN".to_string()),
                ("if_true".to_string(), r"THEN\s+(\w+)".to_string()),
                ("if_false".to_string(), r"ELSE\s+(\w+)".to_string()),
            ]),
        });
        
        Self { patterns }
    }
    
    /// Parse a command from agent response text
    pub fn parse_command(&self, text: &str) -> GraphResult<Command> {
        // Check for GOTO pattern
        if text.contains("GOTO:") {
            if let Some(node) = self.extract_goto_target(text) {
                return Ok(Command::goto(node));
            }
        }
        
        // Check for END pattern
        if text.contains("END") {
            return Ok(Command::end());
        }
        
        // Check for CONDITIONAL pattern
        if text.contains("IF") && text.contains("THEN") && text.contains("ELSE") {
            if let Some((condition, if_true, if_false)) = self.extract_conditional(text) {
                return Ok(Command::conditional(condition, if_true, if_false));
            }
        }
        
        // Default to Continue
        Ok(Command::continue_())
    }
    
    /// Extract GOTO target from text
    fn extract_goto_target(&self, text: &str) -> Option<String> {
        // Simple extraction: GOTO: node_name
        if let Some(start) = text.find("GOTO:") {
            let after_goto = &text[start + 5..];
            if let Some(end) = after_goto.find(char::is_whitespace) {
                Some(after_goto[..end].trim().to_string())
            } else {
                Some(after_goto.trim().to_string())
            }
        } else {
            None
        }
    }
    
    /// Extract conditional routing from text
    fn extract_conditional(&self, text: &str) -> Option<(String, String, String)> {
        // Simple extraction: IF condition THEN node1 ELSE node2
        if let Some(if_pos) = text.find("IF") {
            if let Some(then_pos) = text.find("THEN") {
                if let Some(else_pos) = text.find("ELSE") {
                    let condition = text[if_pos + 2..then_pos].trim().to_string();
                    let if_true_part = &text[then_pos + 4..else_pos];
                    let if_false_part = &text[else_pos + 4..];
                    
                    let if_true = if_true_part.split_whitespace().next()?.to_string();
                    let if_false = if_false_part.split_whitespace().next()?.to_string();
                    
                    return Some((condition, if_true, if_false));
                }
            }
        }
        None
    }
    
    /// Add a custom command pattern
    pub fn add_pattern(&mut self, name: String, pattern: CommandPattern) {
        self.patterns.insert(name, pattern);
    }
}

impl Default for CommandParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Command execution context
#[derive(Debug, Clone)]
pub struct CommandContext {
    /// Current node ID
    pub current_node: String,
    /// Available nodes in the graph
    pub available_nodes: Vec<String>,
    /// Execution history
    pub execution_path: Vec<String>,
}

impl CommandContext {
    /// Create a new command context
    pub fn new(current_node: String, available_nodes: Vec<String>) -> Self {
        Self {
            current_node,
            available_nodes,
            execution_path: Vec::new(),
        }
    }
    
    /// Validate that a command can be executed
    pub fn validate_command(&self, command: &Command) -> GraphResult<()> {
        match command {
            Command::Goto { node, .. } => {
                if !self.available_nodes.contains(node) {
                    return Err(crate::error::GraphError::validation_error(
                        format!("Target node '{}' not found in graph", node)
                    ));
                }
            }
            Command::Conditional { if_true, if_false, .. } => {
                if !self.available_nodes.contains(if_true) {
                    return Err(crate::error::GraphError::validation_error(
                        format!("Conditional target node '{}' not found in graph", if_true)
                    ));
                }
                if !self.available_nodes.contains(if_false) {
                    return Err(crate::error::GraphError::validation_error(
                        format!("Conditional target node '{}' not found in graph", if_false)
                    ));
                }
            }
            Command::Parallel { nodes, .. } => {
                for node in nodes {
                    if !self.available_nodes.contains(node) {
                        return Err(crate::error::GraphError::validation_error(
                            format!("Parallel target node '{}' not found in graph", node)
                        ));
                    }
                }
            }
            _ => {} // Continue and End are always valid
        }
        Ok(())
    }
    
    /// Add a node to the execution path
    pub fn add_to_path(&mut self, node: String) {
        self.execution_path.push(node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_creation() {
        let cmd = Command::goto("next_node");
        assert!(matches!(cmd, Command::Goto { .. }));
        assert_eq!(cmd.target_node(), Some("next_node"));
        
        let cmd = Command::end();
        assert!(cmd.is_end());
        
        let cmd = Command::continue_();
        assert!(cmd.is_continue());
    }

    #[test]
    fn test_command_parser() {
        let parser = CommandParser::new();
        
        // Test GOTO parsing
        let cmd = parser.parse_command("I think we should GOTO: review_node").unwrap();
        assert_eq!(cmd.target_node(), Some("review_node"));
        
        // Test END parsing
        let cmd = parser.parse_command("The task is complete. END").unwrap();
        assert!(cmd.is_end());
        
        // Test default Continue
        let cmd = parser.parse_command("Just continue with normal processing").unwrap();
        assert!(cmd.is_continue());
    }

    #[test]
    fn test_command_context() {
        let nodes = vec!["node1".to_string(), "node2".to_string(), "node3".to_string()];
        let context = CommandContext::new("node1".to_string(), nodes);
        
        let cmd = Command::goto("node2");
        assert!(context.validate_command(&cmd).is_ok());
        
        let cmd = Command::goto("invalid_node");
        assert!(context.validate_command(&cmd).is_err());
    }
}
