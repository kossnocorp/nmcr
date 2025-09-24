use crate::prelude::*;

#[derive(Args, Debug)]
pub struct CliCommandProject<T: Args> {
    #[command(flatten)]
    pub global: CliCommandProjectArgs,

    #[command(flatten)]
    pub local: T,
}

impl<T: Args> CliCommandProject<T> {
    pub fn resolve_project_path(&self) -> PathBuf {
        self.global
            .project
            .clone()
            .unwrap_or_else(Config::default_path)
    }

    pub fn load_project(&self) -> Result<Project> {
        let path = self.resolve_project_path();
        Project::load(Some(path))
    }
}

#[derive(Args, Debug, Default)]
pub struct CliCommandProjectArgs {
    /// Path to the project config file or directory containing it.
    #[arg(short, long, value_name = "PROJECT_PATH")]
    pub project: Option<PathBuf>,
}
