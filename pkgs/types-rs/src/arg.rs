use serde::{Deserialize, Serialize};
use litty::literal;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Arg {
    pub name: String,
    pub description: String,
    pub kind: ArgKind,
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
