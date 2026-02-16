# GEMINI.md

This file provides instructional context for the Gemini AI agent when working with the **forge-tlp** codebase.

## Project Overview

**forge-tlp** is a Rust-based modular framework that implements Traffic Light Protocol (TLP) file access control for AI coding tools. It ensures that the AI never sees content it shouldn't by enforcing sensitivity-based access policies (RED, AMBER, GREEN, CLEAR) at the tool level.

### Key Components
- **tlp-guard:** A `PreToolUse` hook binary that intercepts Read, Edit, and Write tool calls and blocks access to `RED` files.
- **safe-read:** A CLI tool that reads `AMBER` files while redacting inline `#tlp/red` sections and sensitive secrets (API keys, etc.).
- **safe-write:** A CLI tool for TLP-aware file writing.
- **blind-metadata:** A CLI tool for performing bulk YAML frontmatter operations without reading the file body.

### Architecture
1.  **Vault Discovery:** Walks up from a file's parent directory to find a `.tlp` configuration file.
2.  **Classification:** Uses path-based glob patterns from `.tlp` and `tlp:` frontmatter values.
3.  **Policy Enforcement:** The effective level is the most restrictive of path-based and frontmatter-based levels. Frontmatter can escalate but never downgrade protection.
4.  **Fail-Closed:** If a `.tlp` exists but is unreadable, all files are treated as `RED`.

## Building and Running

The project uses the standard Rust toolchain. Shell wrappers in the `bin/` directory provide lazy compilation on first use.

### Key Commands
- `cargo build --release`: Build all binaries in release mode.
- `cargo test`: Run all unit and integration tests (approx. 106 tests).
- `cargo test tlp::tests`: Run unit tests for a specific module.
- `cargo test --test safe_read`: Run integration tests for a specific binary.
- `cargo clippy -- -D warnings`: Run lints; must pass clean (pedantic level enabled).
- `cargo fmt`: Format code according to `rustfmt.toml` (max_width = 100).

## Development Conventions

### Code Style & Structure
- **No `unsafe` code:** Forbidden in `Cargo.toml`.
- **Pedantic Lints:** Clippy pedantic lints are enabled and must be satisfied.
- **Project Layout:**
    - `src/lib.rs`: Library crate re-exporting all modules.
    - `src/tlp/`: TLP enum and classification logic.
    - `src/vault/`: Vault discovery logic.
    - `src/redact/`: Redaction and secret detection (patterns from gitleaks).
    - `src/frontmatter/`: YAML frontmatter manipulation.
    - `src/bin/`: Entry points for the four CLI binaries.
    - `tests/`: Integration tests using `assert_cmd`, `predicates`, and `tempfile`.

### Secret Detection
- Patterns are sourced from gitleaks and located in `src/redact/mod.rs`.
- Use synthetic tokens for test fixtures to avoid triggering security scanners. Avoid `sk_test_`, `sk_live_`, etc.

### TLP Hooks
- The `tlp-guard` hook expects a JSON object on `stdin` (e.g., `{"tool_name":"Read","tool_input":{"file_path":"..."}}`).
- Exit code `0` allows the tool call; exit code `2` blocks it.
- Files outside any vault (no `.tlp` found) default to allowed (exit `0`).
