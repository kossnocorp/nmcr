use nmcr_types::Location;
use std::fmt;

#[derive(Debug, Clone)]
pub struct FormattedLocation<'a>(pub &'a Location);

impl<'a> fmt::Display for FormattedLocation<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let location = self.0;
        let path = if location.path.is_empty() {
            "<memory>".to_string()
        } else {
            location.path.clone()
        };

        write!(f, "{}:{}-{}", path, location.span.start, location.span.end)
    }
}
