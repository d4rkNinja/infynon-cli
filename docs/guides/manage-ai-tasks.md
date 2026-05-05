# Manage AI Tasks

Use tasks to give AI agents clear work boundaries and a durable result trail.

## Create a task

```bash
infynon task create review-auth --mutate --workspace app --agent claude --prompt "Review auth module."
```

## Track progress

```bash
infynon task list --status running
infynon task show review-auth
```

## Add notes and results

```bash
infynon task note review-auth --mutate --text "Checking middleware flow."
infynon task result review-auth --mutate --text "Found one risky condition."
```

## Complete or fail

```bash
infynon task complete review-auth --mutate --result "Review complete."
infynon task fail review-auth --mutate --reason "Blocked by missing environment."
```

## Notes

Assign tasks by passing `--agent` during `create`, `fork`, or `update`.

## Related docs

- [task command](../commands/task.md)
- [Use GCCD task format](./use-gccd-task-format.md)
