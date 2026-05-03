# GCCD Task Contracts

GCCD is the task contract format used by INFYNON for agent-oriented work.

```text
G = Goal
C = Context
C = Constraints
D = Done When
```

A normal prompt says what someone wants. A GCCD task also records what matters around the work: the expected outcome, the boundaries, the relevant project context, and the checks that define completion.

## Why It Exists

AI-assisted development fails most often when the task boundary is weak. A prompt like `Build settings page` leaves important questions open:

- which app or package should change
- which existing components should be reused
- which files or systems are out of scope
- which behavior proves the work is finished
- which checks must pass before the task can be called complete

GCCD makes that boundary explicit. INFYNON tasks are not just prompts. They are structured execution contracts.

## Recommended Shape

```json
{
  "id": "task_001",
  "title": "Build business settings page",
  "goal": "Build a settings page for business profile management.",
  "constraints": [
    "Use existing UI components.",
    "Do not change auth logic.",
    "Do not modify unrelated routes.",
    "Keep TypeScript errors at zero."
  ],
  "context": [
    "Workspace: ./apps/web",
    "API path: ./apps/api",
    "Existing business profile API already exists.",
    "Fields: name, timezone, currency, theme."
  ],
  "done_when": [
    "Settings page renders correctly.",
    "User can update allowed fields.",
    "Validation errors are shown.",
    "Build and typecheck pass."
  ],
  "agent": "codex",
  "workspace": "./apps/web",
  "status": "pending"
}
```

The short title is still useful, but it is not enough by itself. The title names the work. GCCD defines the execution contract.

## Creating a Task

Use `infynon task create` to create a task brief. The exact flags available can vary by release, so check the installed command help first:

```bash
infynon task --help
infynon task create --help
```

Example:

```bash
infynon task create task_001 \
  --mutate \
  --workspace ./apps/web \
  --agent codex \
  --prompt "Build a settings page for business profile management. Use existing UI components. Do not change auth logic. Build and typecheck must pass."
```

INFYNON can normalize plain task text into a structured GCCD brief before the agent receives it.

## Agent Prompt Shape

When INFYNON launches an agent, the task contract should be passed clearly:

```text
You are working on an INFYNON task.

Task ID: task_123
Agent: codex
Workspace: ./apps/web

GOAL:
Create the frontend settings screen.

CONTEXT:
- Parent task is building the full business settings flow.
- API already exists at /business/profile.
- Existing settings components are in src/components/settings.

CONSTRAINTS:
- Use existing UI components only.
- Do not touch backend files.
- Do not modify auth logic.

DONE WHEN:
- Settings page renders correctly.
- User can edit allowed fields.
- Typecheck passes.
- Result is recorded with INFYNON task output.
```

## Parent and Child Work

For a parent task, GCCD should be strongly recommended. The parent task may start from a broader product request and become more specific during planning.

For a child task, GCCD should be required. A child agent needs stricter boundaries because it is usually responsible for one slice of the work and should not modify unrelated parts of the repository.

Recommended child task requirements:

- goal is required
- context is required
- constraints are required
- done_when is required
- parent task id is required
- agent is required
- workspace is required

## Validation Rules

Before a task runs, INFYNON should be able to reject weak task definitions.

Recommended validation for normal tasks:

- goal is required
- done_when is required
- workspace is required
- at least one constraint is recommended
- context is recommended

Recommended validation for child tasks:

- goal is required
- constraints are required
- context is required
- done_when is required
- parent task id is required
- agent is required
- workspace is required

## Practical Standard

A task is ready for execution when another agent can read it and answer four questions without guessing:

- What outcome should exist?
- What information matters?
- What must not be broken or changed?
- How will completion be verified?

That is the standard GCCD enforces.
