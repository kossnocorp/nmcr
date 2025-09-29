# Surface Template Catalog via `list`

## Brief

Design the work to introduce an `nmcr list` command that loads the project's template catalog, keeps tree member files associated with their parent trees, and prints both tree and standalone file templates with key metadata in a readable hierarchy.

## Plan

- [x] [Share catalog parsing via new crate](.agents/plans/004-list-command/001-share-catalog-parsing.md): Refactor the template catalog builder into a reusable `nmcr_catalog` crate consumed by both the CLI and MCP crates while keeping tree member files attached to their parent tree definitions.
  - [x] Audit current catalog assembly — CLI `gen` builds separate maps for files/trees and flattens tree files; MCP catalog currently pushes tree tools and standalone files while tracking duplicate IDs via `TemplateCatalogContext`.
  - [x] Scaffold `pkgs/catalog` crate — shared `nmcr_catalog` crate with tree-aware types and workspace registration.
  - [x] Centralize parsing logic — catalog loading now lives in `TemplateCatalog::load`, keeping tree members nested and validating duplicates/nesting.
  - [x] Implement shared ID lookup helpers — exposed `get_file`/`get_tree` and refactored CLI + MCP consumers to use them.
  - [x] Cover crate behavior with tests — exercised nesting and duplicate detection against example templates.
- [x] [Add CLI command to render catalog](.agents/plans/004-list-command/002-add-cli-list-command.md): Define the `list` subcommand, load the catalog via the shared API, and render tree/file templates with indentation and metadata without duplicating tree member files at the root level.
  - [x] Define command interface — Clap enum now includes `list` with its own module.
  - [x] Load catalog via `nmcr_catalog` — command reuses shared loader and surfaces its error context.
  - [x] Format tree-first output — printed trees with indented members and no duplication.
  - [x] Render metadata consistently — outputs ID, source/path, optional description, and args in aligned sections.
  - [x] Wire into CLI entry point — dispatcher routes to `ListCmd::run` and shares standard exit codes.
- [x] [Test and document the listing flow](.agents/plans/004-list-command/003-test-and-document.md): Cover catalog tree membership handling and CLI output with automated tests and update docs or help text to describe the new command.
  - [x] Expand catalog unit tests — added coverage for nested lookups and unmatched prefixes.
  - [x] Add CLI integration coverage — verified `nmcr list` output via writer-based test harness.
  - [x] Update user-facing docs — documented the command in `docs/stack/cli-guide.md`.
  - [x] Confirm developer workflows — smoke-tested MCP tool exposure with shared catalog data.
  - [x] Document follow-up considerations — noted sorting/filtering opportunities in follow-ups.

## Steps

### [Share catalog parsing via new crate](.agents/plans/004-list-command/001-share-catalog-parsing.md)

Build a shared catalog module inside the new `nmcr_catalog` crate that keeps tree file templates scoped to their parent trees, exposes ID lookup helpers that walk tree members when needed, and remains usable from both the CLI and MCP call sites.

**Step Status:** Completed — `nmcr_catalog` centralizes catalog parsing, CLI/MCP now consume shared lookups, and tests cover tree scoping plus duplicate detection.

### [Add CLI command to render catalog](.agents/plans/004-list-command/002-add-cli-list-command.md)

Implement an `nmcr list` command that loads the catalog, prints trees with their files indented alongside ids, paths, descriptions, and args, and ensures each tree file shows its full ID without appearing at the top level.

**Step Status:** Completed — new `nmcr list` subcommand loads the shared catalog and renders tree-first output with consistent metadata formatting.

### [Test and document the listing flow](.agents/plans/004-list-command/003-test-and-document.md)

Ensure tree membership handling and CLI rendering are covered by tests and reflect the new command in user-facing documentation or help output.

**Step Status:** Completed — catalog and CLI tests cover tree membership, docs describe `nmcr list`, and MCP tooling is smoke-tested with the shared loader.

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

Make sure the tree's file templates have the correct id assigned, but ditch adding templates to the root level. This will make special case for id checking unnecessary.

Change template id lookup (both in gen and mcp, so use shared functionality; create crate `nmcr_catalog` at `pkgs/catalog`) to check inside tree's files (since tree's file ids starts with tree's id, it is possible to skip iterating unless specified id starts with the tree id).

We won't need `nested` flag then.

- Consider optional sort/filter flags for `nmcr list` so large catalogs can be scoped by prefix or grouped by source path.
- Evaluate colorized or column-aligned output to improve readability once the command sees frequent use.

### CLI list formatter refresh

Align the `nmcr list` output with the condensed tree-style layout requested after the initial implementation.

#### Plan

- [x] Redesign the list renderer to produce the new tree/file presentation, including language badges and spacing rules. _Renderer now emits tree-style hierarchy with consistent spacing, icons, and language badges derived from paths or fenced code blocks._
- [x] Tweak parsing/formatting to drop inline path markers from descriptions and default `any` args to strings. _Inline path stubs are filtered out of descriptions and argument types fall back to `string`._
- [x] Update docs and tests to reflect the refreshed output. _CLI integration test snapshots the new layout and `docs/stack/cli-guide.md` showcases the format._

#### Prompt

Now we have such an output:

```
Trees
- rust (source: ./examples/basic/tmpls/rust.md)
  Files:
    - rust_package_cargo_toml -> Cargo.toml
      Description: Crate manifest file. Cargo.toml:
    - rust_package_gitignore -> .gitignore
      Description: Rust crate .gitignore file. .gitignore:

- rust_crate_lib (source: ./examples/basic/tmpls/rust-crate.md)
  Description: Rust crate library template.
  Files:
    - rust_crate_lib_cargo_toml -> ./Cargo.toml
    - rust_crate_lib_src_lib_rs -> ./src/lib.rs

- npm_package (source: ./examples/basic/tmpls/npm.md)
  Files:
    - npm_package_package_json -> ./package.json
    - npm_package_index_js -> ./index.js

Files
- react_react_component -> (no path)
  Description: React component file template with optional props.
  Args: props, name
```

I want you to format it differently. Key points:

- Render trees and file templates on the same level.
- Render tree's file and regular file templates using the same format, but different indentation.
- Render trees, similar to `tree` command output
- Less verbose, more space.
- Consistent spacing 2 empty lines between root level templates, 1 empty line between nested templates and template content blocks (title, headers, description, etc.).
- Display the detected language (if any).
- Render argument as a list.

Here's the desired format:

```
📁 rust (./examples/basic/tmpls/rust.md:1)
  ->
  ├── Cargo.toml
  └── .gitignore
  
  Files:

  📄 rust_package_cargo_toml (./examples/basic/tmpls/rust.md:3)
	-> Cargo.toml (toml)
      
    Crate manifest file.
    
    Arguments:
    - name: string
    - description: string

  📄 rust_package_gitignore (./examples/basic/tmpls/rust.md:24)
	-> .gitignore
      
    Rust crate .gitignore file.


📁 rust_crate_lib (./examples/basic/tmpls/rust-crate.md:1)
   ->
   ├── Cargo.toml
   └── src
      └── lib.rs
  
   Rust crate library template.

   📄 rust_crate_lib_cargo_toml (./examples/basic/tmpls/rust-crate.md:7)
     -> ./Cargo.toml (toml)
    
     Arguments:
     - pkg_name: string

   📄 rust_crate_lib_src_lib_rs (./examples/basic/tmpls/rust-crate.md:18)
     -> ./src/lib.rs (rs)

📁 npm_package (./examples/basic/tmpls/npm.md)
   ->
   ├── package.json
   └── index.js

   Files:
  
   📄 npm_package_package_json (./examples/basic/tmpls/npm.md:3)
     -> ./package.json (json)

    Arguments:
    - name [string]
    - description [string]
    - author [string]
      
   📄 npm_package_index_js (./examples/basic/tmpls/npm.md:17)
     -> ./index.js (js)

📄 react_react_component
   -> [string] (tsx)
   
   React component file template with optional props.
 
   Arguments:
   - props [string] Whether to include props interface.
   - name [string] Component name.
```

I also noticed that path assignment "`Cargo.toml`:" is parsed as part of the description, make sure to exclude it when parsing.

And make "string" the default argument type when parsing instead of "any".
