# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

Rust CLI plugin for Claude Code. Four binaries, one library crate.

## Build & test

```bash
cargo test                        # unit + integration
cargo test tlp::tests             # run one module's unit tests
cargo test --test safe_read       # run one integration test file
cargo test test_name              # run a single test by name
cargo clippy -- -D warnings       # pedantic lints enabled, must pass clean
cargo fmt --check                 # max_width = 100
```

## Project layout

```
src/
  lib.rs              # re-exports all modules
  tlp/mod.rs          # TLP enum, classify(), glob pattern matching
  vault/mod.rs        # vault discovery (walk parents to find .tlp)
  redact/mod.rs       # #tlp/red section stripping + regex secret detection
  frontmatter/mod.rs  # YAML frontmatter get/set
  bin/
    tlp-guard.rs      # PreToolUse hook — reads JSON from stdin, exits 0/2
    safe-read.rs      # CLI — redacts then prints to stdout
    safe-write.rs     # CLI — writes/edits preserving hidden content
    blind-metadata.rs # CLI — frontmatter ops without reading file body
tests/
  fixtures/configs/   # .tlp config files for integration tests
  fixtures/content/   # .md content files for integration tests
  tlp_guard.rs        # integration tests for tlp-guard
  safe_read.rs        # integration tests for safe-read
  safe_write.rs       # integration tests for safe-write
  blind_metadata.rs   # integration tests for blind-metadata
```

Each module has a `tests.rs` sibling with unit tests.

## Architecture

Classification pipeline (`tlp::classify_file`):

1. Walk up from file's parent to find vault root (directory containing `.tlp`)
2. No vault found → `None` (file not governed, hook allows access)
3. `.tlp` exists but unreadable → `RED` with `config_error: true` (fail-closed)
4. Parse `.tlp` config: first matching pattern wins, unmatched defaults to `AMBER`
5. Read file's `tlp:` frontmatter value
6. Effective level = `most_restrictive(path_level, frontmatter_level)` — frontmatter can escalate but never downgrade

AMBER handling differs by tool: Read is blocked (stderr suggests `safe-read`), Edit/Write are allowed with a warning. This lets the AI modify AMBER files without seeing their full content via Read.

### Redaction pipeline

`safe-read` chains two stages in order — `safe-write` inverts the same pipeline:

1. **TLP redaction** (`redact_tlp_sections`) — strips `#tlp/red` blocks (block-mode and inline-mode), replaces with `[REDACTED]`
2. **Secret redaction** (`redact_secrets`) — runs regex patterns from `SECRET_PATTERNS` on the TLP-redacted output, replaces matches with `[SECRET REDACTED]`

`safe-write write` reverses this: extracts hidden chunks from the original file (`extract_tlp_blocks`, `extract_inline_tlp_chunks`, `extract_secret_matches`), then `restore_hidden` replaces markers in the new content with the originals. Marker counts must match exactly or the write is refused.

### safe-write modes

- **edit**: `safe-write edit <file> --old <str> --new <str>` — string replacement on original file content (not the safe-read view). Old string must be unique and must not contain redaction markers.
- **write**: `safe-write write <file>` — full file overwrite from stdin. Preserves hidden `#tlp/red` blocks and secrets by extracting them from the original and restoring markers in the new content.
- **insert**: `safe-write insert <file> --before|--after <marker> --content <text>` — line-based insertion using trimmed marker matching.

All modes unescape `\!` → `!` in arguments (zsh history expansion artifact from Claude Code's Bash tool).

### Shell wrappers and hook dual-mode

Shell wrappers in `bin/` source `_build.sh` which calls `cargo build --release` on first invocation if the binary is missing. The CLI wrappers (`safe-read`, `safe-write`, `blind-metadata`) exit 1 on build failure.

The hook (`hooks/pre-tool-use.sh`) works in two modes: standalone plugin (`CLAUDE_PLUGIN_ROOT`) or forge-core module (`FORGE_MODULE_ROOT`). On build failure, the hook exits 0 (graceful degradation — don't block Claude).

## Conventions

- No `unsafe` code (forbidden in Cargo.toml)
- Clippy pedantic — treat warnings as errors
- Secret detection patterns in `src/redact/mod.rs` are sourced from gitleaks. Use the `SECRET_PATTERNS` const with `concat!` macro. Each pattern gets a comment naming the service.
- Test fixtures use synthetic tokens that won't trigger GitHub Push Protection. Avoid `sk_test_`, `sk_live_`, `rk_live_`, `xoxb-` prefixes — use `rk_prod_`, `xoxa-` or similar instead.
- Integration tests use `assert_cmd` + `predicates` + `tempfile`. Write fixture content to a tempdir, run the binary, assert on stdout/stderr/exit code.

## Hook protocol

`tlp-guard` reads a JSON object from stdin:

```json
{"tool_name":"Read","tool_input":{"file_path":"/absolute/path"}}
```

Exit codes: `0` = allow, `2` = block. Blocked calls print the reason to stderr.
