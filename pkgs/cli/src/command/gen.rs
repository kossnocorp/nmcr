use crate::prelude::*;
use anyhow::{Context, anyhow};
use nmcr_md_parser::prelude::*;
use nmcr_types_internal::{FormattedLocation, Location, Template};
use std::collections::HashMap;

#[derive(Args)]
pub struct GenArgs {}

#[derive(Args)]
pub struct GenCmd {}

impl GenCmd {
    pub async fn run(args: &CliCommandProject<GenArgs>) -> Result<()> {
        let project = args.load_project()?;
        let templates = project.template_paths()?;

        println!("Parsing markdown templates:");
        let mut seen: HashMap<String, Location> = HashMap::new();
        for template in templates {
            println!("- {}", template.display());
            match parse_file(&template) {
                Ok(ParsedMarkdown::Template(t)) => {
                    register_template(&t, &mut seen)?;
                    println!("  {:#?}", t);
                }
                Ok(ParsedMarkdown::Collection(c)) => {
                    for template in &c.templates {
                        register_template(template, &mut seen)?;
                    }
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

fn register_template(template: &Template, seen: &mut HashMap<String, Location>) -> Result<()> {
    if let Some(existing) = seen.get(&template.id) {
        let first = FormattedLocation(existing).to_string();
        let duplicate = FormattedLocation(&template.location).to_string();
        return Err(anyhow!("Duplicate template id: {}", template.id))
            .with_context(|| format!("duplicate occurrence at {duplicate}"))
            .with_context(|| format!("first occurrence at {first}"));
    }

    seen.insert(template.id.clone(), template.location.clone());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use nmcr_types_internal::Span;

    fn make_template(path: &str, start: usize, end: usize) -> Template {
        Template {
            id: "duplicate".into(),
            name: "Example".into(),
            description: String::new(),
            args: Vec::new(),
            lang: None,
            content: String::new(),
            location: Location {
                path: path.to_string(),
                span: Span { start, end },
            },
        }
    }

    #[test]
    fn register_template_detects_duplicates() {
        let mut seen = HashMap::new();
        let first = make_template("templates/a.md", 0, 10);
        register_template(&first, &mut seen).expect("first registration succeeds");

        let second = make_template("templates/b.md", 5, 15);
        let err = register_template(&second, &mut seen).expect_err("duplicate should fail");

        let rendered = format!("{err:?}");
        assert!(rendered.contains("Duplicate template id: duplicate"));
        assert!(rendered.contains("duplicate occurrence at templates/b.md:5-15"));
        assert!(rendered.contains("first occurrence at templates/a.md:0-10"));
    }
}
