# Use GCCD Task Format

GCCD is a compact task format for AI coding work.

GCCD means:

- Goal
- Constraint
- Context
- Done When

## Template

```md
Goal:
What needs to be done.

Constraint:
Rules, limits, files to avoid, style, security boundaries.

Context:
Current project state, related files, existing behavior.

Done When:
Clear completion condition.
```

## Example

```md
Goal:
Review the authentication module and find potential bugs.

Constraint:
Do not rewrite the full module. Only report confirmed issues and minimal safe fixes.

Context:
This is a NestJS backend using JWT auth and role-based access control.

Done When:
A result note is added with confirmed bugs, risk level, and patch suggestions.
```

## Related docs

- [task command](../commands/task.md)
- [Manage AI tasks](./manage-ai-tasks.md)
