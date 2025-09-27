# Document Genotype workflow

## Objective

Produce a concise Genotype guide in `docs/stack/genotype.md` tailored for Rust developers so future contributors understand how schema definitions become the generated `nmcr_types` crate and its cross-language counterparts.

## Tasks

- [x] **Review Genotype reference material** — re-read the provided guide ([`cli/examples/guide/guide.type`](https://github.com/kossnocorp/genotype/blob/5e6af10/cli/examples/guide/guide.type)) and inspect `pkgs/types-src` to extract concrete patterns we already use.
      Note language features (imports, spreads, literals, discriminators) relevant to the nmcr type suite.
- [x] **Outline Rust-focused narrative** — draft the document structure covering motivation, basic syntax, complex constructs (unions, inline definitions), and how they translate to Rust structs/enums.
      Include short code snippets showing Genotype input alongside the generated Rust output.
- [x] **Author documentation** — write `docs/stack/genotype.md` with the finalized outline, referencing real nmcr types where possible and keeping prose concise for experienced developers.
      Include guidance on running Genotype (e.g., `gt build pkgs/types-src`) and how generated crates integrate with the MCP server and CLI.
- [x] **Format and lint** — run `prettier docs/stack/genotype.md --write` and skim the result to ensure markdown rendering is clean and links resolve.
      Capture any follow-up sections that might be useful later (for example, language-specific quirks) in the step notes.

## Questions

None.

## Notes

Keep the guide actionable for contributors extending the schema: focus on translating Genotype constructs to Rust typenames, serde behavior, and cross-language guarantees.
- Key constructs to highlight: module docs via `//!`, field docs with `///`, optional fields (`field?:`), literal unions (e.g., `ArgKind`), spreads (`...Type`), inline type declarations, and command `gt build pkgs/types-src` for generation.
- Prettier runs via `mise exec node -- npx prettier docs/stack/genotype.md --write` because Node binaries are managed by `mise`.
