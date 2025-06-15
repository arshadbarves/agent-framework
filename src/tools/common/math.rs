// Mathematical computation tools

use crate::tools::traits::{Tool, ToolError, ToolInput, ToolMetadata, ToolOutput, ToolResult};
use async_trait::async_trait;
use serde_json::json;

/// Tool for basic mathematical calculations
#[derive(Debug)]
pub struct CalculatorTool {
    metadata: ToolMetadata,
}

impl CalculatorTool {
    /// Create a new calculator tool
    pub fn new() -> Self {
        let metadata = ToolMetadata::new(
            "calculator",
            "Calculator",
            "Perform basic mathematical calculations"
        )
        .with_tag("math")
        .with_tag("calculation")
        .with_tag("utility")
        .with_deterministic(true)
        .with_side_effects(false)
        .with_estimated_duration_ms(10);
        
        Self { metadata }
    }
}

#[async_trait]
impl Tool for CalculatorTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(&self, input: ToolInput) -> ToolResult<ToolOutput> {
        let expression = input.data.as_str()
            .ok_or_else(|| ToolError::ValidationError {
                message: "Mathematical expression is required".to_string(),
            })?;

        // Simple calculator - in a real implementation, this would use a proper expression parser
        let result = match expression {
            expr if expr.contains('+') => {
                let parts: Vec<&str> = expr.split('+').collect();
                if parts.len() == 2 {
                    let a: f64 = parts[0].trim().parse().map_err(|_| ToolError::ValidationError {
                        message: "Invalid number format".to_string(),
                    })?;
                    let b: f64 = parts[1].trim().parse().map_err(|_| ToolError::ValidationError {
                        message: "Invalid number format".to_string(),
                    })?;
                    a + b
                } else {
                    return Err(ToolError::ValidationError {
                        message: "Invalid expression format".to_string(),
                    });
                }
            }
            expr if expr.contains('-') => {
                let parts: Vec<&str> = expr.split('-').collect();
                if parts.len() == 2 {
                    let a: f64 = parts[0].trim().parse().map_err(|_| ToolError::ValidationError {
                        message: "Invalid number format".to_string(),
                    })?;
                    let b: f64 = parts[1].trim().parse().map_err(|_| ToolError::ValidationError {
                        message: "Invalid number format".to_string(),
                    })?;
                    a - b
                } else {
                    return Err(ToolError::ValidationError {
                        message: "Invalid expression format".to_string(),
                    });
                }
            }
            expr if expr.contains('*') => {
                let parts: Vec<&str> = expr.split('*').collect();
                if parts.len() == 2 {
                    let a: f64 = parts[0].trim().parse().map_err(|_| ToolError::ValidationError {
                        message: "Invalid number format".to_string(),
                    })?;
                    let b: f64 = parts[1].trim().parse().map_err(|_| ToolError::ValidationError {
                        message: "Invalid number format".to_string(),
                    })?;
                    a * b
                } else {
                    return Err(ToolError::ValidationError {
                        message: "Invalid expression format".to_string(),
                    });
                }
            }
            expr if expr.contains('/') => {
                let parts: Vec<&str> = expr.split('/').collect();
                if parts.len() == 2 {
                    let a: f64 = parts[0].trim().parse().map_err(|_| ToolError::ValidationError {
                        message: "Invalid number format".to_string(),
                    })?;
                    let b: f64 = parts[1].trim().parse().map_err(|_| ToolError::ValidationError {
                        message: "Invalid number format".to_string(),
                    })?;
                    if b == 0.0 {
                        return Err(ToolError::ExecutionError {
                            message: "Division by zero".to_string(),
                        });
                    }
                    a / b
                } else {
                    return Err(ToolError::ValidationError {
                        message: "Invalid expression format".to_string(),
                    });
                }
            }
            _ => {
                // Try to parse as a single number
                expression.trim().parse().map_err(|_| ToolError::ValidationError {
                    message: "Invalid expression or number format".to_string(),
                })?
            }
        };

        let output = ToolOutput::new(json!({
            "expression": expression,
            "result": result
        }))
        .with_metadata("expression", expression)
        .with_metric("result", result);

        Ok(output)
    }

    async fn validate_input(&self, input: &ToolInput) -> ToolResult<()> {
        if input.data.as_str().is_none() {
            return Err(ToolError::ValidationError {
                message: "Mathematical expression is required".to_string(),
            });
        }
        Ok(())
    }
}

/// Tool for statistical calculations
#[derive(Debug)]
pub struct StatisticsTool {
    metadata: ToolMetadata,
}

impl StatisticsTool {
    /// Create a new statistics tool
    pub fn new() -> Self {
        let metadata = ToolMetadata::new(
            "statistics",
            "Statistics Calculator",
            "Calculate statistical measures for datasets"
        )
        .with_tag("math")
        .with_tag("statistics")
        .with_tag("analysis")
        .with_deterministic(true)
        .with_side_effects(false)
        .with_estimated_duration_ms(50);
        
        Self { metadata }
    }
}

#[async_trait]
impl Tool for StatisticsTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(&self, input: ToolInput) -> ToolResult<ToolOutput> {
        let numbers = input.data.as_array()
            .ok_or_else(|| ToolError::ValidationError {
                message: "Array of numbers is required".to_string(),
            })?;

        let values: Result<Vec<f64>, _> = numbers
            .iter()
            .map(|v| v.as_f64().ok_or_else(|| ToolError::ValidationError {
                message: "All array elements must be numbers".to_string(),
            }))
            .collect();

        let values = values?;

        if values.is_empty() {
            return Err(ToolError::ValidationError {
                message: "Array cannot be empty".to_string(),
            });
        }

        let count = values.len() as f64;
        let sum: f64 = values.iter().sum();
        let mean = sum / count;
        
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        // Calculate variance and standard deviation
        let variance = values.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / count;
        let std_dev = variance.sqrt();

        // Calculate median
        let mut sorted_values = values.clone();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = if sorted_values.len() % 2 == 0 {
            let mid = sorted_values.len() / 2;
            (sorted_values[mid - 1] + sorted_values[mid]) / 2.0
        } else {
            sorted_values[sorted_values.len() / 2]
        };

        let output = ToolOutput::new(json!({
            "count": count,
            "sum": sum,
            "mean": mean,
            "median": median,
            "min": min,
            "max": max,
            "variance": variance,
            "standard_deviation": std_dev
        }))
        .with_metadata("dataset_size", count.to_string())
        .with_metric("count", count)
        .with_metric("mean", mean)
        .with_metric("std_dev", std_dev);

        Ok(output)
    }

    async fn validate_input(&self, input: &ToolInput) -> ToolResult<()> {
        let numbers = input.data.as_array()
            .ok_or_else(|| ToolError::ValidationError {
                message: "Array of numbers is required".to_string(),
            })?;

        if numbers.is_empty() {
            return Err(ToolError::ValidationError {
                message: "Array cannot be empty".to_string(),
            });
        }

        for (i, value) in numbers.iter().enumerate() {
            if !value.is_number() {
                return Err(ToolError::ValidationError {
                    message: format!("Element at index {} is not a number", i),
                });
            }
        }

        Ok(())
    }
}
