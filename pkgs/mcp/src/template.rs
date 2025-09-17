use crate::prelude::*;

pub(crate) struct TemplateCatalog {
    tools: Vec<TemplateTool>,
}

impl TemplateCatalog {
    pub(crate) fn load(paths: &[PathBuf]) -> Result<Self> {
        let mut tools = Vec::new();
        let mut used_names: HashMap<String, usize> = HashMap::new();
        let mut fallback_counter: usize = 1;

        for path in paths {
            let parsed = parse_file(path)
                .with_context(|| format!("Failed to parse template file: {}", path.display()))?;
            match parsed {
                ParsedMarkdown::Template(template) => {
                    tools.push(TemplateTool::from_template(
                        template,
                        &mut used_names,
                        &mut fallback_counter,
                    ));
                }
                ParsedMarkdown::Collection(collection) => {
                    for template in collection.templates {
                        tools.push(TemplateTool::from_template(
                            template,
                            &mut used_names,
                            &mut fallback_counter,
                        ));
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

#[derive(Clone)]
pub(crate) struct TemplateTool {
    template: Template,
    tool_name: String,
    display_name: String,
    description: String,
    schema: Arc<JsonMap<String, JsonValue>>,
}

impl TemplateTool {
    fn from_template(
        template: Template,
        used_names: &mut HashMap<String, usize>,
        fallback_counter: &mut usize,
    ) -> Self {
        let mut base = Self::sanitize_tool_name(&template.name);
        if base.is_empty() {
            base = format!("template_{}", *fallback_counter);
            *fallback_counter += 1;
        }

        let suffix = used_names.entry(base.clone()).or_insert(0);
        let tool_name = if *suffix == 0 {
            base.clone()
        } else {
            format!("{}_{}", base, suffix)
        };
        *suffix += 1;

        let display_name = if template.name.trim().is_empty() {
            format!("{} (untitled)", tool_name)
        } else {
            template.name.trim().to_string()
        };

        let description = if template.description.trim().is_empty() {
            "Template execution is not yet implemented. Use the TODO result.".to_string()
        } else {
            template.description.trim().to_string()
        };

        let schema = Arc::new(Self::args_schema(&template.args));

        Self {
            template,
            tool_name,
            display_name,
            description,
            schema,
        }
    }

    pub(crate) fn route<H>(&self) -> ToolRoute<H>
    where
        H: Clone + Send + Sync + 'static,
    {
        let tool = Tool::new(
            self.tool_name.clone(),
            self.description.clone(),
            self.schema.clone(),
        );
        ToolRoute::new_dyn(tool, |_context| {
            Box::pin(async { Ok(CallToolResult::success(vec![Content::text("TODO")])) })
        })
    }

    fn instructions_line(&self) -> String {
        let mut line = format!("- {} → {}", self.tool_name, self.display_name);
        if !self.template.description.trim().is_empty() {
            line.push_str(&format!(" — {}", self.template.description.trim()));
        }
        if !self.template.args.items.is_empty() {
            let arg_names: Vec<_> = self
                .template
                .args
                .items
                .iter()
                .map(|arg| arg.name.as_str())
                .collect();
            line.push_str(&format!(" (args: {})", arg_names.join(", ")));
        }
        line
    }

    fn args_schema(args: &TemplateArgs) -> JsonMap<String, JsonValue> {
        let mut schema = JsonMap::new();
        schema.insert("type".into(), JsonValue::String("object".into()));

        let mut properties = JsonMap::new();
        for arg in &args.items {
            let mut prop = JsonMap::new();
            match arg.kind {
                TemplateArgType::Boolean => {
                    prop.insert("type".into(), JsonValue::String("boolean".into()));
                }
                TemplateArgType::String => {
                    prop.insert("type".into(), JsonValue::String("string".into()));
                }
                TemplateArgType::Number => {
                    prop.insert("type".into(), JsonValue::String("number".into()));
                }
                TemplateArgType::Any => {}
            }

            if !arg.description.trim().is_empty() {
                prop.insert(
                    "description".into(),
                    JsonValue::String(arg.description.clone()),
                );
            }

            properties.insert(arg.name.clone(), JsonValue::Object(prop));
        }

        schema.insert("properties".into(), JsonValue::Object(properties));
        schema.insert("additionalProperties".into(), JsonValue::Bool(false));
        schema
    }

    fn sanitize_tool_name(name: &str) -> String {
        let mut cleaned = String::new();
        let mut last_was_separator = false;
        for ch in name.chars() {
            if ch.is_ascii_alphanumeric() {
                cleaned.push(ch.to_ascii_lowercase());
                last_was_separator = false;
            } else if !cleaned.is_empty() && !last_was_separator {
                cleaned.push('_');
                last_was_separator = true;
            }
        }
        if cleaned.ends_with('_') {
            cleaned.pop();
        }
        cleaned
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nmcr_types::{Location, TemplateArg};
    use pretty_assertions::assert_eq;

    fn make_arg(name: &str, description: &str, kind: TemplateArgType) -> TemplateArg {
        TemplateArg {
            name: name.to_string(),
            description: description.to_string(),
            kind,
        }
    }

    #[test]
    fn sanitize_tool_name_basic() {
        assert_eq!(
            TemplateTool::sanitize_tool_name("React Component"),
            "react_component"
        );
        assert_eq!(
            TemplateTool::sanitize_tool_name("  API::Client  "),
            "api_client"
        );
        assert_eq!(TemplateTool::sanitize_tool_name("???"), "");
    }

    #[test]
    fn instructions_include_descriptions_and_args() {
        let mut args = TemplateArgs::new();
        args.items.push(make_arg(
            "name",
            "Name of the component",
            TemplateArgType::String,
        ));
        args.items.push(make_arg(
            "with_css",
            "Generate CSS module",
            TemplateArgType::Boolean,
        ));

        let template = Template {
            name: "Component".into(),
            description: "Create a React component".into(),
            args,
            lang: None,
            content: String::new(),
            location: Location::default(),
        };

        let mut used_names = HashMap::new();
        let mut fallback_counter = 1;
        let tool = TemplateTool::from_template(template, &mut used_names, &mut fallback_counter);
        let instructions = tool.instructions_line();

        assert_eq!(
            instructions,
            "- component → Component — Create a React component (args: name, with_css)"
        );
    }
}
