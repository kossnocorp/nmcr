# Revise Generation Interfaces

## Objective

Rework the CLI `gen` command and MCP tool schema so callers pass `key=value` argument pairs that feed the shared Handlebars renderer for both paths and content.

## Tasks

- [ ] **Refactor CLI argument parsing** — change the `gen` command signature to accept `Vec<String>` argument pairs, parse them into a context map, and validate required keys before invocation.
      Update help text and error messaging to demonstrate the new usage pattern.
- [ ] **Wire CLI execution to renderer** — pass the parsed arguments into the shared Handlebars utility when generating file content and interpolated paths.
      Ensure tree outputs and directory creation use rendered segments consistently.
- [ ] **Update MCP tool definitions** — adjust the MCP schema so tool arguments reflect the simplified pair format, including JSON Schema updates and any structured content metadata.
      Confirm MCP execution passes the parsed map to the renderer just like the CLI.
- [ ] **Refresh tests and examples** — modify unit/integration tests, documentation snippets, and sample commands to cover the new `nmcr gen template_id arg=value` syntax and successful interpolation flows.
      Add regression coverage for missing-argument failures surfaced by Handlebars.

## Questions

None.

## Notes

Coordinate CLI and MCP messaging so users see identical guidance on required arguments.
