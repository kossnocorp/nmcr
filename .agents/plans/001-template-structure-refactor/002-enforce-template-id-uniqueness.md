# Enforce template ID uniqueness

## Objective

Reject duplicate template IDs during loading and generation so CLI commands fail fast instead of auto-adjusting conflicting names.

## Tasks

- [ ] **Locate existing conflict handling** — search the CLI and generator crates (notably `pkgs/mcp` and `tools/gen`) for logic that appends counters or otherwise deduplicates template names.
      Document the entry points that currently mutate identifiers.
- [ ] **Leverage `anyhow` for error messaging** — plan to surface duplicate ID details using [`anyhow`](https://docs.rs/anyhow/latest/anyhow/) errors enriched with [`Context`](https://docs.rs/anyhow/latest/anyhow/trait.Context.html), capturing both conflicting IDs and their locations without defining new enums.
      Draft the human-readable strings that will be attached via `with_context`.
- [ ] **Implement validation guard** — insert checks in template aggregation code that collect seen IDs and bail out with the new error when a duplicate appears.
      Make sure the guard short-circuits before any downstream processing that depends on unique IDs.
- [ ] **Propagate CLI failures** — update command handlers so they exit with non-zero status when duplicate IDs are detected, and ensure any progress output is rolled back or clarified.
      Add integration-level assertions if the CLI test harness supports it.
- [ ] **Add regression tests** — create unit or integration tests that load templates with clashing IDs and verify the tooling returns the expected error without renaming the templates.
      Cover both project-level collection and single-command generation scenarios.

## Questions

None.

## Notes

Erroring out replaces all previous auto-renaming strategies.
