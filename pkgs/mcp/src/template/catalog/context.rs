use crate::prelude::*;
use nmcr_types_internal::FormattedLocation;
use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct TemplateCatalogContext {
    pub(crate) claimed: HashMap<String, Location>,
}

impl TemplateCatalogContext {
    pub(crate) fn claim_id(&mut self, id: &str, location: &Location) -> Result<()> {
        if let Some(existing) = self.claimed.get(id) {
            let first = FormattedLocation(existing).to_string();
            let duplicate = FormattedLocation(location).to_string();
            return Err(anyhow!("Duplicate template id: {}", id))
                .with_context(|| format!("duplicate occurrence at {duplicate}"))
                .with_context(|| format!("first occurrence at {first}"));
        }

        self.claimed.insert(id.to_string(), location.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nmcr_types::Span;

    fn make_location(path: &str, start: usize, end: usize) -> Location {
        Location {
            path: path.to_string(),
            span: Span { start, end },
        }
    }

    #[test]
    fn duplicate_ids_return_error() {
        let mut context = TemplateCatalogContext::default();
        let first_loc = make_location("templates/a.md", 1, 10);
        context
            .claim_id("duplicate", &first_loc)
            .expect("first registration succeeds");

        let second_loc = make_location("templates/b.md", 5, 15);
        let err = context
            .claim_id("duplicate", &second_loc)
            .expect_err("duplicate should fail");

        let rendered = format!("{err:?}");
        assert!(rendered.contains("Duplicate template id: duplicate"));
        assert!(rendered.contains("duplicate occurrence at templates/b.md:5-15"));
        assert!(rendered.contains("first occurrence at templates/a.md:1-10"));
    }
}
