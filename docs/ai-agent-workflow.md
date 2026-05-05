# AI Agent Workflow

Use INFYNON when one lead developer or agent needs durable coordination for Codex, Claude Code, Gemini CLI, or another coding-agent session.

## Basic Flow

1. Create or inspect the workspace.
2. Create a GCCD task with a goal, context, constraints, and done-when checks.
3. Assign the task to `codex`, `claude`, or `gemini`.
4. Let INFYNON launch the agent from the task workspace folder.
5. Record notes, results, and terminal status on the task.
6. Complete or fail the task explicitly.

```bash
infynon workspace create app --mutate --folder-name backend --path D:/Codeverse/app --default

infynon task create task_backend_review \
  --mutate \
  --workspace app \
  --folder-name backend \
  --agent claude \
  --prompt "Review the auth middleware change. Do not edit frontend files. Done when findings are recorded."

infynon task show task_backend_review
infynon task complete task_backend_review --mutate --result "Review finished and findings recorded."
```

## Task Launches

When a task starts, INFYNON writes a task-specific prompt file under `~/.infynon/ninja/` and launches the selected agent in the resolved workspace folder.

Claude task starts use:

```bash
claude {model_arg} --append-system-prompt-file {quoted_task_start_system_prompt_path}
```

INFYNON uses the prompt file for Claude so Windows launches do not carry the large system prompt through argv.

Gemini task starts set `GEMINI_SYSTEM_MD` to the task prompt Markdown file through `{gemini_task_system_prompt_env}` and use `--prompt-interactive {quoted_task_start_system_prompt}` only for the initial task prompt. Gemini resume uses `{gemini_task_system_prompt_env} gemini ... --resume {quoted_session_id} {quoted_prompt}`.

## Completion Evidence

Use the operational commands as task evidence:

```bash
infynon pkg scan
infynon pkg npm install express --strict high
infynon weave flow run checkout --format json --no-input
infynon trace note add repo-handoff --title "Branch handoff" --body "Summarize what changed."
```

Package install exit codes:

- `0` success
- `1` command, warning, or nonblocking failure
- `2` lookup or upstream scan failure
- `3` strict policy block
- `4` non-interactive decision required

## Distribution Note

The public distribution repo includes installers, npm/go wrappers, docs, and release assets; the core Rust implementation is not included.
