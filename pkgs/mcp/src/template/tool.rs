use super::render_template;
use crate::prelude::*;
use anyhow::bail;

#[allow(dead_code)]
#[derive(Clone)]
pub(crate) struct TemplateTool {
    template: TemplateFile,
    tool_name: String,
    display_name: String,
    description: String,
    schema: Arc<JsonMap<String, JsonValue>>,
}

impl TemplateTool {
    pub(crate) fn from_template(template: TemplateFile) -> Self {
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
                ensure_required_args(&template, &arguments)
                    .map_err(|err| McpError::invalid_params(err.to_string(), None))?;
                let rendered = render_template(&template.id, &template.content, &arguments)
                    .map_err(|err| McpError::invalid_params(err.to_string(), None))?;
                let rendered_path = match &template.path {
                    Some(path_tpl) => Some(
                        render_template(&format!("{}::path", template.id), path_tpl, &arguments)
                            .map_err(|err| McpError::invalid_params(err.to_string(), None))?,
                    ),
                    None => None,
                };
                let out = nmcr_types::OutputFile {
                    path: rendered_path,
                    lang: template.lang.clone(),
                    content: rendered,
                };
                let output_schema = Self::output_schema(&template);
                Ok(CallToolResult::success(vec![
                    Content::json(out)?,
                    Content::json(output_schema)?,
                ]))
            })
        })
    }

    #[allow(dead_code)]
    pub(crate) fn instructions_line(&self) -> String {
        let mut line = format!("- {} → {}", self.tool_name, self.display_name);
        if !self.template.description.trim().is_empty() {
            line.push_str(&format!(" — {}", self.template.description.trim()));
        }
        if !self.template.args.is_empty() {
            let arg_names: Vec<String> = self
                .template
                .args
                .iter()
                .map(|arg| {
                    if arg.required {
                        arg.name.clone()
                    } else {
                        format!("{}?", arg.name)
                    }
                })
                .collect();
            line.push_str(&format!(" (args: {})", arg_names.join(", ")));
        }
        line
    }

    fn args_schema(args: &[Arg]) -> JsonMap<String, JsonValue> {
        let mut schema = JsonMap::new();
        schema.insert("type".into(), JsonValue::String("object".into()));

        let mut properties = JsonMap::new();
        let mut required = Vec::new();
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
            if arg.required {
                required.push(JsonValue::String(arg.name.clone()));
            }
        }

        schema.insert("properties".into(), JsonValue::Object(properties));
        schema.insert("additionalProperties".into(), JsonValue::Bool(false));
        if !required.is_empty() {
            schema.insert("required".into(), JsonValue::Array(required));
        }
        schema
    }

    fn output_schema(t: &TemplateFile) -> JsonMap<String, JsonValue> {
        let mut properties = JsonMap::new();
        properties.insert("content".into(), json_type("string"));
        properties.insert("lang".into(), json_type("string"));
        if t.path.is_some() {
            properties.insert("path".into(), json_type("string"));
        }

        let mut obj = JsonMap::new();
        obj.insert(
            "$schema".into(),
            JsonValue::String("https://json-schema.org/draft/2020-12/schema".into()),
        );
        obj.insert(
            "title".into(),
            JsonValue::String(format!("{}:OutputFile", t.id)),
        );
        obj.insert("type".into(), JsonValue::String("object".into()));
        obj.insert("properties".into(), JsonValue::Object(properties));

        // Required keys: content always; path only if the template has a path defined
        let mut required = vec![JsonValue::String("content".into())];
        if t.path.is_some() {
            required.push(JsonValue::String("path".into()));
        }
        obj.insert("required".into(), JsonValue::Array(required));
        obj
    }
}

pub(super) fn ensure_required_args(
    template: &TemplateFile,
    args: &JsonMap<String, JsonValue>,
) -> Result<()> {
    let missing: Vec<String> = template
        .args
        .iter()
        .filter(|arg| arg.required && !args.contains_key(&arg.name))
        .map(|arg| arg.name.clone())
        .collect();

    if missing.is_empty() {
        Ok(())
    } else {
        bail!(
            "Missing required argument(s) {} for template '{}'.",
            missing.join(", "),
            template.id
        )
    }
}

pub(crate) fn json_type(t: &str) -> JsonValue {
    let mut m = JsonMap::new();
    m.insert("type".into(), JsonValue::String(t.to_string()));
    JsonValue::Object(m)
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
            required: true,
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

        let template = TemplateFile {
            kind: nmcr_types::TemplateFileKindFile,
            id: "component".into(),
            name: "Component".into(),
            description: "Create a React component".into(),
            args,
            lang: None,
            content: String::new(),
            path: None,
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
        let template = TemplateFile {
            kind: nmcr_types::TemplateFileKindFile,
            id: "rust_package_gitignore".into(),
            name: "Package Gitignore".into(),
            description: String::new(),
            args: Vec::new(),
            lang: None,
            content: String::new(),
            path: None,
            location: empty_location(),
        };

        let tool = TemplateTool::from_template(template.clone());
        assert_eq!(tool.tool_name, template.id);
    }

    #[test]
    fn optional_args_marked_in_instructions() {
        let args = vec![
            Arg {
                name: "name".into(),
                description: String::new(),
                kind: ArgKind::String(ArgKindString),
                required: true,
            },
            Arg {
                name: "suffix".into(),
                description: String::new(),
                kind: ArgKind::String(ArgKindString),
                required: false,
            },
        ];

        let template = TemplateFile {
            kind: nmcr_types::TemplateFileKindFile,
            id: "example".into(),
            name: "Example".into(),
            description: String::new(),
            args,
            lang: None,
            content: String::new(),
            path: None,
            location: empty_location(),
        };

        let tool = TemplateTool::from_template(template);
        let instructions = tool.instructions_line();
        assert!(instructions.contains("suffix?"));
    }
}
