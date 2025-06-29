# Professional Rust Project Structure - Google Style
## AgentGraph Framework Restructuring Proposal

### 🎯 Design Principles

1. **Layered Architecture**: Clear separation between core, platform, and application layers
2. **Domain-Driven Design**: Modules organized by business domain, not technical concerns
3. **Dependency Inversion**: Higher-level modules don't depend on lower-level implementation details
4. **Single Responsibility**: Each crate has one clear purpose
5. **API-First Design**: Clear public interfaces with internal implementation hiding

### 📁 Recommended Workspace Structure

```
agentgraph/
├── Cargo.toml                          # Workspace root
├── README.md
├── LICENSE
├── CHANGELOG.md
├── .github/                            # CI/CD workflows
│   ├── workflows/
│   └── ISSUE_TEMPLATE/
├── docs/                               # Documentation
│   ├── architecture/
│   ├── guides/
│   └── api/
├── tools/                              # Development tools
│   ├── codegen/
│   ├── benchmarks/
│   └── scripts/
├── examples/                           # Usage examples
│   ├── basic/
│   ├── advanced/
│   └── enterprise/
├── tests/                              # Integration tests
│   ├── integration/
│   ├── performance/
│   └── e2e/
└── crates/                             # All library crates
    ├── agentgraph/                     # Main facade crate
    ├── agentgraph-core/                # Core abstractions
    ├── agentgraph-runtime/             # Execution runtime
    ├── agentgraph-agents/              # Agent system
    ├── agentgraph-llm/                 # LLM integrations
    ├── agentgraph-tools/               # Tool system
    ├── agentgraph-human/               # Human-in-the-loop
    ├── agentgraph-enterprise/          # Enterprise features
    ├── agentgraph-observability/       # Monitoring & tracing
    ├── agentgraph-storage/             # Persistence layer
    ├── agentgraph-protocols/           # Communication protocols
    └── agentgraph-macros/              # Procedural macros
```

### 🏗️ Crate Architecture (Layered)

#### Layer 1: Foundation
- **agentgraph-core**: Core abstractions, traits, and types
- **agentgraph-macros**: Procedural macros for ergonomic APIs

#### Layer 2: Platform Services
- **agentgraph-runtime**: Graph execution engine
- **agentgraph-storage**: State persistence and checkpointing
- **agentgraph-protocols**: Inter-agent communication
- **agentgraph-observability**: Metrics, tracing, and monitoring

#### Layer 3: Domain Services
- **agentgraph-agents**: Agent system and collaboration
- **agentgraph-llm**: LLM provider integrations
- **agentgraph-tools**: Tool execution framework
- **agentgraph-human**: Human interaction workflows

#### Layer 4: Enterprise & Extensions
- **agentgraph-enterprise**: Multi-tenancy, security, compliance
- **agentgraph**: Main facade crate (re-exports + convenience)

### 📦 Individual Crate Structure

Each crate follows this internal structure:

```
agentgraph-{name}/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs                          # Public API surface
│   ├── error.rs                        # Error types
│   ├── config.rs                       # Configuration
│   ├── api/                            # Public API modules
│   │   ├── mod.rs
│   │   ├── traits.rs                   # Public traits
│   │   └── types.rs                    # Public types
│   ├── internal/                       # Internal implementation
│   │   ├── mod.rs
│   │   ├── engine/                     # Core logic
│   │   ├── adapters/                   # External integrations
│   │   └── utils/                      # Utilities
│   └── prelude.rs                      # Common imports
├── tests/                              # Unit tests
├── benches/                            # Benchmarks
└── examples/                           # Crate-specific examples
```

### 🎨 Code Quality Standards

#### 1. API Design
```rust
// ✅ Good: Clear, composable API
pub struct GraphBuilder<S: State> {
    // Internal fields hidden
}

impl<S: State> GraphBuilder<S> {
    pub fn new() -> Self { /* */ }
    pub fn add_node<N: Node<S>>(self, id: impl Into<NodeId>, node: N) -> Self { /* */ }
    pub fn add_edge(self, from: NodeId, to: NodeId) -> Self { /* */ }
    pub fn build(self) -> Result<Graph<S>, BuildError> { /* */ }
}

// ❌ Bad: Exposing internal complexity
pub struct Graph<S> {
    pub nodes: HashMap<NodeId, Box<dyn Node<S>>>,  // Internal detail exposed
    pub edges: Vec<Edge>,                          // Mutable access
}
```

#### 2. Error Handling
```rust
// ✅ Good: Structured error hierarchy
#[derive(Debug, thiserror::Error)]
pub enum GraphError {
    #[error("Node not found: {id}")]
    NodeNotFound { id: NodeId },
    
    #[error("Execution failed: {source}")]
    ExecutionFailed {
        #[from]
        source: ExecutionError,
    },
}

// ❌ Bad: Generic error types
pub type GraphError = Box<dyn std::error::Error>;
```

#### 3. Module Organization
```rust
// ✅ Good: Clear module boundaries
pub mod api {
    pub use self::graph::{Graph, GraphBuilder};
    pub use self::node::{Node, NodeId};
    
    mod graph;
    mod node;
}

mod internal {
    pub(crate) mod execution;
    pub(crate) mod storage;
}

// ❌ Bad: Everything public
pub mod graph;
pub mod node;
pub mod execution;
pub mod storage;
```

### 🔧 Implementation Strategy

#### Phase 1: Core Foundation (Week 1-2)
1. Create `agentgraph-core` with clean abstractions
2. Implement `agentgraph-runtime` for execution
3. Set up proper error handling and configuration

#### Phase 2: Domain Services (Week 3-4)
1. Migrate agent system to `agentgraph-agents`
2. Extract LLM integrations to `agentgraph-llm`
3. Refactor tools into `agentgraph-tools`

#### Phase 3: Platform Services (Week 5-6)
1. Create `agentgraph-storage` for persistence
2. Implement `agentgraph-observability`
3. Add `agentgraph-protocols` for communication

#### Phase 4: Enterprise & Polish (Week 7-8)
1. Finalize `agentgraph-enterprise`
2. Create main `agentgraph` facade
3. Update documentation and examples

### 📊 Benefits of This Structure

1. **Scalability**: Easy to add new features without affecting core
2. **Maintainability**: Clear ownership and boundaries
3. **Testability**: Each crate can be tested independently
4. **Performance**: Selective compilation of features
5. **Ecosystem**: Other crates can depend on specific layers
6. **Enterprise-Ready**: Clear separation of open-source vs. enterprise features

### 🚀 Migration Path

1. **Parallel Development**: Build new structure alongside existing
2. **Gradual Migration**: Move modules one at a time
3. **Compatibility Layer**: Maintain old API during transition
4. **Feature Flags**: Use Cargo features for smooth transition
5. **Comprehensive Testing**: Ensure no functionality is lost

This structure follows Google's internal practices for large-scale software development and aligns with Rust ecosystem best practices.