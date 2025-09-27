# Align Data Model

## Objective

Expand the Genotype schemas and generated Rust types so templates and their outputs model both single-file and tree variants while staying compatible with the recent type refactors.

## Tasks

- [ ] **Audit existing schemas** — review `pkgs/types-src` definitions and the generated `nmcr_types` output to understand the current `Template`, `TemplateFile`, and related structs before introducing new variants.
      Map any naming differences introduced by previous refactors (for example, `Arg` vs. `TemplateArg`).
- [ ] **Design `TemplateTree` and enum variants** — update the Genotype schema to define `TemplateTree` (including `description` and `files: [TemplateFile]`) and refactor `Template` into a discriminated union with `File` and `Tree` variants that carry shared metadata.
      Ensure `TemplateFile` supports optional `path`, `lang`, `args`, and `Location`.
- [ ] **Introduce `TemplateOutput` union** — add a new Genotype type capturing CLI/MCP output (`TemplateOutput::File`/`TemplateOutput::Tree`) mirroring the runtime structures needed for Structured Content responses.
      Note how optional paths influence the generated JSON Schema so downstream code can surface accurate tool schemas.
- [ ] **Regenerate and propagate types** — run the Genotype generator, update the Rust crate (`pkgs/types-rs`) and any dependent manifests, and adjust imports where the new enum replaces the previous struct.
      Fix compilation issues in crates that construct templates or outputs, keeping changes limited to type alignment.
- [ ] **Document schema rationale** — add notes to the Genotype files or nearby docs explaining the enum layout and output type so future contributors understand why the data model changed.
      Highlight how tree descriptions and file lists map to Markdown constructs.

## Questions

None.

## Notes

- Treat `Template` as the canonical variant-bearing enum used throughout the codebase.
- Preserve `Location` and argument metadata on both file and tree nodes to retain tooling diagnostics.
- Coordinate naming with `.agents/plans/001-template-structure-refactor` so new fields remain compatible with prior ID/location work.
