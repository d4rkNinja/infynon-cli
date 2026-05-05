# trace

Use `trace` to save repo context, implementation notes, and package/file/branch memory.

## When to use

Use Trace when you want to remember why a change happened, retrieve relevant repo context, or keep task handoff notes outside chat.

## Basic usage

```bash
infynon trace <subcommand>
```

## Common commands

| Command | Description |
|---|---|
| `infynon trace overview` | Show Trace overview |
| `infynon trace init` | Initialize Trace in the repo |
| `infynon trace source add-redis <id> --url <url>` | Add Redis source |
| `infynon trace source add-sql <id> --engine <engine> --url <url>` | Add SQL source |
| `infynon trace source list` | List sources |
| `infynon trace source default <id>` | Set default source |
| `infynon trace source remove <id>` | Remove source |
| `infynon trace note add <id> --title <title> --body <body>` | Add note |
| `infynon trace note update <id>` | Update note |
| `infynon trace note list` | List notes |
| `infynon trace note remove <id>` | Remove note |
| `infynon trace retrieve` | Retrieve notes by filters |
| `infynon trace sync --direction pull|push|both` | Sync Trace data |
| `infynon trace compact` | Compact stale notes |
| `infynon trace schema sql|redis` | Print backend schema |
| `infynon trace tui` | Open Trace TUI |
| `infynon trace graph build` | Build knowledge graph |
| `infynon trace graph show` | Show graph |
| `infynon trace graph entity add <name> --kind <kind>` | Add graph entity |
| `infynon trace graph entity list` | List graph entities |
| `infynon trace graph entity remove <id>` | Remove graph entity |
| `infynon trace graph edge add --from <a> --to <b> --relation <type>` | Add graph edge |
| `infynon trace graph edge list` | List graph edges |
| `infynon trace graph edge remove <id>` | Remove graph edge |
| `infynon trace graph path <from> <to>` | Find path |
| `infynon trace graph impact <entity>` | Show impact |
| `infynon trace graph orphans` | Show unconnected entities |
| `infynon trace graph diff <branch-a> <branch-b>` | Diff graphs |
| `infynon trace graph export --format json|dot` | Export graph |
| `infynon trace graph import <file>` | Import graph |
| `infynon trace graph tui` | Open graph TUI |

## Examples

### Add a note

```bash
infynon trace note add auth-change --title "Auth changed" --body "Refresh moved into middleware." --scope branch --target feature/auth
```

### Retrieve package context

```bash
infynon trace retrieve --scope package --target serde_json --format markdown
```

## Notes

Trace data is local by default. Add Redis or SQL sources only if you need shared or remote storage.

## Related docs

- [Generated files](../reference/generated-files.md)
- [Exit codes](../reference/exit-codes.md)
