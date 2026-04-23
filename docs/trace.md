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
infynon trace init
```

What it does:

- creates Trace config for the repo
- prepares a default local SQLite source at `.infynon/trace/trace.db`

### 2. Add optional backend sources

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

If you do nothing after `trace init`, Trace already has a usable local SQLite backend. Add Redis or another SQL source only when you need shared or remote storage.

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

Source command exit codes:

- `0` source command completed successfully
- `30` invalid source input such as unsupported SQL engine
- `31` trace storage or backend validation failure

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

Note command exit codes:

- `0` note command completed successfully
- `30` invalid note input such as unsupported layer, scope, or status
- `31` trace storage failure

## Retrieval

Use `retrieve` when you want Trace to answer a task-specific question instead of reading every note.

```bash
infynon trace retrieve --scope branch --target auth
infynon trace retrieve --scope package --target chrono
infynon trace retrieve --author alien
infynon trace retrieve --file Cargo.toml
infynon trace retrieve --scope package --target chrono --format json
infynon trace retrieve --scope branch --target auth --format markdown --limit 5
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
- `--format json`
  Emit machine-readable retrieval output.
- `--format markdown`
  Produce a pasteable human summary for PRs, docs, or agent context.
- `--limit 5`
  Keep retrieval output focused when the note set is large.

Retrieve exit codes:

- `0` retrieval completed successfully, including empty result sets
- `30` invalid retrieval filter or unsupported output format
- `31` trace storage or backend retrieval failure

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

Sync exit codes:

- `0` sync completed successfully
- `30` invalid sync input such as unsupported direction
- `31` trace storage or backend sync failure

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
- Edit Log
- Graph

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

## Knowledge Graph

Use `trace graph` when you want to understand how entities in your repo relate to each other across branches.

The knowledge graph stores entities (files, packages, people, decisions, vulnerabilities, endpoints, modules, PRs, branches, notes) and edges (relationships like depends_on, modified_by, exposes, decided_by, owns, and more).

Every entity and edge is scoped to a branch.

### Auto-Build

Use `graph build` to automatically populate the graph from git history and existing trace notes.

```bash
infynon trace graph build
infynon trace graph build --branch feature/auth
```

What it does:

- scans `git log` for person-to-file relationships
- creates Person and File entities
- creates ModifiedBy edges between them
- converts existing trace notes into Note entities with Documents edges

### Entity Management

```bash
infynon trace graph entity add alice --kind person
infynon trace graph entity add "POST /login" --kind endpoint --meta method=POST,auth=required
infynon trace graph entity add CVE-2025-1234 --kind vulnerability
infynon trace graph entity add alice --kind person --branch feature/auth
infynon trace graph entity list --kind person
infynon trace graph entity list --branch feature/auth
infynon trace graph entity remove alice
```

Entity kinds: file, package, person, decision, endpoint, module, pr, branch, note, vulnerability.

### Edge Management

```bash
infynon trace graph edge add --from alice --to src/auth.rs --relation modified_by --evidence "commit:abc123"
infynon trace graph edge add --from src/auth.rs --to ratatui --relation depends_on --weight 0.9
infynon trace graph edge add --from CVE-2025-1234 --to ratatui --relation exposes
infynon trace graph edge list --relation modified_by
infynon trace graph edge remove <edge-id>
```

Relation types: depends_on, introduced_by, modified_by, affects, decided_by, relates_to, supersedes, conflicts_with, documents, exposes, owns.

### Queries

```bash
infynon trace graph show --branch main
infynon trace graph show --kind person
infynon trace graph path CVE-2025-1234 alice
infynon trace graph impact src/auth.rs
infynon trace graph orphans
infynon trace graph diff main feature/auth
```

What each query does:

- `show` — display all entities and edges for a branch
- `path` — find the shortest path between two entities (BFS)
- `impact` — show all entities reachable from a starting entity with depth
- `orphans` — find entities with no connections
- `diff` — compare knowledge graphs between two branches

### Export and Import

```bash
infynon trace graph export --format json -o graph.json
infynon trace graph export --format dot -o graph.dot
infynon trace graph import graph.json --branch imported
```

DOT export can be rendered with Graphviz or any compatible tool.

### TUI

```bash
infynon trace graph tui
infynon trace graph tui --branch feature/auth
```

The graph TUI has three views:

- **Entities** — list and manage entities (n=new, Enter=edit, d=delete)
- **Edges** — list and manage edges (n=new, Enter=edit, d=delete)
- **Visual** — graphical view showing entity connections grouped by kind

TUI keyboard shortcuts:

```
Tab       Cycle views (Entities / Edges / Visual)
e/w/v     Switch to Entities / Edges / Visual directly
n         New entity or edge (depending on view)
Enter     Edit selected entity or edge
d         Delete selected entity or edge
b         Open branch picker
a         Toggle all-branches view
B         Auto-build graph for current branch
r         Reload
↑↓/jk     Navigate
q         Quit
```

### Backend Support

The knowledge graph uses the same backends as Trace notes:

- **Local filesystem** — `.infynon/trace/kg/entities/` and `.infynon/trace/kg/edges/`
- **Redis** — keys under `trace:kg:entity:*` and `trace:kg:edge:*`
- **SQL** — `trace_kg_entities` and `trace_kg_edges` tables (SQLite, Postgres, MySQL)

Graph data is included in `trace sync` operations when using push/pull.

## Short Reference

```bash
infynon trace overview
infynon trace init
infynon trace source add-redis <name> --url redis://... --user alien --default
infynon trace source add-sql <name> --engine postgres --url postgres://... --user alien --default
infynon trace source list
infynon trace source default <name>
infynon trace source remove <name>
infynon trace note add <id> --title "..." --body "..."
infynon trace note update <id> --status stale
infynon trace note remove <id>
infynon trace note list
infynon trace retrieve --scope <scope> --target <target> --format table|markdown|json
infynon trace sync --direction push
infynon trace sync --direction pull
infynon trace sync --direction both
infynon trace compact
infynon trace schema sql
infynon trace schema redis
infynon trace tui
infynon trace graph build
infynon trace graph entity add <name> --kind <kind>
infynon trace graph entity list
infynon trace graph edge add --from <a> --to <b> --relation <r>
infynon trace graph edge list
infynon trace graph show
infynon trace graph path <from> <to>
infynon trace graph impact <entity>
infynon trace graph orphans
infynon trace graph diff <branch-a> <branch-b>
infynon trace graph export --format json|dot
infynon trace graph import <file>
infynon trace graph tui
```
