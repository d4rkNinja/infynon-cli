# task

Use `task` to manage AI coding tasks.

## When to use

Use tasks when you want AI agents or humans to work with clear goals, constraints, notes, results, and lifecycle state.

## Basic usage

```bash
infynon task <subcommand>
```

## Common commands

| Command | Description |
|---|---|
| `infynon task create <id> --mutate` | Create a new task |
| `infynon task list` | List tasks |
| `infynon task show <task-id>` | Show task details |
| `infynon task update <task-id> --mutate` | Update task metadata |
| `infynon task note <task-id> --mutate --text <text>` | Add task note |
| `infynon task result <task-id> --mutate --text <text>` | Add task result |
| `infynon task fork <new-id> --from <task-id> --mutate` | Create a child task |
| `infynon task start <task-id> --mutate` | Mark task running and start agent hook |
| `infynon task resume <task-id> --mutate` | Resume saved agent session |
| `infynon task complete <task-id> --mutate --result <text>` | Mark task complete |
| `infynon task fail <task-id> --mutate --reason <text>` | Mark task failed |
| `infynon task kill <task-id> --mutate --pid <pid>` | Kill task process |
| `infynon task remove <task-id> --mutate` | Remove task |

## Important options

| Option | Description |
|---|---|
| `--workspace <name>` | Workspace for the task |
| `--folder-name <name>` | Folder alias inside the workspace |
| `--agent <agent>` | Agent label, such as `claude`, `codex`, or `gemini` |
| `--model <model>` | Model name |
| `--thinking auto|low|medium|high|xhigh` | Thinking level |
| `--prompt <text>` | Task prompt |
| `--session-id <id>` | Agent session id |
| `--blocked-by <task-id>` | Blocking task id |
| `--blocked-reason <text>` | Blocking reason |
| `--status <status>` | Task status |

## Examples

### Create and assign task

```bash
infynon task create review-auth --mutate --workspace app --agent claude --prompt "Review auth module."
```

### Create queued task

```bash
infynon task create review-ui --mutate --workspace app --agent gemini --status queued --prompt "Review UI after backend fix."
```

### Add result

```bash
infynon task result review-auth --mutate --text "Found one risky middleware path."
```

## Recommended task format

Use GCCD:

```md
Goal:
Constraint:
Context:
Done When:
```

## Notes

There is no separate `assign` command. Assign a task by passing `--agent` during `create`, `fork`, or `update`.

## Related docs

- [Manage AI tasks](../guides/manage-ai-tasks.md)
- [Use GCCD task format](../guides/use-gccd-task-format.md)
- [Multi-agent workflow](../guides/multi-agent-workflow.md)
