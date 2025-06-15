//! Core graph execution engine.

use crate::edge::routing::{EdgeResolver, RouteResolution};
use crate::error::{GraphError, GraphResult};
use crate::graph::{ExecutionContext, Graph};
use crate::node::{NodeExecutionContext, NodeId};
use crate::state::State;
use std::collections::HashSet;
use std::time::Duration;
use tokio::time::timeout;


#[cfg(feature = "streaming")]
use crate::streaming::ExecutionEvent;

/// Graph execution engine
#[derive(Debug)]
pub struct GraphEngine<S>
where
    S: State + serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    /// Edge resolver for routing decisions
    edge_resolver: EdgeResolver<S>,
}

impl<S> GraphEngine<S>
where
    S: State + serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    /// Create a new graph engine
    pub fn new() -> Self {
        Self {
            edge_resolver: EdgeResolver::new(),
        }
    }

    /// Execute a graph with the given state
    pub async fn execute(&mut self, graph: &Graph<S>, state: &mut S) -> GraphResult<ExecutionContext> {
        // Validate the graph
        graph.validate()?;

        // Create execution context
        let mut context = ExecutionContext::new();
        
        // Get entry point
        let entry_point = graph.entry_point()
            .ok_or_else(|| GraphError::graph_structure("No entry point defined".to_string()))?
            .clone();

        #[cfg(feature = "streaming")]
        if let Some(ref emitter) = graph.event_emitter {
            emitter.emit_graph_started(context.execution_id, entry_point.clone())?;
        }

        // Start execution from entry point
        let start_time = std::time::Instant::now();
        let result = self.execute_from_node(graph, state, &mut context, entry_point).await;
        let duration_ms = start_time.elapsed().as_millis() as u64;

        #[cfg(feature = "streaming")]
        if let Some(ref emitter) = graph.event_emitter {
            emitter.emit_graph_completed(
                context.execution_id,
                context.current_node.clone(),
                duration_ms,
                result.is_ok(),
            )?;
        }

        result?;
        Ok(context)
    }

    /// Execute starting from a specific node
    async fn execute_from_node(
        &mut self,
        graph: &Graph<S>,
        state: &mut S,
        context: &mut ExecutionContext,
        start_node: NodeId,
    ) -> GraphResult<()> {
        let mut current_node = start_node;
        let mut visited_nodes = HashSet::new();
        let config = graph.config();

        loop {
            // Check execution limits
            if let Some(max_steps) = config.max_steps {
                if context.current_step >= max_steps {
                    return Err(GraphError::execution_error(format!(
                        "Maximum steps ({}) exceeded",
                        max_steps
                    )));
                }
            }

            if let Some(max_time) = config.max_execution_time_seconds {
                if context.duration().num_seconds() as u64 >= max_time {
                    return Err(GraphError::timeout(max_time));
                }
            }

            // Check for cycles (simple detection)
            if visited_nodes.contains(&current_node) {
                tracing::warn!(
                    node_id = %current_node,
                    "Potential cycle detected, continuing execution"
                );
            }
            visited_nodes.insert(current_node.clone());

            // Update context
            context.current_node = Some(current_node.clone());
            context.add_to_path(current_node.clone());
            context.increment_step();

            // Execute the current node
            self.execute_node(graph, state, context, &current_node).await?;

            // Check if we've reached a finish point AFTER executing the node
            if graph.finish_points().contains(&current_node) {
                tracing::info!(
                    node_id = %current_node,
                    steps = context.current_step,
                    "Reached finish point"
                );
                break;
            }

            // Find next node(s)
            let next_nodes = self.find_next_nodes(graph, state, &current_node).await?;

            match next_nodes {
                RouteResolution::Single(next_node) => {
                    current_node = next_node;
                }
                RouteResolution::Multiple(nodes) => {
                    // Execute nodes in parallel
                    if config.enable_parallel {
                        self.execute_parallel_nodes(graph, state, context, nodes).await?;
                    } else {
                        // Execute sequentially if parallel is disabled
                        for node in nodes {
                            self.execute_node(graph, state, context, &node).await?;
                        }
                    }
                    // For parallel execution, we need to determine the next step
                    // This is a simplified approach - in practice, you might want
                    // more sophisticated merging logic
                    break;
                }
                RouteResolution::None => {
                    tracing::info!(
                        node_id = %current_node,
                        "No outgoing edges, execution complete"
                    );
                    break;
                }
            }
        }

        Ok(())
    }

    /// Execute a single node
    async fn execute_node(
        &self,
        graph: &Graph<S>,
        state: &mut S,
        context: &ExecutionContext,
        node_id: &NodeId,
    ) -> GraphResult<()> {
        let node = graph.node_registry()
            .get(node_id)
            .ok_or_else(|| GraphError::node_error(
                node_id.clone(),
                "Node not found in registry".to_string(),
                None,
            ))?;

        let mut node_context = NodeExecutionContext::new(node_id.clone());
        
        #[cfg(feature = "streaming")]
        if let Some(ref emitter) = graph.event_emitter {
            emitter.emit_node_started(
                context.execution_id,
                node_id.clone(),
                node_context.clone(),
            )?;
        }

        tracing::info!(
            node_id = %node_id,
            execution_id = %context.execution_id,
            step = context.current_step,
            "Executing node"
        );

        // Execute with timeout if configured
        let result = if let Some(timeout_seconds) = graph.config().max_execution_time_seconds {
            let timeout_duration = Duration::from_secs(timeout_seconds);
            match timeout(timeout_duration, node.invoke(state)).await {
                Ok(result) => result,
                Err(_) => {
                    let error = GraphError::timeout(timeout_seconds);
                    node_context.mark_failure(error.to_string());
                    return Err(error);
                }
            }
        } else {
            node.invoke(state).await
        };

        // Handle result
        match result {
            Ok(()) => {
                node_context.mark_success();
                
                #[cfg(feature = "streaming")]
                if let Some(ref emitter) = graph.event_emitter {
                    emitter.emit_node_completed(
                        context.execution_id,
                        node_id.clone(),
                        node_context.duration_ms.unwrap_or(0),
                        true,
                        None,
                    )?;
                    
                    emitter.emit_state_updated(
                        context.execution_id,
                        node_id.clone(),
                        None, // TODO: Add snapshot ID if checkpointing is enabled
                    )?;
                }

                tracing::info!(
                    node_id = %node_id,
                    duration_ms = node_context.duration_ms.unwrap_or(0),
                    "Node executed successfully"
                );
            }
            Err(error) => {
                node_context.mark_failure(error.to_string());
                
                #[cfg(feature = "streaming")]
                if let Some(ref emitter) = graph.event_emitter {
                    emitter.emit_node_completed(
                        context.execution_id,
                        node_id.clone(),
                        node_context.duration_ms.unwrap_or(0),
                        false,
                        Some(error.to_string()),
                    )?;
                    
                    emitter.emit_error(
                        context.execution_id,
                        Some(node_id.clone()),
                        error.to_string(),
                        error.category().to_string(),
                    )?;
                }

                tracing::error!(
                    node_id = %node_id,
                    error = %error,
                    "Node execution failed"
                );

                if graph.config().stop_on_error {
                    return Err(error);
                }
            }
        }

        Ok(())
    }

    /// Execute multiple nodes in parallel
    async fn execute_parallel_nodes(
        &self,
        graph: &Graph<S>,
        state: &mut S,
        context: &ExecutionContext,
        node_ids: Vec<NodeId>,
    ) -> GraphResult<()> {
        #[cfg(feature = "streaming")]
        if let Some(ref emitter) = graph.event_emitter {
            emitter.emit(ExecutionEvent::ParallelStarted {
                execution_id: context.execution_id,
                node_ids: node_ids.clone(),
                timestamp: chrono::Utc::now(),
            })?;
        }

        let start_time = std::time::Instant::now();
        
        // Clone state for each parallel execution
        let mut tasks = Vec::new();
        
        for node_id in &node_ids {
            let mut state_clone = state.clone();
            let node = graph.node_registry()
                .get(node_id)
                .ok_or_else(|| GraphError::node_error(
                    node_id.clone(),
                    "Node not found in registry".to_string(),
                    None,
                ))?;

            // Create a task for each node
            let node_id_clone = node_id.clone();
            let task = async move {
                let result = node.invoke(&mut state_clone).await;
                (node_id_clone, result, state_clone)
            };
            
            tasks.push(task);
        }

        // Execute all tasks concurrently
        let results = futures::future::join_all(tasks).await;
        
        // Process results
        let mut success_count = 0;
        let mut node_results = Vec::new();
        
        for (node_id, result, updated_state) in results {
            let success = result.is_ok();
            node_results.push((node_id.clone(), success));
            
            if success {
                success_count += 1;
                // For now, we'll use the last successful state update
                // In practice, you might want a more sophisticated merging strategy
                *state = updated_state;
            } else if graph.config().stop_on_error {
                return result;
            }
        }

        let duration_ms = start_time.elapsed().as_millis() as u64;

        #[cfg(feature = "streaming")]
        if let Some(ref emitter) = graph.event_emitter {
            emitter.emit(ExecutionEvent::ParallelCompleted {
                execution_id: context.execution_id,
                results: node_results,
                timestamp: chrono::Utc::now(),
                duration_ms,
            })?;
        }

        tracing::info!(
            parallel_nodes = node_ids.len(),
            successful_nodes = success_count,
            duration_ms = duration_ms,
            "Parallel execution completed"
        );

        Ok(())
    }

    /// Find the next nodes to execute
    async fn find_next_nodes(
        &mut self,
        graph: &Graph<S>,
        state: &S,
        current_node: &NodeId,
    ) -> GraphResult<RouteResolution> {
        // Find edges from the current node
        let outgoing_edges: Vec<_> = graph.edges()
            .iter()
            .filter(|edge| edge.from == *current_node)
            .collect();

        if outgoing_edges.is_empty() {
            return Ok(RouteResolution::None);
        }

        // For now, take the first edge (in practice, you might want priority-based selection)
        let edge = outgoing_edges[0];

        // For simple edges, just return the target directly
        match &edge.edge_type {
            crate::edge::EdgeType::Simple { target } => {
                Ok(RouteResolution::Single(target.clone()))
            }
            crate::edge::EdgeType::Parallel { targets } => {
                if targets.is_empty() {
                    Ok(RouteResolution::None)
                } else {
                    Ok(RouteResolution::Multiple(targets.clone()))
                }
            }
            _ => {
                // For complex edges, use the resolver
                self.edge_resolver.resolve_edge(edge, state).await
            }
        }
    }
}

impl<S> Default for GraphEngine<S>
where
    S: State + serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edge::Edge;
    use crate::graph::GraphBuilder;
    use crate::node::Node;
    use async_trait::async_trait;

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct TestState {
        value: i32,
    }

    #[derive(Debug)]
    struct IncrementNode {
        amount: i32,
    }

    #[async_trait]
    impl Node<TestState> for IncrementNode {
        async fn invoke(&self, state: &mut TestState) -> GraphResult<()> {
            state.value += self.amount;
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_graph_execution() {
        let node1 = IncrementNode { amount: 5 };
        let node2 = IncrementNode { amount: 3 };

        let graph = GraphBuilder::new()
            .add_node("node1".to_string(), node1).unwrap()
            .add_node("node2".to_string(), node2).unwrap()
            .with_entry_point("node1".to_string()).unwrap()
            .add_finish_point("node2".to_string()).unwrap()
            .add_edge(Edge::simple("node1", "node2")).unwrap()
            .build().unwrap();

        let mut engine = GraphEngine::new();
        let mut state = TestState { value: 0 };

        let context = engine.execute(&graph, &mut state).await.unwrap();
        
        assert_eq!(state.value, 8); // 0 + 5 + 3
        assert_eq!(context.current_step, 2);
        assert_eq!(context.execution_path.len(), 2);
    }
}
