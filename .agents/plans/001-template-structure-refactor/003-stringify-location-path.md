# Adopt relative paths for locations

## Objective

Represent template locations with [`RelativePathBuf`](https://docs.rs/relative-path/latest/relative_path/) from the `relative_path` crate so metadata stays portable and easy to stringify across languages.

## Tasks

- [ ] **Audit `Location` usages** — identify every crate that constructs or reads `Location` to understand formatting expectations and serialization paths.
      Pay attention to deserializers, schema exports, and CLI output code.
- [ ] **Update struct definitions** — change `Location.path` from `PathBuf` to `RelativePathBuf` in `pkgs/types-internal/src/lib.rs` and propagate that change through any public APIs or re-exports.
      Adjust defaults and builders to accept the new type and add the crate dependency where needed.
- [ ] **Normalize path handling** — ensure call sites convert filesystem paths to normalized, forward-slash relative paths before storing them in `RelativePathBuf`.
      Add lightweight helpers if normalization logic is repeated.
- [ ] **Fix serialization contracts** — update Serde derives, schema generators, and TypeScript/Python bindings so they expect strings and serialize via `RelativePathBuf::as_str`, regenerating any JSON fixtures or samples.
      Verify that deserialization from existing manifests still succeeds with the new type.
- [ ] **Refresh tests and docs** — modify unit tests, snapshots, and documentation snippets that reference `PathBuf` to match the `RelativePathBuf` representation.
      Rerun relevant test suites to confirm nothing relies on `PathBuf` behaviors.

## Questions

None.

## Notes

Keep stored paths platform-agnostic and relative to the project root for consistent downstream usage.
