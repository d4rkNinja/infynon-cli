# INFYNON Command Guide

INFYNON is organized as a terminal control plane for agentic engineering. `workspace` and `task` coordinate coding-agent work; `pkg`, `weave`, and `trace` give that work package safety, API validation, and durable repo memory.

Use the built-in help for the exact command surface available in your installed version:

```bash
infynon --help
infynon workspace --help
infynon task --help
infynon pkg --help
infynon weave --help
infynon trace --help
```

## Agent Control Plane: `infynon workspace`, `infynon task`, `infynon coding`

Use the control plane when one lead developer or agent needs to coordinate child coding agents such as Codex, Claude Code, or Gemini CLI.

The pieces fit together like this:

| Command | Role |
|---|---|
| `infynon workspace` | Defines the workspace, folders, model slots, and saved agent root. |
| `infynon task` | Stores each assignment as a GCCD task with agent, status, notes, result, and optional pid/session metadata. |
| `infynon coding` | Launches Codex, Claude Code, Gemini CLI, or the workspace/task TUI from the selected working directory. |

Common entry points:

```bash
infynon workspace agent-root-show
infynon workspace agent-root-set --mutate --path D:/Codeverse/infynon-agent
infynon workspace create app --mutate --folder-name backend --path D:/Codeverse/app --default

infynon task create task_backend_review \
  --mutate \
  --workspace app \
  --folder-name backend \
  --agent claude \
  --prompt "Review the auth middleware change. Do not edit frontend files. Done when findings are recorded."

infynon coding tui
```

Use this area when:

- a parent agent needs to split work into child-agent tasks
- Codex, Claude Code, or Gemini CLI must start in a known workspace folder
- task status, notes, result, pid, and session metadata need to survive beyond a terminal tab
- package checks, API flow runs, and trace notes should become completion evidence

## Package Intelligence: `infynon pkg`

Use `infynon pkg` when you need to understand dependency risk, inspect installed packages, or run package-related security workflows.

Common entry points:

```bash
infynon pkg scan
infynon pkg audit
infynon pkg explain <package>
infynon pkg outdated
infynon pkg diff
infynon pkg doctor
```

Typical workflow:

```bash
infynon pkg scan
infynon pkg audit
infynon pkg explain serde_json
```

Use this area when:

- you are reviewing a dependency change
- you need a local package risk report
- you want to understand why a package is present
- you want package security checks before continuing with development

## API Flow Testing: `infynon weave`

Use `infynon weave` when API validation depends on multiple connected requests rather than one isolated request.

Common entry points:

```bash
infynon weave env set BASE_URL http://localhost:8001
infynon weave node create
infynon weave flow create checkout
infynon weave flow run checkout
```

Typical workflow:

```bash
infynon weave env set BASE_URL http://localhost:8001
infynon weave flow run checkout --format json --no-input
```

Use this area when:

- a request depends on a previous login, setup, or resource creation step
- you need repeatable API validation from a terminal
- you want flow output suitable for automation
- you need request context threaded across steps

## Repository Memory: `infynon trace`

Use `infynon trace` when repository context needs to survive beyond chat messages, local notes, or one-off handoffs.

Common entry points:

```bash
infynon trace init
infynon trace note add repo-handoff --title "Auth changed" --body "Refresh moved into middleware"
infynon trace retrieve --scope branch --target main
infynon trace sync --direction both
infynon trace tui
```

Use this area when:

- you need structured handoff notes
- package ownership or branch context matters
- you want context available for later AI-assisted work
- you need a terminal interface for inspecting repo memory

## Agent Task Contracts: `infynon task`

Use `infynon task` when AI work needs a clear execution contract instead of a loose prompt. INFYNON uses GCCD to keep task instructions bounded and reviewable:

- Goal: the outcome the task must produce
- Context: the project, files, APIs, or decisions the agent needs to know
- Constraints: boundaries the agent must respect
- Done When: the checks that prove the task is complete

Common entry points:

```bash
infynon task --help
infynon task create --help
infynon task create task_001 --mutate --workspace . --prompt "Build the settings API patch"
```

Example GCCD brief:

```text
Goal:
Build a settings page for business profile management.

Context:
- Workspace: ./apps/web
- Existing business profile API already exists.
- Fields: name, timezone, currency, and theme.

Constraints:
- Use existing UI components.
- Do not change auth logic.
- Do not modify unrelated routes.

Done When:
- Settings page renders correctly.
- User can update allowed fields.
- Validation errors are shown.
- Build and typecheck pass.
```

Use this area when:

- an agent task needs strict boundaries
- parent and child work need separate contracts
- task retry, review, or handoff needs durable instructions
- completion should be checked against explicit criteria

## Suggested Team Workflows

### Before merging dependency changes

```bash
infynon pkg scan
infynon pkg audit
```

### Before shipping API changes

```bash
infynon weave flow run checkout --format json --no-input
```

### Before handing off a branch

```bash
infynon trace note add repo-handoff --title "Branch handoff" --body "Summarize the important implementation context here"
infynon trace sync --direction both
```

### Before launching an agent task

```bash
infynon task create task_001 --mutate --workspace ./apps/web --agent codex --prompt "Build the settings page using existing UI components. Do not modify auth logic. Typecheck must pass."
```

## Exit Codes and Automation

Treat INFYNON like a normal CLI in automation:

- inspect command-specific help before wiring a command into CI
- pin the INFYNON version for reproducible environments
- store generated reports as CI artifacts when needed
- fail builds only on command modes your team has agreed to enforce

## More Help

Run:

```bash
infynon <command> --help
```

for the exact flags supported by the installed version.
