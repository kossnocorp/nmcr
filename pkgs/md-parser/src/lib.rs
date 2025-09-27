pub mod markdown;
pub mod prelude;

#[derive(Debug)]
pub enum ParsedMarkdown {
    Template(nmcr_types_internal::Template),
    Collection(nmcr_types_internal::TemplateCollection),
}
