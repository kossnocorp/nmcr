# Handlebars Template Support

## Brief

Plan the integration of the `handlebars` crate so templates can interpolate arguments in both content and generated paths, while automatically merging detected variables with existing metadata and updating CLI/MCP flows to accept inline argument pairs.

## Plan

- [ ] [Embed Handlebars Rendering](.agents/plans/005-handlebars-support/001-embed-handlebars-rendering.md): Add the `handlebars` dependency, centralize engine setup, and expose rendering utilities that work for template content and file paths.
- [ ] [Unify Variable Extraction](.agents/plans/005-handlebars-support/002-unify-variable-extraction.md): Parse templates to detect Handlebars placeholders from Markdown content and path metadata, merging them with manually documented variables without dropping descriptions.
- [ ] [Revise Generation Interfaces](.agents/plans/005-handlebars-support/003-revise-generation-interfaces.md): Update CLI `gen` command argument parsing, template invocation, and MCP tool schemas/output to leverage the new rendering and argument contract.

## Steps

### [Embed Handlebars Rendering](.agents/plans/005-handlebars-support/001-embed-handlebars-rendering.md)

- Introduce the `handlebars` crate to the workspace and wire a reusable renderer that can compile templates with strict missing-variable detection.
- Encapsulate helpers/settings so both CLI and MCP paths share identical behavior, including escaping rules and optional helper registration.
- Ensure template compilation errors surface actionable diagnostics that mention the source template ID and snippet that failed.

### [Unify Variable Extraction](.agents/plans/005-handlebars-support/002-unify-variable-extraction.md)

- Extend the Markdown/template parser to scan both prose content and path strings for `{{variable}}` patterns, normalizing dotted names and trimming whitespace.
- Merge the discovered variables with the template's declared variables list, retaining existing descriptions and adding placeholders for any new entries.
- Introduce validation that flags conflicts (e.g., manual variable marked optional vs. required by usage) so documentation stays accurate.

### [Revise Generation Interfaces](.agents/plans/005-handlebars-support/003-revise-generation-interfaces.md)

- Rework the CLI `gen` command to accept positional `key=value` argument pairs, map them into a context map, and feed them through the Handlebars renderer for both file paths and content.
- Adjust MCP tool schemas to advertise the new argument structure, ensure generated output paths/content are interpolated, and keep Structured Content responses aligned with tree outputs.
- Update tests and examples covering CLI/MCP invocations to demonstrate the simplified argument syntax and successful rendering of Handlebars variables.

## Questions

### Missing Argument Behavior

Should generation fail immediately when a referenced Handlebars variable lacks a provided value, or should we allow optional placeholders with defaults?

#### Answer

Generation must fail fast with a clear "missing arguments" error when any Handlebars placeholder lacks a provided value, rather than treating it as optional or applying implicit defaults.

## Notes

- Audit existing template metadata loaders to ensure variable descriptions defined in Markdown stay authoritative when merging detected variables.
- Confirm tree templates still render correctly when their file paths include Handlebars expressions and that directory creation respects interpolated segments.

## Prompt

I want you to plan a handlebars support feature. Use 005 index, as we're working on something else in parallel.

- Use `handlebars` crate.
- When parsing a template, extract handlebars variables from content and path.
- Use these variables and merge them into the variables list. Make sure to preserve the variable descriptions defined manually in Markdown.
- When generating template output, interpolate passed arguments into the content and path.
- `gen` command should accept args as pairs (define as `Vec<String>` and then parse) in the following format `nmcr gen template_id arg1=value arg2=value`. Currently, args are handled differently (`--arg arg1=value`) but it's much more cumbersome to use, so rework it.
- Make sure `mcp` defines tool arguments properly.
