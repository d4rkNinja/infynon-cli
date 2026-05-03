# Ninja Coding Bootstrap Flow

This document describes the internal INFYNON flow for launching coding agents through the hidden `coding` command.

The flow is intended for application-owned orchestration. It is not a public user configuration surface.

## Command Surface

INFYNON exposes a hidden coding-agent launcher:

```bash
infynon coding codex
infynon coding claude
infynon coding gemini
```

The command is hidden from normal help output:

```bash
infynon --help
```

`coding` is not listed there. Its subcommands are also hidden from:

```bash
infynon coding --help
```

The older hidden internal alias also exists:

```bash
infynon ninja codex
infynon ninja claude
infynon ninja gemini
```

Use `coding` for launching coding agents with bootstrap behavior.

## Source Files

The flow is implemented by these internal files:

- `src/cli/args/actions_ninja.rs`
  Defines hidden agent choices: `codex`, `claude`, and `gemini`.
- `src/cli/args/root.rs`
  Registers hidden root commands: `ninja` and `coding`.
- `src/cli/commands/user_mode.rs`
  Routes hidden CLI actions into Ninja command execution.
- `src/ninja/commands.rs`
  Executes internal launcher and bootstrap commands.
- `src/ninja/storage.rs`
  Loads embedded internal config and installs the hidden runtime prompt file.
- `src/ninja/types.rs`
  Defines the internal command-template schema.
- `src/ninja/agent-commands.json`
  Stores application-owned launcher and bootstrap command templates.
- `src/ninja/systemprompt.md`
  Stores the source Kuro system prompt bundled with the application.

## Internal Command Config

The internal config file is:

```text
src/ninja/agent-commands.json
```

It is compiled into the binary with:

```rust
include_str!("agent-commands.json")
```

Users should not edit this file at runtime. Changes require source edits and a rebuild.

Current template shape:

```json
{
  "codex": {
    "open": "codex",
    "bootstrap": "codex --config model_instructions_file=\"{system_prompt_path}\" {model_arg} --yolo --no-alt-screen",
    "task": {
      "create": "",
      "start": "codex --config model_instructions_file=\"{task_start_system_prompt_path}\" {model_arg} --yolo --no-alt-screen {quoted_task_start_system_prompt}",
      "note": "",
      "update": "",
      "result": "",
      "complete": "",
      "kill": "",
      "remove": ""
    }
  },
  "claude": {
    "open": "claude",
    "bootstrap": "claude --append-system-prompt {quoted_system_prompt} --permission-mode bypassPermissions --dangerously-skip-permissions",
    "task": {
      "create": "",
      "start": "",
      "note": "",
      "update": "",
      "result": "",
      "complete": "",
      "kill": "",
      "remove": ""
    }
  },
  "gemini": {
    "open": "gemini --skip-trust",
    "bootstrap": "gemini --skip-trust --approval-mode=yolo --prompt-interactive {quoted_system_prompt}",
    "task": {
      "create": "",
      "start": "gemini {model_arg} --skip-trust --approval-mode=yolo --prompt-interactive {quoted_task_start_system_prompt}",
      "note": "",
      "update": "",
      "result": "",
      "complete": "",
      "kill": "",
      "remove": ""
    }
  }
}
```

## Runtime Prompt File

The bundled source prompt lives at:

```text
src/ninja/systemprompt.md
```

At runtime, INFYNON copies that prompt into the hidden user INFYNON storage area:

```text
~/.infynon/ninja/systemprompt.md
~/.infynon/ninja/onboarduser-prompt.md
```

On Windows, this resolves to:

```text
C:\Users\<user>\.infynon\ninja\systemprompt.md
```

On Windows, INFYNON marks these paths hidden using `attrib +h`:

- `~/.infynon`
- `~/.infynon/ninja`
- `~/.infynon/ninja/systemprompt.md`
- `~/.infynon/ninja/onboarduser-prompt.md`

The runtime file is installed automatically when `infynon coding <agent>` runs. The installer can also pre-copy it during a local build/install flow.

## Prompt Role

The prompt defines the Kuro coordination agent:

- Agent name: `Kuro`
- Uses INFYNON workspace commands for project context.
- Uses INFYNON task commands for work tracking.
- Uses INFYNON soul commands for stable global user context.
- Coordinates child coding agents through task records.
- Treats workspace/task JSON output as source of truth.
- Avoids exposing hidden launcher commands and internal templates.

The prompt is appended or injected into coding agents depending on what each tool supports.

## Soul Profile

Kuro can use the user-global soul profile when stable user context matters.

The runtime file is:

```text
~/.infynon/soul.md
```

Use:

```bash
infynon soul show
```

The command returns JSON with:

- `soul_path`
- full `content`
- `is_blank`
- `suggested_structure`

If the soul profile is blank, Kuro should collect global details such as name, purpose, profession, current projects, skills, goals, communication style, answer style, decision preferences, coding preferences, and global constraints. Kuro should not invent missing details.

During bootstrap, INFYNON appends `src/ninja/onboarduser-prompt.md` to the hidden system prompt only when `~/.infynon/soul.md` is blank. Once the soul profile has content, the onboarding prompt is not appended.

Use:

```bash
infynon soul update --text "..."
infynon soul update --file <path>
```

or edit `soul_path` directly when direct file editing is more appropriate.

The soul profile is not workspace-specific. Do not store project-only rules there.

## Multi-Agent Orchestration Model

Kuro is the main coordinating agent.

The intended model is that one main Kuro-controlled session can coordinate multiple child coding agents across terminals. A child agent can be:

- Codex
- Claude
- Gemini
- another instance of Codex, Claude, or Gemini

Multiple child agents can run at the same time when their work is independent and their write areas do not conflict.

INFYNON tasks are the management layer for this flow. Every meaningful child-agent job should map to a task record.

Recommended lifecycle:

1. Create or inspect the workspace.
2. Create a parent task for the overall objective when the work spans more than one agent.
3. Fork or create child tasks for each agent-specific slice.
4. Start the child task when the agent begins work.
5. Record a pid when the launcher or operating system provides one.
6. Record the agent session id when the agent provides one.
7. Use `infynon task resume` for follow-up instructions that should continue the same agent session.
8. Add notes for handoff context, blockers, decisions, and coordination updates.
9. Add results when a child agent produces useful output.
10. Complete each child task after validation.
11. Kill or mark failed tasks when a child agent stalls, exits incorrectly, or must stop.
12. Complete the parent task only after all child outputs are integrated and verified.

Foreground execution is for direct observation or interaction. Use it when the child agent may ask questions, needs live steering, or is doing sensitive work.

Background execution is for independent work. Use it when the child prompt is clear, the output is well-defined, and the process can be tracked with pid, notes, result updates, and task status.

## Agent Root Setup

INFYNON stores one user-global agent root folder in `~/.infynon/ninja.yml` as `agent_root_path`. This is the default working directory for:

- `infynon coding codex`
- `infynon coding claude`
- `infynon coding gemini`

The same path is mirrored into workspace `infynon-agent` with folder `root` for inspection and management. It does not replace the user's default project workspace; tasks created without an explicit workspace use the configured default workspace.

Check the current setting:

```bash
infynon workspace agent-root-show
```

Set it once:

```bash
infynon workspace agent-root-set --mutate --path D:/Codeverse/infynon-agent
```

If the agent root is missing, coding launch commands stop before opening an agent. The orchestrating AI must ask the user for the absolute root folder path, save it with `agent-root-set`, and then rerun the launch.

## Coding TUI

Open the isolated workspace/task management UI:

```bash
infynon coding tui
```

The TUI is an isolated workspace/task control plane. It reads the existing `~/.infynon/ninja.yml`, workspace config JSON files, and task JSON files, then runs the existing `infynon workspace ...` and `infynon task ...` commands internally from form actions. The UI renders human-readable results instead of raw JSON.

Controls:

- `Tab`: switch between workspaces and tasks.
- `Up` / `Down`: select an item.
- `PageUp` / `PageDown`: scroll detail.
- `n`: create workspace or task.
- `u`: update selected workspace or task.
- `g`: set the INFYNON agent root.
- Workspace panel: `a` adds a folder, `x` removes a folder, and `d` deletes the selected workspace when no tasks reference it.
- Task panel: `s` starts, `m` resumes, `o` adds a note, `p` adds a result, `c` completes, `f` fails, `k` kills, and `d` removes.
- `Enter`: advance/save form fields.
- `Esc`: cancel a form or quit.
- `r`: reload state from disk.
- `q`: quit.

The hidden coding launcher supports this directly:

```bash
infynon workspace agent-root-show
infynon workspace agent-root-set --mutate --path D:/Codeverse/infynon-agent
infynon coding codex --background false -- --model gpt-5.5
infynon coding claude --background true -- --verbose
infynon coding gemini --background false -- --debug
```

Launch arguments:

- `--background false`
  Opens a new terminal, runs the rendered interactive bootstrap command there, then best-effort closes the original shell process that ran `infynon coding <agent>` so the user is left with only the agent terminal.
- `--background true`
  Starts the rendered non-interactive/headless bootstrap command without opening a terminal and returns the process id when available.
- `--cwd <path>`
  Runs the command from that directory. If omitted, INFYNON uses the stored agent root path from `~/.infynon/ninja.yml`.
- trailing args after `--`
  Appends those args to the rendered command from `src/ninja/agent-commands.json`.

The launcher does not hardcode agent startup commands. It renders `bootstrap` for foreground interactive mode and `bootstrap_background` for background non-interactive mode from `src/ninja/agent-commands.json`, then applies `--cwd` or the saved agent root, `--background`, and forwarded args.

The original-terminal close behavior applies only to foreground `infynon coding codex|claude|gemini` launches. It does not run for `infynon coding tui`, background launches, or task hook launches.

Original-terminal close is best-effort and OS-specific:

- Windows: INFYNON starts a hidden PowerShell helper that finds the parent shell process for the current `infynon.exe` process and stops it after the new agent terminal has opened.
- macOS/Linux: INFYNON starts a small shell helper that resolves the parent process id and sends `SIGTERM` after the new agent terminal has opened.

The agent terminal is opened before the original shell is closed. If the close helper cannot be started, the launch still succeeds and the JSON execution object reports `close_invoking_terminal.scheduled: false`.

If the agent root is missing, the launcher stops. The orchestrating AI must ask the user for the absolute root folder, save it with `infynon workspace agent-root-set --mutate --path <path>`, then rerun the coding command.

Task coordination rules:

- Assign each child agent a clear task id, workspace, folder, prompt, status, and agent name.
- Prefer one child agent per independent responsibility.
- Avoid sending two child agents to edit the same files unless explicitly intended.
- Use `blocked` status when one child task depends on another.
- Use task notes for live coordination and handoff text.
- Use task results for outputs and verification summaries.
- Store session ids on tasks when available.
- Resume the same child agent session with `infynon task resume <task-id> --mutate --session-id <session-id> --prompt "next instruction"`.
- Keep Kuro responsible for final integration, conflict resolution, validation, and reporting.

## Mandatory Discovery And Planning Protocol

Kuro must follow this order before creating or executing meaningful work:

1. Discover first.
2. Think second.
3. Plan third.
4. Ask for confirmation fourth.
5. Load required skills after approval.
6. Execute after skills are loaded.
7. Validate outputs.
8. Re-check rules compliance.
9. Report clearly.

Discovery requires:

- checking for a project-root `rules/` folder and reading it recursively when present
- explicitly noting `No rules/ folder found` when absent
- checking for a project-root or configured `skills/` folder
- explicitly noting `No skills/ folder found` when absent
- reading required source, config, schema, test, manifest, build, or documentation files before planning when needed

Task planning must include:

- Goal
- Context
- Rules Summary
- Files Reviewed
- Approach
- Constraints
- Done When
- Risks

Final reporting must include:

- Completed Items
- Failed Items with Reasons
- Warnings
- Rules Compliance Status
- Output / Deliverables
- Recommended Next Steps

## Workspace ID Handling

INFYNON uses the workspace name as the workspace id in CLI commands. There is no separate `workspace_id` field.

Before creating a workspace-specific task, inspect workspaces:

```bash
infynon workspace list
infynon workspace show <workspace>
```

Create tasks with the workspace name:

```bash
infynon task create <task-id> --mutate --workspace <workspace-name> --prompt "..."
```

When assigning work that should begin immediately, create the task with `--agent codex`, `--agent claude`, or `--agent gemini` and a clear prompt. INFYNON treats new Codex/Claude/Gemini draft tasks as running assignments and runs the configured `task.start` hook during creation. Use `--status queued` or blocked fields only when launch should be delayed.

If the task belongs to a workspace folder, verify that folder exists in `infynon workspace show <workspace>` and pass:

```bash
--folder-name <folder-name>
```

## Bootstrap Command Behavior

### Codex

Codex bootstrap:

```bash
codex --config model_instructions_file="{system_prompt_path}" {model_arg} --yolo --no-alt-screen
```

Behavior:

- Passes the hidden Kuro prompt file through Codex config key `model_instructions_file`.
- Does not pass a visible/direct instruction prompt; Kuro behavior comes from the system prompt file.
- Uses `--no-alt-screen` for interactive Codex sessions so terminal input/output stays inline instead of switching into the full-screen alternate buffer.
- Passes `--model <model>` when the workspace or task has an assigned model.
- Enables Codex YOLO behavior with `--yolo`.
- Does not pass `-C` or override the project path; the current terminal working directory is used.

### Claude

Claude bootstrap:

```bash
claude --append-system-prompt {quoted_system_prompt} --permission-mode bypassPermissions --dangerously-skip-permissions
```

Behavior:

- Appends the hidden Kuro prompt content to Claude Code's default system prompt.
- Starts Claude with bypass permission mode.
- Includes `--dangerously-skip-permissions` so Claude does not stop for normal permission prompts.

### Gemini

Gemini bootstrap:

```bash
gemini --skip-trust --approval-mode=yolo --prompt-interactive {quoted_system_prompt}
```

Behavior:

- Reads the hidden Kuro prompt file internally.
- Passes that prompt to Gemini with `--prompt-interactive` for foreground launches and `--prompt` for background launches.
- Starts Gemini with `--skip-trust` and `--approval-mode=yolo`.

Important: INFYNON does not use `GEMINI_SYSTEM_MD` for Gemini launches because Windows can surface Markdown file-association UI for `.md` prompt paths. Prompt content is passed directly instead.

## Template Placeholders

Bootstrap templates support these placeholders:

- `{agent}`
  The selected agent name: `codex`, `claude`, or `gemini`.
- `{project_path}`
  The current working directory. Currently available for templates, but not used by the Codex bootstrap.
- `{system_prompt_path}`
  The runtime hidden Kuro prompt file path.
- `{system_prompt}`
  The runtime hidden Kuro prompt file content.
- `{quoted_system_prompt}`
  The runtime hidden Kuro prompt content quoted for the current shell.
- `{model}`
  The selected model name from task or workspace settings.
- `{model_arg}`
  Empty when no model is configured, otherwise `--model <model>`.
- `{gemini_system_prompt_env}`
  Cross-platform environment assignment for `GEMINI_SYSTEM_MD`.

Task hook templates continue to support task-specific placeholders such as:

- `{task_id}`
- `{task_full_name}`
- `{workspace}`
- `{folder_name}`
- `{model}`
- `{thinking}`
- `{status}`
- `{prompt}`
- `{session_id}`
- `{quoted_prompt}`
- `{quoted_session_id}`
- `{task_json_path}`
- `{task_markdown_path}`
- `{task_start_system_prompt_path}`
- `{task_start_system_prompt}`
- `{task_command_guide}`
- `{task_lifecycle_guide}`
- `{task_working_directory}`
- `{quoted_task_start_system_prompt}`

## Current Agent Model Guide

Use the configured workspace model slots first. When a task needs an agent-specific model, use these real CLI model names and pick the smallest capable model for the work:

- Codex `gpt-5.5`: current frontier model for complex coding, research, computer-use, and real-world long-horizon work.
- Codex `gpt-5.4`: strong everyday coding and professional work model with lower cost than `gpt-5.5`.
- Codex `gpt-5.4-mini`: fast, cost-efficient model for simpler coding tasks and small edits.
- Codex `gpt-5.3-codex`: coding-optimized model for agentic engineering and code review workflows.
- Codex `gpt-5.3-codex-spark`: ultra-fast coding research preview for quick, low-risk coding tasks.
- Codex `gpt-5.2`: legacy professional-work fallback for long-running agents when newer models are unavailable.
- Claude `claude-opus-4-7` or alias `opus`: most capable Claude Code model for complex reasoning, planning, architecture, and hard reviews.
- Claude `claude-sonnet-4-6` or alias `sonnet`: daily Claude Code model for implementation, refactors, debugging, and normal agent work.
- Claude `claude-haiku-4-5` or alias `haiku`: fastest Claude model for simple, scoped, latency-sensitive tasks.
- Claude `opusplan`: hybrid Claude Code mode that uses Opus for planning and Sonnet for execution.
- Gemini `auto`: default Gemini CLI routing; lets Gemini choose the best available model for the task.
- Gemini `gemini-3.1-pro-preview`: newest Gemini Pro preview when available; use for the hardest reasoning, coding, and review tasks.
- Gemini `gemini-3-pro-preview`: Gemini 3 Pro preview for complex reasoning and coding when 3.1 is unavailable.
- Gemini `gemini-3-flash-preview`: fast Gemini 3 preview for strong speed/capability balance.
- Gemini `gemini-2.5-pro`: stable Pro model for complex coding, reasoning, multimodal understanding, and large-context work.
- Gemini `gemini-2.5-flash`: fast balanced model for most routine tasks and high-volume work.
- Gemini `gemini-2.5-flash-lite`: fastest low-cost Gemini model for simple tasks.

## Task Start Prompt

When a task is started with:

```bash
infynon task start <task-id> --mutate
```

INFYNON writes a task-specific prompt to:

```text
~/.infynon/ninja/task-start-systemprompt-<task-id>.md
```

This prompt is different from the normal Kuro bootstrap prompt. It is specific to the started task and includes:

- current task id
- task full name
- workspace and folder
- resolved workspace folder working directory
- agent, model, and thinking values
- task JSON path
- task markdown path
- soul path
- task lifecycle rules
- commands for show, note, result, update, complete, and kill

The task-start prompt tells the agent to use the current task id for the whole run, treat the resolved workspace folder as the task working directory, inspect code/config/tests there, add notes/results while working, complete the task with a non-empty result, verify the task is terminal, and close the terminal/session at the end.

Agent task templates can use `{task_start_system_prompt_path}` to pass this prompt into the selected coding tool. Empty task hooks do not run shell commands; INFYNON reports a built-in task update result instead.

When `task.start` runs an external agent command, INFYNON resolves the workspace folder before launch, opens a new terminal in that folder, records the opened terminal pid when available, and starts the command there.

When `task.create` is called with `--status running`, or when a new Codex/Claude/Gemini task would otherwise be created as draft, INFYNON performs the same task-start prompt generation and external agent launch as part of creation. This is the preferred assignment path when the user asks Kuro to hand work to Codex, Claude, or Gemini and expects the child agent to start now.

## Session Resume

Follow-up work can be sent to the same agent session by storing a session id on the task and running:

```bash
infynon task resume <task-id> --mutate --session-id <session-id> --prompt "next instruction"
```

Configured resume templates:

- Codex: `codex ... resume <session-id> <prompt>`
- Claude: `claude -r <session-id> <prompt>`
- Gemini: `gemini --resume <session-id> --prompt-interactive <prompt>`

Task hook failures include recovery guidance with the active task id and valid commands, so an AI can continue manually if a hook command fails.

## Execution Flow

1. User runs:

```bash
infynon coding codex --background false --cwd D:/Codeverse/app -- --model gpt-5.5
```

2. CLI parses hidden root command `coding`.
3. CLI parses hidden agent subcommand `codex`.
4. `execute_coding_command` routes to `commands::execute_coding`.
5. INFYNON ensures the hidden runtime prompt file exists:

```text
~/.infynon/ninja/systemprompt.md
```

6. INFYNON loads internal command templates from:

```text
src/ninja/agent-commands.json
```

7. INFYNON selects the agent's `bootstrap` template.
8. INFYNON renders placeholders.
9. INFYNON appends trailing forwarded args.
10. INFYNON opens a new terminal for foreground mode when `--background false`, or starts a detached background process when `--background true`.
11. For foreground `infynon coding <agent>` launches, INFYNON schedules the original shell process to close after the new terminal opens.
12. Raw rendered command text is not included in user-facing JSON output.

## Shell Execution

Foreground bootstrap launch:

- Windows: opens a new PowerShell window with the rendered command.
- macOS: opens Terminal with the rendered command.
- Linux: opens the first available terminal emulator with the rendered command.
- Codex uses interactive `codex [PROMPT]`.
- Codex task start passes `{quoted_task_start_system_prompt}` as the initial prompt while also setting `model_instructions_file` to the generated task prompt file. Without that prompt argument, Codex opens the TUI and waits for manual input.
- Claude uses interactive `claude [PROMPT]`.
- Gemini task start/resume uses `--prompt-interactive <task-start-prompt>` so the full task prompt is submitted and the session remains interactive.

Background launch:

- Windows: `powershell -NoProfile -Command <rendered command>`
- macOS/Linux: `sh -lc <rendered command>`
- Codex uses non-interactive `codex exec [PROMPT]`.
- Claude uses non-interactive `claude --print [PROMPT]`.
- Gemini uses non-interactive `gemini --prompt <PROMPT>`.

Task hook commands capture output for JSON reporting. Coding bootstrap commands open a terminal for foreground mode or detach in background mode.

Foreground coding bootstrap close behavior:

- Applies to `infynon coding codex`, `infynon coding claude`, and `infynon coding gemini`.
- Does not apply to `infynon coding tui`.
- Does not apply to background mode.
- Does not apply to `infynon task start` or `infynon task resume`; those task hook launches keep the orchestrating terminal alive.
- Is best-effort because terminal hosts differ across Windows Terminal, PowerShell, Terminal.app, and Linux terminal emulators.

## Safety Note

The coding bootstrap flow intentionally enables bypass or YOLO behavior for coding agents.

This means the launched agent may be able to edit files and run commands without normal confirmation prompts. Use this mode only in trusted repositories, isolated workspaces, or disposable branches.

The configured bypass mechanisms are:

- Codex: `--yolo`
- Claude: `--permission-mode bypassPermissions` and `--dangerously-skip-permissions`
- Gemini: `--approval-mode=yolo`

## Validation Checklist

After changing this flow, run:

```bash
cargo fmt --check
cargo check
cargo run -- --help
cargo run -- coding --help
cargo run -- coding codex --help
```

Also validate the internal JSON:

```powershell
Get-Content .\src\ninja\agent-commands.json -Raw | ConvertFrom-Json | Out-Null
```

Expected results:

- `cargo check` passes.
- `infynon --help` does not show `coding` or `ninja`.
- `infynon coding --help` does not expose `codex`, `claude`, or `gemini`.
- `infynon coding codex --help` shows `--background`, `--cwd`, and trailing args.
- `src/ninja/agent-commands.json` parses as valid JSON.

## Install Flow

For a local system install on Windows:

```powershell
cargo build --release
$installDir = Join-Path $env:USERPROFILE ".infynon\bin"
New-Item -ItemType Directory -Force -Path $installDir | Out-Null
Copy-Item .\target\release\infynon.exe (Join-Path $installDir "infynon.exe") -Force
Copy-Item .\target\release\infynon.exe (Join-Path $installDir "infynon-pkg.exe") -Force
```

The hidden prompt can be preinstalled with:

```powershell
$promptDir = Join-Path $env:USERPROFILE ".infynon\ninja"
$promptPath = Join-Path $promptDir "systemprompt.md"
New-Item -ItemType Directory -Force -Path $promptDir | Out-Null
Copy-Item .\src\ninja\systemprompt.md $promptPath -Force
attrib +h (Join-Path $env:USERPROFILE ".infynon")
attrib +h $promptDir
attrib +h $promptPath
```

Runtime execution will also recreate or refresh the prompt file when `infynon coding <agent>` starts.
