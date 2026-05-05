# Workspaces

Workspaces tell INFYNON where projects live and which folder an agent should use for a task.

Use workspaces when you have one or more project folders and want tasks to start in the correct directory.

## Basic usage

```bash
infynon workspace create app --mutate --folder-name backend --path D:/Codeverse/app --default
```

## Common commands

| Command | Description |
|---|---|
| `infynon workspace create <name> --mutate` | Create a workspace |
| `infynon workspace list` | List workspaces |
| `infynon workspace show <name>` | Show workspace details |
| `infynon workspace update <name> --mutate` | Update workspace metadata |
| `infynon workspace add-folder <name> --mutate --folder-name <folder> --path <path>` | Add folder alias |
| `infynon workspace remove-folder <name> --mutate --folder-name <folder>` | Remove folder alias |
| `infynon workspace remove <name> --mutate` | Remove workspace |
| `infynon workspace agent-root-show` | Show configured agent root |
| `infynon workspace agent-root-set --mutate --path <path>` | Set default agent root |

## Notes

- Mutating commands require `--mutate`.
- Workspace paths must be absolute existing directories.
- Removing a workspace or folder is blocked if tasks still reference it.

## Related docs

- [workspace command](./commands/workspace.md)
- [Manage AI tasks](./guides/manage-ai-tasks.md)
