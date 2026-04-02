# INFYNON Command Reference

This document explains what each command area is for and when to use each major command.

Claude Code companion:
[d4rkNinja/code-guardian](https://github.com/d4rkNinja/code-guardian)

## Root

```bash
infynon <command>
```

Top-level command areas:

- `infynon pkg`
  Use this when you want to scan, audit, install, trace, or fix dependencies.
- `infynon weave`
  Use this when you want to model, run, validate, or probe API flows.
- `infynon trace`
  Use this when you want to store, retrieve, sync, or inspect repo memory and provenance.

If you use Claude Code with Trace, pair it with `code-guardian` so handoff context can be pulled before work and updated again after the task.

## Package Intelligence

```bash
infynon pkg <subcommand>
```

### `scan`

Use `scan` when you want to inspect the current repo's dependency files for known package risk.

Typical cases:

- scan the current repo before a commit
- scan a specific lockfile or manifest
- generate a report for sharing
- optionally start remediation immediately

```bash
infynon pkg scan
infynon pkg scan --pkg-file <PATH>
infynon pkg scan --output markdown
infynon pkg scan --output pdf
infynon pkg scan --output both
infynon pkg scan --fix
infynon pkg scan --fix high
```

Command behavior:

- `infynon pkg scan`
  Auto-detects supported package files in the current repo and scans them.
- `infynon pkg scan --pkg-file <PATH>`
  Forces the scan to use a specific file instead of auto-detection.
- `infynon pkg scan --output markdown|pdf|both`
  Saves scan results in a report format for sharing or documentation.
- `infynon pkg scan --fix`
  Starts remediation after the scan using the default fix behavior.
- `infynon pkg scan --fix high`
  Focuses fix behavior on high-and-above severity findings.

### Secure Install Commands

Use these commands when you want INFYNON to sit in front of the package manager and inspect package risk while installing.

Typical cases:

- install a new dependency safely
- make AI-assisted installs visible
- enforce strict severity blocking during install

```bash
infynon pkg npm install <pkg>
infynon pkg yarn add <pkg>
infynon pkg pnpm add <pkg>
infynon pkg bun add <pkg>
infynon pkg pip install <pkg>
infynon pkg uv add <pkg>
infynon pkg poetry add <pkg>
infynon pkg cargo add <pkg>
infynon pkg go get <module>
infynon pkg gem install <pkg>
infynon pkg composer require <vendor/pkg>
infynon pkg nuget add <pkg>
infynon pkg hex deps.get
infynon pkg pub add <pkg>
```

How to think about these:

- same package-manager intent
- same ecosystem-specific syntax
- INFYNON adds package inspection, reporting, and policy around the install

### Strict Mode

Use strict mode when you want INFYNON to block installs above a severity threshold instead of only reporting them.

```bash
infynon pkg --strict npm install <pkg>
infynon pkg --strict high npm install <pkg>
infynon pkg --strict pip install <pkg>
infynon pkg --strict cargo add <pkg>
```

Command behavior:

- `--strict`
  Enables blocking behavior using the default severity threshold.
- `--strict high`
  Blocks high and critical issues.
- ecosystem-specific examples like `pip` and `cargo`
  Apply the same policy logic across ecosystems.

### Other Package Commands

These commands support audit, investigation, cleanup, and remediation workflows.

```bash
infynon pkg audit
infynon pkg why <package>
infynon pkg outdated
infynon pkg diff <pkg> <v1> <v2>
infynon pkg doctor
infynon pkg size <pkg>
infynon pkg search <query>
infynon pkg fix --auto
infynon pkg clean
infynon pkg migrate npm pnpm
```

What each one is for:

- `infynon pkg audit`
  Review the repo's current dependency risk without installing anything.
- `infynon pkg why <package>`
  Explain why a package exists in the tree and what pulled it in.
- `infynon pkg outdated`
  Show dependency versions that are behind current releases.
- `infynon pkg diff <pkg> <v1> <v2>`
  Compare two versions of a package to understand change surface.
- `infynon pkg doctor`
  Diagnose package-state or environment issues in the current repo.
- `infynon pkg size <pkg>`
  Inspect package footprint when dependency weight matters.
- `infynon pkg search <query>`
  Search for packages before deciding what to install.
- `infynon pkg fix --auto`
  Apply remediation automatically where the tool can do so safely.
- `infynon pkg clean`
  Remove INFYNON-generated package artifacts or cleanup state.
- `infynon pkg migrate npm pnpm`
  Help move a project from one package manager workflow to another.

### Eagle Eye

Use Eagle Eye when you want ongoing monitoring instead of one-time scanning.

```bash
infynon pkg eagle-eye setup
infynon pkg eagle-eye start
infynon pkg eagle-eye status
infynon pkg eagle-eye enable
infynon pkg eagle-eye disable
```

What each one is for:

- `setup`
  Configure the monitoring feature for the current repo or machine.
- `start`
  Start scheduled monitoring.
- `status`
  Show whether monitoring is currently active and healthy.
- `enable`
  Turn monitoring on for the configured target.
- `disable`
  Turn monitoring off without deleting the config.

## Weave

```bash
infynon weave <subcommand>
```

Weave commands are for building and running API behavior, not just single requests.

### TUI And Environment

```bash
infynon weave tui
infynon weave env set BASE_URL http://localhost:8001
infynon weave env list
infynon weave env get BASE_URL
infynon weave env delete BASE_URL
```

What each one is for:

- `infynon weave tui`
  Opens the terminal UI for flows, runs, variables, and inspection.
- `infynon weave env set`
  Stores shared environment values such as `BASE_URL`, tokens, or common headers.
- `infynon weave env list`
  Shows the current saved environment values.
- `infynon weave env get`
  Prints one specific variable.
- `infynon weave env delete`
  Removes a variable you no longer want to keep.

### Node Commands

Use node commands when you want to define or test one API request unit.

```bash
infynon weave node create
infynon weave node create --ai "POST /auth/login extracts token"
infynon weave node list
infynon weave node get <node-id>
infynon weave node run <node-id> --prompt
infynon weave node clone <node-id> <new-id>
infynon weave node export <node-id>
infynon weave node remove <node-id>
```

What each one is for:

- `node create`
  Creates a new request node interactively.
- `node create --ai "..."`
  Lets AI scaffold the node from a natural-language intent.
- `node list`
  Shows all nodes in the current project.
- `node get`
  Displays one node's current definition.
- `node run`
  Runs one node in isolation for debugging or validation.
- `node clone`
  Copies a node into a new variation without rebuilding it manually.
- `node export`
  Exports a node as a reusable external representation.
- `node remove`
  Deletes a node definition.

### Flow Commands

Use flow commands when the behavior spans multiple requests and shared context.

```bash
infynon weave flow create "checkout" --ai "login then create order"
infynon weave flow list
infynon weave flow show <flow-id>
infynon weave flow run <flow-id>
infynon weave flow run-all
infynon weave flow remove <flow-id>
infynon weave flow merge <flow1-id> <flow2-id> --join-at <node-id>
```

What each one is for:

- `flow create`
  Creates a new flow manually or with AI guidance.
- `flow list`
  Shows all saved flows.
- `flow show`
  Displays a flow graph and its node connections.
- `flow run`
  Executes one flow from start to finish.
- `flow run-all`
  Executes every flow in the current project.
- `flow remove`
  Deletes a flow without removing the underlying nodes.
- `flow merge`
  Combines two flows into one larger behavior path.

### Flow Wiring Commands

Use these when you want to connect or disconnect behavior between nodes.

```bash
infynon weave attach <from-node-id> <to-node-id>
infynon weave attach login get-profile --carry token,user_id
infynon weave attach create-user send-email --condition "status == 201"
infynon weave attach login get-profile --ai
infynon weave detach <from-node-id> <to-node-id>
```

What each one is for:

- `attach`
  Creates an edge between nodes.
- `attach --carry`
  Explicitly passes context values along the edge.
- `attach --condition`
  Adds conditional branching behavior.
- `attach --ai`
  Lets AI infer useful carry behavior.
- `detach`
  Removes the connection between two nodes.

### Import, Validation, And AI

```bash
infynon weave import openapi.yaml --flow "My Flow"
infynon weave validate
infynon weave ai probe <flow-id>
infynon weave ai suggest --after <node-id>
infynon weave ai attach --after <node-id>
infynon weave ai complete <flow-id>
infynon weave ai explain <flow-id>
infynon weave ai assert <node-id>
infynon weave ai branch <node-id>
```

What each one is for:

- `import`
  Builds nodes or flows from OpenAPI or Swagger definitions.
- `validate`
  Checks that nodes, flows, edges, and definitions are internally valid.
- `ai probe`
  Runs AI-assisted security or behavior probes against a flow.
- `ai suggest`
  Recommends the next useful node after a selected node.
- `ai attach`
  Connects the next node automatically using AI assistance.
- `ai complete`
  Fills out unconnected or incomplete flow sections.
- `ai explain`
  Explains why a run failed or behaved unexpectedly.
- `ai assert`
  Generates assertions for a node.
- `ai branch`
  Suggests branch logic or alternate behavior paths.

For deeper Weave usage, see `docs/weave.md`.

## Trace

```bash
infynon trace <subcommand>
```

Trace commands manage repo memory, package provenance, handoff context, and backend sync.

Recommended Claude Code companion:
[d4rkNinja/code-guardian](https://github.com/d4rkNinja/code-guardian)

### Overview And Setup

```bash
infynon trace overview
infynon trace init --owner team --user alien
infynon trace source add-redis team-redis --url redis://localhost:6379/0 --namespace infynon --user alien --default
infynon trace source add-sql team-db --engine sqlite --url sqlite://.infynon/trace/trace.db --user alien --default
infynon trace source list
infynon trace source default team-db
infynon trace source remove team-db
```

What each one is for:

- `overview`
  Prints the current Trace state summary for the repo.
- `init`
  Creates the local Trace configuration for the repo and stores owner/default-user identity.
- `source add-redis`
  Adds a Redis backend for lower-latency retrieval and live-style coordination.
- `source add-sql`
  Adds a SQL backend for durable history and structured filtering.
- `source list`
  Shows all configured Trace sources.
- `source default`
  Chooses which source is used as the default target.
- `source remove`
  Removes a configured source from local Trace configuration.

### Notes

```bash
infynon trace note add repo-handoff --title "Auth changed" --body "Refresh moved to middleware" --layer team --scope branch --target feature/auth-refresh --files src/auth.rs --tags auth,handoff
infynon trace note update repo-handoff --status stale
infynon trace note remove repo-handoff
infynon trace note list
```

What each one is for:

- `note add`
  Creates a new note and attaches it to a layer and scope.
- `note update`
  Changes note content or state after the fact.
- `note remove`
  Deletes a note you no longer want to keep.
- `note list`
  Shows the current local notes and their states.

### Retrieval

```bash
infynon trace retrieve --scope branch --target auth
infynon trace retrieve --scope package --target chrono
infynon trace retrieve --author alien
infynon trace retrieve --file Cargo.toml
```

Use `retrieve` when you want the right context for a repo task instead of reading all notes manually.

What each pattern is for:

- `--scope branch --target auth`
  Pull branch-related context for a branch or branch-like target.
- `--scope package --target chrono`
  Pull package ownership or package-risk memory for one package.
- `--author alien`
  Filter memory by author.
- `--file Cargo.toml`
  Pull notes related to a specific file.

### Sync, Schema, And TUI

```bash
infynon trace sync --direction push
infynon trace sync --direction pull
infynon trace sync --direction both
infynon trace compact
infynon trace schema sql
infynon trace schema redis
infynon trace tui
```

What each one is for:

- `sync --direction push`
  Pushes local notes and findings to the configured backend.
- `sync --direction pull`
  Pulls remote notes and findings into local Trace storage.
- `sync --direction both`
  Runs a bidirectional sync.
- `compact`
  Reduces note noise and archives stale material where possible.
- `schema sql`
  Prints the SQL schema so users can provision or review the database.
- `schema redis`
  Prints the Redis key model for Redis-backed setups.
- `tui`
  Opens the Trace terminal UI for sources, notes, and package ownership inspection.

For deeper Trace usage, see `docs/trace.md`.
