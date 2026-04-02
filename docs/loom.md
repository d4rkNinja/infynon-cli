# Loom

`infynon loom` is the shared coding-memory layer in INFYNON.

It is designed for:

- canonical memory that should stay durable
- team memory for handoffs and repo-wide notes
- user memory for local or personal context
- package notes that can explain who introduced a risky dependency
- SQL or Redis backends with the same logical schema

## Identity

Loom works better when the repo has a default user:

```bash
infynon loom init --repo infynon-cli --owner team --user alien
```

That user becomes the default author for `loom note add` when `--author` is not passed.

Backend sources can also carry an owner user:

```bash
infynon loom source add-sql team-sqlite \
  --engine sqlite \
  --url sqlite://.infynon/loom/loom.db \
  --user alien \
  --default
```

## Command Flow

### 1. Initialize Loom for the repo

```bash
infynon loom init --owner team --user alien
```

### 2. Add a backend

Redis:

```bash
infynon loom source add-redis team-redis \
  --url redis://localhost:6379/0 \
  --namespace infynon \
  --user alien \
  --default
```

SQL:

```bash
infynon loom source add-sql team-db \
  --engine postgres \
  --url postgres://user:pass@db.example.com:5432/infynon \
  --user alien \
  --default
```

## Why Redis vs SQL

Redis is better when you want:

- lower-latency retrieval
- live coordination
- active session state
- conflict and overlap checks

SQL is better when you want:

- durable structured history
- better reporting and filtering
- canonical memory
- easier audits and exports

## Notes

Create notes:

```bash
infynon loom note add repo-handoff \
  --title "Auth flow changed" \
  --body "Refresh logic moved into middleware." \
  --layer team \
  --scope branch \
  --target feature/auth-refresh \
  --files src/auth.rs \
  --tags auth,handoff
```

Retrieve notes:

```bash
infynon loom retrieve --scope branch --target auth
infynon loom retrieve --scope package --target chrono
```

Update and compact:

```bash
infynon loom note update repo-handoff --status stale
infynon loom compact
```

## Sync

Push local notes and package findings:

```bash
infynon loom sync --direction push
```

Pull remote notes back into the local Loom store:

```bash
infynon loom sync --direction pull
```

Bidirectional sync:

```bash
infynon loom sync --direction both
```

## TUI

```bash
infynon loom tui
```

Current tabs:

- Overview
- Sources
- Notes
- Packages

The Packages tab reuses package scanning and shows `installed_by` when a package-scoped Loom note exists for that package.
