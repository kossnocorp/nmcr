# Introduce template IDs

## Objective

Add plain `String` identifiers to templates and centralize their generation so every tool and consumer works with consistent, pregenerated IDs.

## Tasks

- [x] **Map template creation pipeline** — review `pkgs/types-internal`, `pkgs/mcp`, and any generator code to list every spot that constructs or deserializes `Template` values.
      Ensure we know where IDs must be injected or read before editing shared types.
- [x] **Extend `Template` data model** — add an `id: String` with Serde defaults in `pkgs/types-internal/src/lib.rs` and adjust constructors or builders so callers can supply IDs.
      Update default impls and tests to keep compiling without newtypes or aliases.
- [x] **Bootstrap `nmcr_id` crate** — scaffold a new package (Cargo manifest, `src/lib.rs`, module docs) that exposes a `TemplateIdGenerator` struct with helpers to derive lowercase snake_case IDs from hierarchy metadata.
      Include unit tests covering multi-level headers like `Rust Crate` → `rust_crate` and nested segments.
- [x] **Integrate generator into template tooling** — update the flows identified earlier so they call into `nmcr_id` when instantiating templates.
      Remove any ad hoc name derivation logic left in `mcp` or related crates.
- [x] **Verify primitive-friendly structs** — confirm the updated data models remain composed of plain Rust standard types so future cross-language bindings can adopt them without additional wrappers.
      Leave documentation notes for any follow-up tooling work once bindings are introduced.

## Questions

None.

## Notes

Skip backward compatibility shims because the project is greenfield.
