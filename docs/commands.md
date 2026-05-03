# Command Areas

## `infynon pkg`

Use `pkg` for dependency scanning, audit flows, secure installs, remediation, and package monitoring.

Examples:

```bash
infynon pkg scan
infynon pkg audit
infynon pkg explain serde_json
```

## `infynon weave`

Use `weave` for multi-step API flow creation, execution, and validation.

Examples:

```bash
infynon weave env set BASE_URL http://localhost:8001
infynon weave flow run checkout
```

## `infynon trace`

Use `trace` for repository memory, notes, provenance, sync, and TUI inspection.

Examples:

```bash
infynon trace init
infynon trace sync --direction both
infynon trace tui
```
