# Command Overview

INFYNON is organized into command groups for package safety, API flows, repo memory, task management, workspaces, and coding-agent launch.

## When to use

Use this page when you need the main command map before opening a specific command page.

## Basic usage

```bash
infynon <command>
```

## Common commands

| Command | Description |
|---|---|
| `infynon pkg` | Package search, install verification, dependency scanning |
| `infynon weave` | API workflow creation, execution, assertions, and replay |
| `infynon trace` | Repo notes, context retrieval, sync, and knowledge graph |
| `infynon task` | AI task records, notes, results, lifecycle state |
| `infynon workspace` | Workspace records, folders, and agent root |
| `infynon coding` | Launch Codex, Claude, Gemini, or the workspace/task TUI |
| `infynon soul` | Show or update user-level agent context |
| `infynon doctor npm` | Diagnose npm wrapper issues |

## Examples

### Scan dependencies

```bash
infynon pkg scan
```

### Run an API flow

```bash
infynon weave flow run auth-flow --format json --no-input
```

### Retrieve repo context

```bash
infynon trace retrieve --scope branch --target feature/auth
```

### Create a task

```bash
infynon task create task-review --mutate --workspace app --prompt "Review auth module."
```

## Notes

Mutating workspace and task commands require `--mutate`.

## Related docs

- [pkg](./pkg.md)
- [weave](./weave.md)
- [trace](./trace.md)
- [task](./task.md)
- [workspace](./workspace.md)
- [coding](./coding.md)
