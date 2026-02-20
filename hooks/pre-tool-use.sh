#!/usr/bin/env bash
# PreToolUse hook: TLP file access gate.
# Dual-mode: works standalone (CLAUDE_PLUGIN_ROOT) or as forge-core module (FORGE_MODULE_ROOT).
set -euo pipefail

MODULE_ROOT="${FORGE_MODULE_ROOT:-${CLAUDE_PLUGIN_ROOT:-$(command cd "$(dirname "$0")/.." && pwd)}}"
export CLAUDE_PLUGIN_ROOT="$MODULE_ROOT"  # So _build.sh finds Cargo.toml

source "$MODULE_ROOT/bin/_build.sh"
ensure_built tlp-guard || exit 0  # Graceful degradation: don't block Claude

exec "$BIN_DIR/tlp-guard"
