use crate::prelude::*;

mod init;
pub use init::*;

#[derive(clap::Subcommand)]
pub enum Command {
    /// Initialize a new nmcr project in an existing directory
    Init(InitArgs),
}

impl Command {
    pub async fn run(cli: &Cli) -> Result<()> {
        match &cli.command {
            Some(Command::Init(args)) => Ok(InitCmd::run(cli, args).await?),

            None => unreachable!("No command was provided"),
        }
    }
}
