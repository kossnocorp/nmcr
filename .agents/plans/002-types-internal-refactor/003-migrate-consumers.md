# Migrate consumers to `nmcr_types`

## Objective

Update every crate that previously relied on `nmcr_types_internal` to import the generated `nmcr_types` models and construct `ArgKind` values using the literal-backed API.

## Tasks

- [x] **Locate import sites** — run ripgrep for `nmcr_types_internal` across the workspace to list all modules, tests, and binaries that consume the old crate.
      Group the call sites by crate (`pkgs/mcp`, `pkgs/md-parser`, etc.) to plan edits efficiently.
- [x] **Redirect base model imports** — replace usages of `Template`, `TemplateArg`, `Location`, `Span`, and similar data models to come from `nmcr_types`, while leaving helpers that remain in `nmcr_types_internal` (for example, `FormattedLocation`) untouched.
      Update module paths and re-export statements where those models were reintroduced for convenience.
- [x] **Update argument construction** — refactor every spot that builds `TemplateArg` values to instantiate the new literal enums (`ArgKind::String(ArgKindString)` and peers) rather than raw strings or the old enum.
      Adjust serde expectations and confirm default behaviors remain correct.
- [x] **Align tests and fixtures** — rewrite test data and fixtures to match the new `ArgKind` representation, regenerating snapshots or JSON payloads if their shape changes.
      Re-run the affected test suites (`cargo test -p <crate>`) to verify behavior stays intact.
- [x] **Run workspace checks** — execute `cargo check --workspace` (or the subset impacted) to catch cross-crate fallout once migrations compile locally.
      Log any residual issues that need Genotype updates or further refactoring.

## Questions

None.

## Notes

- Coordinate the migration so all crates build in the same commit, avoiding intermediate states where generated and manual types mix.
- Current importers include `pkgs/cli`, `pkgs/mcp`, and `pkgs/md-parser` crates along with their `prelude` modules and manifest dependencies.
- Snapshot fixtures updated for the md-parser tests; `cargo test -p nmcr_md_parser`, `cargo test -p nmcr_mcp`, and `cargo test -p nmcr` all pass.
