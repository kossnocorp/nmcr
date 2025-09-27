# Tests and Docs

## Objective

Backstop the new tree template functionality with automated tests and refreshed documentation/examples so contributors can rely on the feature confidently.

## Tasks

- [x] **Expand parser test coverage** — add fixtures exercising inline path extraction, tree detection, per-file template emission, and error cases (missing paths, malformed hierarchies) in `pkgs/md-parser/tests`.
      Verify legacy single-file templates continue to pass.
- [ ] **Add CLI integration tests** — script end-to-end runs of `nmcr gen` covering `--print`, directory output, path-missing errors, and tree generation into nested directories.
      Use temporary workspaces to avoid polluting the repo.
- [ ] **Test MCP structured content** — introduce tests (unit or contract) that ensure Structured Content responses include the expected JSON Schema and payload for both file and tree outputs.
      Confirm optional path handling matches the schema.
- [ ] **Refresh examples and docs** — update Markdown templates (`examples/basic/tmpls/*.md`) plus `docs/aspects/template.md` and related guides to demonstrate the new syntax and CLI/MCP flows.
      Include before/after snippets showing inline path markers and tree descriptions.
- [ ] **Document migration notes** — add a changelog or migration entry summarizing behavioral changes for existing users, highlighting breaking changes (CLI behavior shift, Structured Content output).
      Coordinate with README or CLI help updates as needed.

## Questions

None.

## Notes

- Where possible, reuse generated outputs from tests as documentation snippets to keep examples up to date.
- Ensure docs mention the requirement for paths when writing to disk and how to use `--print` for inspection.
