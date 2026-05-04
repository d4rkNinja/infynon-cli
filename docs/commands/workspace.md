# workspace

Use `workspace` to manage project locations, folder aliases, model slots, and the agent root.

## When to use

Use workspaces when tasks or coding agents need to start from a known project folder.

## Basic usage

```bash
infynon workspace <subcommand>
```

## Common commands

| Command | Description |
|---|---|
| `infynon workspace create <name> --mutate` | Create workspace |
| `infynon workspace list` | List workspaces |
| `infynon workspace show <name>` | Show workspace |
| `infynon workspace update <name> --mutate` | Update workspace |
| `infynon workspace add-folder <name> --mutate --folder-name <folder> --path <path>` | Add folder |
| `infynon workspace remove-folder <name> --mutate --folder-name <folder>` | Remove folder |
| `infynon workspace remove <name> --mutate` | Remove workspace |
| `infynon workspace agent-root-show` | Show agent root |
| `infynon workspace agent-root-set --mutate --path <path>` | Set agent root |

## Important options

| Option | Description |
|---|---|
| `--folder-name <name>` | Folder alias |
| `--path <absolute-dir>` | Existing absolute directory |
| `--description <text>` | Workspace description |
| `--default` | Set as default workspace |
| `--lite-model <model>` | Low-cost model slot |
| `--frontier-model <model>` | Default strong model slot |
| `--highest-frontier-model <model>` | Highest-capability model slot |
| `--super-lite-model <model>` | Fastest model slot |

## Examples

### Create workspace

```bash
infynon workspace create app --mutate --folder-name backend --path D:/Codeverse/app --default
```

### Add frontend folder

```bash
infynon workspace add-folder app --mutate --folder-name frontend --path D:/Codeverse/app/frontend
```

## Notes

Mutating commands require `--mutate`. Workspace paths must be absolute existing directories.

## Related docs

- [Workspaces](../workspaces.md)
- [Coding](./coding.md)
