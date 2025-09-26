pub mod markdown;
pub mod prelude;

#[derive(Debug)]
pub enum ParsedMarkdown {
    Template(nmcr_types::Template),
    Collection(nmcr_types::TemplateCollection),
}
