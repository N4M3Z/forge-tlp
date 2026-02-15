# forge-tlp — Verification

> **For AI agents**: Complete this checklist after installation. Every check must pass before declaring the module installed.

## Quick check

```bash
cargo test --manifest-path Modules/forge-tlp/Cargo.toml
```

Expected: 106 tests pass (49 unit + 57 integration across 3 test suites).

## Binaries available

```bash
command -v safe-read        # or: Modules/forge-tlp/bin/safe-read --help
command -v blind-metadata   # or: Modules/forge-tlp/bin/blind-metadata --help
command -v tlp-guard        # (no --help, reads JSON from stdin)
```

## Manual checks

### tlp-guard (PreToolUse hook)

```bash
echo '{"tool_name":"Read","tool_input":{"file_path":"/tmp/test.md"}}' | \
  Modules/forge-tlp/bin/tlp-guard
# File outside any .tlp vault → exit 0 (allowed)
echo $?   # should be 0
```

### safe-read

```bash
echo -e "---\ntitle: test\n---\nVisible content\n#tlp/red\nSecret content\n#tlp/amber\nMore visible" > /tmp/test-amber.md
Modules/forge-tlp/bin/safe-read /tmp/test-amber.md
# Should show: Visible content / [REDACTED] / More visible
```

### blind-metadata

```bash
mkdir -p /tmp/tlp-test && echo -e "---\ntitle: example\n---\nBody" > /tmp/tlp-test/note.md
Modules/forge-tlp/bin/blind-metadata get /tmp/tlp-test title
# Should show: note.md → example
```

## Integration tests

```bash
cargo test --test safe_read        # 21 tests — redaction, secrets, RED refusal
cargo test --test tlp_guard        # 25 tests — classification, gating, frontmatter
cargo test --test blind_metadata   # 11 tests — get, set, has, edge cases
```

## Expected results

- All 3 binaries compile and are available in PATH (or via bin/ wrappers)
- tlp-guard allows files outside TLP vaults (exit 0)
- safe-read strips `#tlp/red` sections and detects secret patterns
- blind-metadata reads/writes frontmatter without exposing file content
- All 106 tests pass
