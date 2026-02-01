#!/usr/bin/env bash
set -euo pipefail

PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT:-$(cd "$(dirname "$0")/.." && pwd)}"
BIN_DIR="$PLUGIN_ROOT/target/release"
BINARY="$BIN_DIR/tlp-guard"

# Build if binary missing
if [ ! -x "$BINARY" ]; then
  CARGO=""
  if command -v cargo >/dev/null 2>&1; then
    CARGO=cargo
  else
    for candidate in "$HOME/.cargo/bin/cargo" /opt/homebrew/bin/cargo /usr/local/bin/cargo; do
      if [ -x "$candidate" ]; then
        CARGO="$candidate"
        break
      fi
    done
  fi

  if [ -z "$CARGO" ]; then
    echo "pai-tlp: cargo not found — install Rust: https://rustup.rs" >&2
    exit 0  # Graceful degradation: don't block Claude
  fi

  "$CARGO" build --release --manifest-path "$PLUGIN_ROOT/Cargo.toml" >&2
fi

exec "$BINARY"
