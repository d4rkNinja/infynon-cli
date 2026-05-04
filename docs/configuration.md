# Configuration

INFYNON stores user-level configuration under `~/.infynon/` and project-level data under `.infynon/` when a command needs project state.

## Workspace setup

Create a workspace:

```bash
infynon workspace create app --mutate --folder-name backend --path D:/Codeverse/app --default
```

Add another folder:

```bash
infynon workspace add-folder app --mutate --folder-name frontend --path D:/Codeverse/app/frontend
```

## Agent root

The agent root is the default folder for direct coding-agent launches.

```bash
infynon workspace agent-root-show
infynon workspace agent-root-set --mutate --path D:/Codeverse/infynon-agent
```

## Weave environment

API flow variables are stored per project.

```bash
infynon weave env set BASE_URL http://localhost:8000
infynon weave env list
```

## Trace setup

Initialize Trace for a repo:

```bash
infynon trace init
```

Add a shared backend only when you need one:

```bash
infynon trace source add-redis team-redis --url redis://localhost:6379/0 --default
infynon trace source add-sql team-db --engine postgres --url postgres://user:pass@host:5432/db --default
```

## Related docs

- [Config reference](./reference/config.md)
- [Generated files](./reference/generated-files.md)
- [Environment variables](./reference/environment-variables.md)
