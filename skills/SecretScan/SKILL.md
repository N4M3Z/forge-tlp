---
name: SecretScan
description: Commit-time secret scanning with gitleaks — prevent credentials and PII from entering git history. USE WHEN scanning for leaked secrets, setting up pre-commit hooks, or auditing repositories for credentials.
---

# SecretScan

Prevent secrets from entering git history. Complements `safe-read` (runtime redaction) with commit-time detection using [gitleaks](https://github.com/gitleaks/gitleaks).

## Two layers of defense

| Layer | Tool | When | What happens |
|-------|------|------|-------------|
| **Runtime** | `safe-read` | AI reads a file | Secrets replaced with `[SECRET REDACTED]` in output |
| **Commit-time** | `gitleaks` | Before git commit/push | Commit blocked if secrets detected |

`safe-read` prevents the AI from seeing secrets. `gitleaks` prevents humans from committing them. Both use the same pattern source (gitleaks regex library).

## Setup

### Install

```bash
brew install gitleaks
```

### Scan the working tree

```bash
gitleaks detect --source . --no-git
```

### Scan git history

```bash
gitleaks detect --source .
```

### Baseline known findings

If the repo has historical secrets that have been rotated, create a baseline so future scans only flag new leaks:

```bash
gitleaks detect --source . --report-path .gitleaks-baseline.json
```

Then scan with the baseline:

```bash
gitleaks detect --source . --baseline-path .gitleaks-baseline.json
```

## Pre-commit hook

Add to `.githooks/pre-commit` (or your hooks path):

```bash
#!/usr/bin/env bash
# Secret scanning — block commits containing credentials
if command -v gitleaks >/dev/null 2>&1; then
  gitleaks protect --staged --no-banner
  if [ $? -ne 0 ]; then
    echo ""
    echo "gitleaks: secrets detected in staged files. Commit blocked."
    echo "Fix the issue or use --no-verify to bypass (not recommended)."
    exit 1
  fi
fi
```

Activate custom hooks path:

```bash
git config core.hooksPath .githooks
```

## Makefile target

Add a `scan` target for on-demand scanning:

```makefile
scan:
	@command -v gitleaks >/dev/null || { echo "Install gitleaks: brew install gitleaks"; exit 1; }
	gitleaks detect --source . --no-git --no-banner
```

## .gitleaks.toml

Optional config file at the project root for custom rules or allowlists:

```toml
[allowlist]
# Files that are expected to contain secret-like patterns (test fixtures, docs)
paths = [
  '''tests/fixtures/''',
  '''.env.example''',
]
```

## Related Skills

- `/TLP` — file classification and access control
- `/SafeRead` — runtime redaction tools

!`"${CLAUDE_PLUGIN_ROOT}/hooks/skill-load.sh" 2>/dev/null`
!`"${CLAUDE_PLUGIN_ROOT}/Modules/forge-tlp/hooks/skill-load.sh" 2>/dev/null`
