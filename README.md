# INFYNON CLI

[![npm version](https://img.shields.io/npm/v/infynon?style=flat-square&logo=npm)](https://www.npmjs.com/package/infynon)
[![GitHub release](https://img.shields.io/github/v/release/d4rkNinja/infynon-cli?style=flat-square&logo=github)](https://github.com/d4rkNinja/infynon-cli/releases)
[![MIT License](https://img.shields.io/badge/license-MIT-0f172a?style=flat-square)](LICENSE)
[![Docs](https://img.shields.io/badge/docs-cli.infynon.com-14b8a6?style=flat-square)](https://cli.infynon.com/docs)
[![Agent control plane](https://img.shields.io/badge/agent%20control%20plane-Codex%20%7C%20Claude%20%7C%20Gemini-7c3aed?style=flat-square)](docs/agent-control-plane.md)
[![Package security](https://img.shields.io/badge/pkg-secure%20installs-ef4444?style=flat-square)](#package-safety-pkg)
[![API flows](https://img.shields.io/badge/weave-API%20flows-0ea5e9?style=flat-square)](#api-workflow-testing-weave)
[![Repo memory](https://img.shields.io/badge/trace-repo%20memory-10b981?style=flat-square)](#repo-memory-trace)
[![npm downloads](https://img.shields.io/badge/dynamic/json?style=flat-square&logo=npm&label=npm%20downloads&query=%24.downloads&url=https%3A%2F%2Fapi.npmjs.org%2Fdownloads%2Fpoint%2F2025-05-04%3A2050-12-31%2Finfynon&cacheSeconds=3600)](https://www.npmjs.com/package/infynon)

Security-first CLI for AI-driven development.

INFYNON helps developers verify packages before install, trace repo context, test API flows, and manage AI coding tasks with more control.

> Install. Verify. Trace. Orchestrate.

---

## Why INFYNON?

AI coding tools can generate code fast, but they can also install unknown packages, miss dependency risk, lose context, or create messy task flows.

INFYNON gives developers a control layer for modern AI-assisted development:

- Check package risk before installation
- Audit dependency files and lockfiles
- Trace repo decisions and execution context
- Build and replay API workflows
- Manage AI-agent coding tasks across workspaces
- Coordinate Claude, Codex, Gemini, or other agents through task state

---

## Installation

```bash
npm i -g infynon
```

Verify installation:

```bash
infynon --version
```

Other install paths:

```bash
go install github.com/d4rkNinja/infynon-cli/go/cmd/infynon@latest
curl -fsSL https://raw.githubusercontent.com/d4rkNinja/infynon-cli/main/install.sh | bash
```

Windows:

```powershell
irm https://raw.githubusercontent.com/d4rkNinja/infynon-cli/main/install.ps1 | iex
```

---

## Core Commands

INFYNON is built around a few major command groups:

| Command | Purpose |
|---|---|
| `pkg` | Package search, install verification, audits, dependency safety |
| `trace` | Repo memory, context, file/package/PR/branch tracking |
| `weave` | API workflow testing, replay, chained requests |
| `task` | AI task management and agent coordination |
| `workspace` | Workspace setup, routing, and agent root configuration |
| `coding` | Launch coding agents like Claude, Codex, Gemini |
| `soul` | Store stable user-level agent context |
| `doctor` | Diagnose installation and wrapper issues |

### Complete command map

Use this as the quick command checklist.

```bash
infynon pkg scan [--output markdown|pdf|both] [--fix [level]] [--pkg-file <file>]
infynon pkg audit [--pkg-file <file>]
infynon pkg why <package> [--pkg-file <file>]
infynon pkg explain <package> [--ecosystem <name>] [--pkg-file <file>]
infynon pkg outdated [--pkg-file <file>]
infynon pkg diff <package> <version-a> <version-b> [--ecosystem <name>]
infynon pkg doctor [--pkg-file <file>]
infynon pkg size <package...> [--ecosystem <name>]
infynon pkg search <query> [--ecosystem <name>]
infynon pkg fix [--auto] [--pkg-file <file>]
infynon pkg clean [--pkg-file <file>]
infynon pkg migrate <from> <to>
infynon pkg eagle-eye setup|start|status|enable|disable
infynon pkg <npm|yarn|pnpm|bun|pip|uv|poetry|cargo|...> <package-manager-args>
```

```bash
infynon weave tui [flow-id]
infynon weave env list
infynon weave env set <key> <value>
infynon weave env get <key> [--reveal]
infynon weave env delete <key>
infynon weave node create [--ai <description>]
infynon weave node list
infynon weave node get <node-id>
infynon weave node clone <node-id> <new-id>
infynon weave node run <node-id> [--base-url <url>] [--set key=value] [--prompt]
infynon weave node export <node-id> [--format curl|json] [--base-url <url>]
infynon weave node remove <node-id>
infynon weave node assertion <node-id> list
infynon weave node assertion <node-id> add <check> [--on-fail stop|continue]
infynon weave node assertion <node-id> enable <index>
infynon weave node assertion <node-id> disable <index>
infynon weave node assertion <node-id> toggle <index>
infynon weave node assertion <node-id> remove <index>
infynon weave node prompt <node-id> list
infynon weave node prompt <node-id> add <var> [--label <label>] [--secret] [--default <value>] [--type text|secret|boolean|select|multiselect] [--options a,b,c]
infynon weave node prompt <node-id> remove <index>
infynon weave flow create <name> [--ai <description>]
infynon weave flow list
infynon weave flow show <flow-id>
infynon weave flow run <flow-id> [--base-url <url>] [--set key=value] [--format json|markdown|junit] [--output markdown|pdf|both] [--no-input]
infynon weave flow run-all [--base-url <url>] [--set key=value] [--format json|markdown|junit] [--output markdown|pdf|both] [--no-input]
infynon weave flow merge <flow-a> <flow-b> --join-at <node-id> [--name <name>]
infynon weave flow remove <flow-id>
infynon weave attach <from-node> <to-node> [--carry var1,var2] [--condition <expr>] [--ai]
infynon weave detach <from-node> <to-node>
infynon weave import <spec> [--flow <name>] [--base-url <url>] [--prefix <prefix>] [--dry-run]
infynon weave validate
infynon weave ai suggest --after <node-id>
infynon weave ai attach --after <node-id> [--flow <flow-id>]
infynon weave ai complete <flow-id>
infynon weave ai probe <flow-id> [--base-url <url>]
infynon weave ai build-flow --nodes <node-a,node-b> [--name <name>]
infynon weave ai explain <flow-id> [--run <index>]
infynon weave ai assert <node-id>
infynon weave ai branch <node-id>
```

```bash
infynon trace overview
infynon trace init [--repo <name>] [--owner <owner>] [--user <user>]
infynon trace source add-redis <id> --url <url> [--namespace <ns>] [--notes <text>] [--user <user>] [--default]
infynon trace source add-sql <id> --engine postgres|mysql|sqlite --url <url> [--database <db>] [--username <user>] [--password-env <env>] [--notes <text>] [--user <user>] [--default]
infynon trace source list
infynon trace source default <id>
infynon trace source remove <id>
infynon trace note add <id> --title <title> --body <body> [--layer canonical|team|user] [--scope repo|branch|pr|file|user|session|package] [--target <value>] [--author <name>] [--actor <name>] [--files a,b] [--tags a,b] [--related-pr <n>]
infynon trace note update <id> [--title <title>] [--body <body>] [--status <status>]
infynon trace note list
infynon trace note remove <id>
infynon trace retrieve [--layer <layer>] [--scope <scope>] [--target <value>] [--author <name>] [--file <path>] [--tag <tag>] [--format table|markdown|json] [--limit <n>]
infynon trace sync [--source <id>] [--direction pull|push|both]
infynon trace compact
infynon trace schema sql|redis
infynon trace tui
infynon trace graph build [--branch <branch>] [--all-branches]
infynon trace graph show [--branch <branch>] [--kind <kind>]
infynon trace graph entity add <name> --kind <kind> [--branch <branch>] [--meta key=value,key=value]
infynon trace graph entity list [--branch <branch>] [--kind <kind>]
infynon trace graph entity remove <id>
infynon trace graph edge add --from <entity> --to <entity> --relation <type> [--weight <0.0-1.0>] [--branch <branch>] [--evidence <text>]
infynon trace graph edge list [--branch <branch>] [--relation <type>]
infynon trace graph edge remove <id>
infynon trace graph path <from> <to> [--branch <branch>]
infynon trace graph impact <entity> [--branch <branch>]
infynon trace graph orphans [--branch <branch>]
infynon trace graph diff <branch-a> <branch-b>
infynon trace graph export [--format json|dot] [--branch <branch>] [-o <file>]
infynon trace graph import <file> [--format json] [--branch <branch>]
infynon trace graph tui [--branch <branch>]
```

```bash
infynon workspace create <name> --mutate [--folder-name <folder>] [--path <absolute-dir>] [--description <text>] [--default] [model flags]
infynon workspace list
infynon workspace show <name>
infynon workspace update <name> --mutate [--folder-name <folder>] [--path <absolute-dir>] [--description <text>] [--default] [model flags]
infynon workspace add-folder <name> --mutate --folder-name <folder> --path <absolute-dir>
infynon workspace remove-folder <name> --mutate --folder-name <folder>
infynon workspace remove <name> --mutate
infynon workspace agent-root-show
infynon workspace agent-root-set --mutate --path <absolute-dir>
```

```bash
infynon task create <id> --mutate [--workspace <name>] [--folder-name <folder>] [--agent <agent>] [--model <model>] [--thinking auto|low|medium|high|xhigh] [--prompt <text>] [--command <cmd>] [--pid <pid>] [--session-id <id>] [--notes <text>] [--result <text>] [--blocked-by <task-id>] [--blocked-reason <text>] [--status <status>]
infynon task list [--workspace <name>] [--status <status>] [--agent <agent>]
infynon task show <id>
infynon task update <id> --mutate [task metadata flags]
infynon task note <id> --mutate --text <text>
infynon task result <id> --mutate --text <text>
infynon task fork <new-id> --from <task-id> --mutate [task metadata flags]
infynon task start <id> --mutate [--pid <pid>] [--session-id <id>]
infynon task resume <id> --mutate [--session-id <id>] [--prompt <text>]
infynon task complete <id> --mutate [--notes <text>] [--result <text>] [--close-terminal] [--keep-terminal]
infynon task fail <id> --mutate [--reason <text>] [--result <text>] [--close-terminal] [--keep-terminal]
infynon task kill <id> --mutate [--pid <pid>] [--reason <text>] [--force]
infynon task remove <id> --mutate
```

```bash
infynon coding tui
infynon coding codex [--background true|false] [--cwd <path>] [-- <agent args>]
infynon coding claude [--background true|false] [--cwd <path>] [-- <agent args>]
infynon coding gemini [--background true|false] [--cwd <path>] [-- <agent args>]
infynon soul show
infynon soul update [--text <text>] [--file <path>]
infynon doctor npm
```

---

## Package Safety: `pkg`

Use `pkg` to search, verify, audit, and manage dependencies safely.

### Search packages

```bash
infynon pkg search react
infynon pkg search fastapi --ecosystem pypi
```

Search works across supported ecosystems and helps compare packages using stronger trust signals.

### Install with verification

```bash
infynon pkg npm install axios
infynon pkg uv add fastapi
infynon pkg cargo add serde
```

INFYNON checks package risk before execution, not after.

### Scan dependency files

```bash
infynon pkg scan
infynon pkg scan --pkg-file package-lock.json
infynon pkg scan --pkg-file uv.lock
infynon pkg scan --json
```

### Auto-fix vulnerable packages

```bash
infynon pkg scan --fix
infynon pkg scan --fix high
infynon pkg fix --auto
```

### Other useful package commands

```bash
infynon pkg audit
infynon pkg outdated
infynon pkg why <package>
infynon pkg explain <package>
infynon pkg diff <package> <version-a> <version-b>
infynon pkg size <package>
infynon pkg doctor
infynon pkg clean
infynon pkg migrate <from> <to>
infynon pkg eagle-eye setup
```

Global package flags:

```bash
--strict [level]
--pkg-file <path>
--json
--no-input
--yes
--skip-vulnerable
--auto-fix
```

---

## API Workflow Testing: `weave`

Use `weave` to build, run, save, and replay API flows from the terminal.

Good for testing real backend flows like:

- Login
- Fetch profile
- Create resource
- Pass token to the next request
- Use a response ID in another request
- Replay the full flow safely in CI

Set shared environment values:

```bash
infynon weave env set BASE_URL http://localhost:8000
infynon weave env list
```

Create and inspect nodes:

```bash
infynon weave node create
infynon weave node create --ai "POST /auth/login extracts token"
infynon weave node list
infynon weave node get <node-id>
```

Run one node:

```bash
infynon weave node run <node-id>
infynon weave node run <node-id> --base-url http://localhost:8000
infynon weave node run <node-id> --set token=abc123
```

Create and run flows:

```bash
infynon weave flow create auth-flow
infynon weave flow create auth-flow --ai "login then get profile"
infynon weave flow list
infynon weave flow show <flow-id>
infynon weave flow run <flow-id>
infynon weave flow run <flow-id> --format json --no-input
infynon weave flow run-all --format junit --no-input
```

Assertions and runtime prompts:

```bash
infynon weave node assertion <node-id> add "status == 200"
infynon weave node assertion <node-id> list
infynon weave node prompt <node-id> add otp --label "OTP code" --type text
infynon weave node prompt <node-id> list
```

Security and AI helpers:

```bash
infynon weave ai probe <flow-id>
infynon weave ai explain <flow-id>
infynon weave ai build-flow --nodes login,get-profile --name auth-flow
```

Open the API flow TUI:

```bash
infynon weave tui
```

---

## Repo Memory: `trace`

Use `trace` to remember why something changed, not just what changed.

Useful for tracking:

- Packages
- Files
- Branches
- Pull requests
- Repo decisions
- AI-generated changes
- Context behind implementation

Initialize Trace:

```bash
infynon trace init
infynon trace overview
```

Create notes:

```bash
infynon trace note add auth-change --title "Auth changed" --body "Refresh logic moved into middleware."
infynon trace note add package-risk --title "Package risk" --body "Review before next release." --scope package --target serde_json
```

List, update, and remove notes:

```bash
infynon trace note list
infynon trace note update auth-change --status stale
infynon trace note remove auth-change
```

Retrieve context:

```bash
infynon trace retrieve --scope branch --target feature/auth
infynon trace retrieve --scope package --target serde_json
infynon trace retrieve --file src/auth.rs
infynon trace retrieve --format markdown --limit 5
```

Sync and inspect:

```bash
infynon trace sync --direction both
infynon trace compact
infynon trace schema sql
infynon trace schema redis
infynon trace tui
```

Knowledge graph:

```bash
infynon trace graph build
infynon trace graph show
infynon trace graph entity add src/auth.rs --kind file
infynon trace graph edge add --from src/auth.rs --to serde_json --relation depends_on
infynon trace graph path <from> <to>
infynon trace graph impact src/auth.rs
infynon trace graph diff main feature/auth
infynon trace graph export --format dot -o graph.dot
infynon trace graph tui
```

---

## AI Task Management: `task`

Use `task` to create, assign, track, block, fork, resume, and complete coding tasks.

This is useful when using multiple AI agents as subagents.

### Create task

```bash
infynon task create <task-id> --mutate --workspace <workspace> --prompt "Review the authentication module."
```

Assign the task during creation:

```bash
infynon task create <task-id> --mutate --workspace <workspace> --agent claude --prompt "Review auth code."
infynon task create <task-id> --mutate --workspace <workspace> --agent codex --prompt "Implement the focused fix."
infynon task create <task-id> --mutate --workspace <workspace> --agent gemini --prompt "Review edge cases."
```

If `--agent` is `claude`, `codex`, or `gemini`, INFYNON can start the agent task immediately. Use `--status queued` to create the task without launching it.

### List tasks

```bash
infynon task list
infynon task list --workspace <workspace>
infynon task list --status running
infynon task list --agent codex
```

### Show task details

```bash
infynon task show <task-id>
```

### Update task metadata

```bash
infynon task update <task-id> --mutate --status running
infynon task update <task-id> --mutate --model gpt-5.5 --thinking high
infynon task update <task-id> --mutate --session-id <session-id>
```

### Add task note

```bash
infynon task note <task-id> --mutate --text "Blocked on CI output."
```

Task notes can be used by subagents or the main agent to understand updates, blockers, and execution context.

### Add task result

```bash
infynon task result <task-id> --mutate --text "Found one auth edge case."
```

### Resume task

```bash
infynon task resume <task-id> --mutate --session-id <session-id> --prompt "Continue with the next failing test."
```

### Mark task complete

```bash
infynon task complete <task-id> --mutate --result "Task completed with findings recorded."
```

### Mark task failed

```bash
infynon task fail <task-id> --mutate --reason "Blocked by missing environment variables."
```

### Block task

Block during create or update:

```bash
infynon task create <task-id> --mutate --workspace <workspace> --blocked-by <other-task-id> --blocked-reason "Waiting for backend task."
infynon task update <task-id> --mutate --blocked-by <other-task-id> --blocked-reason "Waiting for backend task."
```

### Fork task

```bash
infynon task fork <child-task-id> --from <parent-task-id> --mutate --agent codex --prompt "Handle the backend slice only."
```

### TUI

INFYNON includes a TUI for workspace and task management:

```bash
infynon coding tui
```

---

## GCCD Task Format

INFYNON tasks work best with the GCCD format:

```md
Goal:
What needs to be done.

Constraint:
Rules, limits, files to avoid, style, framework, security boundaries.

Context:
Current project state, related files, existing behavior, previous notes.

Done When:
Clear completion condition.
```

Example:

```md
Goal:
Review the authentication module and find possible bugs.

Constraint:
Do not rewrite the whole module. Only suggest minimal safe fixes.

Context:
This is a NestJS backend using JWT-based auth and role permissions.

Done When:
List confirmed bugs, risky patterns, and safe patch suggestions.
```

---

## Workspace Management

INFYNON supports multiple workspaces and agent roots.

```bash
infynon workspace list
infynon workspace show <name>
infynon workspace create <name> --mutate --folder-name <folder> --path <path>
infynon workspace update <name> --mutate --description "Primary workspace"
infynon workspace add-folder <name> --mutate --folder-name <folder> --path <path>
infynon workspace remove-folder <name> --mutate --folder-name <folder>
infynon workspace remove <name> --mutate
infynon workspace agent-root-show
infynon workspace agent-root-set --mutate --path <path>
```

Example:

```bash
infynon workspace create app --mutate --folder-name backend --path D:\Codeverse\app --default
infynon workspace agent-root-set --mutate --path D:\infyn
```

Workspace model slots:

```bash
--lite-model <model>
--frontier-model <model>
--highest-frontier-model <model>
--super-lite-model <model>
```

---

## Launch Coding Agents

INFYNON can launch coding agents from the terminal.

```bash
infynon coding claude
infynon coding codex
infynon coding gemini
```

Use this with tasks to create cleaner AI-agent workflows.

Optional launch flags:

```bash
infynon coding codex --cwd D:/Codeverse/app
infynon coding claude --background true
infynon coding gemini -- --debug
```

---

## Multi-Agent Workflow

INFYNON can be used as a coordination layer for AI coding agents.

Example flow:

```bash
infynon workspace create app --mutate --folder-name backend --path D:/Codeverse/app --default
infynon task create parent-task --mutate --workspace app --prompt "Review and fix auth risk."
infynon task fork backend-task --from parent-task --mutate --agent codex --folder-name backend --prompt "Implement the backend fix only."
infynon task fork review-task --from parent-task --mutate --agent gemini --folder-name backend --status queued --prompt "Review backend-task after it finishes."
infynon task note backend-task --mutate --text "Backend worker started."
infynon task result backend-task --mutate --text "Patch completed."
infynon task complete backend-task --mutate --result "Backend fix completed."
infynon task start review-task --mutate
```

Recommended model:

- Main agent owns the parent task
- Child agents handle focused subtasks
- Every subtask has a real task record
- Notes and results are added back into INFYNON
- Main agent reviews before completion

---

## Example Use Cases

### Secure package install

```bash
infynon pkg npm install express
```

### Audit a lockfile

```bash
infynon pkg scan --pkg-file package-lock.json
```

### Test API login flow

```bash
infynon weave flow run auth-flow --format json --no-input
```

### Create code review task

```bash
infynon task create review-auth --mutate --workspace app --agent gemini --prompt "Review auth module for bugs."
```

### Launch Claude Code inside workspace

```bash
infynon coding claude --cwd D:/Codeverse/app
```

---

## Security Philosophy

INFYNON treats package installation and AI-generated execution as trust boundaries.

The goal is simple:

```txt
Do not blindly install.
Do not blindly execute.
Do not lose context.
Do not trust AI-generated dependency changes without verification.
```

INFYNON helps developers add control before damage happens.

---

## Roadmap

- Better multi-agent orchestration
- Stronger task lifecycle management
- More package ecosystem support
- Improved package trust scoring
- Better workflow replay and API security probes
- Deeper AI coding agent integrations
- Better workspace-level memory and traceability

---

## Links

- Website: [https://cli.infynon.com](https://cli.infynon.com)
- GitHub: [https://github.com/d4rkNinja/infynon-cli](https://github.com/d4rkNinja/infynon-cli)
- npm: [https://www.npmjs.com/package/infynon](https://www.npmjs.com/package/infynon)
- Docs: [https://cli.infynon.com/docs](https://cli.infynon.com/docs)
- Docs home: [docs/README.md](docs/README.md)
- Command overview: [docs/commands/overview.md](docs/commands/overview.md)
- Package safety: [docs/commands/pkg.md](docs/commands/pkg.md)
- API workflows: [docs/commands/weave.md](docs/commands/weave.md)
- Repo trace: [docs/commands/trace.md](docs/commands/trace.md)
- AI tasks: [docs/commands/task.md](docs/commands/task.md)
- Workspaces: [docs/commands/workspace.md](docs/commands/workspace.md)
- Coding agents: [docs/commands/coding.md](docs/commands/coding.md)

Install:

```bash
npm i -g infynon
```

---

## License

MIT License. See [LICENSE](LICENSE).
