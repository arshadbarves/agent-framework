# AgentGraph API Reference

This document provides a comprehensive reference for the AgentGraph API.

## Core Modules

### `agent_graph::agents`

#### `Agent`

The main agent struct that represents an intelligent agent.

```rust
pub struct Agent {
    // Internal fields...
}

impl Agent {
    /// Create a new agent
    pub fn new(
        config: AgentConfig,
        llm_manager: Arc<LLMManager>,
        tool_registry: Arc<ToolRegistry>,
        tool_executor: Arc<ToolExecutor>,
    ) -> Result<Self, AgentError>;

    /// Execute a task
    pub async fn execute_task(&mut self, task: String) -> Result<String, AgentError>;

    /// Get agent state
    pub fn state(&self) -> &AgentState;

    /// Get mutable reference to memory
    pub fn memory_mut(&mut self) -> &mut AgentMemory;

    /// Get reference to memory
    pub fn memory(&self) -> &AgentMemory;
}
```

#### `AgentConfig`

Configuration for creating an agent.

```rust
pub struct AgentConfig {
    pub name: String,
    pub role: String,
    pub system_prompt: String,
    pub provider: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: Option<u32>,
    pub tools: Vec<String>,
    pub memory_config: MemoryConfig,
}
```

#### `AgentMemory`

Memory system for agents.

```rust
impl AgentMemory {
    /// Store an interaction
    pub async fn store_interaction(
        &mut self,
        input: &str,
        output: &str,
    ) -> Result<(), MemoryError>;

    /// Get relevant context
    pub async fn get_relevant_context(&mut self, query: &str) -> Result<String, MemoryError>;

    /// Get memory statistics
    pub fn get_stats(&self) -> MemoryStats;

    /// Clear memory
    pub async fn clear(&mut self) -> Result<(), MemoryError>;
}
```

### `agent_graph::llm`

#### `LLMManager`

Manages multiple LLM providers.

```rust
impl LLMManager {
    /// Create a new LLM manager
    pub fn new(config: LLMConfig) -> Self;

    /// Register a provider
    pub fn register_provider(&mut self, name: String, provider: Arc<dyn LLMProvider>);

    /// Get a provider
    pub fn get_provider(&self, name: &str) -> Option<Arc<dyn LLMProvider>>;

    /// Complete with specific provider
    pub async fn complete_with_provider(
        &self,
        provider_name: &str,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, LLMError>;
}
```

#### `CompletionRequest`

Request structure for LLM completions.

```rust
pub struct CompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub stop: Option<Vec<String>>,
    pub stream: Option<bool>,
}
```

#### `CompletionResponse`

Response structure from LLM completions.

```rust
pub struct CompletionResponse {
    pub id: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
    pub model: String,
    pub created: u64,
}
```

### `agent_graph::tools`

#### `ToolRegistry`

Registry for managing tools.

```rust
impl ToolRegistry {
    /// Create a new tool registry
    pub fn new() -> Self;

    /// Register a tool
    pub fn register<T: Tool + 'static>(&mut self, tool: T);

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<&dyn Tool>;

    /// List all available tools
    pub fn list_tools(&self) -> Vec<String>;
}
```

#### `ToolExecutor`

Executes tools safely.

```rust
impl ToolExecutor {
    /// Create a new tool executor
    pub fn new() -> Self;

    /// Execute a tool
    pub async fn execute(
        &self,
        tool: &dyn Tool,
        input: ToolInput,
        config: ToolConfig,
        context: ToolExecutionContext,
    ) -> Result<ToolOutput, ToolError>;
}
```

### `agent_graph::state`

#### `StateManager`

Manages application state with snapshots.

```rust
impl<T: Clone + Serialize + DeserializeOwned> StateManager<T> {
    /// Create a new state manager
    pub fn new(initial_state: T) -> Self;

    /// Get current state
    pub fn current_state(&self) -> &T;

    /// Update state
    pub fn update_state<F>(&mut self, updater: F) -> Result<(), StateError>
    where
        F: FnOnce(&mut T) -> Result<(), StateError>;

    /// Create a snapshot
    pub fn create_snapshot(&mut self) -> Uuid;

    /// Restore from snapshot
    pub fn restore_snapshot(&mut self, snapshot_id: Uuid) -> Result<(), StateError>;
}
```

### `agent_graph::enterprise`

#### Security

```rust
pub struct SecurityManager {
    // Internal fields...
}

impl SecurityManager {
    /// Create a new security manager
    pub fn new() -> Self;

    /// Authenticate a user
    pub async fn authenticate(
        &self,
        credentials: &Credentials,
    ) -> Result<AuthContext, SecurityError>;

    /// Authorize an action
    pub async fn authorize(
        &self,
        context: &AuthContext,
        permission: Permission,
    ) -> Result<bool, SecurityError>;
}
```

#### Monitoring

```rust
pub struct MonitoringManager {
    // Internal fields...
}

impl MonitoringManager {
    /// Record a metric
    pub async fn record_metric(
        &self,
        name: String,
        metric_type: MetricType,
        value: f64,
        labels: HashMap<String, String>,
    ) -> Result<(), MonitoringError>;

    /// Get metrics
    pub async fn get_metrics(&self, name: &str) -> Result<Vec<MetricPoint>, MonitoringError>;
}
```

#### Resources

```rust
pub struct ResourceManager {
    // Internal fields...
}

impl ResourceManager {
    /// Get resource usage
    pub async fn get_usage(&self, tenant_id: &str) -> Result<ResourceUsage, ResourceError>;

    /// Allocate resources
    pub async fn allocate_resource(
        &self,
        tenant_id: &str,
        resource_type: ResourceType,
        amount: f64,
    ) -> Result<(), ResourceError>;
}
```

### `agent_graph::human`

#### Approval System

```rust
pub struct ApprovalManager {
    // Internal fields...
}

impl ApprovalManager {
    /// Create a new approval manager
    pub fn new(interaction_provider: Arc<dyn HumanInteraction>) -> Self;

    /// Create an approval request
    pub async fn create_approval(
        &self,
        request: ApprovalRequest,
    ) -> Result<String, ApprovalError>;

    /// Get approval status
    pub async fn get_approval(&self, id: &str) -> Result<Approval, ApprovalError>;

    /// Approve a request
    pub async fn approve(
        &self,
        id: &str,
        approver: String,
        comment: Option<String>,
    ) -> Result<(), ApprovalError>;

    /// Reject a request
    pub async fn reject(
        &self,
        id: &str,
        approver: String,
        reason: String,
    ) -> Result<(), ApprovalError>;
}
```

## Error Types

### `AgentError`

```rust
pub enum AgentError {
    ConfigurationError(String),
    LLMError(LLMError),
    ToolError(ToolError),
    MemoryError(MemoryError),
    InvalidInput(String),
}
```

### `LLMError`

```rust
pub enum LLMError {
    ProviderNotFound(String),
    RequestFailed(String),
    InvalidResponse(String),
    RateLimited,
    AuthenticationFailed,
    NetworkError(String),
}
```

### `ToolError`

```rust
pub enum ToolError {
    ToolNotFound(String),
    ExecutionFailed(String),
    InvalidInput(String),
    PermissionDenied,
    Timeout,
}
```

## Configuration Structures

### `LLMConfig`

```rust
pub struct LLMConfig {
    pub default_provider: String,
    pub timeout: Duration,
    pub max_retries: u32,
    pub retry_delay: Duration,
}
```

### `MemoryConfig`

```rust
pub struct MemoryConfig {
    pub max_entries: usize,
    pub embedding_model: String,
    pub similarity_threshold: f32,
    pub cleanup_interval: Duration,
}
```

### `CollaborationConfig`

```rust
pub struct CollaborationConfig {
    pub max_agents: usize,
    pub message_timeout: Duration,
    pub heartbeat_interval: Duration,
}
```

## Traits

### `LLMProvider`

```rust
#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LLMError>;
    fn name(&self) -> &str;
    fn models(&self) -> Vec<String>;
}
```

### `Tool`

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> serde_json::Value;
    
    async fn execute(
        &self,
        input: ToolInput,
        config: ToolConfig,
        context: ToolExecutionContext,
    ) -> Result<ToolOutput, ToolError>;
}
```

### `HumanInteraction`

```rust
#[async_trait]
pub trait HumanInteraction: Send + Sync {
    async fn request_interaction(
        &self,
        request: InteractionRequest,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>;

    async fn get_interaction_status(
        &self,
        interaction_id: &str,
    ) -> Result<InteractionStatus, Box<dyn std::error::Error + Send + Sync>>;

    async fn cancel_interaction(
        &self,
        interaction_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
```

## Constants and Enums

### `Permission`

```rust
pub struct Permission {
    pub resource: String,
    pub action: String,
}
```

### `RiskLevel`

```rust
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}
```

### `ApprovalStatus`

```rust
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
    Cancelled,
}
```

This API reference covers the main public interfaces of AgentGraph. For more detailed information about specific methods and their parameters, please refer to the generated rustdoc documentation.
