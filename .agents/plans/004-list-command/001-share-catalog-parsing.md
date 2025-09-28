# Share catalog parsing via new crate

## Objective

Stand up a reusable `nmcr_catalog` crate that loads template catalogs with tree members scoped to their parent tree definitions and exposes helpers for ID lookup so both the generator and MCP paths can consume the shared logic without relying on a `nested` flag.

## Tasks

- [ ] **Audit current catalog assembly** — locate the existing catalog-loading and ID lookup code paths in the CLI generator and MCP crates to catalogue what must be shared.
      Document any implicit behaviors around tree/file relationships so they can be preserved when refactoring.
- [ ] **Scaffold `pkgs/catalog` crate** — create the new package (Cargo manifest, lib module, feature flags) and move or re-export the template catalog data structures so they can be imported from both consumers.
      Ensure the crate exposes stable types for tree templates that carry member file lists with fully qualified IDs.
- [ ] **Centralize parsing logic** — extract the template loading routines into the crate, keeping file templates attached to their tree parents rather than flattening them at the root level.
      Provide constructors that validate duplicate IDs by walking tree members without needing a `nested` marker.
- [ ] **Implement shared ID lookup helpers** — add functions that resolve templates by ID, checking tree member files when the requested ID shares a prefix with the tree ID.
      Update generator and MCP call sites to use these helpers instead of their local implementations.
- [ ] **Cover crate behavior with tests** — add unit tests verifying tree-scoped IDs, duplicate ID detection, and lookup behavior so regressions surface quickly when consumers change.
      Include fixtures mirroring existing catalog examples to ensure parity after the move.

## Questions

None.

## Notes

Keep the crate free of CLI-specific concerns so it can be reused by additional tools in the future.
