use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CliConfig {
    pub default_provider: Option<String>,
    pub providers: ProviderConfigs,
    pub output: OutputConfig,
    pub execution: ExecutionConfig,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ProviderConfigs {
    pub openai_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub google_api_key: Option<String>,
    pub openrouter_api_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OutputConfig {
    pub default_format: String,
    pub colors: bool,
    pub verbose: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            default_format: "pretty".to_string(),
            colors: true,
            verbose: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionConfig {
    pub default_timeout: u64,
    pub max_retries: u32,
    pub parallel_execution: bool,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            default_timeout: 300,
            max_retries: 3,
            parallel_execution: true,
        }
    }
}

impl CliConfig {
    pub async fn load(config_path: Option<&Path>) -> anyhow::Result<Self> {
        if let Some(path) = config_path {
            let content = tokio::fs::read_to_string(path).await?;
            let config = toml::from_str(&content)?;
            Ok(config)
        } else {
            // Try to load from default locations
            let default_paths = [
                "agentgraph.toml",
                "~/.config/agentgraph/config.toml",
                "~/.agentgraph.toml",
            ];

            for path_str in &default_paths {
                let path = Path::new(path_str);
                if path.exists() {
                    let content = tokio::fs::read_to_string(path).await?;
                    let config = toml::from_str(&content)?;
                    return Ok(config);
                }
            }

            Ok(Self::default())
        }
    }
}