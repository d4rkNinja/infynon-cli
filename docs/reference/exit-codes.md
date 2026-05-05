# Exit Codes

INFYNON uses exit codes so scripts and CI jobs can react to results.

## General

| Code | Meaning |
|---:|---|
| `0` | Success |
| `1` | Runtime error |
| `2` | Invalid command input |

## Weave flow runs

| Code | Meaning |
|---:|---|
| `0` | All assertions passed |
| `20` | Flow execution failed |
| `21` | Required runtime input missing in non-interactive mode |
| `22` | Invalid flow definition or missing node |

## Trace

| Code | Meaning |
|---:|---|
| `0` | Success |
| `30` | Invalid Trace input |
| `31` | Trace storage or backend failure |

## Package scans

Package scan exit behavior depends on output mode and fix mode. Use `--json` in CI when you need machine-readable output.
