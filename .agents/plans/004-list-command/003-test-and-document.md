# Test and document the listing flow

## Objective

Validate the shared catalog behavior and the `nmcr list` command output while updating documentation so users understand how to inspect available templates.

## Tasks

- [x] **Expand catalog unit tests** — exercise the `nmcr_catalog` crate with scenarios covering tree member IDs, duplicate detection, and lookup helpers.
      Include cases for both matching and non-matching tree prefixes to guard against regressions. _Added tests covering nested lookups, duplicate detection, and unmatched ID prefixes._
- [x] **Add CLI integration coverage** — write tests (or golden-output fixtures) that execute `nmcr list` against a sample catalog and assert the rendered hierarchy and metadata formatting.
      Verify standalone file templates and tree members appear exactly once in the expected order. _`render_catalog_to_writer` is exercised against example templates to ensure hierarchy and deduping._
- [x] **Update user-facing docs** — revise CLI help text, README sections, or docs site pages to describe the new `list` command and explain the tree-plus-file hierarchy.
      Mention how IDs relate between trees and their member files so users can map output to generator inputs. _Added `docs/stack/cli-guide.md` detailing the command behavior and ID relationships._
- [x] **Confirm developer workflows** — ensure MCP and generator flows continue to resolve template IDs correctly after adopting the shared crate.
      Run or script smoke tests that mimic their lookups to catch regressions beyond the CLI path. _New MCP catalog test validates tree/file tool exposure alongside catalog lookup coverage._
- [x] **Document follow-up considerations** — note any future enhancements (e.g., sorting, filtering) discovered while testing so they can be tracked separately without blocking release.
      Capture gaps or edge cases the current scope intentionally leaves unresolved. _Captured ideas in plan follow-ups for future filtering and richer metadata._

## Questions

None.

## Notes

Coordinate with documentation maintainers if additional guides or release notes are required.
