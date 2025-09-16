use crate::prelude::*;

#[derive(Parser)]
#[command(name = "nmcr")]
#[command(about = "Agent-enabled scaffolding & code generation tool", long_about = None)]
#[command(arg_required_else_help = true)]
pub struct Cli {
    /// Set custom config file.
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

impl Cli {
    pub async fn run() -> Result<()> {
        let cli = Self::parse();
        Command::run(&cli).await
    }
}
