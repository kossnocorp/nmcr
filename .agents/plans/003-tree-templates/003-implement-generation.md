# Implement Generation

## Objective

Deliver working MCP and CLI generation flows that render templates (file or tree) using the new data model, returning Structured Content or writing files instead of debug output.

## Tasks

- [ ] **Refactor MCP tool responses** — update `pkgs/mcp` so each template tool renders through the new `TemplateOutput` type and returns Structured Content complying with MCP 2025-06-18.
      Produce per-tool JSON Schemas that encode optional file paths and tree structures. (Pending.)
- [x] **Design CLI UX contract** — specify argument parsing for `gen` (template ID/slug, destination path vs. `--print`, overwrite policy, dry-run messaging) and document expected behaviors before coding.
      Align error messages with the plan requirements (missing paths, path/tree mismatches).
- [x] **Rebuild `gen` command implementation** — replace the current playground logic with real rendering: load templates, render the content with arguments, and either print the structured JSON or write files/directories when a path is provided.
      Ensure tree outputs validate that every file has a path and create directories as needed.
- [ ] **Share rendering utilities** — extract or enhance helper functions that iterate through tree nodes, apply arguments, and format output for both CLI and MCP so behavior stays consistent.
      Consider locating shared code in a library crate if duplication grows. (Render is a stub; shared utilities to follow.)
- [x] **Integrate with existing tooling** — update any callers or tests that relied on the previous debug prints to consume the new outputs, and ensure command-line help/documentation reflects the new behavior.
      Capture exit codes and logging expectations for success and failure cases. (CLI and MCP catalog updated.)

## Questions

None.

## Notes

- Respect the plan’s rule: if a template lacks a path and the user supplies a destination directory, exit with an error before writing anything.
- When `--print` is used, emit the structured JSON (matching `TemplateOutput`) to stdout so scripts can consume it.
- Maintain parity between MCP and CLI rendering for deterministic outputs.
