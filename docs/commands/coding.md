# coding

Use `coding` to launch coding agents or open the workspace/task TUI.

## When to use

Use this command when you want INFYNON to start Claude, Codex, or Gemini with workspace/task context.

## Basic usage

```bash
infynon coding <agent>
```

## Common commands

| Command | Description |
|---|---|
| `infynon coding tui` | Open workspace/task TUI |
| `infynon coding claude` | Launch Claude Code |
| `infynon coding codex` | Launch Codex |
| `infynon coding gemini` | Launch Gemini |
| `infynon coding claude --cwd <path>` | Launch Claude in a specific directory |
| `infynon coding codex --background true` | Launch Codex in background mode |
| `infynon coding gemini -- <args>` | Forward extra args to Gemini |

## Options

| Option | Description |
|---|---|
| `--background true|false` | Start in background or foreground mode |
| `--cwd <path>` | Override the configured agent root for this launch |
| `-- <agent args>` | Forward trailing arguments to the underlying agent CLI |

## Examples

### Launch Claude

```bash
infynon coding claude --cwd D:/Codeverse/app
```

### Open task TUI

```bash
infynon coding tui
```

## Notes

Agent CLIs must already be installed separately. If no `--cwd` is supplied, INFYNON uses the configured agent root.

## Related docs

- [Agent launch issues](../troubleshooting/agent-launch-issues.md)
- [Manage AI tasks](../guides/manage-ai-tasks.md)
