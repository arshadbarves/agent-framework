use super::Command;
use crate::{config::CliConfig, OutputFormat};
use async_trait::async_trait;
use clap::Args;

#[derive(Args)]
pub struct InitCommand {
    /// Project name
    #[arg(short, long)]
    name: String,
}

#[async_trait]
impl Command for InitCommand {
    async fn execute(&self, _config: &CliConfig, _format: &OutputFormat) -> anyhow::Result<()> {
        println!("🚀 Initializing AgentGraph project: {}", self.name);
        // Implementation would create project structure
        Ok(())
    }
}