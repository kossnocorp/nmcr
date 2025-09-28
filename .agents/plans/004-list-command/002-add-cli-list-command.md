# Add CLI command to render catalog

## Objective

Introduce an `nmcr list` subcommand that loads the shared catalog, prints tree templates with their member files indented beneath them, and surfaces metadata (ID, path, description, args) without showing tree members twice.

## Tasks

- [ ] **Define command interface** — extend the CLI argument parser to register a `list` subcommand with help text describing the catalog output.
      Confirm the command can coexist with existing options and follows the project's CLI conventions.
- [ ] **Load catalog via `nmcr_catalog`** — invoke the shared crate to build the catalog when the command runs, reusing its error types for missing or invalid template data.
      Handle initialization failures gracefully so the command exits with a helpful message.
- [ ] **Format tree-first output** — iterate over tree templates, printing each tree's ID, path, description, and args summary followed by its file templates indented underneath.
      Ensure standalone file templates appear once at the root level and tree members only appear inside their tree block.
- [ ] **Render metadata consistently** — decide on field ordering, indentation, and styling (e.g., bullet markers) that mirror other CLI listings.
      Include arguments only when present and fall back to placeholders where tree metadata is incomplete.
- [ ] **Wire into CLI entry point** — connect the new subcommand to the main command dispatch flow and ensure exit codes reflect success or failure of catalog rendering.
      Add logging or verbose output hooks if the project expects them for debugging.

## Questions

None.

## Notes

Reuse the shared ID lookup helpers when cross-referencing IDs during rendering to avoid duplicating traversal logic.
