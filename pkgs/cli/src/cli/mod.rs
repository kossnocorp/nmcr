use crate::prelude::*;

mod command;
pub use command::*;

#[derive(Parser)]
#[command(name = "nmcr", arg_required_else_help = true)]
#[command(about = "Agent-enabled scaffolding & code generation tool", long_about = None)]
#[command(arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

impl Cli {
    pub async fn run() -> Result<()> {
        let cli = Self::parse();
        Command::run(&cli).await
    }
}
