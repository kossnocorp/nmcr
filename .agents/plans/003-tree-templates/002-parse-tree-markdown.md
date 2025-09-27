# Parse Tree Markdown

## Objective

Update the Markdown parser to recognize paths and tree structures, producing the new `Template` enum variants without regressing existing collection behavior.

## Tasks

- [ ] **Survey current parsing flow** — trace `pkgs/md-parser/src/markdown.rs` to understand how sections become templates today, noting where paths, descriptions, and collections are derived using the current `nmcr_types` structs.
- [ ] **Capture inline path metadata** — extend the parser to detect inline code lines ending with `:` (for example, `` `Cargo.toml`: ``) before a template block and store the extracted path on the corresponding `TemplateFile`.
      Ensure header-derived names remain a fallback when no inline path appears.
- [ ] **Emit tree descriptions and files** — adjust section grouping so prose between a tree heading and its first file becomes the tree description, and assemble child sections into `TemplateFile` entries including args, language, and spans.
      Treat the presence of paths on every child file as the signal to produce a `Template::Tree` node.
- [ ] **Support individual file templates** — make sure each tree file also emits its own `Template::File` variant so callers can generate single files when desired.
      Validate that collections such as `rust-crate.md` still produce per-template outputs alongside the tree wrapper.
- [ ] **Add validation and error messaging** — surface clear errors when expected paths are missing (tree detection fails), when code blocks are absent, or when malformed hierarchies appear, keeping backward-compatible behavior for existing single-file templates.
      Update tests or add new ones to capture each edge case.

## Questions

None.

## Notes

- Reuse existing span/location helpers to keep diagnostics consistent.
- Remember to update examples (`npm.md`, `rust-crate.md`) once parsing rules change so they remain valid fixtures for tests.
