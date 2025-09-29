use super::{TreeTool, tool::TemplateTool};
use crate::prelude::*;
use nmcr_catalog::TemplateCatalog as SharedCatalog;

pub(crate) struct TemplateCatalog {
    file_tools: Vec<TemplateTool>,
    tree_tools: Vec<TreeTool>,
}

impl TemplateCatalog {
    pub(crate) fn load(paths: &[PathBuf]) -> Result<Self> {
        let catalog = SharedCatalog::load(paths)?;

        let mut file_tools: Vec<TemplateTool> = Vec::new();
        for file in catalog.standalone_files() {
            file_tools.push(TemplateTool::from_template(file.clone()));
        }
        let mut tree_tools: Vec<TreeTool> = Vec::new();
        for tree in catalog.tree_templates() {
            tree_tools.push(TreeTool::from_tree(tree.clone()));
            for file in tree.files() {
                file_tools.push(TemplateTool::from_template(file.clone()));
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
