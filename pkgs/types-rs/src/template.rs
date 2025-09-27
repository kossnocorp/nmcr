use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub description: String,
    pub args: Vec<super::arg::Arg>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    pub content: String,
    pub location: super::location::Location,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplateCollection {
    pub name: String,
    pub description: String,
    pub templates: Vec<Template>,
    pub location: super::location::Location,
}
