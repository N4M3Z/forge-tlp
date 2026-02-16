# forge-tlp

TLP file access control for AI coding agents. Classifies vault files as
RED/AMBER/GREEN/CLEAR and enforces read/write restrictions. Rust crate
(`context-tlp`), four CLI binaries, one library.

## Build & Test

```bash
cargo build --release                    # release build (stripped + LTO)
cargo test                               # all unit + integration tests
cargo test test_name                     # single test by name substring
cargo test --test safe_read              # single integration test file
cargo test tlp::tests                    # single module's unit tests
cargo clippy -- -D warnings             # lint — must pass clean
cargo fmt --check                        # format check (max_width = 100)
```

Always run `cargo clippy -- -D warnings` and `cargo fmt --check` before
committing. Both must pass with zero warnings.

### Binaries

| Binary           | Purpose                                              |
|------------------|------------------------------------------------------|
| `tlp-guard`      | PreToolUse hook — reads JSON from stdin, exits 0/2   |
| `safe-read`      | Read AMBER files with `#tlp/red` sections redacted   |
| `safe-write`     | Write/edit/insert AMBER files preserving hidden data  |
| `blind-metadata` | Bulk frontmatter ops on RED files without reading body|

## Project Layout

```
src/
  lib.rs                 # re-exports: tlp, vault, redact, frontmatter
  tlp/mod.rs             # Tlp enum, classify(), glob pattern matching
  tlp/tests.rs           # 13 unit tests
  vault/mod.rs           # vault discovery (walk parents to find .tlp)
  vault/tests.rs         # 3 unit tests
  redact/mod.rs          # #tlp/red stripping, regex secret detection, restoration
  redact/tests.rs        # 33 unit tests
  frontmatter/mod.rs     # YAML frontmatter get/set, .md file listing
  frontmatter/tests.rs   # 7 unit tests
  bin/
    tlp-guard.rs         # PreToolUse hook binary
    safe-read.rs         # redacting reader binary
    safe-write.rs        # safe writer binary (edit/write/insert modes)
    blind-metadata.rs    # frontmatter bulk ops binary
tests/
  tlp_guard.rs           # ~25 integration tests
  safe_read.rs           # ~21 integration tests
  safe_write.rs          # ~50 integration tests
  blind_metadata.rs      # ~11 integration tests
  fixtures/configs/      # .tlp config files (8 fixtures)
  fixtures/content/      # .md content files (19 fixtures)
```

## Architecture

Classification pipeline (`tlp::classify_file`): walk parents to find `.tlp` ->
no vault = `None` (allow) -> unreadable config = `RED` (fail-closed) -> first
matching pattern wins, default `AMBER` -> frontmatter can escalate but never
downgrade. AMBER Read is blocked (suggests `safe-read`); Edit/Write are allowed.

`safe-read` chains `redact_tlp_sections` then `redact_secrets`. `safe-write write`
inverts this via `restore_hidden` (marker counts must match exactly).

`tlp-guard` reads `{"tool_name":"Read","tool_input":{"file_path":"..."}}` from
stdin. Exit `0` = allow, `2` = block.

## Code Style

### Rust edition and safety

- **Edition 2021**, **`unsafe` forbidden** (`unsafe_code = "forbid"` in Cargo.toml)
- **Clippy pedantic** enabled, treated as errors (`-D warnings`). Four lints
  allowed: `module_name_repetitions`, `must_use_candidate`, `missing_errors_doc`,
  `missing_panics_doc`
- **Max line width**: 100 characters (set in `rustfmt.toml`)
- `#[rustfmt::skip]` is used on `SECRET_PATTERNS` -- `concat!` macro formatting
  is manually managed

### Error handling

- **`Result<T, String>`** -- no `anyhow` or `thiserror` crates
- Binary entry points return **`ExitCode`** directly (not `Result`)
- Error messages go to stderr via `eprintln!()`
- `Option<T>` for optional results (e.g., `classify_file` returns `Option<Classification>`)
- **let-else** pattern used frequently: `let Ok(x) = expr else { return ... }`

### Imports

Order: **crate-local first, then external, then `std::`**

```rust
use context_tlp::redact;      // crate-local
use context_tlp::tlp;
use serde::Deserialize;        // external
use std::io::{self, Read};     // std
use std::process::ExitCode;
```

### Naming conventions

- **Snake_case** for functions and variables
- **CamelCase** for types: `Tlp`, `Classification`, `HookInput`
- **SCREAMING_SNAKE_CASE** for constants: `TLP_RED_MARKER`, `SECRET_PATTERNS`
- **Module pattern**: directory with `mod.rs` + sibling `tests.rs`
- Integration test files use underscores matching binaries: `safe_read.rs` for `safe-read`

### Lazy regex

Use `OnceLock` -- no `lazy_static` crate:
```rust
fn secret_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(SECRET_PATTERNS).expect("secret patterns must compile"))
}
```

### Dependencies

Production (4 only): `regex`, `serde` (with `derive`), `serde_json`, `serde_yaml`.
Dev: `tempfile`, `assert_cmd`, `predicates`. Keep the footprint minimal. Do not add
`anyhow`, `thiserror`, `clap`, `lazy_static`, or other convenience crates.

## Testing

### Unit tests

Located in `src/*/tests.rs` for each module (tlp, vault, redact, frontmatter).
Use `#[cfg(test)] mod tests` and standard `assert_eq!` / `assert!` macros.

### Integration tests

Located in `tests/`. Each file corresponds to a binary. Use `assert_cmd` +
`predicates` + `tempfile`: write fixture files to a `TempDir`, run the binary
via `Command::cargo_bin()`, assert on stdout/stderr/exit code.

Integration tests use `#![allow(deprecated)]` at the top (standard `assert_cmd` API).

### Fixtures

Config fixtures in `tests/fixtures/configs/`, content in `tests/fixtures/content/`.
Loaded at compile time with `include_str!()`.

**Use synthetic tokens** in test fixtures to avoid triggering GitHub Push Protection.
Avoid `sk_test_`, `sk_live_`, `rk_live_`, `xoxb-` prefixes -- use `rk_prod_`,
`xoxa-` or non-standard prefixes instead.

### safe-write modes

- **edit**: `safe-write edit <file> --old <str> --new <str>` -- unique match required
- **write**: `safe-write write <file>` (reads new content from stdin)
- **insert**: `safe-write insert <file> --before|--after <marker> --content <text>`

All modes unescape `\!` to `!` in arguments (zsh history expansion artifact).
