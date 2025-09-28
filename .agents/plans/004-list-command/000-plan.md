# Surface Template Catalog via `list`

## Brief

Design the work to introduce an `nmcr list` command that loads the project's template catalog, keeps tree member files associated with their parent trees, and prints both tree and standalone file templates with key metadata in a readable hierarchy.

## Plan

- [ ] [Share catalog parsing via new crate](.agents/plans/004-list-command/001-share-catalog-parsing.md): Refactor the template catalog builder into a reusable `nmcr_catalog` crate consumed by both the CLI and MCP crates while keeping tree member files attached to their parent tree definitions.
- [ ] [Add CLI command to render catalog](.agents/plans/004-list-command/002-add-cli-list-command.md): Define the `list` subcommand, load the catalog via the shared API, and render tree/file templates with indentation and metadata without duplicating tree member files at the root level.
- [ ] [Test and document the listing flow](.agents/plans/004-list-command/003-test-and-document.md): Cover catalog tree membership handling and CLI output with automated tests and update docs or help text to describe the new command.

## Steps

### [Share catalog parsing via new crate](.agents/plans/004-list-command/001-share-catalog-parsing.md)

Build a shared catalog module inside the new `nmcr_catalog` crate that keeps tree file templates scoped to their parent trees, exposes ID lookup helpers that walk tree members when needed, and remains usable from both the CLI and MCP call sites.

### [Add CLI command to render catalog](.agents/plans/004-list-command/002-add-cli-list-command.md)

Implement an `nmcr list` command that loads the catalog, prints trees with their files indented alongside ids, paths, descriptions, and args, and ensures each tree file shows its full ID without appearing at the top level.

### [Test and document the listing flow](.agents/plans/004-list-command/003-test-and-document.md)

Ensure tree membership handling and CLI rendering are covered by tests and reflect the new command in user-facing documentation or help output.

## Questions

None.

## Notes

- Ensure tree member files carry their tree-scoped IDs without duplicating them at the root level, keeping duplicate ID detection straightforward.
- Tree templates may aggregate args from member files; plan to show whichever metadata the shared catalog exposes without inventing new fields.
- Follow existing CLI output styling conventions (e.g., indentation, bullet markers) when formatting the list.
- Provide shared helper functions so CLI and MCP code can look up template IDs by checking tree members when the ID prefix matches the tree ID.

## Prompt

I want you to plan `list` command feature. It should load project catalog and then print file and tree templates with their data: id, path, description and args (tree templates will not have all the data). Render tree templates with their file templates indented. When parsing templates catalog, add flag `nested` and when rendering the list filter out these templates as they will be rendered as a part of the tree templates.

### Follow-Ups

Make sure the tree's file templates have the correct id assigned, but ditch adding templates to the root level. This will make special case for id checking unncessary.

Change template id lookup (both in gen and mcp, so use shared functionality; create crate `nmcr_catalog` at `pkgs/catalog`) to check inside tree's files (since tree's file ids starts with tree's id, it is possible to skip iterating unless specified id starts with the tree id).

We won't need `nested` flag then.
