//! High-level graph execution utilities and convenience methods.

use crate::error::GraphResult;
use crate::graph::{ExecutionContext, Graph};
use crate::graph::engine::GraphEngine;
use crate::state::State;

#[cfg(feature = "streaming")]
use crate::streaming::{ExecutionStream, create_execution_stream, EventEmitter};

#[cfg(feature = "checkpointing")]
use crate::state::checkpointing::Checkpointer;

impl<S> Graph<S>
where
    S: State + serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    /// Execute the graph with the given state
    pub async fn run(&self, state: &mut S) -> GraphResult<ExecutionContext> {
        let mut engine = GraphEngine::new();
        engine.execute(self, state).await
    }

    /// Execute the graph and return both the final state and execution context
    pub async fn run_with_context(&self, mut state: S) -> GraphResult<(S, ExecutionContext)> {
        let context = self.run(&mut state).await?;
        Ok((state, context))
    }

    #[cfg(feature = "streaming")]
    /// Execute the graph with streaming events
    pub async fn run_streaming(&mut self, state: &mut S) -> GraphResult<(ExecutionContext, ExecutionStream)> {
        // Create event emitter if not already set
        if self.event_emitter.is_none() {
            let (emitter, receiver) = EventEmitter::new();
            let stream = create_execution_stream(receiver);
            self.set_event_emitter(emitter);
            
            // Execute the graph
            let context = self.run(state).await?;
            Ok((context, stream))
        } else {
            // If emitter is already set, we can't create a new stream
            // This is a limitation of the current design
            let context = self.run(state).await?;
            // Return an empty stream as a placeholder
            let (_, receiver) = tokio::sync::mpsc::unbounded_channel();
            let stream = create_execution_stream(receiver);
            Ok((context, stream))
        }
    }

    #[cfg(feature = "checkpointing")]
    /// Execute the graph with automatic checkpointing
    pub async fn run_with_checkpointing<C>(
        &mut self,
        state: &mut S,
        checkpointer: C,
    ) -> GraphResult<ExecutionContext>
    where
        C: Checkpointer<S> + 'static,
    {
        self.set_checkpointer(checkpointer);
        self.run(state).await
    }

    /// Validate that the graph can be executed
    pub fn can_execute(&self) -> GraphResult<()> {
        self.validate()
    }

    /// Get execution statistics for the graph
    pub fn execution_stats(&self) -> ExecutionStats {
        ExecutionStats {
            node_count: self.node_ids().len(),
            edge_count: self.edges().len(),
            has_entry_point: self.entry_point().is_some(),
            finish_point_count: self.finish_points().len(),
            has_parallel_edges: self.edges().iter().any(|e| e.is_parallel_safe()),
            estimated_complexity: self.estimate_complexity(),
        }
    }

    /// Estimate the computational complexity of the graph
    fn estimate_complexity(&self) -> ComplexityEstimate {
        let node_count = self.node_ids().len();
        let edge_count = self.edges().len();
        
        // Simple heuristic for complexity estimation
        let complexity_score = node_count + edge_count * 2;
        
        match complexity_score {
            0..=10 => ComplexityEstimate::Low,
            11..=50 => ComplexityEstimate::Medium,
            51..=200 => ComplexityEstimate::High,
            _ => ComplexityEstimate::VeryHigh,
        }
    }

    /// Get a summary of the graph structure
    pub fn summary(&self) -> GraphSummary {
        GraphSummary {
            name: self.metadata().name.clone(),
            version: self.metadata().version.clone(),
            node_count: self.node_ids().len(),
            edge_count: self.edges().len(),
            entry_point: self.entry_point().cloned(),
            finish_points: self.finish_points().to_vec(),
            tags: self.metadata().tags.clone(),
            complexity: self.estimate_complexity(),
        }
    }
}

/// Statistics about graph execution capabilities
#[derive(Debug, Clone)]
pub struct ExecutionStats {
    /// Number of nodes in the graph
    pub node_count: usize,
    /// Number of edges in the graph
    pub edge_count: usize,
    /// Whether the graph has an entry point
    pub has_entry_point: bool,
    /// Number of finish points
    pub finish_point_count: usize,
    /// Whether the graph has parallel edges
    pub has_parallel_edges: bool,
    /// Estimated computational complexity
    pub estimated_complexity: ComplexityEstimate,
}

/// Complexity estimate for graph execution
#[derive(Debug, Clone, PartialEq)]
pub enum ComplexityEstimate {
    /// Low complexity (simple linear graphs)
    Low,
    /// Medium complexity (some branching)
    Medium,
    /// High complexity (complex branching and parallel execution)
    High,
    /// Very high complexity (large graphs with many nodes and edges)
    VeryHigh,
}

/// Summary of graph structure
#[derive(Debug, Clone)]
pub struct GraphSummary {
    /// Graph name
    pub name: String,
    /// Graph version
    pub version: String,
    /// Number of nodes
    pub node_count: usize,
    /// Number of edges
    pub edge_count: usize,
    /// Entry point node ID
    pub entry_point: Option<String>,
    /// Finish point node IDs
    pub finish_points: Vec<String>,
    /// Graph tags
    pub tags: Vec<String>,
    /// Complexity estimate
    pub complexity: ComplexityEstimate,
}

impl std::fmt::Display for GraphSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Graph '{}' v{}: {} nodes, {} edges, complexity: {:?}",
            self.name, self.version, self.node_count, self.edge_count, self.complexity
        )
    }
}

/// Execution result with detailed information
#[derive(Debug, Clone)]
pub struct ExecutionResult<S> {
    /// Final state after execution
    pub state: S,
    /// Execution context with timing and path information
    pub context: ExecutionContext,
    /// Whether execution was successful
    pub success: bool,
    /// Error message if execution failed
    pub error: Option<String>,
    /// Execution statistics
    pub stats: ExecutionResultStats,
}

/// Statistics about a completed execution
#[derive(Debug, Clone)]
pub struct ExecutionResultStats {
    /// Total execution time in milliseconds
    pub total_duration_ms: u64,
    /// Number of nodes executed
    pub nodes_executed: usize,
    /// Number of steps taken
    pub steps_taken: u64,
    /// Average time per node in milliseconds
    pub avg_node_duration_ms: f64,
    /// Whether parallel execution was used
    pub used_parallel_execution: bool,
    /// Number of checkpoints created (if checkpointing was enabled)
    pub checkpoints_created: usize,
}

impl<S> ExecutionResult<S> {
    /// Create a successful execution result
    pub fn success(state: S, context: ExecutionContext) -> Self {
        let stats = ExecutionResultStats {
            total_duration_ms: context.duration_ms(),
            nodes_executed: context.execution_path.len(),
            steps_taken: context.current_step,
            avg_node_duration_ms: if context.execution_path.is_empty() {
                0.0
            } else {
                context.duration_ms() as f64 / context.execution_path.len() as f64
            },
            used_parallel_execution: false, // TODO: Track this in context
            checkpoints_created: 0, // TODO: Track this in context
        };

        Self {
            state,
            context,
            success: true,
            error: None,
            stats,
        }
    }

    /// Create a failed execution result
    pub fn failure(state: S, context: ExecutionContext, error: String) -> Self {
        let stats = ExecutionResultStats {
            total_duration_ms: context.duration_ms(),
            nodes_executed: context.execution_path.len(),
            steps_taken: context.current_step,
            avg_node_duration_ms: if context.execution_path.is_empty() {
                0.0
            } else {
                context.duration_ms() as f64 / context.execution_path.len() as f64
            },
            used_parallel_execution: false,
            checkpoints_created: 0,
        };

        Self {
            state,
            context,
            success: false,
            error: Some(error),
            stats,
        }
    }

    /// Get the execution duration
    pub fn duration(&self) -> chrono::Duration {
        self.context.duration()
    }

    /// Check if execution was successful
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Get the error message if execution failed
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}

/// Convenience function to execute a graph and get detailed results
pub async fn execute_graph<S>(
    graph: &Graph<S>,
    state: S,
) -> ExecutionResult<S>
where
    S: State + serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    let mut state = state;
    match graph.run(&mut state).await {
        Ok(context) => ExecutionResult::success(state, context),
        Err(error) => {
            let context = ExecutionContext::new(); // Create a minimal context for error case
            ExecutionResult::failure(state, context, error.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::graph::GraphBuilder;
    use crate::node::Node;
    use async_trait::async_trait;

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct TestState {
        value: i32,
    }



    #[derive(Debug)]
    struct TestNode {
        increment: i32,
    }

    #[async_trait]
    impl Node<TestState> for TestNode {
        async fn invoke(&self, state: &mut TestState) -> GraphResult<()> {
            state.value += self.increment;
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_graph_run() {
        let node = TestNode { increment: 5 };
        let graph = GraphBuilder::new()
            .add_node("test".to_string(), node).unwrap()
            .with_entry_point("test".to_string()).unwrap()
            .add_finish_point("test".to_string()).unwrap()
            .build().unwrap();

        let mut state = TestState { value: 0 };
        let context = graph.run(&mut state).await.unwrap();

        assert_eq!(state.value, 5);
        assert_eq!(context.current_step, 1);
    }

    #[tokio::test]
    async fn test_execution_result() {
        let node = TestNode { increment: 10 };
        let graph = GraphBuilder::new()
            .add_node("test".to_string(), node).unwrap()
            .with_entry_point("test".to_string()).unwrap()
            .add_finish_point("test".to_string()).unwrap()
            .build().unwrap();

        let state = TestState { value: 0 };
        let result = execute_graph(&graph, state).await;

        assert!(result.is_success());
        assert_eq!(result.state.value, 10);
        assert_eq!(result.stats.nodes_executed, 1);
    }

    #[test]
    fn test_graph_summary() {
        let node = TestNode { increment: 1 };
        let graph = GraphBuilder::new()
            .add_node("test".to_string(), node).unwrap()
            .with_entry_point("test".to_string()).unwrap()
            .add_finish_point("test".to_string()).unwrap()
            .build().unwrap();

        let summary = graph.summary();
        assert_eq!(summary.node_count, 1);
        assert_eq!(summary.edge_count, 0);
        assert_eq!(summary.complexity, ComplexityEstimate::Low);
    }
}
