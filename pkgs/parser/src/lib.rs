pub mod markdown;

// ParsedMarkdown is a parser-specific enum that references shared types
pub enum ParsedMarkdown {
    Template(nmcr_types::Template),
    Collection(nmcr_types::TemplateCollection),
}

pub mod prelude {
    pub use crate::markdown::{parse_file, parse_str};
    pub use crate::ParsedMarkdown;
    pub use anyhow::{anyhow, bail, Context, Result};
    pub use nmcr_types::{Template, TemplateArg, TemplateArgType, TemplateArgs, TemplateCollection};
}
