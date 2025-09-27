//! Deterministic identifier utilities for nmcr entities.

/// Generates deterministic identifiers for hierarchical entities.
#[derive(Debug, Default, Clone, Copy)]
pub struct EntityId;

impl EntityId {
    /// Create a new generator.
    pub fn new() -> Self {
        Self
    }

    /// Produce an identifier from an ordered set of human-readable segments.
    ///
    /// Empty or fully sanitized segments are skipped. Remaining segments are
    /// joined with underscores in lowercase `snake_case` form.
    pub fn from_segments<I, S>(&self, segments: I) -> String
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let sanitized: Vec<String> = segments
            .into_iter()
            .map(|segment| Self::normalize_segment(segment.as_ref()))
            .filter(|segment| !segment.is_empty())
            .collect();

        sanitized.join("_")
    }

    /// Normalize a single segment to a `snake_case` component.
    pub fn normalize_segment(input: &str) -> String {
        let mut out = String::new();
        let mut last_was_separator = false;

        for ch in input.chars() {
            if ch.is_ascii_alphanumeric() {
                out.push(ch.to_ascii_lowercase());
                last_was_separator = false;
            } else if ch.is_alphanumeric() {
                for lower in ch.to_lowercase() {
                    out.push(lower);
                }
                last_was_separator = false;
            } else if !out.is_empty() && !last_was_separator {
                out.push('_');
                last_was_separator = true;
            } else {
                // Skip duplicated separators and leading punctuation.
            }
        }

        if out.ends_with('_') {
            out.pop();
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_segments() {
        let cases = [
            ("Rust Crate", "rust_crate"),
            ("API::Client", "api_client"),
            ("  hello-world  ", "hello_world"),
            ("???", ""),
            ("Tęmplaté", "tęmplaté"),
        ];

        for (input, expected) in cases {
            assert_eq!(EntityId::normalize_segment(input), expected);
        }
    }

    #[test]
    fn builds_ids_from_segments() {
        let generator = EntityId::new();
        let id = generator.from_segments(["Rust Crate", "Manifest"]);
        assert_eq!(id, "rust_crate_manifest");
    }

    #[test]
    fn drops_empty_segments() {
        let generator = EntityId::new();
        let id = generator.from_segments(["???", " Rust File "]);
        assert_eq!(id, "rust_file");
    }

    #[test]
    fn avoids_double_underscores() {
        let generator = EntityId::new();
        let id = generator.from_segments(["API ::: Client -- HTTP"]);
        assert_eq!(id, "api_client_http");
        assert!(!id.contains("__"));
    }
}
