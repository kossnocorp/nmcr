use crate::prelude::*;
use nmcr_parser::prelude::*;

#[derive(Args)]
pub struct GenArgs {}

#[derive(Args)]
pub struct GenCmd {}

impl GenCmd {
    pub async fn run<'a>(args: &CliCommandProject<GenArgs>) -> Result<()> {
        let project = args.load_project()?;
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
