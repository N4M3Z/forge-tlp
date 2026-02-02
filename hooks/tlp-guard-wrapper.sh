#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../bin/_build.sh"
ensure_built tlp-guard || exit 0  # Graceful degradation: don't block Claude
exec "$BIN_DIR/tlp-guard"
