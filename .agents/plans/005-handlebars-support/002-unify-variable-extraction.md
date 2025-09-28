# Unify Variable Extraction

## Objective

Detect Handlebars placeholders in template Markdown and path definitions, then merge them into the documented variable list without losing author-provided descriptions.

## Tasks

- [ ] **Survey template parsing code** — map where Markdown frontmatter, content, and path metadata are currently parsed so we know where to plug in Handlebars placeholder scanning.
      Note existing variable-merging logic and any assumptions about argument sources.
- [ ] **Implement placeholder discovery** — extend the parser to scan content and path strings for `{{ }}` expressions, normalize identifier formatting, and deduplicate results.
      Capture location metadata where possible for future diagnostics.
- [ ] **Merge detected variables with metadata** — update the variable collection routines to union manual definitions and discovered placeholders while preserving descriptions and optional flags.
      Ensure newly discovered variables receive sensible placeholder entries without overwriting manual docs.
- [ ] **Add validation for conflicts** — flag when manual metadata conflicts with detected usage (such as optional vs. required) and emit actionable warnings or errors during parsing.
      Cover cases where placeholders reference unknown dotted paths or helpers.

## Questions

None.

## Notes

Maintain backward compatibility for templates that already declare all variables manually.
