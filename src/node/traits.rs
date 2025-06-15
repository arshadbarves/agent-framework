//! Additional traits and utilities for nodes.

use crate::error::GraphResult;
use crate::node::{Node, NodeMetadata};
use crate::state::State;
use async_trait::async_trait;
use std::time::Duration;

/// Trait for nodes that can be retried on failure
#[async_trait]
pub trait RetryableNode<S>: Node<S>
where
    S: State,
{
    /// Maximum number of retry attempts
    fn max_retries(&self) -> u32 {
        3
    }

    /// Delay between retry attempts
    fn retry_delay(&self) -> Duration {
        Duration::from_millis(1000)
    }

    /// Determine if an error is retryable
    fn is_retryable_error(&self, error: &crate::error::GraphError) -> bool {
        error.is_recoverable()
    }

    /// Execute with retry logic
    async fn invoke_with_retry(&self, state: &mut S) -> GraphResult<()> {
        let mut attempts = 0;
        let max_attempts = self.max_retries() + 1; // +1 for initial attempt

        loop {
            attempts += 1;
            
            match self.invoke(state).await {
                Ok(()) => {
                    if attempts > 1 {
                        tracing::info!(
                            node_type = self.node_type(),
                            attempts = attempts,
                            "Node succeeded after retry"
                        );
                    }
                    return Ok(());
                }
                Err(error) => {
                    if attempts >= max_attempts || !self.is_retryable_error(&error) {
                        tracing::error!(
                            node_type = self.node_type(),
                            attempts = attempts,
                            error = %error,
                            "Node failed after all retry attempts"
                        );
                        return Err(error);
                    }

                    tracing::warn!(
                        node_type = self.node_type(),
                        attempt = attempts,
                        max_attempts = max_attempts,
                        error = %error,
                        "Node failed, retrying"
                    );

                    tokio::time::sleep(self.retry_delay()).await;
                }
            }
        }
    }
}

/// Trait for nodes that can timeout
#[async_trait]
pub trait TimeoutNode<S>: Node<S>
where
    S: State,
{
    /// Timeout duration for node execution
    fn timeout(&self) -> Duration {
        Duration::from_secs(30)
    }

    /// Execute with timeout
    async fn invoke_with_timeout(&self, state: &mut S) -> GraphResult<()> {
        let timeout_duration = self.timeout();
        
        match tokio::time::timeout(timeout_duration, self.invoke(state)).await {
            Ok(result) => result,
            Err(_) => {
                tracing::error!(
                    node_type = self.node_type(),
                    timeout_seconds = timeout_duration.as_secs(),
                    "Node execution timed out"
                );
                Err(crate::error::GraphError::timeout(timeout_duration.as_secs()))
            }
        }
    }
}

/// Trait for nodes that can be cached
#[async_trait]
pub trait CacheableNode<S>: Node<S>
where
    S: State + Clone + PartialEq,
{
    /// Generate a cache key for the given state
    fn cache_key(&self, state: &S) -> String;

    /// Check if the result should be cached
    fn should_cache(&self, _state: &S) -> bool {
        true
    }

    /// Cache TTL in seconds
    fn cache_ttl(&self) -> Option<u64> {
        Some(3600) // 1 hour default
    }
}

/// Trait for nodes that can validate their input state
#[async_trait]
pub trait ValidatingNode<S>: Node<S>
where
    S: State,
{
    /// Validate the input state before execution
    async fn validate_input(&self, state: &S) -> GraphResult<()>;

    /// Validate the output state after execution
    async fn validate_output(&self, state: &S) -> GraphResult<()>;

    /// Execute with validation
    async fn invoke_with_validation(&self, state: &mut S) -> GraphResult<()> {
        // Validate input
        self.validate_input(state).await?;
        
        // Clone state for comparison if needed
        let _original_state = state.clone();
        
        // Execute the node
        let result = self.invoke(state).await;
        
        // Validate output only if execution was successful
        if result.is_ok() {
            self.validate_output(state).await?;
        }
        
        result
    }
}

/// Trait for nodes that can be monitored
pub trait MonitorableNode<S>: Node<S>
where
    S: State,
{
    /// Get metrics about this node
    fn metrics(&self) -> NodeMetrics {
        NodeMetrics::default()
    }

    /// Record execution metrics
    fn record_execution(&self, _duration: Duration, _success: bool) {
        // Default implementation does nothing
        // Implementations can override to record metrics
    }
}

/// Metrics for a node
#[derive(Debug, Clone, Default)]
pub struct NodeMetrics {
    /// Total number of executions
    pub execution_count: u64,
    /// Number of successful executions
    pub success_count: u64,
    /// Number of failed executions
    pub failure_count: u64,
    /// Average execution time in milliseconds
    pub avg_execution_time_ms: f64,
    /// Minimum execution time in milliseconds
    pub min_execution_time_ms: u64,
    /// Maximum execution time in milliseconds
    pub max_execution_time_ms: u64,
}

impl NodeMetrics {
    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        if self.execution_count == 0 {
            0.0
        } else {
            self.success_count as f64 / self.execution_count as f64
        }
    }

    /// Calculate failure rate
    pub fn failure_rate(&self) -> f64 {
        1.0 - self.success_rate()
    }
}

/// Trait for nodes that can be configured
pub trait ConfigurableNode<S>: Node<S>
where
    S: State,
{
    /// Configuration type for this node
    type Config: Clone + Send + Sync;

    /// Apply configuration to the node
    fn configure(&mut self, config: Self::Config) -> GraphResult<()>;

    /// Get current configuration
    fn config(&self) -> &Self::Config;
}

/// Trait for nodes that can be composed with other nodes
#[async_trait]
pub trait ComposableNode<S>: Node<S>
where
    S: State,
{
    /// Compose this node with another node in sequence
    fn then<N>(self, next: N) -> SequentialComposition<S, Self, N>
    where
        Self: Sized + 'static,
        N: Node<S> + 'static,
    {
        SequentialComposition::new(self, next)
    }

    /// Compose this node with another node in parallel
    fn parallel<N>(self, other: N) -> ParallelComposition<S, Self, N>
    where
        Self: Sized + 'static,
        N: Node<S> + 'static,
        S: Clone,
    {
        ParallelComposition::new(self, other)
    }
}

/// Automatic implementation for all nodes
impl<S, T> ComposableNode<S> for T
where
    S: State,
    T: Node<S>,
{
}

/// Sequential composition of two nodes
#[derive(Debug)]
pub struct SequentialComposition<S, A, B>
where
    S: State,
    A: Node<S>,
    B: Node<S>,
{
    first: A,
    second: B,
    _phantom: std::marker::PhantomData<S>,
}

impl<S, A, B> SequentialComposition<S, A, B>
where
    S: State,
    A: Node<S>,
    B: Node<S>,
{
    /// Create a new sequential composition
    pub fn new(first: A, second: B) -> Self {
        Self {
            first,
            second,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<S, A, B> Node<S> for SequentialComposition<S, A, B>
where
    S: State,
    A: Node<S>,
    B: Node<S>,
{
    async fn invoke(&self, state: &mut S) -> GraphResult<()> {
        self.first.invoke(state).await?;
        self.second.invoke(state).await?;
        Ok(())
    }

    fn metadata(&self) -> NodeMetadata {
        NodeMetadata::new("SequentialComposition")
            .with_description("Sequential composition of two nodes")
            .with_tag("composition")
    }
}

/// Parallel composition of two nodes
#[derive(Debug)]
pub struct ParallelComposition<S, A, B>
where
    S: State + Clone,
    A: Node<S>,
    B: Node<S>,
{
    first: A,
    second: B,
    _phantom: std::marker::PhantomData<S>,
}

impl<S, A, B> ParallelComposition<S, A, B>
where
    S: State + Clone,
    A: Node<S>,
    B: Node<S>,
{
    /// Create a new parallel composition
    pub fn new(first: A, second: B) -> Self {
        Self {
            first,
            second,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<S, A, B> Node<S> for ParallelComposition<S, A, B>
where
    S: State + Clone,
    A: Node<S>,
    B: Node<S>,
{
    async fn invoke(&self, state: &mut S) -> GraphResult<()> {
        let mut state_a = state.clone();
        let mut state_b = state.clone();

        let (result_a, result_b) = tokio::join!(
            self.first.invoke(&mut state_a),
            self.second.invoke(&mut state_b)
        );

        result_a?;
        result_b?;

        // For parallel composition, we need a strategy to merge states
        // This is a simplified approach - in practice, you might want
        // a more sophisticated merging strategy
        *state = state_a; // Use the first node's result for now

        Ok(())
    }

    fn metadata(&self) -> NodeMetadata {
        NodeMetadata::new("ParallelComposition")
            .with_description("Parallel composition of two nodes")
            .with_tag("composition")
            .with_parallel_safe(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[derive(Debug, Clone, PartialEq)]
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

    #[async_trait]
    impl RetryableNode<TestState> for IncrementNode {
        fn max_retries(&self) -> u32 {
            2
        }

        fn retry_delay(&self) -> Duration {
            Duration::from_millis(10)
        }
    }

    #[tokio::test]
    async fn test_sequential_composition() {
        let node1 = IncrementNode { amount: 5 };
        let node2 = IncrementNode { amount: 3 };
        let composed = node1.then(node2);

        let mut state = TestState { value: 0 };
        composed.invoke(&mut state).await.unwrap();
        assert_eq!(state.value, 8);
    }

    #[tokio::test]
    async fn test_parallel_composition() {
        let node1 = IncrementNode { amount: 5 };
        let node2 = IncrementNode { amount: 3 };
        let composed = node1.parallel(node2);

        let mut state = TestState { value: 0 };
        composed.invoke(&mut state).await.unwrap();
        // With our simple merging strategy, we get the first node's result
        assert_eq!(state.value, 5);
    }
}
