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
    pub(crate) fn from_template(template: Template) -> Self {
        let tool_name = template.id.clone();

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
        if !self.template.args.is_empty() {
            let arg_names: Vec<_> = self
                .template
                .args
                .iter()
                .map(|arg| arg.name.as_str())
                .collect();
            line.push_str(&format!(" (args: {})", arg_names.join(", ")));
        }
        line
    }

    fn args_schema(args: &[Arg]) -> JsonMap<String, JsonValue> {
        let mut schema = JsonMap::new();
        schema.insert("type".into(), JsonValue::String("object".into()));

        let mut properties = JsonMap::new();
        for arg in args {
            let mut prop = JsonMap::new();
            match &arg.kind {
                ArgKind::Boolean(_) => {
                    prop.insert("type".into(), JsonValue::String("boolean".into()));
                }
                ArgKind::String(_) => {
                    prop.insert("type".into(), JsonValue::String("string".into()));
                }
                ArgKind::Number(_) => {
                    prop.insert("type".into(), JsonValue::String("number".into()));
                }
                ArgKind::Any(_) => {}
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use nmcr_types::{ArgKindBoolean, ArgKindString, Span};
    use pretty_assertions::assert_eq;

    fn make_arg(name: &str, description: &str, kind: ArgKind) -> Arg {
        Arg {
            name: name.to_string(),
            description: description.to_string(),
            kind,
        }
    }

    fn empty_location() -> Location {
        Location {
            path: String::new(),
            span: Span { start: 0, end: 0 },
        }
    }

    #[test]
    fn instructions_include_descriptions_and_args() {
        let mut args = Vec::new();
        args.push(make_arg(
            "name",
            "Name of the component",
            ArgKind::String(ArgKindString),
        ));
        args.push(make_arg(
            "with_css",
            "Generate CSS module",
            ArgKind::Boolean(ArgKindBoolean),
        ));

        let template = Template {
            id: "component".into(),
            name: "Component".into(),
            description: "Create a React component".into(),
            args,
            lang: None,
            content: String::new(),
            location: empty_location(),
        };

        let tool = TemplateTool::from_template(template);
        let instructions = tool.instructions_line();

        assert_eq!(
            instructions,
            "- component → Component — Create a React component (args: name, with_css)"
        );
    }

    #[test]
    fn tool_name_uses_template_id() {
        let template = Template {
            id: "rust_package_gitignore".into(),
            name: "Package Gitignore".into(),
            description: String::new(),
            args: Vec::new(),
            lang: None,
            content: String::new(),
            location: empty_location(),
        };

        let tool = TemplateTool::from_template(template.clone());
        assert_eq!(tool.tool_name, template.id);
    }
}
