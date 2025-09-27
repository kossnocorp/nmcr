//! Template schema: single-file and tree templates, plus a collection wrapper.

use litty::literal;
use serde::{Deserialize, Serialize};

/// A single-file template node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplateFile {
    /// Discriminator for unions.
    pub kind: TemplateFileKindFile,
    pub id: String,
    pub name: String,
    pub description: String,
    pub args: Vec<super::arg::Arg>,
    /// Optional language hint from the fenced code block.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    /// Raw template content.
    pub content: String,
    /// Optional relative path to use when writing to disk.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub location: super::location::Location,
}

#[literal("file")]
pub struct TemplateFileKindFile;

/// A tree of template files grouped under a single heading.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplateTree {
    /// Discriminator for unions.
    pub kind: TemplateTreeKindTree,
    pub id: String,
    pub name: String,
    /// Prose between the tree heading and the first file.
    pub description: String,
    pub files: Vec<Template>,
    pub location: super::location::Location,
}

#[literal("tree")]
pub struct TemplateTreeKindTree;

/// Union of templates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Template {
    TemplateTree(TemplateTree),
    TemplateFile(TemplateFile),
}

/// A collection of top-level templates parsed from a single markdown file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplateCollection {
    pub name: String,
    pub description: String,
    pub templates: Vec<Template>,
    pub location: super::location::Location,
}
