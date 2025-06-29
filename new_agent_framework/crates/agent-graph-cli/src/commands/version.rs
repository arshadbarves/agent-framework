use super::Command;
use crate::{config::CliConfig, utils::output, OutputFormat};
use async_trait::async_trait;
use clap::Args;
use serde::{Deserialize, Serialize};

#[derive(Args)]
pub struct VersionCommand {
    /// Show detailed version information
    #[arg(long)]
    detailed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct VersionInfo {
    version: String,
    git_commit: Option<String>,
    build_date: Option<String>,
    rust_version: String,
    target_triple: String,
    features: Vec<String>,
    dependencies: Vec<DependencyInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DependencyInfo {
    name: String,
    version: String,
}

#[async_trait]
impl Command for VersionCommand {
    async fn execute(&self, _config: &CliConfig, format: &OutputFormat) -> anyhow::Result<()> {
        use colored::*;

        if self.detailed {
            let version_info = self.get_detailed_version_info();
            output::print_result(&version_info, format)?;
        } else {
            match format {
                OutputFormat::Pretty => {
                    println!("{} {}", 
                        "AgentGraph".bright_blue().bold(),
                        env!("CARGO_PKG_VERSION").cyan()
                    );
                    println!("Production-grade multi-agent framework for Rust");
                    println!("Built with ❤️  by the AgentGraph community");
                }
                _ => {
                    let simple_version = serde_json::json!({
                        "name": "AgentGraph",
                        "version": env!("CARGO_PKG_VERSION")
                    });
                    output::print_result(&simple_version, format)?;
                }
            }
        }

        Ok(())
    }
}

impl VersionCommand {
    fn get_detailed_version_info(&self) -> VersionInfo {
        VersionInfo {
            version: env!("CARGO_PKG_VERSION").to_string(),
            git_commit: option_env!("GIT_COMMIT").map(|s| s.to_string()),
            build_date: option_env!("BUILD_DATE").map(|s| s.to_string()),
            rust_version: env!("RUSTC_VERSION").to_string(),
            target_triple: env!("TARGET").to_string(),
            features: self.get_enabled_features(),
            dependencies: self.get_key_dependencies(),
        }
    }

    fn get_enabled_features(&self) -> Vec<String> {
        let mut features = Vec::new();
        
        #[cfg(feature = "agents")]
        features.push("agents".to_string());
        
        #[cfg(feature = "llm")]
        features.push("llm".to_string());
        
        #[cfg(feature = "tools")]
        features.push("tools".to_string());
        
        #[cfg(feature = "human")]
        features.push("human".to_string());
        
        #[cfg(feature = "enterprise")]
        features.push("enterprise".to_string());
        
        #[cfg(feature = "visualization")]
        features.push("visualization".to_string());
        
        #[cfg(feature = "streaming")]
        features.push("streaming".to_string());
        
        features
    }

    fn get_key_dependencies(&self) -> Vec<DependencyInfo> {
        vec![
            DependencyInfo {
                name: "tokio".to_string(),
                version: "1.0".to_string(),
            },
            DependencyInfo {
                name: "serde".to_string(),
                version: "1.0".to_string(),
            },
            DependencyInfo {
                name: "tracing".to_string(),
                version: "0.1".to_string(),
            },
            DependencyInfo {
                name: "reqwest".to_string(),
                version: "0.11".to_string(),
            },
        ]
    }
}