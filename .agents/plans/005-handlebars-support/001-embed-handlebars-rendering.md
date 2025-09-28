# Embed Handlebars Rendering

## Objective

Introduce the `handlebars` crate and expose a shared rendering utility that compiles templates for both file contents and generated paths with strict missing-variable errors.

## Tasks

- [ ] **Review current rendering flows** — trace how template content and paths are produced today across CLI and MCP crates, noting shared helpers and spots that need to swap to Handlebars.
      Capture existing error handling so new diagnostics stay consistent.
- [ ] **Add Handlebars dependency and engine wrapper** — update the workspace Cargo manifests to include `handlebars`, create a centralized renderer module, and configure strict missing-variable detection plus any necessary helpers.
      Ensure the renderer exposes functions usable from both CLI and MCP layers without duplicate setup.
- [ ] **Integrate renderer for content and path generation** — replace current string interpolation logic with calls into the shared Handlebars wrapper when producing template outputs.
      Cover single-file and tree templates so nested paths also pass through the renderer.
- [ ] **Propagate actionable error reporting** — surface template ID, failing segment, and original Handlebars error details whenever rendering fails.
      Add unit tests around error cases to lock in the "missing arguments" messaging requirement.

## Questions

None.

## Notes

Keep escaping and helper registration centralized so CLI and MCP outputs remain identical.
