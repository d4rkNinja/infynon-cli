# INFYNON Contracts

This page documents the current machine-readable output and exit-code contracts for the CLI areas that already expose structured stdout.

## `pkg`

### `infynon pkg scan --json`

Schema version: `infynon.pkg.scan.v1`

Top-level fields:

- `schema_version`
- `status`
- `packages_scanned`
- `vulnerabilities`
- `summary`

Example shape:

```json
{
  "schema_version": "infynon.pkg.scan.v1",
  "status": "vulnerable",
  "packages_scanned": 42,
  "vulnerabilities": [
    {
      "package": "serde_json",
      "ecosystem": "crates.io",
      "current_version": "1.0.117",
      "cve_id": "RUSTSEC-2024-0001",
      "severity": "HIGH",
      "summary": "Example advisory summary",
      "safe_version": "1.0.120",
      "fix_verified": true,
      "fix_cmd": "cargo add serde_json@1.0.120"
    }
  ],
  "summary": {
    "critical": 0,
    "high": 1,
    "medium": 0,
    "low": 0,
    "informational": 0,
    "total": 1
  }
}
```

Exit codes:

- `0` no known vulnerabilities found
- `1` warnings-only result
- `2` vulnerabilities found or scan failed upstream

### Secure install machine mode

Commands:

```bash
infynon pkg npm install <pkg> --json
infynon pkg npm install <pkg> --json --no-input
```

Schema version: `infynon.pkg.install.v1`

Top-level fields vary by outcome, but the stable envelope is:

- `schema_version`
- `status`
- `packages_checked`
- `installed`
- optional `vulnerabilities`
- optional `blocked_by`
- optional `error`

Example blocked shape:

```json
{
  "schema_version": "infynon.pkg.install.v1",
  "status": "blocked",
  "packages_checked": ["express"],
  "installed": false,
  "blocked_by": "--strict high",
  "vulnerabilities": [
    {
      "package": "express",
      "cve_id": "GHSA-xxxx-yyyy-zzzz",
      "severity": "HIGH",
      "summary": "Example advisory summary",
      "fixed_version": "4.21.0",
      "upgrade_cmd": "npm install express@4.21.0",
      "fix_is_clean": true
    }
  ]
}
```

Exit codes:

- `0` install completed successfully
- `2` security-gate lookup failed
- `3` blocked by strict policy
- `4` explicit non-interactive decision required

## `trace`

### `infynon trace retrieve --format json`

Schema version: `infynon.trace.retrieve.v1`

Top-level fields:

- `schema_version`
- `count`
- `notes`

Example shape:

```json
{
  "schema_version": "infynon.trace.retrieve.v1",
  "count": 2,
  "notes": [
    {
      "id": "repo-handoff",
      "title": "Auth changed",
      "body": "Refresh moved into middleware.",
      "layer": "team",
      "scope": "branch",
      "target": "feature/auth-refresh",
      "files": ["src/auth.rs"],
      "tags": ["auth", "handoff"],
      "author": "alien",
      "status": "active",
      "created_at": "2026-04-23T10:00:00Z",
      "updated_at": "2026-04-23T10:00:00Z"
    }
  ]
}
```

Current exit behavior:

- `0` retrieval command completed, including empty result sets
- `30` invalid trace filter or unsupported output format
- `31` trace storage or backend retrieval failure

### `infynon trace source ...`

Current exit behavior:

- `0` source command completed successfully
- `30` invalid source input such as an unsupported SQL engine
- `31` trace storage or backend validation failure

### `infynon trace note ...`

Current exit behavior:

- `0` note command completed successfully
- `30` invalid note input such as unsupported layer, scope, or status
- `31` trace storage failure

### `infynon trace sync ...`

Current exit behavior:

- `0` sync completed successfully
- `30` invalid sync input such as unsupported direction
- `31` trace storage or backend sync failure
