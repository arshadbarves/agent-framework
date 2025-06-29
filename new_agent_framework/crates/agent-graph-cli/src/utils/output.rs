use crate::OutputFormat;
use serde::Serialize;

pub fn print_result<T: Serialize>(data: &T, format: &OutputFormat) -> anyhow::Result<()> {
    match format {
        OutputFormat::Pretty => {
            println!("{}", serde_json::to_string_pretty(data)?);
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string(data)?);
        }
        OutputFormat::Yaml => {
            println!("{}", serde_yaml::to_string(data)?);
        }
        OutputFormat::Table => {
            // For table format, we'd need to implement custom formatting
            // For now, fall back to pretty JSON
            println!("{}", serde_json::to_string_pretty(data)?);
        }
    }
    Ok(())
}