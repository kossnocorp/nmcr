# Template ID Generation

## Overview

Template identifiers are deterministic lowercase `snake_case` strings derived from the hierarchy of headings that introduces the template in Markdown. Commands such as `gen` and the MCP server use these IDs to refer to project templates and expose them as tool names.

## Format

- Start from the outermost heading that scopes the template and include each nested heading down to the template entry itself.
- For each heading, trim whitespace, lowercase ASCII letters, and replace runs of non-alphanumeric characters with a single underscore. Non-ASCII letters are kept in their lowercase form.
- Omit empty segments. Join the remaining segments with underscores.

An empty result is considered an error; the parser will abort rather than inventing an identifier. Downstream loaders also fail fast if duplicate IDs appear in a project.

## Examples

```md
# Rust Crate

## Manifest
```

Produces the ID `rust_crate_manifest`.

```md
# API Client

## SDK

### Template
```

Produces the ID `api_client_sdk`, because only the structural headings contribute and the helper subheading `Template` is ignored.

```md
# API ::: Client -- HTTP
```

Produces the ID `api_client_http`. Runs of punctuation never create double underscores; segments are always separated by a single `_`.

## Implementation

The `nmcr_id` crate (see `pkgs/nmcr_id`) exposes `EntityId::from_segments`, which normalizes headings and returns the final identifier. The Markdown parser (`nmcr_md_parser`) calls this generator while loading templates and stores the result on `Template.id`. Runtime tooling simply trusts that field, and the CLI plus MCP server terminate with a contextual error if duplicated IDs slip through.
