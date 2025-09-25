use crate::prelude::*;

#[derive(Default)]
pub struct TemplateCatalogContext {
    pub used_names: HashMap<String, usize>,
    pub fallback_counter: usize,
}
