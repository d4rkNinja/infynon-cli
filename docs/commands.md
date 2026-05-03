# INFYNON Command Reference

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
- `infynon workspace`
  Use this when you want user-global workspace definitions under `~/.infynon`.
- `infynon task`
  Use this when you want user-global task tracking with agent, model, prompt, and pid metadata.
- `infynon soul`
  Use this when you want to inspect or update the user-global soul profile under `~/.infynon/soul.md`.

If you use Claude Code with Trace, pair it with `code-guardian` so handoff context can be pulled before work and updated again after the task.

Internal coding-agent bootstrap details are documented in `docs/ninja-coding.md`. The hidden `coding` launcher reads templates from `src/ninja/agent-commands.json`, supports `--background true|false`, `--cwd <path>`, and appends trailing args after `--`.

## Package Intelligence

```bash
infynon pkg <subcommand>
```

Contract-focused flags available across `pkg` workflows:

- `--json`
  Emit machine-readable JSON to stdout for automation and CI.
- `--no-input`
  Disable prompts and fail when an install would otherwise require a decision.

Machine-readable and exit-code details for `pkg` and `trace` are documented in `docs/contracts.md`.

## Workspace

```bash
infynon workspace <subcommand>
```

Workspace commands store user-global metadata in `~/.infynon/ninja.yml` and `~/.infynon/workspaces/<workspace>/config.json`; output is JSON only.

```bash
infynon workspace create app --mutate --default
infynon workspace create docs --mutate --folder-name docs-site --path D:/Codeverse/docs
infynon workspace create api --mutate --folder-name server --path D:/Codeverse/api --lite-model gpt-5.4-mini --frontier-model gpt-5.5
infynon workspace add-folder docs --mutate --folder-name api --path D:/Codeverse/api
infynon workspace remove-folder docs --mutate --folder-name api
infynon workspace remove docs --mutate
infynon workspace agent-root-show
infynon workspace agent-root-set --mutate --path D:/Codeverse/infynon-agent
infynon workspace list
infynon workspace show app
infynon workspace update app --mutate --path D:/Codeverse/app --description "Primary app workspace"
```

Use these commands to manage user-global workspace metadata and folder mappings.

`agent-root-set` stores the user-global INFYNON agent root in `~/.infynon/ninja.yml` and creates or updates workspace `infynon-agent` with folder `root`. It does not change the default project workspace for new tasks. Coding launchers use the agent root by default for `infynon coding codex`, `infynon coding claude`, and `infynon coding gemini`. If it is not configured, an AI should ask the user for the absolute folder path, save it with `agent-root-set`, and only then launch an agent.

`workspace remove <name> --mutate` deletes a workspace only when no saved tasks still reference it. Remove or reassign those tasks first, then delete the workspace.

Agent root setup flow:

```bash
infynon workspace agent-root-show
infynon workspace agent-root-set --mutate --path D:/Codeverse/infynon-agent
infynon coding codex
```

When the root is missing, `infynon coding ...` returns a JSON error with the setup command instead of opening Codex, Claude, or Gemini in the wrong folder.

Validation rules:

- names and folder names must use only ASCII letters, digits, `-`, and `_`
- mutating commands require `--mutate`
- `--path` must be an existing absolute directory
- `--folder-name` and `--path` must be provided together on create/update
- workspace model slots are fixed to `lite_model`, `frontier_model`, `highest_frontier_model`, and `super_lite_model`
- each model slot carries a thinking level; default is `auto`
- thinking values allowed: `auto`, `low`, `medium`, `high`, `xhigh`
- `remove-folder` is blocked if any task in that workspace still references that folder name
- `update` requires at least one actual change flag

## Soul

```bash
infynon soul <subcommand>
```

Soul commands manage user-global profile context in `~/.infynon/soul.md`.
Soul command output is JSON only.

```bash
infynon soul show
infynon soul update --text "# Soul Profile..."
Get-Content .\profile.md | infynon soul update
infynon soul update --file D:/Codeverse/profile.md
```

`show` prints the full soul file path, content, blank status, and suggested collection structure. `update` replaces the soul profile from `--text`, `--file`, or piped stdin.

When `soul.md` is blank, `infynon coding codex|claude|gemini` appends the internal `onboarduser-prompt.md` instructions to the bootstrap system prompt so the agent collects stable user context and saves it with `infynon soul update`. When `soul.md` has content, onboarding instructions are not appended.

The soul profile is for stable global user context only:

- name
- purpose
- profession
- current projects
- skills
- goals
- communication style
- answer style
- decision preferences
- coding preferences
- global constraints

Do not store workspace-specific rules in `soul.md`. Put workspace-specific information in workspace config, project files, task notes, or task results.

## Task

```bash
infynon task <subcommand>
```

Task commands store user-global metadata in `~/.infynon/ninja.yml` and `~/.infynon/tasks/<task-id>/`.
Agent command templates live in `~/.infynon/agent-commands.json`.
Task command output is JSON only.

When creating a task for a workspace, pass the workspace name with `--workspace <workspace-name>`. If a folder is required, first verify it exists in the workspace config and pass `--folder-name <folder-name>`.

```bash
infynon task create 550e8400-e29b-41d4-a716-446655440000 --mutate --workspace app --agent ninja --model gpt-5.5 --prompt "Ship the API patch" --folder-name backend
infynon task create 550e8400-e29b-41d4-a716-446655440004 --mutate --workspace app --agent codex --model gpt-5.5 --prompt "Review the API patch" --folder-name backend --status running
infynon task create 550e8400-e29b-41d4-a716-446655440001 --mutate --workspace app --model gpt-5.5 --thinking high --prompt "Ship the API patch" --result "initial notes"
infynon task create 550e8400-e29b-41d4-a716-446655440002 --mutate --workspace docs --command "pnpm docs:build" --status queued
infynon task create 550e8400-e29b-41d4-a716-446655440003 --mutate --workspace docs --blocked-by 550e8400-e29b-41d4-a716-446655440002 --blocked-reason "Waiting for API worker"
infynon task list
infynon task list --workspace app
infynon task list --status running
infynon task list --agent planner
infynon task show 550e8400-e29b-41d4-a716-446655440000
infynon task update 550e8400-e29b-41d4-a716-446655440000 --mutate --model gpt-5.4 --notes "waiting on review"
infynon task update 550e8400-e29b-41d4-a716-446655440000 --mutate --thinking medium --result "updated summary"
infynon task update 550e8400-e29b-41d4-a716-446655440000 --mutate --blocked-by 550e8400-e29b-41d4-a716-446655440002 --blocked-reason "Waiting for backend output"
infynon task note 550e8400-e29b-41d4-a716-446655440000 --mutate --text "handoff ready for worker"
infynon task result 550e8400-e29b-41d4-a716-446655440000 --mutate --text "context packaged"
infynon task fork 550e8400-e29b-41d4-a716-446655440010 --from 550e8400-e29b-41d4-a716-446655440000 --mutate --agent worker-ui --status queued --prompt "Ship the UI slice"
infynon task fork 550e8400-e29b-41d4-a716-446655440011 --from 550e8400-e29b-41d4-a716-446655440000 --mutate --blocked-by 550e8400-e29b-41d4-a716-446655440002 --blocked-reason "Waiting for backend output"
infynon task start 550e8400-e29b-41d4-a716-446655440000 --mutate --pid 4242
infynon task resume 550e8400-e29b-41d4-a716-446655440000 --mutate --session-id abc123 --prompt "Continue with the next failing test"
infynon task complete 550e8400-e29b-41d4-a716-446655440000 --mutate --result "merged to main"
infynon task complete 550e8400-e29b-41d4-a716-446655440000 --mutate --result "merged to main" --close-terminal
infynon task complete 550e8400-e29b-41d4-a716-446655440000 --mutate --result "merged to main" --keep-terminal
infynon task fail 550e8400-e29b-41d4-a716-446655440000 --mutate --reason "review blocked by missing dependency"
infynon task kill 550e8400-e29b-41d4-a716-446655440000 --mutate --pid 4242 --reason "stuck process" --force
infynon task remove 550e8400-e29b-41d4-a716-446655440002 --mutate
```

What each one is for:

- `create`
  Creates a task record plus a markdown tracker file named like `<task-id>-<workspace>-<folder>.md`. If `--agent` is `codex`, `claude`, or `gemini`, INFYNON also checks `~/.infynon/agent-commands.json` and runs the configured `task.create` command when it is non-empty. If the task is created with `--status running`, or if a Codex/Claude/Gemini task would otherwise be created as draft, INFYNON immediately writes the task-start prompt and runs the configured `task.start` hook, so assignment and launch happen in one command. Use `--status queued` or blocked fields when you want to create without launching.
- `list`
  Filters task summaries by workspace, status, or agent.
- `show`
  Prints the full JSON definition for one task.
- `update`
  Changes saved metadata, rewrites the markdown tracker file, and runs the configured `task.update` template command when available.
- `note`
  Appends a handoff or coordination note to the markdown tracker and runs the configured `task.note` template command when available.
- `result`
  Appends a result update to the markdown tracker and runs the configured `task.result` template command when available.
- `fork`, `start`, `resume`, `complete`, `fail`, `kill`, and `remove`
  Manage task lineage, running state, same-session follow-up, successful completion, failed completion, process termination, and deletion.

Each task directory contains:

- `task.json`
  Machine-readable task state.
- `<task-id>-<workspace>-<folder>.md`
  Fixed-format tracking file for AI-readable task context.

The agent template file has `codex`, `claude`, and `gemini` sections. Each section can define `open`, `bootstrap`, and task hooks for `create`, `start`, `note`, `update`, `result`, `complete`, `fail`, `kill`, and `remove`. If a task hook is an empty string, INFYNON uses the built-in task state update and reports `mode: "built_in"` instead of shelling out.

Supported placeholders inside agent command templates:

- `{task_id}`
- `{task_full_name}`
- `{workspace}`
- `{folder_name}`
- `{agent}`
- `{model}`
- `{thinking}`
- `{status}`
- `{prompt}`
- `{session_id}`
- `{quoted_prompt}`
- `{quoted_session_id}`
- `{task_json_path}`
- `{task_markdown_path}`
- `{task_start_system_prompt_path}`
- `{task_start_system_prompt}`
- `{task_command_guide}`
- `{task_lifecycle_guide}`
- `{task_working_directory}`

When `task.start` runs, INFYNON writes a task-specific prompt to `~/.infynon/ninja/task-start-systemprompt-<task-id>.md`. Agent templates can pass `{task_start_system_prompt_path}` to Codex, Claude, Gemini, or another runner. INFYNON resolves the task workspace folder, opens a new terminal in that folder, records the opened terminal pid when available, and starts the agent command there. If the task has no explicit model, INFYNON assigns one from the workspace model slots based on thinking level. The prompt tells the started agent to use the current task id, inspect code/config/tests from the working directory, update notes/results, complete the task with a non-empty result, verify terminal status, and close the terminal/session. Codex task starts pass the rendered task prompt as the initial interactive prompt as well as using `model_instructions_file`, so Codex starts work instead of opening an idle TUI.

`task.complete` and `task.fail` close the recorded task terminal by default when the task has a PID. Pass `--keep-terminal` only when the terminal must stay open for inspection.

Current Codex, Claude, and Gemini model names and one-line capability guidance are maintained in `docs/ninja-coding.md#current-agent-model-guide`.

If an agent task hook fails, INFYNON returns command guidance in the error so an AI can recover with commands like:

```bash
infynon task show <task-id>
infynon task note <task-id> --mutate --text "note"
infynon task result <task-id> --mutate --text "result"
infynon task complete <task-id> --mutate --result "final result"
infynon task fail <task-id> --mutate --reason "reason"
```

Template commands run through:

- Windows: `powershell -NoProfile -Command`
- macOS and Linux: `sh -lc`

Hidden coding launches are separate from task hooks:

```bash
infynon coding tui
infynon coding codex --background false --cwd D:/Codeverse/app -- --model gpt-5.5
infynon coding claude --background true --cwd D:/Codeverse/app -- --verbose
infynon coding gemini --background false -- --debug
```

`infynon coding tui` opens an isolated workspace/task management TUI. It uses form actions to run the existing workspace/task commands internally and renders human-readable results instead of JSON. Use `Tab` to switch lists, `n` to create, `u` to update, `g` to set agent root, workspace `a`/`x`/`d` to add folder/remove folder/delete workspace, and task `s`/`m`/`o`/`p`/`c`/`f`/`k`/`d` to start/resume/note/result/complete/fail/kill/remove.

`--background false` opens a new terminal with the selected interactive bootstrap command and best-effort closes the original shell process that ran `infynon coding <agent>`. `--background true` starts the selected non-interactive bootstrap command without opening a terminal. The launcher never hardcodes Codex, Claude, or Gemini startup strings; it renders templates from `src/ninja/agent-commands.json` and appends trailing args.

The original-shell close behavior is intentionally narrow:

- Applies only to foreground `infynon coding codex|claude|gemini`.
- Does not apply to `infynon coding tui`.
- Does not apply to background launches.
- Does not apply to task hook launches such as `infynon task start` or `infynon task resume`.
- Runs after the new agent terminal is opened and is best-effort across Windows, macOS, and Linux.

The markdown file stores:

- task id
- task full name
- parent task id
- blocked by task id
- blocked reason
- task name
- workspace name
- folder name
- model name
- thinking power
- task status
- start time
- end time
- pid
- command
- task description
- task notes
- task results

Validation rules:

- task ids must always be valid UUIDv4 values
- workspace names and folder names must use only ASCII letters, digits, `-`, and `_`
- mutating commands require `--mutate`
- `--thinking` must be one of: `auto`, `low`, `medium`, `high`, `xhigh`
- `--blocked-by` and `--blocked-reason` must be provided together
- `--blocked-by` must point to an existing task id and cannot point to the current task id
- `--pid` must be greater than zero
- started agent tasks should have a recorded pid when INFYNON opens the terminal
- `complete` requires a non-empty final result, either already stored or passed with `--result`
- `update` requires at least one actual change flag
- `kill` requires an explicit or previously recorded pid
- `fork` target id must differ from the source task id
- if a workspace is selected, the task folder must exist in that workspace config
- terminal tasks (`completed`, `failed`, `killed`) cannot be started, completed again, or killed again

### `scan`

Use `scan` when you want to inspect the current repo's dependency files for known package risk.

Typical cases:

- scan the current repo before a commit
- scan a specific lockfile or manifest
- generate a report for sharing
- optionally start remediation immediately

```bash
infynon pkg scan
infynon pkg scan --json
infynon pkg scan --output markdown
infynon pkg scan --output pdf
infynon pkg scan --output both
infynon pkg scan --pkg-file <PATH>
infynon pkg scan --fix
infynon pkg scan --fix high
```

Command behavior:

- `infynon pkg scan`
  Auto-detects supported package files in the current repo and scans them.
- `infynon pkg scan --pkg-file <PATH>`
  Forces the scan to use a specific file instead of auto-detection.
- `infynon pkg scan --json`
  Emits versioned machine-readable output to stdout for CI, agents, and automation.
- `infynon pkg scan --output markdown|pdf|both`
  Saves a shareable report file for humans when you need an artifact in addition to terminal output.
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
infynon pkg npm install <pkg> --strict high --no-input
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
- use `--no-input` in CI so installs fail instead of prompting
- use `--json` for machine-readable results
- use `--skip-vulnerable`, `--auto-fix`, or `--yes` only when you want an explicit non-interactive install policy

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
infynon pkg explain <package>
infynon pkg explain requests --ecosystem pip
infynon pkg explain tokio --pkg-file Cargo.lock
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
- `infynon pkg explain <package>`
  Show direct/transitive status, advisory context, and a remediation plan for one installed package.
  Typical examples: `infynon pkg explain serde_json`, `infynon pkg explain requests --ecosystem pip`, `infynon pkg explain tokio --pkg-file Cargo.lock`.
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
  Supports npm/yarn/pnpm/bun, pip/uv/poetry, cargo, go, gem, composer, nuget, hex, and pub.
  PyPI-backed search currently uses exact package lookup when the registry search endpoint is unavailable.
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
infynon weave flow run <flow-id> --format json --no-input
infynon weave flow run-all
infynon weave flow run-all --format junit --no-input
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
- `flow run --format json|markdown|junit`
  Emits structured stdout for CI, reports, or agent consumption.
- `flow run --no-input`
  Disables runtime prompts and fails with an explicit exit code when required input is missing.
  Exit codes: `0` pass, `20` flow failed, `21` input missing in non-interactive mode, `22` invalid flow definition.
- `flow run-all`
  Executes every flow in the current project.
  Exit codes: `0` all passed, `20` at least one flow failed, `21` input missing, `22` invalid flow definition.
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
infynon trace init
infynon trace source add-redis team-redis --url redis://localhost:6379/0 --namespace infynon --user alien --default
infynon trace source list
infynon trace source default local-sqlite
infynon trace source remove team-redis
```

What each one is for:

- `overview`
  Prints the current Trace state summary for the repo.
- `init`
  Creates the local Trace configuration for the repo and prepares a default local SQLite source at `.infynon/trace/trace.db`.
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
infynon trace retrieve --scope package --target chrono --format json
infynon trace retrieve --scope branch --target auth --format markdown --limit 5
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

### Knowledge Graph

```bash
infynon trace graph build
infynon trace graph build --branch feature/auth
infynon trace graph entity add <name> --kind <kind> [--branch <branch>] [--meta key=val,key=val]
infynon trace graph entity list [--kind <kind>] [--branch <branch>]
infynon trace graph entity remove <id>
infynon trace graph edge add --from <entity> --to <entity> --relation <type> [--weight 0.9] [--evidence "..."]
infynon trace graph edge list [--relation <type>] [--branch <branch>]
infynon trace graph edge remove <id>
infynon trace graph show [--branch <branch>] [--kind <kind>]
infynon trace graph path <from> <to> [--branch <branch>]
infynon trace graph impact <entity> [--branch <branch>]
infynon trace graph orphans [--branch <branch>]
infynon trace graph diff <branch-a> <branch-b>
infynon trace graph export --format json|dot [--branch <branch>] [-o file]
infynon trace graph import <file> [--branch <branch>]
infynon trace graph tui [--branch <branch>]
```

Graph commands build, inspect, connect, diff, import, export, and browse Trace entities and relationships. Use `graph tui` for interactive inspection and `graph export --format json|dot` for artifacts.

For deeper Trace usage, see `docs/trace.md`.
