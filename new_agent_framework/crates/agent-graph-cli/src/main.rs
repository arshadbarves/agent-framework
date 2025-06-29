//! AgentGraph CLI - Command-line interface for the AgentGraph framework

use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;
mod config;
mod utils;

use commands::*;
use config::CliConfig;

#[derive(Parser)]
#[command(name = "agentgraph")]
#[command(about = "AgentGraph - Production-grade multi-agent framework for Rust")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(author = "AgentGraph Contributors")]
struct Cli {
    /// Configuration file path
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,

    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Output format
    #[arg(long, global = true, value_enum, default_value = "pretty")]
    format: OutputFormat,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new AgentGraph project
    Init(InitCommand),
    /// Run a graph workflow
    Run(RunCommand),
    /// Validate a graph definition
    Validate(ValidateCommand),
    /// List available providers, tools, and agents
    List(ListCommand),
    /// Test LLM providers and tools
    Test(TestCommand),
    /// Generate graph visualizations
    Visualize(VisualizeCommand),
    /// Benchmark graph performance
    Benchmark(BenchmarkCommand),
    /// Manage enterprise features
    Enterprise(EnterpriseCommand),
    /// Interactive shell mode
    Shell(ShellCommand),
    /// Show version information
    Version(VersionCommand),
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum OutputFormat {
    Pretty,
    Json,
    Yaml,
    Table,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(format!("agentgraph={},agent_graph={}", log_level, log_level))
        .init();

    // Load configuration
    let config = CliConfig::load(cli.config.as_deref()).await?;

    // Execute command
    match cli.command {
        Commands::Init(cmd) => cmd.execute(&config, &cli.format).await,
        Commands::Run(cmd) => cmd.execute(&config, &cli.format).await,
        Commands::Validate(cmd) => cmd.execute(&config, &cli.format).await,
        Commands::List(cmd) => cmd.execute(&config, &cli.format).await,
        Commands::Test(cmd) => cmd.execute(&config, &cli.format).await,
        Commands::Visualize(cmd) => cmd.execute(&config, &cli.format).await,
        Commands::Benchmark(cmd) => cmd.execute(&config, &cli.format).await,
        Commands::Enterprise(cmd) => cmd.execute(&config, &cli.format).await,
        Commands::Shell(cmd) => cmd.execute(&config, &cli.format).await,
        Commands::Version(cmd) => cmd.execute(&config, &cli.format).await,
    }
}