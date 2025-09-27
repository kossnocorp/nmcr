use serde::{Deserialize, Serialize};
use std::fmt;

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
pub struct Template {
    pub id: String,
    pub name: String,
    pub description: String,
    pub args: Vec<TemplateArg>,
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
    pub path: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FormattedLocation<'a>(pub &'a Location);

impl<'a> fmt::Display for FormattedLocation<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let location = self.0;
        let path = if location.path.is_empty() {
            "<memory>".to_string()
        } else {
            location.path.clone()
        };

        write!(f, "{}:{}-{}", path, location.span.start, location.span.end)
    }
}
