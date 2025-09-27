use super::render_template;
use crate::prelude::*;

#[derive(Clone)]
pub(crate) struct TemplateTool {
    template: Template,
    tool_name: String,
    display_name: String,
    description: String,
    schema: Arc<JsonMap<String, JsonValue>>,
}

impl TemplateTool {
    pub(crate) fn from_template(template: Template, context: &mut TemplateCatalogContext) -> Self {
        let mut segments: Vec<String> = Vec::new();
        if let Some(group) = template.collection.as_ref() {
            let sanitized = Self::sanitize_tool_name(group);
            if !sanitized.is_empty() {
                segments.push(sanitized);
            }
        }

        let name_segment = Self::sanitize_tool_name(&template.name);
        if !name_segment.is_empty() {
            segments.push(name_segment);
        }

        let mut base = if segments.is_empty() {
            String::new()
        } else {
            segments.join("_")
        };

        if base.is_empty() {
            base = format!("template_{}", context.fallback_counter);
            context.fallback_counter += 1;
        }

        let suffix = context.used_names.entry(base.clone()).or_insert(0);
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
            format!("Render the {} template.", display_name)
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
        let template = self.template.clone();

        ToolRoute::new_dyn(tool, move |mut context| {
            let template = template.clone();
            Box::pin(async move {
                let arguments = context.arguments.take().unwrap_or_default();
                let rendered = render_template(&template.content, &arguments);
                Ok(CallToolResult::success(vec![Content::text(rendered)]))
            })
        })
    }

    pub(crate) fn instructions_line(&self) -> String {
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
    use nmcr_types_internal::{Location, TemplateArg};
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
            collection: None,
            args,
            lang: None,
            content: String::new(),
            location: Location::default(),
        };

        let tool = TemplateTool::from_template(template, &mut Default::default());
        let instructions = tool.instructions_line();

        assert_eq!(
            instructions,
            "- component → Component — Create a React component (args: name, with_css)"
        );
    }

    #[test]
    fn tool_name_includes_collection_header() {
        let template = Template {
            name: "Package Gitignore".into(),
            description: String::new(),
            collection: Some("Rust".into()),
            args: TemplateArgs::new(),
            lang: None,
            content: String::new(),
            location: Location::default(),
        };

        let tool = TemplateTool::from_template(template, &mut Default::default());
        assert_eq!(tool.tool_name, "rust_package_gitignore");
    }
}
