use crate::prelude::*;

#[derive(clap::Args)]
pub struct InitArgs {
    /// Optional path to initialize the project in
    #[arg()]
    path: Option<PathBuf>,
    /// Force initialization. Overwrite existing files.
    #[arg(short, long, default_value_t = false)]
    force: bool,
}

pub struct InitCmd {}

impl InitCmd {
    pub async fn run<'a>(cli: &'a Cli, args: &'a InitArgs) -> Result<()> {
        let path = args
            .path
            .clone()
            .unwrap_or_else(|| cli.resolve_project_path());

        let templates_glob = UiConfig::inquire_templates_glob()?;

        let mut config = Config::init(&path, args.force)?;
        config.user.templates = templates_glob;

        config.write()?;

        Ok(())
    }
}
