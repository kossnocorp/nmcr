#!/usr/bin/env bash

set -eo pipefail

echo -e "⚡️ Setting up dev environment\n"

root_dir="$(dirname "$0")/.."

# Link local Codex config to home directory, as it doesn't support relative
# paths yet: https://github.com/openai/codex/issues/3706

echo -e "🌀 Using local Codex config template to generate ~/.codex/config.toml"

codex_config_path_chunk=".codex/config.toml"
local_codex_config_tmpl_path="$root_dir/.codex/config.tmpl.toml"
global_codex_config_path="$HOME/.codex/config.toml"

rm -f "$global_codex_config_path"
sed "s#{{project_root}}#${project_root}#g" "$local_codex_config_tmpl_path" > "$global_codex_config_path"

echo -e "\n💚 Dev environment setup complete!"