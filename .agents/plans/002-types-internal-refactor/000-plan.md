# Types Internal Migration

## Brief

Refactor the project to rely on the generated `nmcr_types` crate for shared template data models, retire duplicate structs from `pkgs/types-internal`, and document the Genotype schema language for future cross-language tooling.

## Plan

- [x] [Validate generated type coverage](.agents/plans/002-types-internal-refactor/001-validate-generated-types.md): Compare existing internal structs with the Genotype output, note gaps, and confirm how `ArgKind` maps onto current usage.
- [x] [Restructure `types-internal` crate](.agents/plans/002-types-internal-refactor/002-restructure-types-internal.md): Remove manual base structs, add dependencies on `nmcr_types`, and keep only internal helpers like `FormattedLocation`.
- [x] [Migrate consumers to `nmcr_types`](.agents/plans/002-types-internal-refactor/003-migrate-consumers.md): Update all crates to import the generated types, adjust `ArgKind` handling, and align serialization plus tests.
- [x] [Document Genotype workflow](.agents/plans/002-types-internal-refactor/004-document-genotype.md): Capture Genotype syntax, Rust translation examples, and cross-language motivation in `docs/stack/genotype.md`.

## Steps

### [Validate generated type coverage](.agents/plans/002-types-internal-refactor/001-validate-generated-types.md)

- Inventory the structs and enums currently in `pkgs/types-internal` and map each to the corresponding Genotype-generated Rust types.
- Identify any behavioral mismatches (serde attributes, derives, default handling) that must be addressed before consumers switch crates.
- Confirm how the new `ArgKind` literals should be instantiated in place of the old string-backed `TemplateArgType` enum (`ArgKind::Number(ArgKindNumber)`, etc.).

### [Restructure `types-internal` crate](.agents/plans/002-types-internal-refactor/002-restructure-types-internal.md)

- Remove local definitions for `Template`, `TemplateArg`, `Location`, `Span`, and related enums in favor of imports from `nmcr_types`.
- Decide whether to re-export the generated types or leave `types-internal` focused on supplementary helpers, updating the public API accordingly.
- Update Cargo metadata and ensure the crate compiles against the new dependency graph.

### [Migrate consumers to `nmcr_types`](.agents/plans/002-types-internal-refactor/003-migrate-consumers.md)

- Replace `nmcr_types_internal` imports across the codebase with `nmcr_types`, handling any namespace renames or module paths.
- Update construction sites to use the literal-backed `ArgKind` variants provided by `litty` instead of raw strings or legacy enums.
- Adjust tests, serializers, and tool integrations so they compile and pass using the new type definitions.

### [Document Genotype workflow](.agents/plans/002-types-internal-refactor/004-document-genotype.md)

- Author `docs/stack/genotype.md` summarizing the Genotype language, focusing on how definitions translate into idiomatic Rust types.
- Highlight the single-source-of-truth motivation and explain how generated packages integrate with the CLI and MCP server.
- Include concise examples drawn from the current type suite to guide future contributors.

## Questions

### Fate of `nmcr_types_internal`

Should the `nmcr_types_internal` crate remain as a thin façade (re-exporting and housing helpers) or should we plan a follow-up removal once consumers migrate?

#### Answer

Keep the crate as a façade so we can share internal-only types and helper newtypes (for example, `FormattedLocation`) without exposing them externally.

## Notes

- Generated Rust types live under `pkgs/types-rs` with crate name `nmcr_types`; they introduce `ArgKind` variants backed by the `litty` crate (`ArgKind::Number(ArgKindNumber)` style usage).
- Existing helpers like `FormattedLocation` remain unique to `nmcr_types_internal`, so the plan must account for keeping or relocating them.

## Prompt

Generate plan for types-internal refactoring:

- I used Genotype to generate `pkgs/types-rs` (along with other languages) from `pkgs/types-src`. They largely replicate base types currently defined in [lib.rs](pkgs/types-internal/src/lib.rs). There might be mistakes (so make sure it address them).
- Remove manual base type definitions from `pkgs/types-internal` that already generated at `pkgs/types-rs`.
- Instead of consuming these base types from `pkgs/types-internal`, everywhere in the source code, use `pkgs/types-rs`.
- Consider that `ArgKind` is now different and defined using `litty` crate (provided by Genotype). This is intentional. So instead of assigning a `String`, use the `ArgKind` enum with correct literal structs (e.g. `ArgKind::Number(ArgKindNumber`).

Additionally, use this guide from Genotype repo (https://github.com/kossnocorp/genotype/blob/main/cli/examples/guide/guide.type):

```type
//! Welcome to the Genotype language guide!
//!
//! Comments at the beginning of a file prefixed with `//!` are treated as
//! the module documentation and translated to the corresponding documentation
//! comments in the target language.

/// Comments before types and fields prefixed with `///` are treated as
/// documentation comments for the type or field.
User = {
  /// Full user name field
  name: /* Inline comments... */ string,
  // ...just like `//` comments are allowed too, but won't be included in
  // the generated source code.
}

// Btw, that was a simple object type alias `User` with a single field `name`.
// It will translate to a corresponding type in the target language. For
// instance. In Python, that will define `class Name`. In TypeScript, it will be
// `interface Name`l In Rust, `struct Name`, etc.

// You can define type aliases for all kinds of types, for example, primitives
// such as string, int, float, boolean, and null:
Nothing = boolean

Account = {
  /// You can define nested object types:
  name: {
    first: string,
    /// Fields can be optional:
    last?: string,
  }

  /// You can also assign the name inline, which will create a separate type:
  address: Address = {
    street: string,
    city: string,
    zip: string,
  },
}

Order = {
  /// You can reference other types, including the ones defined inline:
  address: Address,
  /// ...or the ones defined later:
  cart: Card,
}

Card = {
  /// Btw, this is how you define arrays:
  items: [Item],

  /// ...tuples:
  fees: (float, float, float),

  /// ...and maps:
  discounts: { [string]: float },
  /// Unless the key is an int, float, or bool, you can skip the key type to make
  /// it a string:
  prices: { []: float },
}

/// You can reuse types...
ItemBase = {
  title: string,
  price: float,
}

ItemBook = {
  // ...by extending other types:
  ...ItemBase,
  isbn: string,
}

ItemGame = {
  ...ItemBase,
  platform: string,
}

/// You can union types:
Item = ItemBook | ItemGame

Payload = {
  /// You can use literal types to define constants:
  version: 1,
  /// And branded types to define unique identifiers:
  id: PayloadId = @string
}

/// There's also a special type any that will translate into any JSON value:
JSON = any

// Genotype features module system, so you can import types from other files:
// use ./module/FullName

Contact = {
  /// And use them in your types:
  // name: FullName,
  /// You can also import types inline:
  email: ./module/Email,
}

// You can import multiple types, as well as rename them:
use ./module/{FullName, Email as EmailAddress}

// Or import all types from a module implicitly:
use ./module/*

// ...in the target language it will be translated to corresponding imports,
// e.g. `use super::module` and `module::Zip` in Rust:
ZipAlias = Zip

// Finally, it's worth mentioning that Genotype supports annotating types and
// fields with attributes, which can be used to add metadata.

#[discriminator = "type"]
Animal = Dog | Cat

Dog = {
  type: "dog",
}

Cat = {
  type: "cat",
}

// For now, only the `discriminator` attribute is supported, which will assign
// discriminator to the field in Python.

// That is is. Thank you for reading the guide! For more examples, see
// the `examples` directory: https://github.com/kossnocorp/genotype
```
