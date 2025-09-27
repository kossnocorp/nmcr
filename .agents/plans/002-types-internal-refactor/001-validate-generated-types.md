# Validate generated type coverage

## Objective

Confirm every struct, enum, and helper previously defined in `pkgs/types-internal` exists in the Genotype-generated `nmcr_types` crate and note any behavioral gaps before touching the codebase.

## Tasks

- [x] **Inventory legacy types** — read `pkgs/types-internal/src/lib.rs` and document all exported structs, enums, and helper implementations we currently expose.
      Capture derives, serde attributes, and default implementations that consumers rely on.
- [x] **Map to generated equivalents** — inspect `pkgs/types-rs/src/*.rs` to match each legacy type to its `nmcr_types` counterpart, recording any mismatched field names, visibility, or serde settings.
      Highlight missing pieces that will require schema or Genotype adjustments (for example, `Template::content` or optional fields).
- [x] **Evaluate `ArgKind` migration** — prototype the replacement of `TemplateArgType` with the literal-backed enum (`ArgKind::Number(ArgKindNumber)`, etc.) and list the construction sites that will need updates.
      Note any serialization or deserialization differences introduced by the new representation.
- [x] **Summarize findings** — create a short checklist in step notes outlining confirmed matches, outstanding gaps, and any required Genotype schema changes to schedule for Step 2.
      Include links or file paths so follow-up work can reference the exact locations quickly.

## Questions

None.

## Notes

- Keep the summary focused on actionable gaps (missing fields, serde differences, literal enum usage) so later steps know whether extra Genotype changes are required.
- Legacy `TemplateArgType`: derives `Default`, `Debug`, `Clone`, `Serialize`, `Deserialize`, `PartialEq`, `Eq`; default variant `Any`.
- Legacy `TemplateArg`: derives `Debug`, `Clone`, `Serialize`, `Deserialize`, `Default`; fields `name`, `description`, `kind`.
- Legacy `Template`: derives `Debug`, `Clone`, `Serialize`, `Deserialize`, `Default`; includes `id`, `name`, `description`, `args`, `lang`, `content`, `location`.
- Legacy `TemplateCollection`: derives `Debug`, `Clone`, `Serialize`, `Deserialize`, `Default`; contains `name`, `description`, `templates`, `location`.
- Legacy `Location`: derives `Debug`, `Clone`, `Serialize`, `Deserialize`, `Default`; holds `path`, `span`.
- Legacy `Span`: derives `Debug`, `Clone`, `Serialize`, `Deserialize`, `Default`; stores `start`, `end` indices.
- Legacy helper `FormattedLocation<'a>` implements `Display` for pretty-printing `Location` ranges.
- Generated `Arg` corresponds to `TemplateArg` but omits `Default` and renames the type; fields match and include `ArgKind` instead of `TemplateArgType`.
- Generated `Template` lacks the `content: String` field present internally and derives `PartialEq`; no `Default` implementation.
- Generated `TemplateCollection` matches legacy fields and adds `PartialEq` derive; defaults absent.
- Generated `Location` and `Span` mirror legacy shapes and derive `PartialEq`; they also skip `Default` implementations.
- `TemplateArgType` consumers exist in `pkgs/md-parser/src/markdown.rs` (defaulting to `Any`) and `pkgs/mcp/src/template/tool.rs` (mapping to JSON schema, building args via `make_arg`).
- New `ArgKind` values require constructing variants such as `ArgKind::String(ArgKindString)` and `ArgKind::Boolean(ArgKindBoolean)`; serialization stays literal-backed via `litty`.
- Action: Extend `Template` in Genotype schema to include `content: string` so the generated Rust struct matches internal expectations.
- Action: Decide whether to rename `TemplateArg` usages to `Arg` or add a Genotype alias; current plan is to migrate consumers to the generated `Arg` name.
- Action: Verify absence of `Default` derives is acceptable; add helper constructors if downstream code relies on `Default` implementations.
