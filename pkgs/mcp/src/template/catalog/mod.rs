use super::{TreeTool, tool::TemplateTool};
use crate::prelude::*;

mod context;
use context::TemplateCatalogContext;

pub(crate) struct TemplateCatalog {
    file_tools: Vec<TemplateTool>,
    tree_tools: Vec<TreeTool>,
}

impl TemplateCatalog {
    pub(crate) fn load(paths: &[PathBuf]) -> Result<Self> {
        let mut file_tools: Vec<TemplateTool> = Vec::new();
        let mut tree_tools: Vec<TreeTool> = Vec::new();
        let mut context = TemplateCatalogContext::default();

        for path in paths {
            let parsed = parse_file(path)
                .with_context(|| format!("Failed to parse template file: {}", path.display()))?;

            match parsed {
                ParsedMarkdown::Template(t) => match t {
                    Template::TemplateFile(f) => {
                        context.claim_id(&f.id, &f.location)?;
                        file_tools.push(TemplateTool::from_template(f));
                    }
                    Template::TemplateTree(tr) => {
                        // Add a tree tool
                        tree_tools.push(TreeTool::from_tree(tr.clone()));
                        for tf in tr.files {
                            if let Template::TemplateFile(f) = tf {
                                context.claim_id(&f.id, &f.location)?;
                                file_tools.push(TemplateTool::from_template(f));
                            }
                        }
                    }
                },

                ParsedMarkdown::Tree(tree) => {
                    tree_tools.push(TreeTool::from_tree(tree.clone()));
                    for tf in tree.files {
                        if let Template::TemplateFile(f) = tf {
                            context.claim_id(&f.id, &f.location)?;
                            file_tools.push(TemplateTool::from_template(f));
                        }
                    }
                }

                ParsedMarkdown::Collection(collection) => {
                    for t in collection.templates {
                        match t {
                            Template::TemplateFile(f) => {
                                context.claim_id(&f.id, &f.location)?;
                                file_tools.push(TemplateTool::from_template(f));
                            }
                            Template::TemplateTree(tr) => {
                                // Add a tree tool; files are already emitted as TemplateFile entries.
                                tree_tools.push(TreeTool::from_tree(tr));
                            }
                        }
                    }
                }
            }
        }

        Ok(Self {
            file_tools,
            tree_tools,
        })
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.file_tools.is_empty() && self.tree_tools.is_empty()
    }

    pub(crate) fn file_tools(&self) -> &[TemplateTool] {
        &self.file_tools
    }

    pub(crate) fn tree_tools(&self) -> &[TreeTool] {
        &self.tree_tools
    }

    pub(crate) fn instructions(&self) -> Option<String> {
        if self.is_empty() {
            return None;
        }
        Some("Templates available via tools: refer to the tool list in your client".to_string())
    }
}
