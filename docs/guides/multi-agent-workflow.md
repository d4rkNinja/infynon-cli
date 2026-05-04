# Multi-Agent Workflow

Use INFYNON as a coordination layer when multiple agents or humans are working on related tasks.

## Recommended model

- Main agent owns the parent task
- Child agents handle focused subtasks
- Every subtask has a real task record
- Notes and results are added back into INFYNON
- Main agent reviews before completion

## Example

```bash
infynon task create parent-task --mutate --workspace app --prompt "Review and fix auth risk."
infynon task fork backend-task --from parent-task --mutate --agent codex --folder-name backend --prompt "Implement backend fix only."
infynon task fork review-task --from parent-task --mutate --agent gemini --folder-name backend --status queued --prompt "Review backend-task after it finishes."
infynon task complete backend-task --mutate --result "Backend fix completed."
infynon task start review-task --mutate
```

## Notes

Avoid assigning two agents to edit the same files unless a human is coordinating the merge.

## Related docs

- [task command](../commands/task.md)
- [workspace command](../commands/workspace.md)
- [coding command](../commands/coding.md)
