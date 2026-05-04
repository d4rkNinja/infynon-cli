# Agent Launch Issues

Use this page when `infynon coding claude`, `infynon coding codex`, or `infynon coding gemini` does not launch correctly.

## Check agent CLI

Make sure the underlying agent CLI is installed:

```bash
claude --version
codex --version
gemini --version
```

## Check agent root

```bash
infynon workspace agent-root-show
```

Set it if missing:

```bash
infynon workspace agent-root-set --mutate --path D:/Codeverse/infynon-agent
```

## Launch with explicit directory

```bash
infynon coding claude --cwd D:/Codeverse/app
infynon coding codex --cwd D:/Codeverse/app
infynon coding gemini --cwd D:/Codeverse/app
```

## Notes

INFYNON launches agent CLIs. It does not install Claude, Codex, or Gemini for you.

## Related docs

- [coding command](../commands/coding.md)
- [workspace command](../commands/workspace.md)
