use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TemplateArgType {
    #[default]
    Any,
    Boolean,
    String,
    Number,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemplateArg {
    pub name: String,
    pub description: String,
    pub kind: TemplateArgType,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemplateArgs {
    pub items: Vec<TemplateArg>,
}

impl TemplateArgs {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
    pub fn push(&mut self, name: impl Into<String>, description: impl Into<String>) {
        self.items.push(TemplateArg {
            name: name.into(),
            description: description.into(),
            kind: TemplateArgType::Any,
        });
    }
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    pub fn len(&self) -> usize {
        self.items.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Template {
    pub name: String,
    pub description: String,
    pub args: TemplateArgs,
    pub lang: Option<String>,
    pub content: String,
    pub location: Location,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemplateCollection {
    pub name: String,
    pub description: String,
    pub templates: Vec<Template>,
    pub location: Location,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Location {
    pub path: PathBuf,
    pub span: Span,
}
