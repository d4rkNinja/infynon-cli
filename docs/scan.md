# infynon pkg scan

Scan your project's dependency tree for known CVEs using the OSV.dev database.

If you also use Trace with Claude Code, pair it with the `code-guardian` companion so package findings can connect back to repo memory:
[d4rkNinja/code-guardian](https://github.com/d4rkNinja/code-guardian)

## Usage

```
infynon pkg scan [OPTIONS]
```

## Options

| Flag | Description |
|------|-------------|
| `--output <FORMAT>` | Export high-quality security report: `markdown`, `pdf`, `both` |
| `--fix [LEVEL]` | Run the auto-fix remediation workflow with live progress tracking, error reporting, and strict/warn security gating. Levels: `critical`, `high`, `medium`, `low`, `informational`, `all` (default: `all`) |
| `--pkg-file <PATH>` | Scan a specific lock/manifest file instead of auto-detecting |

## Examples

```bash
# Auto-detect lock files and scan
infynon pkg scan

# Scan a specific file
infynon pkg scan --pkg-file ./Cargo.lock

# Export professional PDF & Markdown reports
infynon pkg scan --output both

# Auto-fix all fixable vulnerabilities with live progress tracking
infynon pkg scan --fix

# Auto-fix only critical and high severity
infynon pkg scan --fix high
```

## Supported Lock Files

| Ecosystem | Files |
|-----------|-------|
| npm | `package-lock.json` |
| yarn | `yarn.lock` |
| pnpm | `pnpm-lock.yaml` |
| pip | `requirements.txt` |
| uv | `uv.lock` |
| poetry | `poetry.lock` |
| cargo | `Cargo.lock` |
| go | `go.sum`, `go.mod` |
| gem | `Gemfile.lock` |
| composer | `composer.lock` |
| nuget | `packages.lock.json` |
| hex | `mix.lock` |
| pub | `pubspec.lock` |
| pip/uv/poetry | `pyproject.toml` |

## Report Output

The scan produces a unified table with columns:

- **Risk** — Severity level (CRITICAL, HIGH, MEDIUM, LOW, INFORMATIONAL)
- **Package** — Package name
- **Version** — Currently installed version
- **CVE / ID** — OSV vulnerability identifier
- **Remediation** — Fix version from OSV, or latest stable from registry if no DB fix exists
