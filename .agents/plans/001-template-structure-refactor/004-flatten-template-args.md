# Flatten template args

## Objective

Remove the `TemplateArgs` wrapper so templates expose their arguments directly as `Vec<TemplateArg>` across Rust and downstream bindings.

## Tasks

- [ ] **Trace `TemplateArgs` usage** — inspect Rust crates and generated bindings to list constructors, helpers, and serializers that depend on the wrapper type.
      Note any builder patterns or macros that will need updates.
- [ ] **Refactor data structures** — replace the `TemplateArgs` struct with a plain vector field on `Template`, updating Serde annotations and default implementations accordingly.
      Provide convenience helpers if common operations (such as `is_empty`) are still needed.
- [ ] **Rewrite call sites** — adjust code that previously instantiated or mutated `TemplateArgs` so it now works with `Vec<TemplateArg>`.
      Double-check CLI commands, template parsers, and test fixtures for compilation warnings.
- [ ] **Sync generated bindings** — regenerate TypeScript, Python, and schema outputs to reflect the array-based representation.
      Ensure optionality and defaults align with previous behavior.
- [ ] **Update tests and docs** — modify any unit tests, snapshots, and documentation that reference `TemplateArgs` symbols.
      Add new tests exercising the vector-based API if coverage gaps appear.

## Questions

None.

## Notes

Keep method naming consistent by relocating helper functions onto utility modules if needed.
