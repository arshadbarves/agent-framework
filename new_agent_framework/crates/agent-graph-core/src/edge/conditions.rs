//! Edge condition evaluation system.

use crate::error::{CoreError, CoreResult};
use crate::state::State;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Trait for edge conditions
#[async_trait]
pub trait EdgeCondition<S>: Send + Sync + std::fmt::Debug
where
    S: State,
{
    /// Evaluate the condition against the current state
    async fn evaluate(&self, state: &S) -> CoreResult<bool>;

    /// Get a description of this condition
    fn description(&self) -> String;

    /// Get condition metadata
    fn metadata(&self) -> HashMap<String, serde_json::Value> {
        HashMap::new()
    }
}

/// Always true condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlwaysCondition;

#[async_trait]
impl<S> EdgeCondition<S> for AlwaysCondition
where
    S: State,
{
    async fn evaluate(&self, _state: &S) -> CoreResult<bool> {
        Ok(true)
    }

    fn description(&self) -> String {
        "Always true".to_string()
    }
}

/// Always false condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeverCondition;

#[async_trait]
impl<S> EdgeCondition<S> for NeverCondition
where
    S: State,
{
    async fn evaluate(&self, _state: &S) -> CoreResult<bool> {
        Ok(false)
    }

    fn description(&self) -> String {
        "Always false".to_string()
    }
}

/// Function-based condition
pub struct FunctionCondition<S, F>
where
    S: State,
    F: Fn(&S) -> CoreResult<bool> + Send + Sync,
{
    function: F,
    description: String,
    _phantom: std::marker::PhantomData<S>,
}

impl<S, F> std::fmt::Debug for FunctionCondition<S, F>
where
    S: State,
    F: Fn(&S) -> CoreResult<bool> + Send + Sync,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FunctionCondition")
            .field("description", &self.description)
            .finish()
    }
}

impl<S, F> FunctionCondition<S, F>
where
    S: State,
    F: Fn(&S) -> CoreResult<bool> + Send + Sync,
{
    /// Create a new function condition
    pub fn new(function: F, description: String) -> Self {
        Self {
            function,
            description,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<S, F> EdgeCondition<S> for FunctionCondition<S, F>
where
    S: State,
    F: Fn(&S) -> CoreResult<bool> + Send + Sync,
{
    async fn evaluate(&self, state: &S) -> CoreResult<bool> {
        (self.function)(state)
    }

    fn description(&self) -> String {
        self.description.clone()
    }
}

/// Composite AND condition
pub struct AndCondition<S>
where
    S: State,
{
    conditions: Vec<Box<dyn EdgeCondition<S>>>,
}

impl<S> std::fmt::Debug for AndCondition<S>
where
    S: State,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AndCondition")
            .field("condition_count", &self.conditions.len())
            .finish()
    }
}

impl<S> AndCondition<S>
where
    S: State,
{
    /// Create a new AND condition
    pub fn new(conditions: Vec<Box<dyn EdgeCondition<S>>>) -> Self {
        Self { conditions }
    }
}

#[async_trait]
impl<S> EdgeCondition<S> for AndCondition<S>
where
    S: State,
{
    async fn evaluate(&self, state: &S) -> CoreResult<bool> {
        for condition in &self.conditions {
            if !condition.evaluate(state).await? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn description(&self) -> String {
        let descriptions: Vec<_> = self.conditions.iter().map(|c| c.description()).collect();
        format!("AND({})", descriptions.join(", "))
    }
}

/// Composite OR condition
pub struct OrCondition<S>
where
    S: State,
{
    conditions: Vec<Box<dyn EdgeCondition<S>>>,
}

impl<S> std::fmt::Debug for OrCondition<S>
where
    S: State,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OrCondition")
            .field("condition_count", &self.conditions.len())
            .finish()
    }
}

impl<S> OrCondition<S>
where
    S: State,
{
    /// Create a new OR condition
    pub fn new(conditions: Vec<Box<dyn EdgeCondition<S>>>) -> Self {
        Self { conditions }
    }
}

#[async_trait]
impl<S> EdgeCondition<S> for OrCondition<S>
where
    S: State,
{
    async fn evaluate(&self, state: &S) -> CoreResult<bool> {
        for condition in &self.conditions {
            if condition.evaluate(state).await? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn description(&self) -> String {
        let descriptions: Vec<_> = self.conditions.iter().map(|c| c.description()).collect();
        format!("OR({})", descriptions.join(", "))
    }
}

/// NOT condition wrapper
pub struct NotCondition<S>
where
    S: State,
{
    condition: Box<dyn EdgeCondition<S>>,
}

impl<S> std::fmt::Debug for NotCondition<S>
where
    S: State,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NotCondition")
            .field("inner_condition", &self.condition.description())
            .finish()
    }
}

impl<S> NotCondition<S>
where
    S: State,
{
    /// Create a new NOT condition
    pub fn new(condition: Box<dyn EdgeCondition<S>>) -> Self {
        Self { condition }
    }
}

#[async_trait]
impl<S> EdgeCondition<S> for NotCondition<S>
where
    S: State,
{
    async fn evaluate(&self, state: &S) -> CoreResult<bool> {
        Ok(!self.condition.evaluate(state).await?)
    }

    fn description(&self) -> String {
        format!("NOT({})", self.condition.description())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestState {
        value: i32,
        flag: bool,
    }

    #[tokio::test]
    async fn test_always_condition() {
        let condition = AlwaysCondition;
        let state = TestState { value: 42, flag: true };
        
        assert!(condition.evaluate(&state).await.unwrap());
    }

    #[tokio::test]
    async fn test_never_condition() {
        let condition = NeverCondition;
        let state = TestState { value: 42, flag: true };
        
        assert!(!condition.evaluate(&state).await.unwrap());
    }

    #[tokio::test]
    async fn test_function_condition() {
        let condition = FunctionCondition::new(
            |state: &TestState| Ok(state.value > 10),
            "value > 10".to_string(),
        );
        
        let state1 = TestState { value: 42, flag: true };
        let state2 = TestState { value: 5, flag: true };
        
        assert!(condition.evaluate(&state1).await.unwrap());
        assert!(!condition.evaluate(&state2).await.unwrap());
    }

    #[tokio::test]
    async fn test_and_condition() {
        let condition1 = Box::new(FunctionCondition::new(
            |state: &TestState| Ok(state.value > 10),
            "value > 10".to_string(),
        ));
        let condition2 = Box::new(FunctionCondition::new(
            |state: &TestState| Ok(state.flag),
            "flag is true".to_string(),
        ));
        
        let and_condition = AndCondition::new(vec![condition1, condition2]);
        
        let state1 = TestState { value: 42, flag: true };
        let state2 = TestState { value: 42, flag: false };
        let state3 = TestState { value: 5, flag: true };
        
        assert!(and_condition.evaluate(&state1).await.unwrap());
        assert!(!and_condition.evaluate(&state2).await.unwrap());
        assert!(!and_condition.evaluate(&state3).await.unwrap());
    }
}