---
name: tlp
description: Traffic Light Protocol (TLP) file access control. Use when the user asks about TLP, file classification, safe-read, blind-metadata, redacted reading, or file access policies.
version: 0.1.0
---

# Traffic Light Protocol (TLP)

TLP classifies files by sensitivity. A `.tlp` config at the directory root defines path-based defaults. First match wins. Unlisted files default to AMBER.

## Levels

| Level | Read | Edit/Write |
|-------|------|------------|
| RED | Blocked entirely | Blocked entirely |
| AMBER | Blocked — requires user approval, then use `safe-read` | Allowed (never output content verbatim) |
| GREEN | Allowed | Allowed |
| CLEAR | Allowed | Allowed |

## How It Works

The `tlp-guard` hook intercepts Read, Edit, and Write tool calls. It walks up from the file path to the nearest `.tlp` config, classifies the file, and enforces the level.

### AMBER approval flow

1. You try to Read a file → `tlp-guard` blocks (exit 2)
2. The block message tells you to ask the user and provides a `safe-read` command
3. User approves → use the `safe-read` command via Bash
4. `safe-read` outputs the file with inline `#tlp/red` sections stripped
5. User declines → do not read the file

## The `.tlp` Config File

Place a `.tlp` file at the root of any directory tree to protect. Patterns are glob-style against relative paths.

```
RED:
  - "*.pdf"
  - "Resources/Contacts/**"

AMBER:
  - "Resources/Journals/**"

GREEN:
  - "Topics/**"
  - "Resources/Agents/**"

CLEAR:
  - ".tlp"
  - "CLAUDE.md"
```

Supported patterns:
- `*.ext` — match files by extension anywhere
- `dir/**` — match all files under a directory prefix
- `exact/path.md` — match a specific file

## Inline Redaction Markers

Within AMBER files, use `#tlp/red` on its own line to start a redacted section, and `#tlp/amber`, `#tlp/green`, or `#tlp/clear` on its own line to end it:

```markdown
Normal content visible to the AI.

#tlp/red
Private content the AI must not see.
#tlp/amber

Back to normal content.
```

- Marker must be the only content on the line (trimmed)
- `#tlp/red` in the middle of a sentence is NOT a marker
- Unterminated `#tlp/red` redacts to end of file (fail-safe)
- Each redacted section is replaced with a single `[REDACTED]` line

## CLI Tools

### safe-read

Read a file with inline `#tlp/red` sections stripped:

```bash
<plugin-path>/bin/safe-read "/path/to/file.md"
```

### blind-metadata

Bulk YAML frontmatter operations. Useful for managing `tlp:` fields across files without reading content:

```bash
# Set a key on all .md files in a directory
<plugin-path>/bin/blind-metadata set <directory> <key> <value>

# Get a key from all .md files
<plugin-path>/bin/blind-metadata get <directory> <key>

# List files missing a key
<plugin-path>/bin/blind-metadata has <directory> <key>
```

Supports absolute paths and vault-relative paths (walks up to find `.tlp` root).
