use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Location {
    pub path: String,
    pub span: super::span::Span,
}
