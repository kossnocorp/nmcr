# Genotype Schema Workflow

## Why Genotype

Genotype lets us describe the nmcr data model once and fan it out into idiomatic packages for Rust, TypeScript, and Python. The schema in `pkgs/types-src` acts as the single source of truth, so the CLI, the MCP server, and any external tooling consume compatible structs and serialization rules without divergence. Keeping the schema declarative also makes it easy to add new bindings later while enforcing the same shape across languages.

## Source Layout

All Genotype inputs live in `pkgs/types-src/src`. Each `.type` file defines a module; `genotype.toml` wires those modules to the Rust, TypeScript, and Python generators. The Rust artefact lands in `pkgs/types-rs` under the crate name `nmcr_types`, which we now treat as the canonical model crate. When you need helpers that are internal-only (for example, `FormattedLocation`), place them in `pkgs/types-internal` and keep them thin wrappers around the generated types.

## Language Essentials

### Module and Type Docs

Use leading `//!` comments for module-level documentation and `///` for types or fields. Genotype copies them straight into the generated code:

```type
//! Template schema shared across nmcr packages.

/// Argument metadata collected from markdown.
Arg = {
  /// Raw identifier of the argument.
  name: string,
  /// Human-readable help text.
  description: string,
  kind: ArgKind,
  /// Whether the template requires this argument at render time.
  required: bool,
}
```

becomes

```rust
#[doc = "Template schema shared across nmcr packages."]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Arg {
    /// Raw identifier of the argument.
    pub name: String,
    /// Human-readable help text.
    pub description: String,
    pub kind: ArgKind,
    /// Whether the template requires this argument at render time.
    pub required: bool,
}
```

### Objects and Optional Fields

Genotype object literals map to Rust structs. Optional fields use the `?` suffix and turn into `Option<T>` with Serde defaults. The `Template` definition below mirrors what the CLI consumes today:

```type
Template = {
  id: string,
  name: string,
  description: string,
  args: [./arg/Arg],
  lang?: string,
  content: string,
  location: ./location/Location,
}
```

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub description: String,
    pub args: Vec<Arg>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    pub content: String,
    pub location: Location,
}
```

### Inline Types, Imports, and Spreads

Import other modules with `use` and compose objects with spreads. Inline assignments (`address: Address = { ... }`) create a named type and use it immediately. Spreads (`...Base`) work the same way as Rust struct update syntax but stay purely declarative in the schema.

### Literal Unions and `ArgKind`

Unioning string literals creates tagged enums in Rust. In our schema we model argument kinds as literal strings:

```type
ArgKind = "any" | "boolean" | "string" | "number"
```

Genotype generates an `ArgKind` enum plus unit structs such as `ArgKindString` backed by the [`litty`](https://crates.io/crates/litty) crate, so constructing a string argument looks like `ArgKind::String(ArgKindString)`. The literal structs ensure the JSON representation stays as the raw string while keeping enum exhaustiveness on the Rust side.

## Rust Translation Cheatsheet

- `string`, `int`, `float`, and `boolean` map to `String`, `i64`, `f64`, and `bool`.
- Arrays (`[T]`) become `Vec<T>`; maps (`{ []: T }`) translate to `BTreeMap<String, T>`.
- Optional fields (`field?:`) emit `Option<T>` with `#[serde(default, skip_serializing_if = "Option::is_none")]`.
- Literal unions (`"foo" | "bar"`) produce enums enriched with `litty` literal wrappers.
- Inline type definitions create dedicated Rust structs or enums, so reuse them to avoid duplication across modules.

## Generation Workflow

1. Edit the schema under `pkgs/types-src/src`.
2. Regenerate the targets:

   ```bash
   gt build pkgs/types-src
   ```

   The `gt` binary ships with the Genotype CLI (`cargo install genotype_cli` if it is missing).

3. Commit the regenerated packages (`pkgs/types-rs`, and the TypeScript/Python outputs when relevant`).
4. Update dependents to import from `nmcr_types` instead of maintaining ad hoc copies.

## Integration Notes

- `nmcr_types` is the only crate that should define base models. Downstream crates import it directly; `nmcr_types_internal` remains for helper newtypes such as `FormattedLocation`.
- The MCP server and the CLI both rely on the generated structs, so schema changes must be regenerated before they compile.
- Keep tests in sync with literal enums (for example, use `ArgKind::Boolean(ArgKindBoolean)` instead of bare strings) to verify that the serialized payloads stay stable across languages.
