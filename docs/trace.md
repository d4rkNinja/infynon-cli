# Trace

`infynon trace` is the repo memory and provenance layer in INFYNON.

Claude Code companion:
[d4rkNinja/code-guardian](https://github.com/d4rkNinja/code-guardian)

Use `code-guardian` when you want Claude Code to:

- retrieve the latest Trace memory before a task starts
- update Trace after the task ends
- connect that workflow through Claude Code hooks

It is designed for:

- canonical memory that should stay durable
- team memory for shared repo context
- user memory for local working notes
- package notes that can explain who introduced a risky dependency
- Redis or SQL backends with the same logical schema

## Mental Model

Trace is not one giant notes file.

Think about it as:

- `sources`
  where the memory lives
- `notes`
  the actual memory records
- `retrieve`
  the query path for getting relevant memory back
- `sync`
  moving memory between local state and configured backends
- `compact`
  reducing noise after work accumulates
- `tui`
  inspecting memory and package ownership visually

## Claude Code Integration

If you want Trace to work smoothly with Claude Code, pair it with the `code-guardian` companion:

- skill repo: [d4rkNinja/code-guardian](https://github.com/d4rkNinja/code-guardian)
- best fit for hook-based Trace retrieval and update flows
- useful when you want agent memory behavior without building a separate custom bridge

## Typical Flow

### 1. Initialize Trace for the repo

Use this once per repo to define owner and default user identity.

```bash
infynon trace init --repo infynon-cli --owner team --user alien
```

What it does:

- creates Trace config for the repo
- stores the repo owner label
- stores the default user so later notes have a sensible author

### 2. Add a backend source

Use Redis when you want fast retrieval and live-feeling coordination:

```bash
infynon trace source add-redis team-redis \
  --url redis://localhost:6379/0 \
  --namespace infynon \
  --user alien \
  --default
```

Use SQL when you want durable history and better filtering:

```bash
infynon trace source add-sql team-db \
  --engine postgres \
  --url postgres://user:pass@db.example.com:5432/infynon \
  --user alien \
  --default
```

What `source add-redis` is for:

- lower-latency retrieval
- active session style coordination
- teams that already run Redis and want a faster live layer

What `source add-sql` is for:

- long-term note storage
- structured filtering
- canonical memory and durable history

### 3. Inspect configured sources

Use these when you want to understand or change source configuration.

```bash
infynon trace source list
infynon trace source default team-db
infynon trace source remove old-db
```

What each command does:

- `source list`
  Prints all configured sources and shows which one is the default.
- `source default <name>`
  Switches the default source used by Trace operations.
- `source remove <name>`
  Removes a source from local Trace configuration.

## Notes

Notes are the core memory objects in Trace.

### Create a note

Use `note add` when you want to capture a repo fact, handoff, package note, or branch-specific warning.

```bash
infynon trace note add repo-handoff \
  --title "Auth flow changed" \
  --body "Refresh logic moved into middleware." \
  --layer team \
  --scope branch \
  --target feature/auth-refresh \
  --files src/auth.rs \
  --tags auth,handoff
```

What the main fields mean:

- note id: `repo-handoff`
  A stable identifier for the note.
- `--title`
  Short human-facing summary.
- `--body`
  Actual explanation or handoff text.
- `--layer`
  Where the note belongs: canonical, team, or user.
- `--scope`
  What the note attaches to: repo, branch, file, package, and similar scopes.
- `--target`
  The specific thing inside that scope.
- `--files`
  Optional file links for retrieval and inspection.
- `--tags`
  Extra retrieval handles.

### Update a note

Use `note update` when the note is still relevant but its content or status has changed.

```bash
infynon trace note update repo-handoff --status stale
infynon trace note update repo-handoff --title "Auth flow updated"
```

Typical uses:

- mark a note stale after merges
- correct a title or body
- change lifecycle state without deleting the note

### Remove a note

Use this when the note should no longer exist at all.

```bash
infynon trace note remove repo-handoff
```

### List notes

Use this to inspect the current local note set.

```bash
infynon trace note list
```

## Retrieval

Use `retrieve` when you want Trace to answer a task-specific question instead of reading every note.

```bash
infynon trace retrieve --scope branch --target auth
infynon trace retrieve --scope package --target chrono
infynon trace retrieve --author alien
infynon trace retrieve --file Cargo.toml
```

What each pattern is good for:

- `--scope branch --target auth`
  Find branch-specific handoffs or context.
- `--scope package --target chrono`
  Find package ownership or risk memory.
- `--author alien`
  Pull notes created by one user.
- `--file Cargo.toml`
  Pull notes attached to a file.

## Sync

Use sync when you want local Trace state and backend state to match.

### Push local data to the backend

```bash
infynon trace sync --direction push
```

Use this when:

- you added local notes and want them stored remotely
- you want package findings available to the rest of the team

### Pull remote data into local Trace

```bash
infynon trace sync --direction pull
```

Use this when:

- another machine or teammate already updated Trace
- you want the latest memory before starting work

### Bidirectional sync

```bash
infynon trace sync --direction both
```

Use this as the normal "bring everything up to date" command.

## Compact

Use `compact` when Trace has accumulated temporary or stale note noise.

```bash
infynon trace compact
```

Typical reasons to run it:

- after merges
- after large branch work finishes
- when temporary notes should be reduced or archived

## Schema Commands

Use schema commands when you want to provision or inspect backend storage.

```bash
infynon trace schema sql
infynon trace schema redis
```

What each one does:

- `schema sql`
  Prints the SQL schema for SQL-backed Trace setups.
- `schema redis`
  Prints the Redis key layout for Redis-backed Trace setups.

## TUI

Use the TUI when you want to browse sources, notes, and package ownership from one place.

```bash
infynon trace tui
```

Current tabs:

- Overview
- Sources
- Notes
- Packages

Why the Packages tab matters:

- it reuses package findings
- it can show `installed_by` when package-scoped Trace notes exist
- it helps answer "who introduced this compromised package?"

## Recommended Pairing

```text
Claude Code + code-guardian + INFYNON Trace
```

Use that stack when you want:

- Claude Code to read the latest shared context before editing
- Trace to remain the structured memory backend
- package ownership and handoff notes to stay queryable from the CLI and TUI

## Short Reference

```bash
infynon trace overview
infynon trace init --owner team --user alien
infynon trace source add-redis <name> --url redis://... --user alien --default
infynon trace source add-sql <name> --engine postgres --url postgres://... --user alien --default
infynon trace source list
infynon trace source default <name>
infynon trace source remove <name>
infynon trace note add <id> --title "..." --body "..."
infynon trace note update <id> --status stale
infynon trace note remove <id>
infynon trace note list
infynon trace retrieve --scope <scope> --target <target>
infynon trace sync --direction push
infynon trace sync --direction pull
infynon trace sync --direction both
infynon trace compact
infynon trace schema sql
infynon trace schema redis
infynon trace tui
```
