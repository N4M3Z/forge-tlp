#!/usr/bin/env bash
# SessionStart: emit TLP conventions.
set -euo pipefail

MODULE_ROOT="$(command cd "$(dirname "$0")/.." && pwd)"
PROJECT_ROOT="${FORGE_ROOT:-$(command cd "$MODULE_ROOT/../.." && pwd)}"

FORGE_LOAD="$PROJECT_ROOT/Modules/forge-load/src"
if [ -f "$FORGE_LOAD/load.sh" ]; then
    source "$FORGE_LOAD/load.sh"
    load_context "$MODULE_ROOT" "$PROJECT_ROOT" --index-only
else
    awk '/^---$/{if(n++)exit;next} n{print}' "$MODULE_ROOT/skills/TLP/SKILL.md"
fi
