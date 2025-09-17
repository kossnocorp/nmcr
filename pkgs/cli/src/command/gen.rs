use crate::prelude::*;

#[derive(clap::Args)]
pub struct GenArgs {}

pub struct GenCmd {}

impl GenCmd {
    pub async fn run<'a>(cli: &'a Cli, args: &'a GenArgs) -> Result<()> {
        let path = cli.resolve_project_path();
        let project = Project::load(Some(path))?;
        let templates = project.template_paths()?;

        println!("Available templates:");
        for template in templates {
            println!("- {}", template.display());
        }

        Ok(())
    }
}
