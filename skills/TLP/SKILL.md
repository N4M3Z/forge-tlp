---
name: TLP
description: TLP file access control — RED/AMBER/GREEN/CLEAR classification, .tlp config, frontmatter overrides. USE WHEN accessing protected files or configuring file access policies.
---

# Traffic Light Protocol (TLP)

TLP classifies files by sensitivity. A `.tlp` config at the directory root defines path-based defaults. First match wins. Unlisted files default to AMBER.

## Levels

| Level | Read | Edit/Write |
|-------|------|------------|
| `RED` | Blocked entirely | Blocked entirely |
| `AMBER` | Blocked — requires user approval, then use `safe-read` | Allowed (never output content verbatim). Edit/Write emit a warning. |
| `GREEN` | Allowed | Allowed |
| `CLEAR` | Allowed | Allowed |

## How It Works

The `tlp-guard` hook intercepts Read, Edit, and Write tool calls. It walks up from the file path to the nearest `.tlp` config, classifies the file, and enforces the level.

If the `.tlp` config file exists but cannot be read (e.g., corrupted or permission error), all files are treated as RED until fixed (fail-closed).

### AMBER approval flow

1. You try to Read a file → `tlp-guard` blocks (exit 2)
2. The block message tells you to ask the user and provides a `safe-read` command
3. User approves → use the `safe-read` command via Bash (see `/SafeRead` skill)
4. `safe-read` outputs the file with `#tlp/red` sections and secrets stripped
5. User declines → do not read the file

## The `.tlp` Config File

Place a `.tlp` file at the root of any directory tree to protect it. Patterns are glob-style against relative paths in a .yaml file

```yaml
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

## Frontmatter Override

Files can override their path-based classification with a `tlp:` field in YAML frontmatter:

```yaml
---
tlp: RED
---
```

The effective level is the **more restrictive** of path-based and frontmatter-based classification. This means a file can escalate its protection (e.g., `GREEN` path + `RED` frontmatter = `RED`), but never downgrade it (e.g., `AMBER` path + `GREEN` frontmatter = `AMBER`).

Valid values: `RED`, `AMBER`, `GREEN`, `CLEAR` (case-insensitive). Unrecognized values are ignored.

## Inline Redaction Markers

Within AMBER files, `#tlp/red` marks the start of redacted content. Processed by `safe-read` (see `/SafeRead` skill).

### Block mode

`#tlp/red` alone on a line starts a multi-line redacted section, ended by `#tlp/amber`, `#tlp/green`, or `#tlp/clear` alone on a line:

```markdown
Normal content visible to the AI.

#tlp/red
Private content the AI must not see.
#tlp/amber

Back to normal content.
```

Output: the entire section between markers is replaced with a single `[REDACTED]` line.

### Inline mode

`#tlp/red` mid-line redacts from the marker to the next `#tlp/*` boundary tag on the same line, or to end of line:

```markdown
Normal text #tlp/red secret text #tlp/amber more normal text.
Text with #tlp/red secret to end of line
```

Output:
```
Normal text [REDACTED] more normal text.
Text with [REDACTED]
```

### Rules

- Unterminated `#tlp/red` redacts to end of file (block mode) or end of line (inline mode) — fail-safe
- Each block-mode redacted section is replaced with a single `[REDACTED]` line
- Each inline redacted span is replaced with `[REDACTED]` in place

## Related Skills

- `/SafeRead` — runtime redaction tools (`safe-read`, `blind-metadata`)
- `/SecretScan` — commit-time secret scanning with gitleaks

!`"${CLAUDE_PLUGIN_ROOT}/hooks/skill-load.sh" 2>/dev/null`
!`"${CLAUDE_PLUGIN_ROOT}/Modules/forge-tlp/hooks/skill-load.sh" 2>/dev/null`
