pub mod init;
pub mod run;
pub mod validate;
pub mod list;
pub mod test;
pub mod visualize;
pub mod benchmark;
pub mod enterprise;
pub mod shell;
pub mod version;

pub use init::InitCommand;
pub use run::RunCommand;
pub use validate::ValidateCommand;
pub use list::ListCommand;
pub use test::TestCommand;
pub use visualize::VisualizeCommand;
pub use benchmark::BenchmarkCommand;
pub use enterprise::EnterpriseCommand;
pub use shell::ShellCommand;
pub use version::VersionCommand;

use crate::{config::CliConfig, OutputFormat};
use async_trait::async_trait;

#[async_trait]
pub trait Command {
    async fn execute(&self, config: &CliConfig, format: &OutputFormat) -> anyhow::Result<()>;
}