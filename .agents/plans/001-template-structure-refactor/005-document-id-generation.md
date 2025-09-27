# Document ID generation

## Objective

Capture the canonical ID format and examples so contributors understand how template identifiers are derived and where the implementation lives.

## Tasks

- [ ] **Outline specification** — draft `docs/architecture/id.md` with a concise description of the lowercase snake_case convention and hierarchy-based segments.
      Reference how headings map to segments and note constraints such as allowed characters.
- [ ] **Provide worked examples** — include short Markdown snippets demonstrating multi-level headings and the resulting IDs (for example, `Rust Crate` → `rust_crate` and `Manifest` subheaders).
      Highlight how non-alphanumeric characters are handled if applicable.
- [ ] **Document tooling source of truth** — describe the `nmcr_id` package as the implementation home, summarizing the main functions exposed for other crates.
      Mention any expectations around normalization or validation enforced by the generator.
- [ ] **Cross-link ecosystem** — update any existing developer docs or READMEs that describe template metadata to point to the new architecture document.
      Ensure navigation menus or indexes include the new page if necessary.

## Questions

None.

## Notes

Keep the document tightly scoped to ID semantics so future tooling changes stay centralized.
