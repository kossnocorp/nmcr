# Normalize location paths

## Objective

Keep location metadata portable by storing paths as normalized strings while using [`relative-path`](https://docs.rs/relative-path/latest/relative_path/) helpers to sanitize inputs where helpful.

## Tasks

- [x] **Audit `Location` usages** — identify every crate that constructs or reads `Location` to understand formatting expectations and serialization paths.
      Pay attention to deserializers, schema exports, and CLI output code.
- [x] **Update struct definitions** — change `Location.path` from `PathBuf` to a normalized `String` in `pkgs/types-internal/src/lib.rs` and propagate that change through any public APIs or re-exports.
      Introduce `relative-path` helpers only where needed for normalization without changing the stored type.
- [x] **Normalize path handling** — ensure call sites convert filesystem paths to normalized, forward-slash relative paths (using `relative-path` utilities internally) before storing them.
      Add lightweight helpers if normalization logic is repeated.
- [x] **Fix serialization contracts** — ensure Serde derives, schema generators, and TypeScript/Python bindings continue to expect strings, and regenerate any JSON fixtures or samples.
      Verify that deserialization from existing manifests still succeeds with the normalized representation.
- [x] **Refresh tests and docs** — modify unit tests, snapshots, and documentation snippets that referenced `PathBuf` so they match the normalized string representation.
      Rerun relevant test suites to confirm nothing relies on the old type.

## Questions

None.

## Notes

Store strings in the canonical form expected by future bindings, leaning on `relative-path` utilities during parsing to avoid ad hoc normalization.
