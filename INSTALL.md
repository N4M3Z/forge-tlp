# forge-tlp — Installation

> **For AI agents**: This guide covers installation of forge-tlp. Follow the steps for your deployment mode.

## As part of forge-core (submodule)

Already included as a submodule. Build with:

```bash
make install    # builds all modules including forge-tlp
```

Or build individually:

```bash
cargo build --release --manifest-path Modules/forge-tlp/Cargo.toml
```

Ensure the module is listed in `forge.yaml` under modules (it should be — order matters, TLP must run before forge-journals in PreToolUse):

```yaml
modules:
  - forge-tlp    # PreToolUse — TLP file access gate
```

## Standalone (Claude Code plugin)

```bash
claude plugin install forge-tlp
```

Or install from a local path during development:

```bash
claude plugin install /path/to/forge-tlp
```

Standalone mode uses the module's own `hooks/hooks.json` and `lib/load.sh`. No forge-core dependency required.

## What gets installed

| Binary | Purpose |
|--------|---------|
| `tlp-guard` | PreToolUse hook — gates Read/Edit/Write based on TLP classification |
| `safe-read` | CLI — reads AMBER files with `#tlp/red` sections and secrets redacted |
| `blind-metadata` | CLI — bulk frontmatter operations without reading file content |

Shell wrappers in `bin/` handle lazy compilation — the Rust binary is built on first invocation if missing.

## Configuration

### .tlp config file

Place a `.tlp` file at the root of any directory tree to protect it:

```yaml
RED:
  - "*.pdf"
  - "Resources/Contacts/**"
AMBER:
  - "Resources/Journals/**"
GREEN:
  - "Topics/**"
CLEAR:
  - ".tlp"
```

### Module config override

Create `config.yaml` (gitignored) to override compiled defaults:

```yaml
# Override event subscriptions
events:
  - PreToolUse
```

Setting `events: []` disables all hooks for this module.

## Dependencies

| Dependency | Required | Purpose |
|-----------|----------|---------|
| Rust + cargo | Yes | Build the 3 binaries |

No external runtime dependencies. All pattern matching is compiled into the binaries.

## Verify

See [VERIFY.md](VERIFY.md) for the post-installation checklist.
