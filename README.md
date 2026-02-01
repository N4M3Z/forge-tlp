# context-tlp

Traffic Light Protocol (TLP) file access control plugin for Claude Code.

## What It Does

Enforces sensitivity-based file access policies using a `.tlp` config file.
Files are classified as RED (blocked), AMBER (requires approval), GREEN (open),
or CLEAR (public). AMBER reads go through `safe-read`, which strips inline
`#tlp/red` sections before the AI sees the content.

## Components

- **tlp-guard** (hook) — PreToolUse hook that intercepts Read/Edit/Write
- **safe-read** (CLI) — Reads files with inline `#tlp/red` redaction
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

## Architecture

```
Read request
  → tlp-guard-wrapper.sh (builds if needed)
    → tlp-guard binary
      → walks up to .tlp config
      → classifies file
      → RED: block (exit 2)
      → AMBER + Read: block, suggest safe-read
      → AMBER + Edit/Write: allow + warn
      → GREEN/CLEAR: allow
```

## License

MIT
