pub mod markdown;
pub mod prelude;

pub enum ParsedMarkdown {
    Template(nmcr_types::Template),
    Collection(nmcr_types::TemplateCollection),
}
