// Human input collection system

use super::traits::{HumanInput, HumanResponse, HumanResult, InteractionError, HumanInteraction};
use super::{HumanContext, HumanConfig};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
// use tokio::sync::oneshot;

/// Request for human input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputRequest {
    /// Unique request ID
    pub request_id: String,
    /// Human input specification
    pub input: HumanInput,
    /// Context for the request
    pub context: HumanContext,
    /// Configuration for the request
    pub config: HumanConfig,
}

impl InputRequest {
    /// Create a new input request
    pub fn new(
        request_id: String,
        input: HumanInput,
        context: HumanContext,
        config: HumanConfig,
    ) -> Self {
        Self {
            request_id,
            input,
            context,
            config,
        }
    }
}

/// Validator for human input
pub trait InputValidator: Send + Sync + std::fmt::Debug {
    /// Validate the input value
    fn validate(&self, value: &serde_json::Value) -> HumanResult<()>;
    
    /// Get validation error message
    fn error_message(&self) -> String;
}

/// Required field validator
#[derive(Debug)]
pub struct RequiredValidator;

impl InputValidator for RequiredValidator {
    fn validate(&self, value: &serde_json::Value) -> HumanResult<()> {
        if value.is_null() {
            Err(InteractionError::ValidationError {
                message: self.error_message(),
            })
        } else {
            Ok(())
        }
    }
    
    fn error_message(&self) -> String {
        "This field is required".to_string()
    }
}

/// String length validator
#[derive(Debug)]
pub struct LengthValidator {
    min_length: Option<usize>,
    max_length: Option<usize>,
}

impl LengthValidator {
    /// Create a new length validator
    pub fn new(min_length: Option<usize>, max_length: Option<usize>) -> Self {
        Self { min_length, max_length }
    }
    
    /// Create a minimum length validator
    pub fn min(min_length: usize) -> Self {
        Self::new(Some(min_length), None)
    }
    
    /// Create a maximum length validator
    pub fn max(max_length: usize) -> Self {
        Self::new(None, Some(max_length))
    }
    
    /// Create a range length validator
    pub fn range(min_length: usize, max_length: usize) -> Self {
        Self::new(Some(min_length), Some(max_length))
    }
}

impl InputValidator for LengthValidator {
    fn validate(&self, value: &serde_json::Value) -> HumanResult<()> {
        let text = value.as_str().ok_or_else(|| InteractionError::ValidationError {
            message: "Value must be a string".to_string(),
        })?;
        
        let length = text.len();
        
        if let Some(min) = self.min_length {
            if length < min {
                return Err(InteractionError::ValidationError {
                    message: format!("Text must be at least {} characters long", min),
                });
            }
        }
        
        if let Some(max) = self.max_length {
            if length > max {
                return Err(InteractionError::ValidationError {
                    message: format!("Text must be at most {} characters long", max),
                });
            }
        }
        
        Ok(())
    }
    
    fn error_message(&self) -> String {
        match (self.min_length, self.max_length) {
            (Some(min), Some(max)) => format!("Text must be between {} and {} characters", min, max),
            (Some(min), None) => format!("Text must be at least {} characters", min),
            (None, Some(max)) => format!("Text must be at most {} characters", max),
            (None, None) => "Invalid text length".to_string(),
        }
    }
}

/// Numeric range validator
#[derive(Debug)]
pub struct NumericValidator {
    min_value: Option<f64>,
    max_value: Option<f64>,
}

impl NumericValidator {
    /// Create a new numeric validator
    pub fn new(min_value: Option<f64>, max_value: Option<f64>) -> Self {
        Self { min_value, max_value }
    }
    
    /// Create a minimum value validator
    pub fn min(min_value: f64) -> Self {
        Self::new(Some(min_value), None)
    }
    
    /// Create a maximum value validator
    pub fn max(max_value: f64) -> Self {
        Self::new(None, Some(max_value))
    }
    
    /// Create a range validator
    pub fn range(min_value: f64, max_value: f64) -> Self {
        Self::new(Some(min_value), Some(max_value))
    }
}

impl InputValidator for NumericValidator {
    fn validate(&self, value: &serde_json::Value) -> HumanResult<()> {
        let number = value.as_f64().ok_or_else(|| InteractionError::ValidationError {
            message: "Value must be a number".to_string(),
        })?;
        
        if let Some(min) = self.min_value {
            if number < min {
                return Err(InteractionError::ValidationError {
                    message: format!("Value must be at least {}", min),
                });
            }
        }
        
        if let Some(max) = self.max_value {
            if number > max {
                return Err(InteractionError::ValidationError {
                    message: format!("Value must be at most {}", max),
                });
            }
        }
        
        Ok(())
    }
    
    fn error_message(&self) -> String {
        match (self.min_value, self.max_value) {
            (Some(min), Some(max)) => format!("Value must be between {} and {}", min, max),
            (Some(min), None) => format!("Value must be at least {}", min),
            (None, Some(max)) => format!("Value must be at most {}", max),
            (None, None) => "Invalid numeric value".to_string(),
        }
    }
}

/// Input collector for managing human input requests
#[derive(Debug)]
pub struct InputCollector {
    /// Registered validators
    validators: HashMap<String, Box<dyn InputValidator>>,
    /// Human interaction provider
    interaction_provider: Arc<dyn HumanInteraction>,
}

impl InputCollector {
    /// Create a new input collector
    pub fn new(interaction_provider: Arc<dyn HumanInteraction>) -> Self {
        let mut validators: HashMap<String, Box<dyn InputValidator>> = HashMap::new();
        
        // Register built-in validators
        validators.insert("required".to_string(), Box::new(RequiredValidator));
        
        Self {
            validators,
            interaction_provider,
        }
    }
    
    /// Register a custom validator
    pub fn register_validator(&mut self, name: String, validator: Box<dyn InputValidator>) {
        self.validators.insert(name, validator);
    }
    
    /// Collect input from human
    pub async fn collect_input(&self, request: InputRequest) -> HumanResult<HumanResponse> {
        // Validate the input request
        self.validate_request(&request.input).await?;
        
        // Request input from human
        let response = self.interaction_provider
            .request_input(request.input.clone(), &request.context, &request.config)
            .await?;
        
        // Validate the response
        self.validate_response(&request.input, &response)?;
        
        Ok(response)
    }
    
    /// Validate input request
    async fn validate_request(&self, input: &HumanInput) -> HumanResult<()> {
        // Use the interaction provider's validation
        self.interaction_provider.validate_input(input).await
    }
    
    /// Validate human response
    fn validate_response(&self, input: &HumanInput, response: &HumanResponse) -> HumanResult<()> {
        // Apply validation rules
        for (rule_name, rule_value) in &input.validation_rules {
            if let Some(validator) = self.validators.get(rule_name) {
                // Check if rule is enabled
                if rule_value.as_bool().unwrap_or(false) {
                    validator.validate(&response.value)?;
                }
            }
        }
        
        // Apply type-specific validation
        match input.interaction_type {
            super::traits::InteractionType::MultipleChoice => {
                if let Some(options) = &input.options {
                    let selected = response.value.as_str().ok_or_else(|| {
                        InteractionError::ValidationError {
                            message: "Multiple choice response must be a string".to_string(),
                        }
                    })?;
                    
                    if !options.contains(&selected.to_string()) {
                        return Err(InteractionError::ValidationError {
                            message: format!("Invalid choice: {}. Valid options: {:?}", selected, options),
                        });
                    }
                }
            }
            _ => {} // Other types handled by custom validators
        }
        
        Ok(())
    }
    
    /// Create a simple text input request
    pub fn text_input(
        &self,
        request_id: String,
        prompt: String,
        context: HumanContext,
    ) -> InputRequest {
        let input = HumanInput::text_input(prompt);
        let config = HumanConfig::default();
        InputRequest::new(request_id, input, context, config)
    }
    
    /// Create an approval request
    pub fn approval(
        &self,
        request_id: String,
        prompt: String,
        context: HumanContext,
    ) -> InputRequest {
        let input = HumanInput::approval(prompt);
        let config = HumanConfig::default();
        InputRequest::new(request_id, input, context, config)
    }
    
    /// Create a multiple choice request
    pub fn multiple_choice(
        &self,
        request_id: String,
        prompt: String,
        options: Vec<String>,
        context: HumanContext,
    ) -> InputRequest {
        let input = HumanInput::multiple_choice(prompt, options);
        let config = HumanConfig::default();
        InputRequest::new(request_id, input, context, config)
    }
}

/// Console-based human interaction provider for testing
#[derive(Debug)]
pub struct ConsoleInteraction {
    name: String,
}

impl ConsoleInteraction {
    /// Create a new console interaction provider
    pub fn new() -> Self {
        Self {
            name: "console".to_string(),
        }
    }
}

impl Default for ConsoleInteraction {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl HumanInteraction for ConsoleInteraction {
    async fn request_input(
        &self,
        input: HumanInput,
        _context: &HumanContext,
        _config: &HumanConfig,
    ) -> HumanResult<HumanResponse> {
        use std::io::{self, Write};
        
        let start_time = std::time::Instant::now();
        
        // Display prompt
        println!("\n🤔 Human Input Required:");
        println!("   {}", input.prompt);
        
        if let Some(context) = &input.context {
            println!("   Context: {}", context);
        }
        
        // Handle different interaction types
        match input.interaction_type {
            super::traits::InteractionType::Approval => {
                println!("   Please respond with 'yes' or 'no':");
                print!("   > ");
                io::stdout().flush().unwrap();
                
                let mut response = String::new();
                io::stdin().read_line(&mut response).map_err(|e| {
                    InteractionError::CommunicationError {
                        message: format!("Failed to read input: {}", e),
                    }
                })?;
                
                let response = response.trim().to_lowercase();
                let value = match response.as_str() {
                    "yes" | "y" | "true" | "1" => serde_json::Value::Bool(true),
                    "no" | "n" | "false" | "0" => serde_json::Value::Bool(false),
                    _ => return Err(InteractionError::ValidationError {
                        message: "Please respond with 'yes' or 'no'".to_string(),
                    }),
                };
                
                let elapsed = start_time.elapsed().as_millis() as u64;
                Ok(HumanResponse::human(value, elapsed))
            }
            
            super::traits::InteractionType::MultipleChoice => {
                if let Some(options) = &input.options {
                    println!("   Options:");
                    for (i, option) in options.iter().enumerate() {
                        println!("   {}. {}", i + 1, option);
                    }
                    print!("   Choose (1-{}): ", options.len());
                    io::stdout().flush().unwrap();
                    
                    let mut response = String::new();
                    io::stdin().read_line(&mut response).map_err(|e| {
                        InteractionError::CommunicationError {
                            message: format!("Failed to read input: {}", e),
                        }
                    })?;
                    
                    let choice: usize = response.trim().parse().map_err(|_| {
                        InteractionError::ValidationError {
                            message: "Please enter a valid number".to_string(),
                        }
                    })?;
                    
                    if choice == 0 || choice > options.len() {
                        return Err(InteractionError::ValidationError {
                            message: format!("Please choose between 1 and {}", options.len()),
                        });
                    }
                    
                    let selected = &options[choice - 1];
                    let elapsed = start_time.elapsed().as_millis() as u64;
                    Ok(HumanResponse::human(serde_json::Value::String(selected.clone()), elapsed))
                } else {
                    Err(InteractionError::ValidationError {
                        message: "Multiple choice options not provided".to_string(),
                    })
                }
            }
            
            _ => {
                print!("   > ");
                io::stdout().flush().unwrap();
                
                let mut response = String::new();
                io::stdin().read_line(&mut response).map_err(|e| {
                    InteractionError::CommunicationError {
                        message: format!("Failed to read input: {}", e),
                    }
                })?;
                
                let response = response.trim();
                let elapsed = start_time.elapsed().as_millis() as u64;
                Ok(HumanResponse::human(serde_json::Value::String(response.to_string()), elapsed))
            }
        }
    }
    
    async fn is_available(&self) -> bool {
        true // Console is always available
    }
    
    async fn cancel_interaction(&self, _interaction_id: &str) -> HumanResult<()> {
        // Console interactions can't be cancelled
        Ok(())
    }
    
    fn provider_name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_required_validator() {
        let validator = RequiredValidator;
        
        assert!(validator.validate(&json!("test")).is_ok());
        assert!(validator.validate(&json!(42)).is_ok());
        assert!(validator.validate(&json!(null)).is_err());
    }

    #[test]
    fn test_length_validator() {
        let validator = LengthValidator::range(3, 10);
        
        assert!(validator.validate(&json!("hello")).is_ok());
        assert!(validator.validate(&json!("hi")).is_err()); // Too short
        assert!(validator.validate(&json!("this is too long")).is_err()); // Too long
        assert!(validator.validate(&json!(42)).is_err()); // Not a string
    }

    #[test]
    fn test_numeric_validator() {
        let validator = NumericValidator::range(0.0, 100.0);
        
        assert!(validator.validate(&json!(50.0)).is_ok());
        assert!(validator.validate(&json!(42)).is_ok());
        assert!(validator.validate(&json!(-1.0)).is_err()); // Too small
        assert!(validator.validate(&json!(101.0)).is_err()); // Too large
        assert!(validator.validate(&json!("not a number")).is_err()); // Not a number
    }

    #[test]
    fn test_input_request_creation() {
        let input = HumanInput::text_input("Enter your name:".to_string());
        let context = HumanContext::new("test_interaction".to_string());
        let config = HumanConfig::default();
        
        let request = InputRequest::new("req_1".to_string(), input, context, config);
        
        assert_eq!(request.request_id, "req_1");
        assert_eq!(request.input.prompt, "Enter your name:");
    }
}
