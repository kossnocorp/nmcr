# Tree Template Generation

## Brief

Plan the remaining work to support tree-capable templates end to end: finalize the shared data model, parse the enhanced Markdown syntax, emit structured multi-file output through MCP/CLI, and document/test the new flows.

## Plan

- [x] [Align Data Model](.agents/plans/003-tree-templates/001-align-data-model.md): Extend the Genotype schemas and generated Rust types to represent single-file and tree templates plus their outputs while staying consistent with prior refactors.
- [x] [Parse Tree Markdown](.agents/plans/003-tree-templates/002-parse-tree-markdown.md): Update the Markdown parser to extract paths, descriptions, and file nodes, populating the new tree structures without regressing collection support.
- [ ] [Implement Generation](.agents/plans/003-tree-templates/003-implement-generation.md): Build real MCP/CLI execution paths—replacing the current `gen` playground—with logic that renders files or trees, honors path requirements, and supports the new `--print` flag semantics. (CLI completed; MCP Structured Content pending.)
- [ ] [Tests and Docs](.agents/plans/003-tree-templates/004-tests-docs.md): Cover the new behavior with automated tests and refresh documentation/examples so contributors can author and consume tree templates confidently. (Parser tests added; CLI/MCP tests and docs pending.)

## Steps

- Reflect the updated Mermaid diagram (`Template` enum with `File`/`Tree` variants) inside `pkgs/types-src` by modelling `TemplateTree` (with description and file list) and ensuring `TemplateFile` carries optional path, language, args, and `Location` metadata.
- Add a Genotype-defined `TemplateOutput` discriminated union with `File` and `Tree` variants so MCP responses and CLI dry runs can share the same representation.
  (Implemented as `TemplateOutputFile` and `TemplateOutputTree` due to current Genotype union constraints.)
- Regenerate `nmcr_types`, update downstream crates (prepped by plans 001 and 002) to compile with the new enum variants, and capture any serialization versioning notes needed for structured content consumers.

### [Parse Tree Markdown](.agents/plans/003-tree-templates/002-parse-tree-markdown.md)

- Enhance the parser to capture inline "`path`:" indicators before code blocks, falling back to header-derived names, and persist the path on the associated `TemplateFile`.
- Detect tree templates by grouping sections whose files all produce paths; attach any prose between the tree heading and first file as the tree description, and keep generating child `TemplateFile` templates (with their own args/content/lang data).
- Ensure collections like `rust-crate.md` still materialize both collection-level templates and their nested tree files, adding validation for missing paths or malformed hierarchies.

### [Implement Generation](.agents/plans/003-tree-templates/003-implement-generation.md)

- Update MCP tools to return Structured Content JSON conforming to the new `TemplateOutput` schema, including generation of per-tool JSON Schema that reflects optional `path` fields.
- Replace the CLI `gen` playground with a full command: accept either a destination path or `--print`; enforce errors when paths are required but missing, and when tree outputs contain files without paths; surface concise success/error messaging instead of debug dumps.
- Implement filesystem writes for tree templates by creating directories as needed, rendering each file relative to the supplied root, and preserving existing single-file behavior.

### [Tests and Docs](.agents/plans/003-tree-templates/004-tests-docs.md)

- Add parser unit tests covering inline path extraction, tree detection, and failure cases for missing paths.
- Introduce CLI and MCP integration tests verifying Structured Content payloads, schema generation, and filesystem output rules.
- Update documentation and examples (`npm.md`, `rust-crate.md`, Template aspect docs) to demonstrate both single-file and tree workflows, noting the new CLI flag semantics.

## Questions

### Multi-file Output Contract

What format should generation interfaces (CLI output, MCP tool response) use to deliver multiple files—direct filesystem writes, structured JSON payloads, archives, or another convention?

#### Answer

MCP must return Structured Content JSON backed by a new Genotype `TemplateOutput` type with `File` and `Tree` variants, and the CLI should either print this output or write files to a user-specified path (erroring when required paths are missing).

## Notes

- Incorporate recent refactors from `.agents/plans/001-template-structure-refactor` and `.agents/plans/002-types-internal-refactor` so schema updates remain Genotype-first and consumers already depending on `nmcr_types` continue to compile.
- Existing examples (`examples/basic/tmpls/npm.md`, `examples/basic/tmpls/rust-crate.md`) provide coverage for both tree and collection cases—keep them in sync with the new parsing rules.

## Prompt

I want you to plan tree template generation feature.

Look at `npm.md` it is a tree template where each `##` represent file names names to use when generating the template. The content of the section is a regular template like we have right now defined at files like `react.md`. Using this template will generate multiple files in a specified directory.

Another example is `rust-crate.md`. It is a template collection where `##` header representing a template that has multiple `###` headers that represent file names.

Extend and use the current data structures using the mermaid scheme from `template.md`.

Heads up: I moved the plans to `.agents/plans/003-tree-templates`.

Also, look at the work we did since the last message and revisit the generated plan accordingly:

- `.agents/plans/001-template-structure-refactor`
- `.agents/plans/002-types-internal-refactor`

I want you to update how MCP and gen commands work.

- MCP should respond with a JSON. Define a new type in `pkgs/types-src/src` using Genotype to represent a template output. There should be two different types: File and Tree. Make sure to utilize Structured Content feature of MCP. Generate content-aware JSON Schema for each tool, if a File has no path, then it should be reflected in the output schema.
- `gen` command should either accept a path or `--print`. If path is present but the target template doesn't have assigned path, the command should exit with error. If path is present and that's a tree template, make sure it's a directory and then generate files with corresponding relative paths and generated content. If any of the tree files have path missing, exit with an error.

Additional things to incorporate:

- When parsing Markdown, and there's inline code block with `:` after it, before the template content block assign it as a path.
- Parse individual templates for each of the tree files.
- Change the way trees are detected. Instead of relying on each of the headers to have path in the header, check if each of the section templates has `path` assigned, add the tree template with all of the templates as the files.
- TemplateTree should have description as well, parsed from the content between the tree header and the first file header.
