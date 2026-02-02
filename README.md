# context-tlp

Traffic Light Protocol (TLP) file access control plugin for Claude Code.

## What It Does

Enforces sensitivity-based file access policies using a `.tlp` config file.
Files are classified as RED (blocked), AMBER (requires approval), GREEN (open),
or CLEAR (public). AMBER reads go through `safe-read`, which strips inline
`#tlp/red` sections before the AI sees the content.

## Components

- **tlp-guard** (hook) — PreToolUse hook that intercepts Read/Edit/Write
- **safe-read** (CLI) — Reads files with inline `#tlp/red` redaction + secret detection
- **blind-metadata** (CLI) — Bulk YAML frontmatter operations

## Requirements

- Rust toolchain ([rustup.rs](https://rustup.rs)) — binaries build on first use
- A `.tlp` file at the root of each directory tree to protect

## Installation

### From marketplace

```
/plugin marketplace add <owner>/<repo>
/plugin install context-tlp@pai-plugins
```

### Local testing

```bash
claude --plugin-dir /path/to/Plugins/context-tlp
```

### Post-install

Whitelist the CLI tools in your project or global `settings.local.json`:

```json
{
  "permissions": {
    "allow": [
      "Bash(<plugin-path>/bin/safe-read:*)",
      "Bash(<plugin-path>/bin/blind-metadata:*)"
    ]
  }
}
```

## Configuration

Create a `.tlp` file at your directory root. See [examples/example.tlp](examples/example.tlp).

### Pattern syntax

Patterns are listed under level headers (`RED:`, `AMBER:`, `GREEN:`, `CLEAR:`) as quoted strings with a `- ` prefix:

| Pattern | Matches | Example |
|---------|---------|---------|
| `*.ext` | Any file with that extension, anywhere in the tree | `"*.pdf"` matches `docs/report.pdf` |
| `dir/**` | All files under a directory (recursive) | `"Contacts/**"` matches `Contacts/john.md` |
| `exact/path.md` | Exact relative path only | `"README.md"` matches only `README.md` at the root |

First match wins. Files not matched by any pattern default to AMBER.

### Frontmatter override

Files can escalate their own protection level via a `tlp:` field in YAML frontmatter:

```yaml
---
tlp: RED
---
```

The effective level is the **more restrictive** of the path-based and frontmatter-based classification. A file can escalate (GREEN path + RED frontmatter = RED) but never downgrade (AMBER path + GREEN frontmatter = AMBER).

### Fail-closed behavior

If `.tlp` exists but cannot be read (permissions, corruption), all files in that vault are treated as RED and access is blocked until the config is fixed. This prevents accidental exposure from a broken config.

Files outside any vault (no `.tlp` in any parent directory) are not affected by the hook.

## Architecture

```
Read request
  → tlp-guard-wrapper.sh (builds if needed)
    → tlp-guard binary
      → walks up to .tlp config
      → classifies file (path pattern + frontmatter override)
      → RED: block (exit 2)
      → AMBER + Read: block, suggest safe-read
      → AMBER + Edit/Write: allow + warn
      → GREEN/CLEAR: allow
```

`safe-read` also checks TLP classification and refuses RED files.

## Development

```bash
# Run all tests (unit + integration)
cargo test

# Check for warnings
cargo clippy -- -D warnings

# Build release binaries
cargo build --release

# Format code
cargo fmt
```

### Project structure

```
src/
  lib.rs              # Library crate (re-exports modules)
  tlp.rs              # TLP enum, classify(), pattern matching
  vault.rs            # Vault discovery (walk up to .tlp)
  redact.rs           # TLP section redaction + secret detection
  frontmatter.rs      # YAML frontmatter get/set, .md file listing
  bin/
    tlp-guard.rs      # PreToolUse hook binary
    safe-read.rs      # Redacting file reader binary
    blind-metadata.rs # Frontmatter bulk operations binary
tests/
  tlp_guard.rs        # Integration tests for tlp-guard
  safe_read.rs        # Integration tests for safe-read
  blind_metadata.rs   # Integration tests for blind-metadata
```

## License

MIT
