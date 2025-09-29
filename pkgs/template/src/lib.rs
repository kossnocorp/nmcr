use std::collections::BTreeMap;
use std::fmt;

use anyhow::{Context, Result, anyhow};
use handlebars::{Handlebars, RenderError, no_escape};
use serde::Serialize;
use serde_json::{Map as JsonMap, Value as JsonValue};

/// A reusable Handlebars renderer configured for strict argument handling.
#[derive(Debug)]
pub struct TemplateRenderer {
    registry: Handlebars<'static>,
}

impl TemplateRenderer {
    /// Create a new renderer with strict mode enabled.
    pub fn new() -> Self {
        let mut registry = Handlebars::new();
        registry.set_strict_mode(true);
        registry.register_escape_fn(no_escape);
        Self { registry }
    }

    /// Render a template string with the provided JSON object context.
    pub fn render_map(
        &self,
        template_id: &str,
        template: &str,
        context: &JsonMap<String, JsonValue>,
    ) -> Result<String> {
        let value = JsonValue::Object(context.clone());
        self.render_value(template_id, template, &value)
    }

    /// Render a template string with any serializable context.
    pub fn render<T>(&self, template_id: &str, template: &str, context: &T) -> Result<String>
    where
        T: Serialize,
    {
        let value = serde_json::to_value(context)
            .with_context(|| format!("Failed to serialize render context for '{template_id}'"))?;
        self.render_value(template_id, template, &value)
    }

    fn render_value(
        &self,
        template_id: &str,
        template: &str,
        context: &JsonValue,
    ) -> Result<String> {
        if !context.is_object() {
            return Err(anyhow!(
                "Context for template '{}' must be a JSON object, got {}",
                template_id,
                ContextType(context)
            ));
        }

        self.registry
            .render_template(template, context)
            .map_err(|err| format_render_error(template_id, template, err))
    }
}

impl Default for TemplateRenderer {
    fn default() -> Self {
        Self::new()
    }
}

struct ContextType<'a>(&'a JsonValue);

impl<'a> fmt::Display for ContextType<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            JsonValue::Null => write!(f, "null"),
            JsonValue::Bool(_) => write!(f, "boolean"),
            JsonValue::Number(_) => write!(f, "number"),
            JsonValue::String(_) => write!(f, "string"),
            JsonValue::Array(_) => write!(f, "array"),
            JsonValue::Object(_) => write!(f, "object"),
        }
    }
}

fn format_render_error(template_id: &str, template: &str, err: RenderError) -> anyhow::Error {
    let mut message = format!("Failed to render template '{template_id}': {}", err);

    if let Some(line) = err.line_no {
        if let Some(snippet) = line_snippet(template, line) {
            if let Some(column) = err.column_no {
                message.push_str(&format!("\n --> line {line}, column {column}:"));
            } else {
                message.push_str(&format!("\n --> line {line}:"));
            }
            message.push_str(&format!("\n     {snippet}"));
        }
    }

    anyhow!(message)
}

fn line_snippet(src: &str, line: usize) -> Option<String> {
    let line_idx = line.saturating_sub(1);
    src.lines().nth(line_idx).map(|s| s.trim_end().to_string())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaceholderOccurrence {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placeholder {
    pub name: String,
    pub occurrences: Vec<PlaceholderOccurrence>,
}

pub fn discover_placeholders(template: &str) -> Vec<Placeholder> {
    let mut buckets: BTreeMap<String, Vec<PlaceholderOccurrence>> = BTreeMap::new();
    for m in scan_template(template) {
        buckets
            .entry(m.name)
            .or_default()
            .push(PlaceholderOccurrence {
                start: m.start,
                end: m.end,
            });
    }
    buckets
        .into_iter()
        .map(|(name, occurrences)| Placeholder { name, occurrences })
        .collect()
}

#[derive(Debug)]
struct TokenMatch {
    name: String,
    start: usize,
    end: usize,
}

fn scan_template(template: &str) -> Vec<TokenMatch> {
    let mut matches = Vec::new();
    let mut cursor = 0;
    let bytes = template.as_bytes();
    while cursor < bytes.len() {
        let relative = match template[cursor..].find("{{") {
            Some(idx) => idx,
            None => break,
        };
        let start = cursor + relative;
        let mut idx = start + 2;
        if idx < bytes.len() && bytes[idx] == b'{' {
            idx += 1;
        }

        let search_slice = &template[idx..];
        let close_rel = match search_slice.find("}}") {
            Some(idx) => idx,
            None => break,
        };
        let end = idx + close_rel;
        let mut closing_len = 2;
        if template[end..].starts_with("}}}") {
            closing_len = 3;
        }

        cursor = (end + closing_len).min(template.len());

        let trimmed = template[idx..end].trim();
        if trimmed.is_empty() {
            continue;
        }

        let trimmed = trimmed.trim_start_matches('~').trim_end_matches('~').trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with('!') {
            continue;
        }
        if trimmed.starts_with('/') {
            continue;
        }
        if trimmed == "else" || trimmed.starts_with("else ") {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let first = parts[0];
        let rest = &parts[1..];
        let mut collected: Vec<String> = Vec::new();
        if let Some(first_char) = first.chars().next() {
            match first_char {
                '#' | '^' => {
                    for token in rest {
                        if let Some(name) = normalize_placeholder_name(token) {
                            collected.push(name);
                        }
                    }
                }
                '>' => {
                    for (idx, token) in rest.iter().enumerate() {
                        if idx == 0 {
                            continue;
                        }
                        if let Some(name) = normalize_placeholder_name(token) {
                            collected.push(name);
                        }
                    }
                }
                '&' => {
                    if let Some(name) = normalize_placeholder_name(&first[1..]) {
                        collected.push(name);
                    }
                    for token in rest {
                        if let Some(name) = normalize_placeholder_name(token) {
                            collected.push(name);
                        }
                    }
                }
                _ => {
                    if rest.is_empty() {
                        if let Some(name) = normalize_placeholder_name(first) {
                            collected.push(name);
                        }
                    } else {
                        for token in rest {
                            if let Some(name) = normalize_placeholder_name(token) {
                                collected.push(name);
                            }
                        }
                    }
                }
            }
        }

        for name in collected {
            matches.push(TokenMatch {
                name,
                start,
                end: cursor,
            });
        }
    }
    matches
}

fn normalize_placeholder_name(token: &str) -> Option<String> {
    if token.contains('|') {
        return None;
    }
    let mut candidate = token.trim();
    candidate = candidate
        .trim_matches(|c: char| matches!(c, '|' | ',' | ')' | '('))
        .trim();

    while candidate.starts_with("../") {
        candidate = candidate.trim_start_matches("../");
    }

    if let Some((before, _)) = candidate.split_once('=') {
        candidate = before.trim();
    }

    candidate = candidate.trim_matches('|').trim();

    if candidate.is_empty() || candidate == "." || candidate.eq_ignore_ascii_case("this") {
        return None;
    }

    if candidate.starts_with('@') {
        return None;
    }

    const RESERVED: &[&str] = &["as", "in", "let"];
    if RESERVED.iter().any(|kw| candidate.eq_ignore_ascii_case(kw)) {
        return None;
    }

    let valid = candidate
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.');
    if valid {
        Some(candidate.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn renders_with_context() {
        let renderer = TemplateRenderer::new();
        let mut ctx = JsonMap::new();
        ctx.insert("name".into(), JsonValue::String("world".into()));
        let rendered = renderer
            .render_map("greeting", "Hello, {{name}}!", &ctx)
            .expect("rendered");
        assert_eq!(rendered, "Hello, world!");
    }

    #[test]
    fn renders_without_html_escaping() {
        let renderer = TemplateRenderer::new();
        let rendered = renderer
            .render(
                "pkg",
                "{{author}}",
                &json!({"author": "Sasha Koss <koss@nocorp.me>"}),
            )
            .expect("rendered");
        assert_eq!(rendered, "Sasha Koss <koss@nocorp.me>");
    }

    #[test]
    fn errors_on_missing_variable() {
        let renderer = TemplateRenderer::new();
        let err = renderer
            .render_map("greeting", "Hello, {{name}}!", &JsonMap::new())
            .expect_err("missing variable should fail");
        let rendered = format!("{err}");
        assert!(rendered.contains("greeting"));
        assert!(rendered.contains("line"));
    }

    #[test]
    fn rejects_non_object_context() {
        let renderer = TemplateRenderer::new();
        let err = renderer
            .render("tmpl", "{{this}}", &json!([1, 2, 3]))
            .expect_err("array context should fail");
        assert!(err.to_string().contains("must be a JSON object"));
    }

    #[test]
    fn discovers_placeholders() {
        let found = discover_placeholders("Hello {{ name }} {{#if flag}}{{greeting}}{{/if}}");
        let names: Vec<_> = found.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["flag", "greeting", "name"]);
    }

    #[test]
    fn discover_skips_alias_tokens() {
        let tpl = "{{#each items as |item|}}{{item.name}}{{/each}}";
        let found = discover_placeholders(tpl);
        let names: Vec<_> = found.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["item.name", "items"]);
    }
}
