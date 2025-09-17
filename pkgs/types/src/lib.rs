use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TemplateArgType {
    Any,
    Boolean,
    String,
    Number,
}

impl Default for TemplateArgType {
    fn default() -> Self { TemplateArgType::Any }
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
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemplateCollection {
    pub name: String,
    pub description: String,
    pub templates: Vec<Template>,
}
