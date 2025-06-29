//! Human input collection and management.

use crate::{CoreError, CoreResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

/// Trait for collecting human input
#[async_trait]
pub trait HumanInputCollector: Send + Sync + std::fmt::Debug {
    /// Request input from a human
    async fn request_input(&self, request: InputRequest) -> CoreResult<InputResponse>;

    /// Check if the collector is available
    async fn is_available(&self) -> bool;

    /// Get collector metadata
    fn metadata(&self) -> &InputCollectorMetadata;
}

/// Metadata for input collectors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputCollectorMetadata {
    /// Collector ID
    pub id: String,
    /// Collector name
    pub name: String,
    /// Collector type
    pub collector_type: InputCollectorType,
    /// Supported input types
    pub supported_types: Vec<InputType>,
    /// Whether the collector supports real-time interaction
    pub real_time: bool,
    /// Maximum response time
    pub max_response_time: Option<Duration>,
}

/// Type of input collector
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InputCollectorType {
    /// Console/terminal input
    Console,
    /// Web-based input
    Web,
    /// Chat interface
    Chat,
    /// Email-based input
    Email,
    /// Custom collector type
    Custom(String),
}

/// Type of input being requested
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InputType {
    /// Free text input
    Text,
    /// Yes/No choice
    Boolean,
    /// Multiple choice selection
    Choice,
    /// Numeric input
    Number,
    /// File upload
    File,
    /// Custom input type
    Custom(String),
}

/// Request for human input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputRequest {
    /// Unique request ID
    pub id: String,
    /// Type of input requested
    pub input_type: InputType,
    /// Prompt or question for the human
    pub prompt: String,
    /// Additional context or instructions
    pub context: Option<String>,
    /// Available choices (for choice input type)
    pub choices: Option<Vec<String>>,
    /// Default value
    pub default_value: Option<serde_json::Value>,
    /// Whether the input is required
    pub required: bool,
    /// Timeout for the request
    pub timeout: Option<Duration>,
    /// Priority level
    pub priority: InputPriority,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Priority level for input requests
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum InputPriority {
    /// Low priority - can wait
    Low,
    /// Normal priority
    Normal,
    /// High priority - needs attention soon
    High,
    /// Critical priority - needs immediate attention
    Critical,
}

/// Response from human input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputResponse {
    /// Request ID this response is for
    pub request_id: String,
    /// The input value provided
    pub value: Option<serde_json::Value>,
    /// Whether the request was cancelled
    pub cancelled: bool,
    /// Whether the request timed out
    pub timed_out: bool,
    /// Response timestamp
    pub timestamp: SystemTime,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Console-based input collector
#[derive(Debug)]
pub struct ConsoleInputCollector {
    metadata: InputCollectorMetadata,
}

impl ConsoleInputCollector {
    /// Create a new console input collector
    pub fn new() -> Self {
        let metadata = InputCollectorMetadata {
            id: "console".to_string(),
            name: "Console Input Collector".to_string(),
            collector_type: InputCollectorType::Console,
            supported_types: vec![
                InputType::Text,
                InputType::Boolean,
                InputType::Choice,
                InputType::Number,
            ],
            real_time: true,
            max_response_time: None,
        };

        Self { metadata }
    }
}

impl Default for ConsoleInputCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl HumanInputCollector for ConsoleInputCollector {
    async fn request_input(&self, request: InputRequest) -> CoreResult<InputResponse> {
        use std::io::{self, Write};

        println!("\n=== Human Input Required ===");
        println!("Request ID: {}", request.id);
        println!("Priority: {:?}", request.priority);
        println!("Prompt: {}", request.prompt);
        
        if let Some(context) = &request.context {
            println!("Context: {}", context);
        }

        match request.input_type {
            InputType::Text => {
                print!("Enter text: ");
                io::stdout().flush().unwrap();
                
                let mut input = String::new();
                io::stdin().read_line(&mut input).map_err(|e| {
                    CoreError::execution_error(format!("Failed to read input: {}", e))
                })?;
                
                let value = input.trim().to_string();
                Ok(InputResponse {
                    request_id: request.id,
                    value: Some(serde_json::Value::String(value)),
                    cancelled: false,
                    timed_out: false,
                    timestamp: SystemTime::now(),
                    metadata: HashMap::new(),
                })
            }
            InputType::Boolean => {
                print!("Enter y/n: ");
                io::stdout().flush().unwrap();
                
                let mut input = String::new();
                io::stdin().read_line(&mut input).map_err(|e| {
                    CoreError::execution_error(format!("Failed to read input: {}", e))
                })?;
                
                let value = match input.trim().to_lowercase().as_str() {
                    "y" | "yes" | "true" | "1" => true,
                    "n" | "no" | "false" | "0" => false,
                    _ => {
                        return Err(CoreError::validation_error("Invalid boolean input"));
                    }
                };
                
                Ok(InputResponse {
                    request_id: request.id,
                    value: Some(serde_json::Value::Bool(value)),
                    cancelled: false,
                    timed_out: false,
                    timestamp: SystemTime::now(),
                    metadata: HashMap::new(),
                })
            }
            InputType::Choice => {
                if let Some(choices) = &request.choices {
                    println!("Available choices:");
                    for (i, choice) in choices.iter().enumerate() {
                        println!("  {}: {}", i + 1, choice);
                    }
                    
                    print!("Enter choice number: ");
                    io::stdout().flush().unwrap();
                    
                    let mut input = String::new();
                    io::stdin().read_line(&mut input).map_err(|e| {
                        CoreError::execution_error(format!("Failed to read input: {}", e))
                    })?;
                    
                    let choice_num: usize = input.trim().parse().map_err(|_| {
                        CoreError::validation_error("Invalid choice number")
                    })?;
                    
                    if choice_num == 0 || choice_num > choices.len() {
                        return Err(CoreError::validation_error("Choice number out of range"));
                    }
                    
                    let selected_choice = choices[choice_num - 1].clone();
                    Ok(InputResponse {
                        request_id: request.id,
                        value: Some(serde_json::Value::String(selected_choice)),
                        cancelled: false,
                        timed_out: false,
                        timestamp: SystemTime::now(),
                        metadata: HashMap::new(),
                    })
                } else {
                    Err(CoreError::validation_error("No choices provided for choice input"))
                }
            }
            InputType::Number => {
                print!("Enter number: ");
                io::stdout().flush().unwrap();
                
                let mut input = String::new();
                io::stdin().read_line(&mut input).map_err(|e| {
                    CoreError::execution_error(format!("Failed to read input: {}", e))
                })?;
                
                let number: f64 = input.trim().parse().map_err(|_| {
                    CoreError::validation_error("Invalid number format")
                })?;
                
                Ok(InputResponse {
                    request_id: request.id,
                    value: Some(serde_json::json!(number)),
                    cancelled: false,
                    timed_out: false,
                    timestamp: SystemTime::now(),
                    metadata: HashMap::new(),
                })
            }
            _ => {
                Err(CoreError::execution_error(format!(
                    "Unsupported input type: {:?}",
                    request.input_type
                )))
            }
        }
    }

    async fn is_available(&self) -> bool {
        true // Console is always available
    }

    fn metadata(&self) -> &InputCollectorMetadata {
        &self.metadata
    }
}

/// Input manager for coordinating multiple collectors
#[derive(Debug)]
pub struct InputManager {
    /// Registered collectors
    collectors: HashMap<String, Box<dyn HumanInputCollector>>,
    /// Default collector ID
    default_collector: Option<String>,
    /// Pending requests
    pending_requests: HashMap<String, PendingRequest>,
}

/// Pending input request
#[derive(Debug)]
struct PendingRequest {
    request: InputRequest,
    response_sender: oneshot::Sender<CoreResult<InputResponse>>,
    created_at: SystemTime,
}

impl InputManager {
    /// Create a new input manager
    pub fn new() -> Self {
        Self {
            collectors: HashMap::new(),
            default_collector: None,
            pending_requests: HashMap::new(),
        }
    }

    /// Register an input collector
    pub fn register_collector(&mut self, collector: Box<dyn HumanInputCollector>) {
        let id = collector.metadata().id.clone();
        self.collectors.insert(id.clone(), collector);
        
        // Set as default if it's the first collector
        if self.default_collector.is_none() {
            self.default_collector = Some(id);
        }
    }

    /// Set the default collector
    pub fn set_default_collector(&mut self, collector_id: String) -> CoreResult<()> {
        if !self.collectors.contains_key(&collector_id) {
            return Err(CoreError::configuration_error(format!(
                "Collector not found: {}",
                collector_id
            )));
        }
        self.default_collector = Some(collector_id);
        Ok(())
    }

    /// Request input using the default collector
    pub async fn request_input(&mut self, request: InputRequest) -> CoreResult<InputResponse> {
        let collector_id = self.default_collector.as_ref()
            .ok_or_else(|| CoreError::configuration_error("No default collector set"))?;
        
        self.request_input_from(request, collector_id).await
    }

    /// Request input from a specific collector
    pub async fn request_input_from(
        &mut self,
        request: InputRequest,
        collector_id: &str,
    ) -> CoreResult<InputResponse> {
        let collector = self.collectors.get(collector_id)
            .ok_or_else(|| CoreError::configuration_error(format!(
                "Collector not found: {}",
                collector_id
            )))?;

        // Check if collector is available
        if !collector.is_available().await {
            return Err(CoreError::execution_error(format!(
                "Collector {} is not available",
                collector_id
            )));
        }

        // Execute the request
        collector.request_input(request).await
    }

    /// List available collectors
    pub fn list_collectors(&self) -> Vec<String> {
        self.collectors.keys().cloned().collect()
    }

    /// Get collector metadata
    pub fn get_collector_metadata(&self, collector_id: &str) -> Option<&InputCollectorMetadata> {
        self.collectors.get(collector_id).map(|c| c.metadata())
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}

impl InputRequest {
    /// Create a simple text input request
    pub fn text(prompt: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            input_type: InputType::Text,
            prompt,
            context: None,
            choices: None,
            default_value: None,
            required: true,
            timeout: None,
            priority: InputPriority::Normal,
            metadata: HashMap::new(),
        }
    }

    /// Create a boolean input request
    pub fn boolean(prompt: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            input_type: InputType::Boolean,
            prompt,
            context: None,
            choices: None,
            default_value: None,
            required: true,
            timeout: None,
            priority: InputPriority::Normal,
            metadata: HashMap::new(),
        }
    }

    /// Create a choice input request
    pub fn choice(prompt: String, choices: Vec<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            input_type: InputType::Choice,
            prompt,
            context: None,
            choices: Some(choices),
            default_value: None,
            required: true,
            timeout: None,
            priority: InputPriority::Normal,
            metadata: HashMap::new(),
        }
    }

    /// Set the priority
    pub fn with_priority(mut self, priority: InputPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set the timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set the context
    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }
}