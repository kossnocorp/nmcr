# Template Structures Refactor

## Brief

Refactor template data models to include persistent IDs, enforce uniqueness, simplify arguments, and make locations Genotype-friendly.

## Plan

- [ ] [Introduce template IDs](.agents/plans/001-template-structure-refactor/001-introduce-template-id.md): Add plain `String` IDs to `Template` and wire them up through a shared generator.
- [ ] [Enforce template ID uniqueness](.agents/plans/001-template-structure-refactor/002-enforce-template-id-uniqueness.md): Replace the current conflict resolution by rejecting duplicate IDs during collection and generator flows using [`anyhow::Context`](https://docs.rs/anyhow/latest/anyhow/struct.Context.html) for messaging.
- [ ] [Adopt relative location paths](.agents/plans/001-template-structure-refactor/003-stringify-location-path.md): Switch `Location.path` to [`RelativePathBuf`](https://docs.rs/relative-path/latest/relative_path/struct.RelativePathBuf.html) and normalize how paths are stored and serialized.
- [ ] [Flatten template args](.agents/plans/001-template-structure-refactor/004-flatten-template-args.md): Remove the `TemplateArgs` wrapper and adapt APIs, serialization, and tests to use `Vec<TemplateArg>`.
- [ ] [Document ID generation](.agents/plans/001-template-structure-refactor/005-document-id-generation.md): Capture the ID format and examples in project architecture docs and keep Genotype considerations in view ([link](https://github.com/kossnocorp/genotype)).

## Steps

### [Introduce template IDs](.agents/plans/001-template-structure-refactor/001-introduce-template-id.md)

- Add an `id: String` to `Template` with Serde support and ensure default generation for new templates.
- Create a new `nmcr_id` package that exposes an ID generator struct with helpers to derive IDs from template hierarchy data.
- Update template creation code to rely on the shared generator without introducing type aliases or newtypes.

### [Enforce template ID uniqueness](.agents/plans/001-template-structure-refactor/002-enforce-template-id-uniqueness.md)

- Locate the logic that currently mutates conflicting template names and replace it with validation that emits a failure when duplicates occur.
- Propagate the failure up the call stack so `mcp` and `gen` commands exit early with a clear message.
- Add tests covering duplicate detection for both runtime loading and CLI generation flows.

### [Adopt relative location paths](.agents/plans/001-template-structure-refactor/003-stringify-location-path.md)

- Change `Location.path` to use `RelativePathBuf` from the `relative_path` crate and update related types.
- Ensure all call sites now store normalized relative strings and adjust helper utilities accordingly.
- Verify downstream language bindings and schema definitions compile with the new representation.

### [Flatten template args](.agents/plans/001-template-structure-refactor/004-flatten-template-args.md)

- Remove the `TemplateArgs` wrapper, define `Template.args: Vec<TemplateArg>`, and update constructors and builders.
- Adjust Serde defaults, CLI interfaces, and any helper methods relying on `TemplateArgs`.
- Update tests, documentation, and generated bindings to reflect the simplified structure.

### [Document ID generation](.agents/plans/001-template-structure-refactor/005-document-id-generation.md)

- Outline the ID format (lowercase `snake_case` segments derived from hierarchy headers) in `docs/architecture/id.md`.
- Provide concise examples showing heading-to-ID conversion for simple hierarchies.
- Reference the new `nmcr_id` package as the canonical implementation entry point.

## Questions

### Template ID format

What format should the new template `id` use (for example, the same slug-style format currently produced for collections), and where should the ID generation live?

#### Answer

Lowercase `snake_case` built from each hierarchy level's header (for example, `Rust Crate` → `rust_crate`). Implement the logic inside the new `nmcr_id` package and call it from template tooling.

## Notes

No backward compatibility work is required because the project is greenfield.

## Prompt

I want you to plan template structures refactoring:

1. We recently added collection `Option<String>` to produce name for template tool when needed. Instead of that, add pregenerated id to `Template` in the same format.

2. When there's a id conflict inside a project, I want you instead of using a counter or any other measure to generate unique name simply return an error. For now simply exiting mcp/gen commands will do.

3. Inside Location, we keep `path` as `PathBuf`. I want you to use `String` instead. The reason is that in the future I want to utilize meta-programming language Genotype (https://github.com/kossnocorp/genotype) to generate types for TS/Rust/Python and right now it doesn't support types like `PathBuf`.

4. Get rid of `TemplateArgs`, make `Template`'s `args` a simple `Vec<TemplateArg>`.
