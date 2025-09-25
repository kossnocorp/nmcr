#!/usr/bin/env bash

set -eo pipefail

function log {
  if [ -n "$VERBOSE" ]; then
    echo -e "$1"
  fi
}

log "⚡️ Starting up MCP server\n"

root_dir="$(dirname "$0")/.."

log "🌀 Running MCP server for project at $project_path"

project_path="${1:-./examples/basic/}"

cd "$root_dir" || echo "🔴 Can't cd to $root_dir. Does this directory exist?"
RUSTFLAGS=-Awarnings cargo run --quiet -- mcp --project "$project_path"