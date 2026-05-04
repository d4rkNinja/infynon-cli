# INFYNON Agent Control Plane

INFYNON is a terminal control plane for coordinating coding-agent work with package safety, API validation, and repository memory.

The control plane has three durable layers:

| Layer | Command | Role |
|---|---|---|
| Workspace routing | `infynon workspace` | Names the project, folders, model slots, and agent root where work happens. |
| Task contracts | `infynon task` | Stores each assignment as a GCCD brief with status, agent, model, pid, notes, result, and session metadata. |
| Agent launch | `infynon coding` | Opens Codex, Claude Code, or Gemini CLI in the right folder with the right prompt. |

`pkg`, `weave`, and `trace` are the operational layers around that control plane:

- `pkg` checks dependencies before agents build on unsafe packages.
- `weave` validates real API behavior after agents change code.
- `trace` keeps branch, file, package, and handoff context available across sessions.

## Why It Matters

Plain agent delegation usually leaves important state in chat or terminal tabs:

- what workspace the child agent used
- which files or folders were in scope
- which model and agent were assigned
- what completion criteria were agreed to
- whether the task completed, failed, or blocked
- what result the child agent produced

INFYNON keeps that state in workspace and task records so parent-agent work, child-agent work, human review, and retries stay connected.

Codex, Claude Code, and Gemini CLI have built-in launch templates today. Other coding-agent CLIs can still participate by using the same workspace and task records as their contract layer: read `task show`, work from the assigned folder, write notes/results, and complete or fail the task explicitly.

## Workspace First

Create or inspect the workspace before assigning work:

```bash
infynon workspace list
infynon workspace show app
infynon workspace create app --mutate --folder-name backend --path D:/Codeverse/app --default
infynon workspace add-folder app --mutate --folder-name frontend --path D:/Codeverse/app/frontend
```

Set the agent root once. This is the default directory for direct coding-agent launches:

```bash
infynon workspace agent-root-show
infynon workspace agent-root-set --mutate --path D:/Codeverse/infynon-agent
```

## Task Contracts

Every meaningful agent assignment should be a task.

INFYNON normalizes plain prompts into GCCD:

```text
Goal
Context
Constraints
Done When
```

Example child-agent task:

```bash
infynon task create task_backend_review \
  --mutate \
  --workspace app \
  --folder-name backend \
  --agent claude \
  --model sonnet \
  --prompt "Review the auth middleware change. Do not edit frontend files. Done when findings are recorded and the relevant tests are named."
```

When the task agent is `codex`, `claude`, or `gemini`, INFYNON treats a new draft task as a running assignment and starts the configured task launch hook immediately. Use `--status queued` when you want to create the task without launching it yet.

## Subagent Workflow

Use one parent task for the objective and child tasks for independent slices:

```bash
infynon task create task_release \
  --mutate \
  --workspace app \
  --prompt "Prepare the release. Split implementation, docs, and verification into child tasks."

infynon task fork task_release_docs \
  --from task_release \
  --mutate \
  --agent codex \
  --folder-name docs \
  --prompt "Update docs for the release. Do not edit source code."

infynon task fork task_release_review \
  --from task_release \
  --mutate \
  --agent claude \
  --folder-name backend \
  --prompt "Review the backend release changes. Record findings and do not modify files."
```

Track and coordinate:

```bash
infynon task list --status running
infynon task show task_release_docs
infynon task note task_release_docs --mutate --text "Docs worker is waiting for final release tag."
infynon task result task_release_review --mutate --text "No backend blockers found."
```

Complete or fail tasks explicitly:

```bash
infynon task complete task_release_docs --mutate --result "Docs updated and links checked."
infynon task fail task_release_review --mutate --reason "Blocked by missing CI logs."
```

## Direct Agent Launch

For direct agent bootstrapping:

```bash
infynon coding codex
infynon coding claude
infynon coding gemini
infynon coding tui
```

Foreground launches open a new terminal in the selected working directory. Background launches return process metadata for automation.

## Operational Loop

Agent orchestration without operational context is fragile. INFYNON keeps the agent work tied to the same tools that prove the work is safe and complete.

```bash
infynon pkg scan
infynon task create task_fix_deps --mutate --workspace app --agent codex --prompt "Fix dependency risk without changing runtime behavior."
infynon weave flow run checkout --format json --no-input
infynon trace note add repo-handoff --title "Dependency fix handoff" --body "Record what changed and why."
infynon task complete task_fix_deps --mutate --result "Risk fixed, checkout flow passed, handoff note recorded."
```

## Best Practices

- Give every child agent a separate task id.
- Assign one clear responsibility per child task.
- Use workspace folders to prevent agents from starting in the wrong directory.
- Avoid overlapping write areas unless a human is coordinating the merge.
- Record blockers as task notes, not just chat messages.
- Record final output with `task result` or `task complete`.
- Use `pkg`, `weave`, and `trace` as completion evidence, not as afterthoughts.

## Related Docs

- [Command Guide](commands.md)
- [GCCD Task Contracts](gccd.md)
- [Install Guide](install.md)
- [Verification Guide](verification.md)
