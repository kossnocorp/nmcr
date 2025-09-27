pub mod markdown;
pub mod prelude;

#[derive(Debug)]
pub enum ParsedMarkdown {
    Template(nmcr_types::Template),
    Tree(nmcr_types::TemplateTree),
    Collection(nmcr_types::TemplateCollection),
}
