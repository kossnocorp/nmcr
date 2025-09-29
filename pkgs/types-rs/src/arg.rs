use litty::literal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Arg {
    pub name: String,
    pub description: String,
    pub kind: ArgKind,
    #[serde(default = "default_required")]
    pub required: bool,
}

fn default_required() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArgKind {
    Any(ArgKindAny),
    Boolean(ArgKindBoolean),
    String(ArgKindString),
    Number(ArgKindNumber),
}

#[literal("any")]
pub struct ArgKindAny;

#[literal("boolean")]
pub struct ArgKindBoolean;

#[literal("string")]
pub struct ArgKindString;

#[literal("number")]
pub struct ArgKindNumber;
