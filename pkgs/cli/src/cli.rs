use crate::prelude::*;

#[derive(Parser)]
#[command(name = "nmcr")]
#[command(about = "Agent-enabled scaffolding & code generation tool", long_about = None)]
#[command(arg_required_else_help = true)]
pub struct Cli {
    /// Path to the project config file or directory containing it.
    #[arg(short, long, value_name = "PROJECT_PATH")]
    pub project: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

impl Cli {
    pub async fn run() -> Result<()> {
        let cli = Self::parse();
        Command::run(&cli).await
    }

    pub fn resolve_project_path(&self) -> PathBuf {
        self.project
            .as_ref()
            .map(|p| p.clone())
            .unwrap_or_else(|| Config::default_path())
    }
}
