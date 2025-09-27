# Restructure `types-internal` crate

## Objective

Slim the `nmcr_types_internal` crate down to façade duties focused on internal helpers (for example, `FormattedLocation`) while depending on the generated `nmcr_types` models directly.

## Tasks

- [x] **Add dependency on `nmcr_types`** — update `pkgs/types-internal/Cargo.toml` to depend on the generated crate and run `cargo metadata -p nmcr_types_internal` to ensure workspace resolution stays clean.
      Document any feature flags or version alignment required.
- [x] **Patch Genotype schema when required** — for any gaps flagged in Step 1, update `pkgs/types-src` accordingly and regenerate the Rust package with `gt build pkgs/types-src`.
      Re-run the Step 1 checklist to confirm the generated types now match expectations.
- [x] **Remove duplicated data models** — delete the manual definitions for `Template`, `TemplateArg`, `Location`, `Span`, and related enums in `pkgs/types-internal/src/lib.rs` without leaving commented copies or re-exports.
      Keep bespoke helpers such as `FormattedLocation` and adjust visibility so consumers that still rely on them continue compiling.
- [x] **Reintroduce missing conveniences** — if internal consumers expect constructors or defaults provided by the old structs, recreate them as inherent impls or helper functions built on the imported types.
      Ensure these helpers do not leak external dependencies unintentionally.
- [x] **Verify compilation** — run `cargo check -p nmcr_types_internal` to confirm the crate compiles with the new structure and fix any fallout before moving on.
      Record follow-up items (for example, additional derives needed in the generated crate) if compilation fails.

## Questions

None.

## Notes

- External crates should import shared models from `nmcr_types` directly; `nmcr_types_internal` remains only for internals that lack generated counterparts.
- `FormattedLocation` now wraps `nmcr_types::Location`; the crate no longer depends on `serde`.
- No consumer requires the old `Default` impls, so no extra helper constructors were introduced.
