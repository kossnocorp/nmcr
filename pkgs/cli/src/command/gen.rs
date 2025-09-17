use crate::prelude::*;
use nmcr_parser::prelude::*;

#[derive(clap::Args)]
pub struct GenArgs {}

pub struct GenCmd {}

impl GenCmd {
    pub async fn run<'a>(cli: &'a Cli, args: &'a GenArgs) -> Result<()> {
        let path = cli.resolve_project_path();
        let project = Project::load(Some(path))?;
        let templates = project.template_paths()?;

        println!("Parsing markdown templates:");
        for template in templates {
            println!("- {}", template.display());
            match parse_file(&template) {
                Ok(ParsedMarkdown::Template(t)) => {
                    println!("  {:#?}", t);
                }
                Ok(ParsedMarkdown::Collection(c)) => {
                    println!("  {:#?}", c);
                }
                Err(err) => {
                    println!("  ! Failed to parse: {err}");
                }
            }
        }

        Ok(())
    }
}
