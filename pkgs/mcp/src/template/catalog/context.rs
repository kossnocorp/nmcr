use crate::prelude::*;
use nmcr_types_internal::{FormattedLocation, Location};
use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct TemplateCatalogContext {
    pub(crate) claimed: HashMap<String, Location>,
}

impl TemplateCatalogContext {
    pub(crate) fn claim_id(&mut self, template: &Template) -> Result<()> {
        if let Some(existing) = self.claimed.get(&template.id) {
            let first = FormattedLocation(existing).to_string();
            let duplicate = FormattedLocation(&template.location).to_string();
            return Err(anyhow!("Duplicate template id: {}", template.id))
                .with_context(|| format!("duplicate occurrence at {duplicate}"))
                .with_context(|| format!("first occurrence at {first}"));
        }

        self.claimed
            .insert(template.id.clone(), template.location.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nmcr_types_internal::Span;

    fn make_template(path: &str, start: usize, end: usize) -> Template {
        Template {
            id: "duplicate".into(),
            name: "Example".into(),
            description: String::new(),
            args: Vec::new(),
            lang: None,
            content: String::new(),
            location: Location {
                path: path.to_string(),
                span: Span { start, end },
            },
        }
    }

    #[test]
    fn duplicate_ids_return_error() {
        let mut context = TemplateCatalogContext::default();
        let first = make_template("templates/a.md", 1, 10);
        context
            .claim_id(&first)
            .expect("first registration succeeds");

        let second = make_template("templates/b.md", 5, 15);
        let err = context
            .claim_id(&second)
            .expect_err("duplicate should fail");

        let rendered = format!("{err:?}");
        assert!(rendered.contains("Duplicate template id: duplicate"));
        assert!(rendered.contains("duplicate occurrence at templates/b.md:5-15"));
        assert!(rendered.contains("first occurrence at templates/a.md:1-10"));
    }
}
