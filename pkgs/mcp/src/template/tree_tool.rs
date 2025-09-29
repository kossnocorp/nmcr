use super::render_template;
use super::tool::{ensure_required_args, json_type};
use crate::prelude::*;
use nmcr_catalog::CatalogTree;
use nmcr_types::{Arg, ArgKind};
use std::collections::BTreeSet;

#[allow(dead_code)]
#[derive(Clone)]
pub(crate) struct TreeTool {
    tree: CatalogTree,
    tool_name: String,
    #[allow(dead_code)]
    display_name: String,
    description: String,
    schema: Arc<JsonMap<String, JsonValue>>,
}

impl TreeTool {
    pub(crate) fn from_tree(tree: CatalogTree) -> Self {
        let tool_name = tree.id().to_string();
        let display_name = if tree.name().trim().is_empty() {
            format!("{} (tree)", tool_name)
        } else {
            tree.name().trim().to_string()
        };

        let description = if tree.description().trim().is_empty() {
            format!("Render the {} tree.", display_name)
        } else {
            tree.description().trim().to_string()
        };

        let schema = Arc::new(Self::args_schema(&tree));

        Self {
            tree,
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
        let tree = self.tree.clone();

        ToolRoute::new_dyn(tool, move |mut context| {
            let tree = tree.clone();
            Box::pin(async move {
                let args = context.arguments.take().unwrap_or_default();
                let mut files: Vec<nmcr_types::OutputFile> = Vec::new();
                for file in tree.files() {
                    ensure_required_args(file, &args)
                        .map_err(|err| McpError::invalid_params(err.to_string(), None))?;
                    let rendered = render_template(&file.id, &file.content, &args)
                        .map_err(|err| McpError::invalid_params(err.to_string(), None))?;
                    let rendered_path = match &file.path {
                        Some(path_tpl) => Some(
                            render_template(&format!("{}::path", file.id), path_tpl, &args)
                                .map_err(|err| McpError::invalid_params(err.to_string(), None))?,
                        ),
                        None => None,
                    };
                    files.push(nmcr_types::OutputFile {
                        path: rendered_path,
                        lang: file.lang.clone(),
                        content: rendered,
                    });
                }
                let out = nmcr_types::OutputTree { files };
                let output_schema = Self::output_schema(&tree);
                Ok(CallToolResult::success(vec![
                    Content::json(out)?,
                    Content::json(output_schema)?,
                ]))
            })
        })
    }

    #[allow(dead_code)]
    pub(crate) fn instructions_line(&self) -> String {
        format!("- {} → {} (tree)", self.tool_name, self.display_name)
    }

    fn args_schema(tree: &CatalogTree) -> JsonMap<String, JsonValue> {
        // Union of args across files in the tree
        let mut schema = JsonMap::new();
        schema.insert("type".into(), JsonValue::String("object".into()));

        let mut properties = JsonMap::new();
        let mut required = BTreeSet::new();
        for file in tree.files() {
            for arg in &file.args {
                properties.entry(arg.name.clone()).or_insert_with(|| {
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
                    JsonValue::Object(prop)
                });
                if arg.required {
                    required.insert(arg.name.clone());
                }
            }
        }
        schema.insert("properties".into(), JsonValue::Object(properties));
        schema.insert("additionalProperties".into(), JsonValue::Bool(false));
        if !required.is_empty() {
            let req_values = required.into_iter().map(JsonValue::String).collect();
            schema.insert("required".into(), JsonValue::Array(req_values));
        }
        schema
    }

    fn output_schema(tree: &CatalogTree) -> JsonMap<String, JsonValue> {
        // Build item schema; require path only if every file has a path
        let all_have_path = tree.files().iter().all(|f| f.path.is_some());
        let mut item_props = JsonMap::new();
        item_props.insert("content".into(), json_type("string"));
        item_props.insert("lang".into(), json_type("string"));
        if all_have_path {
            item_props.insert("path".into(), json_type("string"));
        }
        let mut item = JsonMap::new();
        item.insert("type".into(), JsonValue::String("object".into()));
        item.insert("properties".into(), JsonValue::Object(item_props));
        let mut req = vec![JsonValue::String("content".into())];
        if all_have_path {
            req.push(JsonValue::String("path".into()));
        }
        item.insert("required".into(), JsonValue::Array(req));

        let mut props = JsonMap::new();
        let mut files = JsonMap::new();
        files.insert("type".into(), JsonValue::String("array".into()));
        files.insert("items".into(), JsonValue::Object(item));
        props.insert("files".into(), JsonValue::Object(files));

        let mut schema = JsonMap::new();
        schema.insert(
            "$schema".into(),
            JsonValue::String("https://json-schema.org/draft/2020-12/schema".into()),
        );
        schema.insert(
            "title".into(),
            JsonValue::String(format!("{}:OutputTree", tree.id())),
        );
        schema.insert("type".into(), JsonValue::String("object".into()));
        schema.insert("properties".into(), JsonValue::Object(props));
        schema.insert(
            "required".into(),
            JsonValue::Array(vec![JsonValue::String("files".into())]),
        );
        schema
    }
}
