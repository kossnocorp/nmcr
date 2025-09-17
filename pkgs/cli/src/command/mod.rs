use crate::prelude::*;

mod init;
pub use init::*;

mod r#gen;
pub use r#gen::*;

mod mcp;
pub use mcp::*;

#[derive(clap::Subcommand)]
pub enum Command {
    /// Initialize a new nmcr project in an existing directory
    Init(InitArgs),

    /// Generate code from a template.
    Gen(GenArgs),

    /// Start the MCP server exposing project templates as tools.
    Mcp(McpArgs),
}

impl Command {
    pub async fn run(cli: &Cli) -> Result<()> {
        match &cli.command {
            Some(Command::Init(args)) => Ok(InitCmd::run(cli, args).await?),

            Some(Command::Gen(args)) => Ok(GenCmd::run(cli, args).await?),

            Some(Command::Mcp(args)) => Ok(McpCmd::run(cli, args).await?),

            None => unreachable!("No command was provided"),
        }
    }
}
