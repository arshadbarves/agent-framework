# AgentGraph Examples

This directory contains comprehensive examples demonstrating the capabilities of the AgentGraph framework.

## Overview

AgentGraph is a production-ready Rust framework for building multi-agent systems with advanced features like LLM integration, collaboration, enterprise security, and human-in-the-loop workflows.

## Examples

### 1. Basic Agent (`basic_agent.rs`)

**What it demonstrates:**
- Creating a simple agent with a role template
- Setting up LLM providers (Mock provider for testing)
- Executing tasks and getting responses
- Using the memory system to store and retrieve context
- Error handling and agent state management

**Key concepts:**
- Agent creation and configuration
- Role templates (Software Developer)
- Memory system operations
- Task execution workflow

**Run the example:**
```bash
cargo run --example basic_agent
```

**Expected output:**
- Agent setup and configuration details
- Task execution with responses
- Memory system demonstration
- Agent performance statistics

### 2. Multi-Agent Collaboration (`multi_agent_collaboration.rs`)

**What it demonstrates:**
- Creating multiple specialized agents
- Setting up agent collaboration
- Coordinating complex workflows across agents
- Cross-agent communication and knowledge sharing
- Performance monitoring across multiple agents

**Key concepts:**
- Collaboration Manager setup
- Agent specialization (Developer, QA, PM, DevOps)
- Capability-based agent discovery
- Workflow coordination
- Shared memory and context

**Run the example:**
```bash
cargo run --example multi_agent_collaboration
```

**Expected output:**
- Multi-agent system setup
- Collaborative project workflow
- Agent specialization demonstration
- Performance metrics across agents

## Role Templates

AgentGraph includes several pre-built role templates:

### Software Developer
- **Focus**: Coding, debugging, architecture
- **Tools**: File operations, code analysis
- **Temperature**: 0.3 (precise, deterministic)

### QA Engineer
- **Focus**: Testing, quality assurance, automation
- **Tools**: Testing frameworks, validation tools
- **Temperature**: 0.4 (balanced precision and creativity)

### Project Manager
- **Focus**: Planning, coordination, requirements
- **Tools**: Project management, communication
- **Temperature**: 0.6 (balanced approach)

### DevOps Engineer
- **Focus**: Deployment, infrastructure, monitoring
- **Tools**: Infrastructure tools, monitoring
- **Temperature**: 0.3 (precise, reliable)

### Research Analyst
- **Focus**: Research, analysis, data gathering
- **Tools**: Web search, data analysis
- **Temperature**: 0.5 (balanced analysis)

### Content Writer
- **Focus**: Writing, content creation, communication
- **Tools**: Text processing, formatting
- **Temperature**: 0.8 (creative, varied output)

### UI/UX Designer
- **Focus**: Design, user experience, interfaces
- **Tools**: Design tools, prototyping
- **Temperature**: 0.7 (creative with structure)

## Common Patterns

### 1. Agent Setup Pattern

```rust
// Setup LLM provider
let llm_config = LLMConfig::default();
let mut llm_manager = LLMManager::new(llm_config);
let provider = MockProvider::new(); // or OpenAIProvider, AnthropicProvider, etc.
llm_manager.register_provider("provider_name".to_string(), Arc::new(provider));

// Setup tools
let tool_registry = Arc::new(ToolRegistry::new());
let tool_executor = Arc::new(ToolExecutor::new());

// Create agent
let template = RoleTemplates::software_developer();
let config = template.to_agent_config("AgentName".to_string(), "provider_name".to_string());
let agent = Agent::new(config, Arc::new(llm_manager), tool_registry, tool_executor)?;
```

### 2. Collaboration Setup Pattern

```rust
// Setup collaboration
let collab_config = CollaborationConfig::default();
let collab_manager = CollaborationManager::new(collab_config);

// Register agents with capabilities
let receiver = collab_manager.register_agent(
    "agent_name".to_string(),
    vec!["capability1".to_string(), "capability2".to_string()]
).await?;

// Find agents by capability
let agents = collab_manager.find_agents_with_capabilities(&["coding".to_string()]).await?;
```

### 3. Memory Usage Pattern

```rust
// Store interaction
agent.memory_mut().store_interaction(
    "Question or context",
    "Response or information"
).await?;

// Retrieve relevant context
let context = agent.memory_mut().get_relevant_context("search query").await?;

// Get memory statistics
let stats = agent.memory().get_stats();
```

## Testing

Each example includes comprehensive tests demonstrating:
- Agent creation and configuration
- Task execution
- Memory system functionality
- Collaboration features
- Error handling

Run tests for specific examples:
```bash
cargo test --example basic_agent
cargo test --example multi_agent_collaboration
```

## Configuration

### LLM Providers

The examples use MockProvider for testing, but you can easily switch to real providers:

```rust
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

### Environment Variables

For real LLM providers, set these environment variables:
```bash
export OPENAI_API_KEY="your-openai-key"
export ANTHROPIC_API_KEY="your-anthropic-key"
export GOOGLE_API_KEY="your-google-key"
export OPENROUTER_API_KEY="your-openrouter-key"
```

## Best Practices

### 1. Error Handling
- Always handle LLM provider errors gracefully
- Implement retry logic for transient failures
- Log errors for debugging and monitoring

### 2. Resource Management
- Use connection pooling for HTTP requests
- Monitor token usage and costs
- Implement rate limiting for API calls

### 3. Security
- Never hardcode API keys in source code
- Use environment variables for sensitive configuration
- Implement proper authentication and authorization

### 4. Performance
- Cache frequently accessed data
- Use appropriate temperature settings for different tasks
- Monitor agent performance and optimize accordingly

## Next Steps

After running these examples, explore:
- [Getting Started Guide](../docs/getting-started.md)
- [API Reference](../docs/api-reference.md)
- [Architecture Documentation](../docs/architecture.md)
- [Security Guide](../docs/security.md)

## Troubleshooting

### Common Issues

1. **Provider not found**: Ensure you've registered the LLM provider before creating agents
2. **Tool execution errors**: Check that required tools are available and properly configured
3. **Memory errors**: Verify memory configuration and available resources
4. **Collaboration timeouts**: Adjust timeout settings in collaboration configuration

### Debug Mode

Enable debug logging to see detailed execution information:
```bash
RUST_LOG=debug cargo run --example basic_agent
```

## Contributing

To add new examples:
1. Create a new `.rs` file in the `examples/` directory
2. Follow the existing pattern for setup and demonstration
3. Include comprehensive tests
4. Update this README with example description
5. Add appropriate documentation comments
