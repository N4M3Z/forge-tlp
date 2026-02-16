# forge-tlp

TLP file access control for AI coding agents. Classifies vault files as RED/AMBER/GREEN/CLEAR and enforces read/write restrictions. Rust crate (`context-tlp`).

## Build & Test

```bash
cargo build --release --manifest-path Cargo.toml
cargo test --manifest-path Cargo.toml                          # all tests
cargo test --manifest-path Cargo.toml test_name                # single test by name
cargo test --manifest-path Cargo.toml --test safe_read         # single integration test file
cargo test --manifest-path Cargo.toml tlp::tests               # single module unit tests
cargo clippy --manifest-path Cargo.toml -- -D warnings         # lint (must pass clean)
cargo fmt --manifest-path Cargo.toml --check                   # format check
```

### Binaries

| Binary | Purpose |
|--------|---------|
| `tlp-guard` | PreToolUse hook — blocks RED/AMBER file access |
| `safe-read` | Read AMBER files with `#tlp/red` sections and secrets redacted |
| `safe-write` | Write/edit/insert AMBER files preserving hidden blocks |
| `blind-metadata` | Edit frontmatter on RED files without reading content |

## Code Style

- **Edition 2021**, `unsafe` forbidden, clippy pedantic enabled
- **Max line width**: 100 (`rustfmt.toml`)
- **Error handling**: `Result<T, String>` — no `anyhow`/`thiserror`
- **Module pattern**: `mod.rs` + sibling `tests.rs`
- **Imports**: crate-local first, then external, then `std::`
- **Lazy regex**: `OnceLock` (no `lazy_static`)
- **Integration tests**: `assert_cmd` + `predicates` + `tempfile`

## Testing

Unit tests in `src/*/tests.rs` (tlp, vault, redact, frontmatter). Integration tests in `tests/` (tlp_guard, safe_read, safe_write, blind_metadata). Fixtures in `tests/fixtures/` — use synthetic tokens to avoid triggering GitHub Push Protection.
