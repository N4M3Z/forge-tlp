---
name: SafeRead
description: Runtime redaction tools — safe-read strips secrets and #tlp/red sections, blind-metadata edits frontmatter without reading content. USE WHEN reading AMBER files, redacting secrets, or managing frontmatter on protected files.
---

# SafeRead

Runtime redaction tools for reading protected files and managing their metadata.

## safe-read

Read a file with inline `#tlp/red` sections stripped and secrets redacted:

```bash
Modules/forge-tlp/bin/safe-read "/path/to/file.md"
```

`RED` files are refused entirely — safe-read only handles AMBER and below.

### Secret detection

`safe-read` automatically scans for known API key and credential patterns (sourced from [gitleaks](https://github.com/gitleaks/gitleaks)) and replaces them with `[SECRET REDACTED]`. A warning is emitted to stderr when secrets are found.

Coverage includes 45+ services:

| Category | Services |
|----------|----------|
| AI/ML | Anthropic, OpenAI, OpenRouter |
| Cloud | AWS, GCP, Azure |
| Code hosting | GitHub, GitLab |
| Communication | Slack, Twilio, SendGrid, Mailchimp |
| Payments | Stripe |
| Package registries | npm |
| Databases | MongoDB connection strings |
| Crypto | PEM private keys, JWTs |

Patterns are compiled into a single regex from `src/redact/mod.rs`. They match token formats (prefix + length + character set), not secret values — so they work without a secrets database.

### Redaction modes

`safe-read` processes two kinds of redaction:

1. **TLP markers** — `#tlp/red` block and inline sections (see `/TLP` skill for marker syntax)
2. **Secret patterns** — regex-matched credentials replaced with `[SECRET REDACTED]`

Both run in a single pass. TLP redaction runs first, then secret scanning on the remaining content.

## blind-metadata

Bulk YAML frontmatter operations without reading file content. Useful for managing `tlp:` fields across files:

```bash
# Set a key on all .md files in a directory
Modules/forge-tlp/bin/blind-metadata set <directory> <key> <value>

# Get a key from all .md files
Modules/forge-tlp/bin/blind-metadata get <directory> <key>

# List files missing a key
Modules/forge-tlp/bin/blind-metadata has <directory> <key>
```

Supports absolute paths and vault-relative paths (walks up to find `.tlp` root).

### Common operations

```bash
# Classify a directory as RED
blind-metadata set Resources/Contacts tlp RED

# Audit which files have TLP frontmatter
blind-metadata has Resources/Journals tlp

# Read TLP values without opening the files
blind-metadata get Resources/Journals tlp
```

## Related Skills

- `/TLP` — classification rules, `.tlp` config, frontmatter overrides
- `/SecretScan` — commit-time secret scanning with gitleaks

!`"${CLAUDE_PLUGIN_ROOT}/hooks/skill-load.sh" 2>/dev/null`
!`"${CLAUDE_PLUGIN_ROOT}/Modules/forge-tlp/hooks/skill-load.sh" 2>/dev/null`
