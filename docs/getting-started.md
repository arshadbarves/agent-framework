# Getting Started with AgentGraph

AgentGraph is a production-ready Rust framework for building multi-agent systems with advanced features like LLM integration, collaboration, enterprise security, and human-in-the-loop workflows.

## Quick Start

### Installation

Add AgentGraph to your `Cargo.toml`:

```toml
[dependencies]
agent_graph = "0.3.0"
tokio = { version = "1.0", features = ["full"] }
serde_json = "1.0"
```

### Basic Example

```rust
use agent_graph::{
    agents::{Agent, roles::RoleTemplates},
    llm::{LLMManager, LLMConfig, providers::MockProvider},
    tools::{ToolRegistry, ToolExecutor},
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup LLM provider
    let llm_config = LLMConfig::default();
    let mut llm_manager = LLMManager::new(llm_config);
    let mock_provider = MockProvider::new();
    llm_manager.register_provider("mock".to_string(), Arc::new(mock_provider));
    let llm_manager = Arc::new(llm_manager);
    
    // Setup tools
    let tool_registry = Arc::new(ToolRegistry::new());
    let tool_executor = Arc::new(ToolExecutor::new());
    
    // Create a software developer agent
    let developer_template = RoleTemplates::software_developer();
    let config = developer_template.to_agent_config("Alice".to_string(), "mock".to_string());
    
    let mut agent = Agent::new(config, llm_manager, tool_registry, tool_executor)?;
    
    // Execute a task
    let response = agent.execute_task("Write a hello world function in Python".to_string()).await?;
    println!("Agent response: {}", response);
    
    Ok(())
}
```

## Core Concepts

### 1. Agents

Agents are the core building blocks of AgentGraph. Each agent has:
- **Role**: Defines the agent's capabilities and behavior
- **Memory**: Stores conversation history and context
- **Tools**: Available functions the agent can call
- **LLM Provider**: The language model backend

### 2. Role Templates

AgentGraph provides pre-built role templates:

```rust
// Available role templates
let developer = RoleTemplates::software_developer();
let researcher = RoleTemplates::research_analyst();
let writer = RoleTemplates::content_writer();
let manager = RoleTemplates::project_manager();
let qa = RoleTemplates::qa_engineer();
let devops = RoleTemplates::devops_engineer();
let designer = RoleTemplates::ui_ux_designer();

// Get all available templates
let all_templates = RoleTemplates::all_templates();
let template_names = RoleTemplates::template_names();
```

### 3. LLM Integration

AgentGraph supports multiple LLM providers:

```rust
use agent_graph::llm::providers::{OpenAIProvider, AnthropicProvider, GoogleProvider, OpenRouterProvider};

// OpenAI
let openai = OpenAIProvider::new("your-api-key".to_string());
llm_manager.register_provider("openai".to_string(), Arc::new(openai));

// Anthropic
let anthropic = AnthropicProvider::new("your-api-key".to_string());
llm_manager.register_provider("anthropic".to_string(), Arc::new(anthropic));

// Google
let google = GoogleProvider::new("your-api-key".to_string());
llm_manager.register_provider("google".to_string(), Arc::new(google));

// OpenRouter
let openrouter = OpenRouterProvider::new("your-api-key".to_string());
llm_manager.register_provider("openrouter".to_string(), Arc::new(openrouter));
```

### 4. Agent Collaboration

Agents can collaborate through the collaboration manager:

```rust
use agent_graph::agents::collaboration::CollaborationManager;

let collab_config = agent_graph::agents::collaboration::CollaborationConfig::default();
let collab_manager = CollaborationManager::new(collab_config);

// Register agents with capabilities
let dev_receiver = collab_manager.register_agent(
    "Alice".to_string(),
    vec!["coding".to_string(), "debugging".to_string()]
).await?;

let qa_receiver = collab_manager.register_agent(
    "Bob".to_string(),
    vec!["testing".to_string(), "quality_assurance".to_string()]
).await?;

// Find agents with specific capabilities
let coding_agents = collab_manager.find_agents_with_capabilities(&["coding".to_string()]).await?;
```

### 5. Memory System

Each agent has a sophisticated memory system:

```rust
// Store interactions
agent.memory_mut().store_interaction(
    "How do I handle errors in Rust?",
    "Use Result<T, E> types and the ? operator for error propagation"
).await?;

// Retrieve relevant context
let context = agent.memory_mut().get_relevant_context("error handling").await?;

// Get memory statistics
let stats = agent.memory().get_stats();
println!("Total entries: {}", stats.total_entries);
```

### 6. Tools

AgentGraph includes built-in tools:

```rust
let tool_registry = ToolRegistry::new();
let available_tools = tool_registry.list_tools();

// Common tools include:
// - file_read, file_write: File operations
// - http_get, http_post: HTTP requests
// - json_parse: JSON parsing
// - text_search: Text search operations
```

## Advanced Features

### Enterprise Security

```rust
use agent_graph::enterprise::security::{SecurityManager, AuthContext, Permission};

let security_manager = SecurityManager::new();

let auth_context = AuthContext {
    user_id: "user123".to_string(),
    roles: vec!["developer".to_string()],
    session_id: "session123".to_string(),
    authenticated_at: std::time::SystemTime::now(),
    expires_at: Some(std::time::SystemTime::now() + std::time::Duration::from_secs(3600)),
    claims: std::collections::HashMap::new(),
};
```

### Human-in-the-Loop

```rust
use agent_graph::human::approval::{ApprovalManager, ApprovalRequest};

let approval_request = ApprovalRequest {
    request_id: "req123".to_string(),
    title: "Deploy to Production".to_string(),
    description: "Deploy version 1.2.3".to_string(),
    risk_level: agent_graph::human::approval::RiskLevel::High,
    data: agent_graph::human::approval::HumanContext::default(),
    min_approvals: 2,
    required_approvers: vec!["admin".to_string()],
    auto_approve_conditions: vec![],
    expires_at: Some(std::time::SystemTime::now() + std::time::Duration::from_secs(3600)),
};
```

### State Management

```rust
use agent_graph::state::StateManager;

let state_manager = StateManager::new(serde_json::json!({
    "project": "my-project",
    "version": "1.0.0"
}));

// Create snapshots
let snapshot = state_manager.create_snapshot();

// Access current state
let current_state = state_manager.current_state();
```

## Best Practices

### 1. Agent Design
- Choose appropriate role templates for your use case
- Configure memory settings based on conversation length
- Set reasonable token limits to control costs

### 2. Error Handling
- Always handle LLM provider errors gracefully
- Implement retry logic for transient failures
- Log errors for debugging and monitoring

### 3. Security
- Use proper authentication and authorization
- Validate all inputs from external sources
- Implement rate limiting for API calls

### 4. Performance
- Use connection pooling for HTTP requests
- Cache frequently accessed data
- Monitor token usage and costs

## Next Steps

- Check out the [Examples](examples/) directory for complete working examples
- Read the [API Documentation](api/) for detailed reference
- See [Architecture Guide](architecture.md) for system design details
- Review [Security Guide](security.md) for production deployment

## Support

- GitHub Issues: Report bugs and feature requests
- Documentation: Comprehensive guides and API reference
- Examples: Working code samples for common use cases
