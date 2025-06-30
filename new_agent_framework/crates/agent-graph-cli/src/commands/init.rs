use super::Command;
use crate::{config::CliConfig, OutputFormat};
use async_trait::async_trait;
use clap::Args;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Args)]
pub struct InitCommand {
    /// Project name
    #[arg(short, long)]
    name: String,

    /// Project directory (defaults to current directory)
    #[arg(short, long)]
    directory: Option<PathBuf>,

    /// Project template
    #[arg(short, long, value_enum, default_value = "basic")]
    template: ProjectTemplate,

    /// Initialize with example workflows
    #[arg(long)]
    with_examples: bool,

    /// Initialize with enterprise features
    #[arg(long)]
    enterprise: bool,

    /// Force overwrite existing files
    #[arg(long)]
    force: bool,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum ProjectTemplate {
    Basic,
    MultiAgent,
    Enterprise,
    Research,
    Automation,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProjectStructure {
    name: String,
    template: String,
    created_files: Vec<String>,
    next_steps: Vec<String>,
}

#[async_trait]
impl Command for InitCommand {
    async fn execute(&self, _config: &CliConfig, format: &OutputFormat) -> anyhow::Result<()> {
        use colored::*;
        use indicatif::{ProgressBar, ProgressStyle};

        println!("{}", "🚀 Initializing AgentGraph Project".bright_blue().bold());
        println!("Project: {}", self.name.cyan());
        println!("Template: {:?}", self.template);

        let project_dir = self.directory.as_ref()
            .map(|d| d.clone())
            .unwrap_or_else(|| PathBuf::from(&self.name));

        // Check if directory exists
        if project_dir.exists() && !self.force {
            if !project_dir.read_dir()?.next().is_none() {
                anyhow::bail!("Directory '{}' already exists and is not empty. Use --force to overwrite.", project_dir.display());
            }
        }

        let progress = ProgressBar::new(8);
        progress.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
        );

        let mut created_files = Vec::new();

        // Create project directory
        progress.set_message("Creating project directory...");
        fs::create_dir_all(&project_dir).await?;
        progress.inc(1);

        // Create Cargo.toml
        progress.set_message("Creating Cargo.toml...");
        let cargo_toml = self.generate_cargo_toml();
        let cargo_path = project_dir.join("Cargo.toml");
        fs::write(&cargo_path, cargo_toml).await?;
        created_files.push("Cargo.toml".to_string());
        progress.inc(1);

        // Create src directory and main.rs
        progress.set_message("Creating source files...");
        let src_dir = project_dir.join("src");
        fs::create_dir_all(&src_dir).await?;
        
        let main_rs = self.generate_main_rs();
        let main_path = src_dir.join("main.rs");
        fs::write(&main_path, main_rs).await?;
        created_files.push("src/main.rs".to_string());
        progress.inc(1);

        // Create configuration file
        progress.set_message("Creating configuration...");
        let config_toml = self.generate_config_toml();
        let config_path = project_dir.join("agentgraph.toml");
        fs::write(&config_path, config_toml).await?;
        created_files.push("agentgraph.toml".to_string());
        progress.inc(1);

        // Create workflows directory
        progress.set_message("Creating workflows...");
        let workflows_dir = project_dir.join("workflows");
        fs::create_dir_all(&workflows_dir).await?;
        
        let example_workflow = self.generate_example_workflow();
        let workflow_path = workflows_dir.join("example.json");
        fs::write(&workflow_path, example_workflow).await?;
        created_files.push("workflows/example.json".to_string());
        progress.inc(1);

        // Create examples if requested
        if self.with_examples {
            progress.set_message("Creating examples...");
            let examples_dir = project_dir.join("examples");
            fs::create_dir_all(&examples_dir).await?;
            
            let examples = self.generate_examples();
            for (filename, content) in examples {
                let example_path = examples_dir.join(&filename);
                fs::write(&example_path, content).await?;
                created_files.push(format!("examples/{}", filename));
            }
        }
        progress.inc(1);

        // Create README
        progress.set_message("Creating documentation...");
        let readme = self.generate_readme();
        let readme_path = project_dir.join("README.md");
        fs::write(&readme_path, readme).await?;
        created_files.push("README.md".to_string());
        progress.inc(1);

        // Create .gitignore
        progress.set_message("Creating .gitignore...");
        let gitignore = self.generate_gitignore();
        let gitignore_path = project_dir.join(".gitignore");
        fs::write(&gitignore_path, gitignore).await?;
        created_files.push(".gitignore".to_string());
        progress.inc(1);

        progress.finish_with_message("✅ Project created successfully!");

        let next_steps = vec![
            format!("cd {}", project_dir.display()),
            "# Configure your API keys in agentgraph.toml".to_string(),
            "cargo run".to_string(),
            "# Or run a workflow:".to_string(),
            "agentgraph run --graph workflows/example.json".to_string(),
        ];

        let project_info = ProjectStructure {
            name: self.name.clone(),
            template: format!("{:?}", self.template),
            created_files,
            next_steps,
        };

        // Print summary
        self.print_summary(&project_info);

        // Output structured data if requested
        if !matches!(format, OutputFormat::Pretty) {
            crate::utils::output::print_result(&project_info, format)?;
        }

        Ok(())
    }
}

impl InitCommand {
    fn generate_cargo_toml(&self) -> String {
        let enterprise_deps = if self.enterprise {
            r#"
# Enterprise features
agent-graph-enterprise = { version = "0.4.0", features = ["full"] }"#
        } else {
            ""
        };

        format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
agent-graph = {{ version = "0.4.0", features = ["full"] }}
tokio = {{ version = "1.0", features = ["full"] }}
anyhow = "1.0"
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"{enterprise_deps}

[[bin]]
name = "{}"
path = "src/main.rs"
"#, self.name, self.name, enterprise_deps = enterprise_deps)
    }

    fn generate_main_rs(&self) -> String {
        match self.template {
            ProjectTemplate::Basic => {
                r#"use agent_graph::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkflowState {
    message: String,
    results: HashMap<String, serde_json::Value>,
}

impl State for WorkflowState {}

impl Default for WorkflowState {
    fn default() -> Self {
        Self {
            message: "Hello, AgentGraph!".to_string(),
            results: HashMap::new(),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    println!("🚀 Starting AgentGraph workflow...");
    
    // Create a simple graph
    let mut graph = GraphBuilder::new()
        .with_name("Basic Workflow")
        .build()?;
    
    // TODO: Add your nodes here
    // Example:
    // let my_node = MyCustomNode::new();
    // graph.add_node("my_node", Box::new(my_node))?;
    // graph.set_entry_point("my_node")?;
    // graph.add_finish_point("my_node")?;
    
    // Execute the workflow
    let mut state = WorkflowState::default();
    // graph.execute(&mut state).await?;
    
    println!("✅ Workflow completed!");
    println!("Final state: {:?}", state);
    
    Ok(())
}
"#.to_string()
            }
            ProjectTemplate::MultiAgent => {
                r#"use agent_graph::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CollaborationState {
    task: String,
    research_results: Option<String>,
    analysis_results: Option<String>,
    final_report: Option<String>,
}

impl State for CollaborationState {}

impl Default for CollaborationState {
    fn default() -> Self {
        Self {
            task: "Analyze market trends for Q4 2024".to_string(),
            research_results: None,
            analysis_results: None,
            final_report: None,
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    println!("🤖 Starting Multi-Agent Collaboration...");
    
    // Create agent runtime
    let mut runtime = AgentRuntime::new(RuntimeConfig::default());
    
    // Create specialized agents
    let researcher = AgentBuilder::new("researcher")
        .with_role(AgentRole::Researcher)
        .with_description("Research specialist")
        .build()?;
    
    let analyst = AgentBuilder::new("analyst")
        .with_role(AgentRole::Analyst)
        .with_description("Data analysis specialist")
        .build()?;
    
    // Register agents
    runtime.register_agent(researcher).await?;
    runtime.register_agent(analyst).await?;
    
    // Create collaboration workflow
    let mut graph = GraphBuilder::new()
        .with_name("Multi-Agent Collaboration")
        .build()?;
    
    // TODO: Add agent nodes and coordination logic
    
    println!("✅ Multi-agent system ready!");
    
    Ok(())
}
"#.to_string()
            }
            ProjectTemplate::Enterprise => {
                r#"use agent_graph::prelude::*;
use agent_graph_enterprise::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EnterpriseState {
    tenant_id: String,
    user_id: String,
    workflow_data: serde_json::Value,
}

impl State for EnterpriseState {}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    println!("🏢 Starting Enterprise AgentGraph...");
    
    // Initialize enterprise features
    let enterprise_config = EnterpriseConfig::default();
    let enterprise_manager = EnterpriseManager::new(enterprise_config)?;
    
    // Set up multi-tenancy
    let tenant = enterprise_manager.create_tenant("demo-tenant").await?;
    println!("Created tenant: {}", tenant.id);
    
    // Set up security
    let security_manager = SecurityManager::new();
    
    // Create enterprise-grade workflow
    let mut graph = GraphBuilder::new()
        .with_name("Enterprise Workflow")
        .with_enterprise_features(true)
        .build()?;
    
    println!("✅ Enterprise system initialized!");
    
    Ok(())
}
"#.to_string()
            }
            ProjectTemplate::Research => {
                r#"use agent_graph::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ResearchState {
    research_topic: String,
    sources: Vec<String>,
    findings: Vec<String>,
    report: Option<String>,
}

impl State for ResearchState {}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    println!("🔬 Starting Research Workflow...");
    
    // Set up LLM client for research
    let llm_client = LLMClientBuilder::new()
        .with_provider("openai", Arc::new(OpenAIProvider::new(OpenAIConfig::default())?))
        .build().await?;
    
    // Create research workflow
    let mut graph = GraphBuilder::new()
        .with_name("Research Pipeline")
        .build()?;
    
    // TODO: Add research nodes
    // - Literature search
    // - Data collection
    // - Analysis
    // - Report generation
    
    println!("✅ Research pipeline ready!");
    
    Ok(())
}
"#.to_string()
            }
            ProjectTemplate::Automation => {
                r#"use agent_graph::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AutomationState {
    input_data: serde_json::Value,
    processing_steps: Vec<String>,
    output_data: Option<serde_json::Value>,
}

impl State for AutomationState {}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    println!("⚙️ Starting Automation Workflow...");
    
    // Set up tool registry
    let tool_registry = create_builtin_registry()?;
    
    // Create automation pipeline
    let mut graph = GraphBuilder::new()
        .with_name("Automation Pipeline")
        .build()?;
    
    // TODO: Add automation nodes
    // - Data ingestion
    // - Processing steps
    // - Quality checks
    // - Output generation
    
    println!("✅ Automation pipeline ready!");
    
    Ok(())
}
"#.to_string()
            }
        }
    }

    fn generate_config_toml(&self) -> String {
        let enterprise_config = if self.enterprise {
            r#"
[enterprise]
multi_tenancy_enabled = true
audit_logging_enabled = true
security_mode = "strict"
rbac_enabled = true"#
        } else {
            r#"
[enterprise]
multi_tenancy_enabled = false
audit_logging_enabled = false
security_mode = "standard""#
        };

        format!(r#"# AgentGraph Configuration for {}

[providers]
# Configure your LLM providers
# openai_api_key = "your-openai-api-key"
# anthropic_api_key = "your-anthropic-api-key"
# google_api_key = "your-google-api-key"
# openrouter_api_key = "your-openrouter-api-key"

[output]
default_format = "pretty"
colors = true
verbose = false

[execution]
default_timeout = 300
max_retries = 3
parallel_execution = true

[logging]
level = "info"
format = "pretty"

[tools]
http_timeout = 30
file_max_size_mb = 10
database_pool_size = 5

[agents]
default_memory_size_mb = 50
collaboration_enabled = true
max_concurrent_agents = 10{enterprise_config}
"#, self.name, enterprise_config = enterprise_config)
    }

    fn generate_example_workflow(&self) -> String {
        match self.template {
            ProjectTemplate::Basic => {
                serde_json::to_string_pretty(&serde_json::json!({
                    "name": "Basic Example Workflow",
                    "description": "A simple workflow to get started",
                    "entry_point": "start",
                    "finish_points": ["end"],
                    "nodes": {
                        "start": {
                            "node_type": "custom",
                            "config": {
                                "message": "Starting workflow..."
                            }
                        },
                        "end": {
                            "node_type": "custom",
                            "config": {
                                "message": "Workflow completed!"
                            }
                        }
                    },
                    "edges": {
                        "start": [{"to": "end"}]
                    }
                })).unwrap()
            }
            ProjectTemplate::MultiAgent => {
                serde_json::to_string_pretty(&serde_json::json!({
                    "name": "Multi-Agent Collaboration",
                    "description": "Agents working together on a task",
                    "entry_point": "research",
                    "finish_points": ["report"],
                    "nodes": {
                        "research": {
                            "node_type": "agent",
                            "config": {
                                "role": "researcher",
                                "task": "Research the given topic"
                            }
                        },
                        "analyze": {
                            "node_type": "agent",
                            "config": {
                                "role": "analyst",
                                "task": "Analyze the research findings"
                            }
                        },
                        "report": {
                            "node_type": "agent",
                            "config": {
                                "role": "writer",
                                "task": "Generate final report"
                            }
                        }
                    },
                    "edges": {
                        "research": [{"to": "analyze"}],
                        "analyze": [{"to": "report"}]
                    }
                })).unwrap()
            }
            _ => {
                serde_json::to_string_pretty(&serde_json::json!({
                    "name": format!("{:?} Workflow", self.template),
                    "description": "Template-specific workflow",
                    "entry_point": "start",
                    "finish_points": ["end"],
                    "nodes": {
                        "start": {
                            "node_type": "custom",
                            "config": {}
                        },
                        "end": {
                            "node_type": "custom", 
                            "config": {}
                        }
                    },
                    "edges": {
                        "start": [{"to": "end"}]
                    }
                })).unwrap()
            }
        }
    }

    fn generate_examples(&self) -> Vec<(String, String)> {
        let mut examples = Vec::new();

        examples.push(("basic_node.rs".to_string(), r#"use agent_graph::prelude::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct BasicNode {
    message: String,
}

impl BasicNode {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

#[async_trait]
impl<S: State> Node<S> for BasicNode {
    async fn execute(&self, state: &mut S) -> CoreResult<NodeOutput> {
        println!("BasicNode: {}", self.message);
        
        // Simulate some work
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        Ok(NodeOutput::success())
    }

    fn id(&self) -> &str {
        "basic_node"
    }

    fn metadata(&self) -> &NodeMetadata {
        static METADATA: std::sync::OnceLock<NodeMetadata> = std::sync::OnceLock::new();
        METADATA.get_or_init(|| NodeMetadata {
            name: "Basic Node".to_string(),
            description: Some("A basic example node".to_string()),
            version: "1.0.0".to_string(),
            parallel_safe: true,
            expected_duration_ms: Some(100),
            tags: vec!["example".to_string()],
            custom_properties: std::collections::HashMap::new(),
        })
    }
}
"#.to_string()));

        examples.push(("llm_example.rs".to_string(), r#"use agent_graph::prelude::*;
use agent_graph_llm::prelude::*;

pub async fn llm_example() -> anyhow::Result<()> {
    // Create LLM client
    let llm_client = LLMClientBuilder::new()
        .with_provider("mock", Arc::new(MockProvider::new()))
        .with_default_provider("mock")
        .build().await?;

    // Create a completion request
    let request = CompletionRequest::simple(
        "mock-gpt-4".to_string(),
        vec![
            Message::system("You are a helpful assistant.".to_string()),
            Message::user("Hello, how are you?".to_string()),
        ],
    );

    // Get response
    let response = llm_client.complete(request, None).await?;
    println!("LLM Response: {:?}", response);

    Ok(())
}
"#.to_string()));

        examples
    }

    fn generate_readme(&self) -> String {
        format!(r#"# {}

An AgentGraph project created with the {:?} template.

## Getting Started

1. Configure your API keys in `agentgraph.toml`
2. Run the project:
   ```bash
   cargo run
   ```

3. Or run a specific workflow:
   ```bash
   agentgraph run --graph workflows/example.json
   ```

## Project Structure

- `src/main.rs` - Main application entry point
- `workflows/` - Graph workflow definitions
- `agentgraph.toml` - Configuration file
- `examples/` - Example code and workflows

## Available Commands

```bash
# List available resources
agentgraph list providers
agentgraph list tools

# Test your setup
agentgraph test all

# Validate workflows
agentgraph validate --graph workflows/example.json

# Generate visualizations
agentgraph visualize --graph workflows/example.json --output workflow.svg
```

## Next Steps

1. Add your API keys to `agentgraph.toml`
2. Customize the workflow in `workflows/example.json`
3. Add custom nodes and agents in `src/`
4. Explore the examples in `examples/`

## Documentation

- [AgentGraph Documentation](https://docs.rs/agent_graph)
- [Getting Started Guide](https://github.com/agent-graph/agent-graph/blob/main/docs/getting-started.md)
- [Examples](https://github.com/agent-graph/agent-graph/tree/main/examples)

## Support

- [GitHub Issues](https://github.com/agent-graph/agent-graph/issues)
- [Discord Community](https://discord.gg/agentgraph)
- [Documentation](https://docs.rs/agent_graph)
"#, self.name, self.template)
    }

    fn generate_gitignore(&self) -> String {
        r#"# Rust
/target/
**/*.rs.bk
*.pdb
Cargo.lock

# IDE
.vscode/
.idea/
*.swp
*.swo
*~

# OS
.DS_Store
.DS_Store?
._*
.Spotlight-V100
.Trashes
ehthumbs.db
Thumbs.db

# Logs
*.log

# Environment
.env
.env.local
.env.*.local

# AgentGraph
/data/
/logs/
/checkpoints/
/outputs/

# API Keys (keep your keys safe!)
agentgraph.toml.local
secrets.toml
"#.to_string()
    }

    fn print_summary(&self, project: &ProjectStructure) {
        use colored::*;

        println!("\n{}", "📁 Project Structure Created".bright_green().bold());
        println!("Name: {}", project.name.cyan());
        println!("Template: {}", project.template.cyan());
        println!("Files created: {}", project.created_files.len().to_string().cyan());

        println!("\n{}", "📄 Created Files:".bright_blue().bold());
        for file in &project.created_files {
            println!("  ✅ {}", file.green());
        }

        println!("\n{}", "🚀 Next Steps:".bright_blue().bold());
        for (i, step) in project.next_steps.iter().enumerate() {
            if step.starts_with('#') {
                println!("  {}", step.dimmed());
            } else {
                println!("  {}. {}", i + 1, step.cyan());
            }
        }

        println!("\n{}", "🎉 Project ready! Happy coding!".bright_green().bold());
    }
}