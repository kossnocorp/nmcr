use crate::prelude::*;

#[derive(Args)]
pub struct InitArgs {
    /// Optional path to initialize the project in
    #[arg()]
    path: Option<PathBuf>,
    /// Force initialization. Overwrite existing files.
    #[arg(short, long, default_value_t = false)]
    force: bool,
}

#[derive(Args)]
pub struct InitCmd {}

impl InitCmd {
    pub async fn run<'a>(args: &'a CliCommandProject<InitArgs>) -> Result<()> {
        let path = args
            .local
            .path
            .clone()
            .unwrap_or_else(|| args.resolve_project_path());

        let templates_glob = UiConfig::inquire_templates_glob()?;

        let mut config = Config::init(&path, args.local.force)?;
        config.user.templates = templates_glob;

        config.write()?;

        Ok(())
    }
}
