use super::tool::TemplateTool;
use crate::prelude::*;

mod context;
pub(crate) use context::*;

pub(crate) struct TemplateCatalog {
    tools: Vec<TemplateTool>,
}

impl TemplateCatalog {
    pub(crate) fn load(paths: &[PathBuf]) -> Result<Self> {
        let mut tools = Vec::new();
        let mut context = TemplateCatalogContext::default();

        for path in paths {
            let parsed = parse_file(path)
                .with_context(|| format!("Failed to parse template file: {}", path.display()))?;

            match parsed {
                ParsedMarkdown::Template(template) => {
                    tools.push(TemplateTool::from_template(template, &mut context));
                }

                ParsedMarkdown::Collection(collection) => {
                    for template in collection.templates {
                        tools.push(TemplateTool::from_template(template, &mut context));
                    }
                }
            }
        }

        Ok(Self { tools })
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    pub(crate) fn tools(&self) -> &[TemplateTool] {
        &self.tools
    }

    pub(crate) fn instructions(&self) -> Option<String> {
        if self.tools.is_empty() {
            return None;
        }

        let mut lines = Vec::with_capacity(self.tools.len() + 1);
        lines.push("Templates available via tools:".to_string());
        for tool in &self.tools {
            lines.push(tool.instructions_line());
        }

        Some(lines.join("\n"))
    }
}
