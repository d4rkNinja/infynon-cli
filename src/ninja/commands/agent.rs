
enum AgentTaskCommand {
    BuiltIn { agent: String },
    External(String),
}

fn resolve_agent_task_command(
    agent: Option<&str>,
    action: &str,
) -> Result<Option<AgentTaskCommand>, String> {
    let Some(agent) = agent else {
        return Ok(None);
    };
    let agent_name = agent.trim().to_ascii_lowercase();
    let templates = storage::load_agent_commands()?;
    let agent_group = match agent_name.as_str() {
        "codex" => &templates.codex,
        "claude" => &templates.claude,
        "gemini" => &templates.gemini,
        _ => return Ok(None),
    };
    let command = match action {
        "create" => &agent_group.task.create,
        "start" => &agent_group.task.start,
        "resume" => &agent_group.task.resume,
        "note" => &agent_group.task.note,
        "update" => &agent_group.task.update,
        "result" => &agent_group.task.result,
        "complete" => &agent_group.task.complete,
        "fail" => &agent_group.task.fail,
        "kill" => &agent_group.task.kill,
        "remove" => &agent_group.task.remove,
        _ => return Ok(None),
    };
    let trimmed = command.trim();
    if trimmed.is_empty() {
        Ok(Some(AgentTaskCommand::BuiltIn { agent: agent_name }))
    } else {
        Ok(Some(AgentTaskCommand::External(trimmed.to_string())))
    }
}

fn run_agent_task_action(
    agent: Option<&str>,
    action: &str,
    task: &TaskRecord,
) -> Result<Option<serde_json::Value>, String> {
    let Some(command) = resolve_agent_task_command(agent, action)? else {
        return Ok(None);
    };
    match command {
        AgentTaskCommand::BuiltIn { agent } => Ok(Some(built_in_agent_task_result(&agent, action, task))),
        AgentTaskCommand::External(template) => {
            validate_agent_task_template(&template, action)?;
            Ok(Some(run_agent_task_command(&template, action, task)?))
        }
    }
}

fn validate_agent_task_template(template: &str, action: &str) -> Result<(), String> {
    if template.contains("{quoted_task_start_system_prompt}")
        && (action == "start" || action == "resume")
    {
        return Err(
            "Invalid agent command template: use `{task_start_system_prompt_arg}` or `{task_start_system_prompt_stdin}` instead of `{quoted_task_start_system_prompt}` for task start/resume prompts. Inline Markdown prompts can break shell parsing on Windows."
                .to_string(),
        );
    }
    if template.contains("{task_start_system_prompt_arg}")
        && !(action == "start" || action == "resume")
    {
        return Err(
            "Invalid agent command template: `{task_start_system_prompt_arg}` is only valid for task start/resume commands."
                .to_string(),
        );
    }
    if (template.contains("{task_start_interactive_prompt}")
        || template.contains("{quoted_task_start_interactive_prompt}")
        || template.contains("{task_resume_interactive_prompt}")
        || template.contains("{quoted_task_resume_interactive_prompt}"))
        && !(action == "start" || action == "resume")
    {
        return Err(
            "Invalid agent command template: task interactive prompt placeholders are only valid for task start/resume commands."
                .to_string(),
        );
    }
    if template.contains("codex exec ")
        && action == "start"
        && !template.contains("{task_start_system_prompt_stdin}")
    {
        return Err(
            "Invalid Codex exec start template: pipe the task payload through `{task_start_system_prompt_stdin}` and use `-` as the Codex prompt argument."
                .to_string(),
        );
    }
    if template.contains("codex exec resume")
        && action == "resume"
        && !template.contains("{prompt_stdin}")
    {
        return Err(
            "Invalid Codex exec resume template: pipe the resume prompt through `{prompt_stdin}` and use `-` as the Codex prompt argument."
                .to_string(),
        );
    }
    Ok(())
}

fn extract_agent_execution_pid(value: &Option<serde_json::Value>) -> Option<u32> {
    value
        .as_ref()
        .and_then(|value| value.get("pid"))
        .and_then(|pid| {
            pid.as_u64()
                .and_then(|value| u32::try_from(value).ok())
                .or_else(|| pid.as_str().and_then(|value| value.trim().parse::<u32>().ok()))
        })
        .filter(|pid| *pid > 0)
}

fn built_in_agent_task_result(agent: &str, action: &str, task: &TaskRecord) -> serde_json::Value {
    json!({
        "ran": false,
        "mode": "built_in",
        "agent": agent,
        "action": action,
        "task_id": task.id,
        "external_command_configured": false,
        "message": "No external agent task command is configured; INFYNON completed the built-in task state update.",
    })
}

fn run_agent_task_command(
    template: &str,
    action: &str,
    task: &TaskRecord,
) -> Result<serde_json::Value, String> {
    let command = render_agent_command_template(template, task);
    let cwd = task_working_directory(task)?;
    let result = if action == "start" || action == "resume" {
        let command = wrap_foreground_task_command(&command, action, task);
        launch_agent_command(&command, &cwd, false)
    } else {
        run_hidden_shell_command(&command, cwd.as_deref())
    };
    result
        .map(|mut value| {
            if let Some(object) = value.as_object_mut() {
                object.insert("cwd".to_string(), json!(cwd));
            }
            value
        })
        .map_err(|err| {
            let crash_report_path =
                write_agent_crash_report(action, task, &command, cwd.as_deref(), &err)
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|report_err| {
                        format!("failed to write crash report: {}", report_err)
                    });
            format!(
                "{}\n\nCrash report: {}\n\n{}",
                err,
                crash_report_path,
                task_hook_error_guide(action, task)
            )
        })
}

fn wrap_foreground_task_command(command: &str, action: &str, task: &TaskRecord) -> String {
    let reason = format!(
        "Agent command exited with non-zero status during task {}.",
        action
    );
    let result = format!("failed: {}", reason);
    let crash_report_path = storage::crash_report_path(&format!("task-{}-{}", task.id, action))
        .display()
        .to_string();
    let crash_report = format!(
        "# INFYNON Agent Crash Report\n\nTask ID: {}\nAction: {}\nAgent: {}\nWorkspace: {}\nFolder: {}\nModel: {}\nReason: {}\n\n## Command\n\n```text\n{}\n```\n",
        task.id,
        action,
        task.agent.as_deref().unwrap_or(""),
        task.workspace.as_deref().unwrap_or(""),
        task.folder_name.as_deref().unwrap_or(""),
        task.model.as_deref().unwrap_or(""),
        reason,
        command,
    );
    if cfg!(windows) {
        format!(
            "& {{ {} }}; $code = $LASTEXITCODE; if ($code -ne 0) {{ $report = {}; $reportDir = Split-Path -Parent $report; New-Item -ItemType Directory -Force -Path $reportDir | Out-Null; Set-Content -LiteralPath $report -Value {} -Encoding UTF8; infynon task fail {} --mutate --reason {} --result {} --keep-terminal; Write-Host \"INFYNON crash report: $report\"; exit $code }}",
            command,
            shell_quote(&crash_report_path),
            shell_quote(&crash_report),
            task.id,
            shell_quote(&reason),
            shell_quote(&result)
        )
    } else {
        format!(
            "( {} ); code=$?; if [ \"$code\" -ne 0 ]; then mkdir -p {}; cat > {} <<'INFYNON_CRASH_REPORT'\n{}\nINFYNON_CRASH_REPORT\ninfynon task fail {} --mutate --reason {} --result {} --keep-terminal; printf 'INFYNON crash report: %s\\n' {}; fi; exit \"$code\"",
            command,
            shell_quote(
                &std::path::Path::new(&crash_report_path)
                    .parent()
                    .map(|path| path.display().to_string())
                    .unwrap_or_default()
            ),
            shell_quote(&crash_report_path),
            crash_report,
            shell_quote(&task.id),
            shell_quote(&reason),
            shell_quote(&result),
            shell_quote(&crash_report_path)
        )
    }
}

fn write_agent_crash_report(
    action: &str,
    task: &TaskRecord,
    command: &str,
    cwd: Option<&str>,
    error: &str,
) -> Result<std::path::PathBuf, String> {
    let content = format!(
        "# INFYNON Agent Crash Report\n\nTask ID: {task_id}\nAction: {action}\nAgent: {agent}\nWorkspace: {workspace}\nFolder: {folder}\nModel: {model}\nWorking Directory: {cwd}\nError: {error}\n\n## Command\n\n```text\n{command}\n```\n",
        task_id = task.id,
        action = action,
        agent = task.agent.as_deref().unwrap_or(""),
        workspace = task.workspace.as_deref().unwrap_or(""),
        folder = task.folder_name.as_deref().unwrap_or(""),
        model = task.model.as_deref().unwrap_or(""),
        cwd = cwd.unwrap_or(""),
        error = error,
        command = command,
    );
    storage::write_crash_report(&format!("task-{}-{}", task.id, action), &content)
}

#[derive(Debug)]
struct AgentLaunchRequest {
    agent: &'static str,
    background: bool,
    cwd: Option<String>,
    args: Vec<String>,
}

impl AgentLaunchRequest {
    fn from_action(action: NinjaAction) -> Self {
        match action {
            NinjaAction::Tui => unreachable!("TUI actions are handled before agent launch"),
            NinjaAction::Codex {
                background,
                cwd,
                args,
            } => Self {
                agent: "codex",
                background,
                cwd,
                args,
            },
            NinjaAction::Claude {
                background,
                cwd,
                args,
            } => Self {
                agent: "claude",
                background,
                cwd,
                args,
            },
            NinjaAction::Gemini {
                background,
                cwd,
                args,
            } => Self {
                agent: "gemini",
                background,
                cwd,
                args,
            },
        }
    }
}

fn run_project_agent_open(request: AgentLaunchRequest) -> Result<(), String> {
    let (default_cwd, default_model) = default_launch_context()?;
    let cwd = request.cwd.clone().or(default_cwd);
    let templates = storage::load_internal_agent_commands()?;
    let agent_group = match request.agent {
        "codex" => &templates.codex,
        "claude" => &templates.claude,
        "gemini" => &templates.gemini,
        _ => return Err(format!("Unsupported ninja agent '{}'.", request.agent)),
    };
    let template = agent_group.open.trim();
    if template.is_empty() {
        return Err(format!(
            "No internal open command configured for '{}'. Update src/ninja/agent-commands.json.",
            request.agent
        ));
    }
    let command = append_forwarded_args(
        &render_project_agent_command_template(
            template,
            request.agent,
            None,
            default_model.as_deref(),
        ),
        &request.args,
    );
    let execution = launch_agent_command(&command, &cwd, request.background)?;
    print_json_pretty(&json!({
        "status": "ok",
        "command": "ninja.open",
        "agent": request.agent,
        "background": request.background,
        "cwd": cwd,
        "agent_command_template_source": "internal",
        "agent_command_execution": execution,
    }));
    Ok(())
}

fn run_agent_bootstrap(request: AgentLaunchRequest) -> Result<(), String> {
    let prompt_path = storage::ensure_hidden_system_prompt_file()?;
    let (default_cwd, default_model) = default_launch_context()?;
    let cwd = request.cwd.clone().or(default_cwd);
    let templates = storage::load_internal_agent_commands()?;
    let agent_group = match request.agent {
        "codex" => &templates.codex,
        "claude" => &templates.claude,
        "gemini" => &templates.gemini,
        _ => return Err(format!("Unsupported coding agent '{}'.", request.agent)),
    };
    let template = if request.background {
        agent_group.bootstrap_background.trim()
    } else {
        agent_group.bootstrap.trim()
    };
    if template.is_empty() {
        return Err(format!(
            "No internal bootstrap command configured for '{}'. Update src/ninja/agent-commands.json.",
            request.agent
        ));
    }
    let prompt_path_text = prompt_path.display().to_string();
    let command = append_forwarded_args(
        &render_project_agent_command_template(
            template,
            request.agent,
            Some(prompt_path_text.as_str()),
            default_model.as_deref(),
        ),
        &request.args,
    );
    let mut execution = launch_agent_command(&command, &cwd, request.background)?;
    let close_invoking_terminal = None::<serde_json::Value>;
    if let Some(object) = execution.as_object_mut() {
        object.insert(
            "close_invoking_terminal".to_string(),
            json!(close_invoking_terminal),
        );
    }
    if !request.background {
        return Ok(());
    }
    print_json_pretty(&json!({
        "status": "ok",
        "command": "coding.bootstrap",
        "agent": request.agent,
        "background": request.background,
        "cwd": cwd,
        "system_prompt_path": prompt_path,
        "agent_command_template_source": "internal",
        "agent_command_execution": execution,
    }));
    Ok(())
}

fn run_hidden_shell_command(
    command: &str,
    cwd: Option<&str>,
) -> Result<serde_json::Value, String> {
    let mut process = if cfg!(windows) {
        let mut cmd = Command::new("powershell");
        cmd.args(["-NoProfile", "-Command", command]);
        cmd
    } else {
        let mut cmd = Command::new("sh");
        cmd.args(["-lc", command]);
        cmd
    };
    if let Some(cwd) = cwd {
        process.current_dir(cwd);
    }
    let output = process
        .output()
        .map_err(|e| format!("Failed to execute agent task command: {}", e))?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !output.status.success() {
        return Err(format!(
            "Agent command failed: {}",
            if stderr.is_empty() {
                format!("exit code {:?}", output.status.code())
            } else {
                stderr
            }
        ));
    }
    Ok(json!({
        "ran": true,
        "exit_code": output.status.code(),
        "stdout": stdout,
        "stderr": stderr,
    }))
}

fn render_agent_command_template(template: &str, task: &TaskRecord) -> String {
    let full_name = task_full_name(task);
    let workspace = task.workspace.as_deref().unwrap_or("");
    let folder_name = task.folder_name.as_deref().unwrap_or("");
    let prompt = task.prompt.as_deref().unwrap_or("");
    let session_id = task.session_id.as_deref().unwrap_or("");
    let model = task.model.as_deref().unwrap_or("");
    let model_arg = model_arg(model);
    let thinking = task.thinking.as_deref().unwrap_or("auto");
    let status = task.status.as_str();
    let task_file = storage::task_file(&task.id).display().to_string();
    let markdown_path = storage::task_markdown_file(
        &task.id,
        task.workspace.as_deref(),
        task.folder_name.as_deref(),
    )
    .display()
    .to_string();
    let task_start_system_prompt_path = storage::task_start_system_prompt_path(&task.id)
        .display()
        .to_string();
    let gemini_task_system_prompt_env =
        gemini_system_prompt_env(&task_start_system_prompt_path);
    let task_command_guide = task_command_guide(task);
    let task_lifecycle_guide = task_lifecycle_guide(task);
    let task_start_system_prompt =
        render_task_start_system_prompt(task, &task_start_system_prompt_path);
    let task_start_interactive_prompt = task_start_interactive_prompt(task);
    let task_resume_interactive_prompt = task_resume_interactive_prompt(task);
    let quoted_task_start_system_prompt = if uses_task_system_prompt_file_template(template) {
        shell_quote(&task_start_initial_prompt(
            task,
            &task_start_system_prompt_path,
        ))
    } else {
        shell_quote(&task_start_system_prompt)
    };
    let task_start_prompt_payload = if uses_task_system_prompt_file_template(template) {
        task_start_initial_prompt(task, &task_start_system_prompt_path)
    } else {
        task_start_system_prompt.clone()
    };
    let task_start_system_prompt_arg = command_arg_from_content_file(
        &format!("task-{}-prompt", task.id),
        &task_start_prompt_payload,
    )
    .unwrap_or_else(|_| shell_quote(&task_start_prompt_payload));
    let task_start_system_prompt_stdin = command_stdin_from_content_file(
        &format!("task-{}-prompt", task.id),
        &task_start_prompt_payload,
    )
    .unwrap_or_default();
    let prompt_stdin = command_stdin_from_content_file(
        &format!("task-{}-resume-prompt", task.id),
        prompt,
    )
    .unwrap_or_default();
    let task_working_directory = task_working_directory(task)
        .ok()
        .flatten()
        .unwrap_or_default();

    template
        .replace("{task_id}", &task.id)
        .replace("{task_full_name}", &full_name)
        .replace("{workspace}", workspace)
        .replace("{folder_name}", folder_name)
        .replace("{agent}", task.agent.as_deref().unwrap_or(""))
        .replace("{model}", model)
        .replace("{model_arg}", &model_arg)
        .replace("{thinking}", thinking)
        .replace("{status}", status)
        .replace("{prompt}", prompt)
        .replace("{session_id}", session_id)
        .replace("{quoted_prompt}", &shell_quote(prompt))
        .replace("{quoted_session_id}", &shell_quote(session_id))
        .replace("{task_json_path}", &task_file)
        .replace("{task_markdown_path}", &markdown_path)
        .replace(
            "{task_start_system_prompt_path}",
            &task_start_system_prompt_path,
        )
        .replace(
            "{quoted_task_start_system_prompt_path}",
            &shell_quote(&task_start_system_prompt_path),
        )
        .replace(
            "{quoted_codex_task_start_system_prompt_config}",
            &shell_quote(&format!(
                "model_instructions_file=\"{}\"",
                task_start_system_prompt_path
            )),
        )
        .replace("{task_start_system_prompt}", &task_start_system_prompt)
        .replace(
            "{quoted_task_start_system_prompt}",
            &quoted_task_start_system_prompt,
        )
        .replace(
            "{task_start_interactive_prompt}",
            &task_start_interactive_prompt,
        )
        .replace(
            "{quoted_task_start_interactive_prompt}",
            &shell_quote(&task_start_interactive_prompt),
        )
        .replace(
            "{task_resume_interactive_prompt}",
            &task_resume_interactive_prompt,
        )
        .replace(
            "{quoted_task_resume_interactive_prompt}",
            &shell_quote(&task_resume_interactive_prompt),
        )
        .replace(
            "{task_start_system_prompt_arg}",
            &task_start_system_prompt_arg,
        )
        .replace(
            "{task_start_system_prompt_stdin}",
            &task_start_system_prompt_stdin,
        )
        .replace("{prompt_stdin}", &prompt_stdin)
        .replace("{task_command_guide}", &task_command_guide)
        .replace("{task_lifecycle_guide}", &task_lifecycle_guide)
        .replace("{task_working_directory}", &task_working_directory)
        .replace(
            "{gemini_task_system_prompt_env}",
            &gemini_task_system_prompt_env,
        )
}

fn command_stdin_from_content_file(stem: &str, content: &str) -> Result<String, String> {
    let path = write_temp_agent_content(stem, content)?;
    let path = path.display().to_string();
    if cfg!(windows) {
        Ok(format!("Get-Content -LiteralPath {} -Raw |", shell_quote(&path)))
    } else {
        Ok(format!("cat {} |", shell_quote(&path)))
    }
}

fn command_arg_from_content_file(stem: &str, content: &str) -> Result<String, String> {
    let path = write_temp_agent_content(stem, content)?;
    let path = path.display().to_string();
    if cfg!(windows) {
        Ok(format!("(Get-Content -LiteralPath {} -Raw)", shell_quote(&path)))
    } else {
        Ok(format!("\"$(cat {})\"", shell_quote(&path)))
    }
}

fn write_temp_agent_content(stem: &str, content: &str) -> Result<std::path::PathBuf, String> {
    let stamp = timestamp_now()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>();
    let path = std::env::temp_dir().join(format!(
        "infynon-agent-content-{}-{}-{}.md",
        std::process::id(),
        safe_file_stem(stem),
        stamp
    ));
    std::fs::write(&path, content)
        .map(|_| path)
        .map_err(|e| format!("Failed to write agent content file: {}", e))
}

fn uses_task_system_prompt_file_template(template: &str) -> bool {
    let uses_codex_file = template.contains("codex")
        && template.contains("--config")
        && template.contains("{quoted_codex_task_start_system_prompt_config}");
    let uses_claude_file = template.contains("claude")
        && template.contains("--append-system-prompt-file")
        && template.contains("{quoted_task_start_system_prompt_path}");
    let uses_gemini_file = template.contains("gemini")
        && template.contains("{gemini_task_system_prompt_env}");
    (uses_codex_file || uses_claude_file || uses_gemini_file)
        && (template.contains("{quoted_task_start_system_prompt}")
            || template.contains("{task_start_system_prompt_arg}")
            || template.contains("{task_start_system_prompt_stdin}"))
}

fn task_start_initial_prompt(task: &TaskRecord, prompt_path: &str) -> String {
    let task_file = storage::task_file(&task.id).display().to_string();
    let markdown_path = storage::task_markdown_file(
        &task.id,
        task.workspace.as_deref(),
        task.folder_name.as_deref(),
    )
    .display()
    .to_string();
    let task_working_directory = task_working_directory(task)
        .ok()
        .flatten()
        .unwrap_or_default();
    let prompt = task.prompt.as_deref().unwrap_or("").trim();
    let notes = task.notes.as_deref().unwrap_or("").trim();
    let result = task.result.as_deref().unwrap_or("").trim();
    let blocked_by = task.blocked_by.as_deref().unwrap_or("").trim();
    let blocked_reason = task.blocked_reason.as_deref().unwrap_or("").trim();
    let session_id = task.session_id.as_deref().unwrap_or("").trim();

    format!(
        "# INFYNON Task Payload\n\nTask ID: {id}\nWorkspace: {workspace}\nFolder: {folder}\nWorking directory: {cwd}\nTask status: {status}\nAgent: {agent}\nModel: {model}\nThinking: {thinking}\nPID: {pid}\nSession ID: {session_id}\nBlocked By: {blocked_by}\nBlocked Reason: {blocked_reason}\nTask record: {task_file}\nTask notes file: {markdown_path}\nSystem instruction file: {prompt_path}\n\n## Task Description\n\n{prompt}\n\n## Existing Notes\n\n{notes}\n\n## Existing Results\n\n{result}\n\nStart now. Follow the system instruction file for INFYNON behavior. Treat this payload as the real task content. Update the task record with notes/results and complete, fail, or kill the task according to the actual outcome.",
        id = task.id,
        workspace = task.workspace.as_deref().unwrap_or(""),
        folder = task.folder_name.as_deref().unwrap_or(""),
        cwd = task_working_directory,
        status = task.status.as_str(),
        agent = task.agent.as_deref().unwrap_or(""),
        model = task.model.as_deref().unwrap_or(""),
        thinking = task.thinking.as_deref().unwrap_or("auto"),
        pid = task.pid.map(|value| value.to_string()).unwrap_or_default(),
        session_id = if session_id.is_empty() { "(none)" } else { session_id },
        blocked_by = if blocked_by.is_empty() { "(none)" } else { blocked_by },
        blocked_reason = if blocked_reason.is_empty() { "(none)" } else { blocked_reason },
        task_file = task_file,
        markdown_path = markdown_path,
        prompt_path = prompt_path,
        prompt = if prompt.is_empty() {
            "(No task brief was provided. Inspect the task record before editing.)"
        } else {
            prompt
        },
        notes = if notes.is_empty() { "(none)" } else { notes },
        result = if result.is_empty() { "(none)" } else { result },
    )
}

fn task_start_interactive_prompt(task: &TaskRecord) -> String {
    format!(
        "Start INFYNON task {id}. First run: infynon task show {id}. Treat that durable task record as the assignment. Follow the INFYNON lifecycle and complete, fail, or kill the task before exit.",
        id = task.id
    )
}

fn task_resume_interactive_prompt(task: &TaskRecord) -> String {
    format!(
        "Continue INFYNON task {id}. First run: infynon task show {id}. Treat the durable task record and latest prompt as authoritative, then complete, fail, or kill the task before exit.",
        id = task.id
    )
}

#[cfg(test)]
mod agent_task_template_tests {
    use super::{uses_task_system_prompt_file_template, validate_agent_task_template};

    #[test]
    fn detects_claude_task_prompt_file_template() {
        let template = "claude {model_arg} --append-system-prompt-file {quoted_task_start_system_prompt_path} {quoted_task_start_system_prompt}";

        assert!(uses_task_system_prompt_file_template(template));
    }

    #[test]
    fn detects_codex_task_prompt_file_template() {
        let template = "codex --config {quoted_codex_task_start_system_prompt_config} {quoted_task_start_system_prompt}";

        assert!(uses_task_system_prompt_file_template(template));
    }

    #[test]
    fn detects_codex_task_prompt_file_template_with_file_arg() {
        let template = "codex --config {quoted_codex_task_start_system_prompt_config} {task_start_system_prompt_arg}";

        assert!(uses_task_system_prompt_file_template(template));
    }

    #[test]
    fn detects_codex_task_prompt_file_template_with_stdin_arg() {
        let template = "{task_start_system_prompt_stdin} codex exec --config {quoted_codex_task_start_system_prompt_config} -";

        assert!(uses_task_system_prompt_file_template(template));
    }

    #[test]
    fn detects_gemini_task_system_prompt_env_template() {
        let template = "{gemini_task_system_prompt_env} gemini --prompt-interactive {quoted_task_start_system_prompt}";

        assert!(uses_task_system_prompt_file_template(template));
    }

    #[test]
    fn permits_codex_interactive_task_prompt_for_start() {
        let template = "codex --config {quoted_codex_task_start_system_prompt_config} {quoted_task_start_interactive_prompt}";

        assert!(validate_agent_task_template(template, "start").is_ok());
    }

    #[test]
    fn rejects_task_interactive_prompt_outside_start_resume() {
        let template = "codex {quoted_task_start_interactive_prompt}";

        assert!(validate_agent_task_template(template, "complete").is_err());
    }
}

fn ensure_task_start_system_prompt(task: &TaskRecord) -> Result<std::path::PathBuf, String> {
    let path = storage::task_start_system_prompt_path(&task.id);
    let content = render_task_start_system_prompt(task, path.display().to_string().as_str());
    storage::write_task_start_system_prompt(&task.id, &content)
}

fn render_task_start_system_prompt(task: &TaskRecord, prompt_path: &str) -> String {
    let template = include_str!("../task-start-systemprompt.md");
    let task_file = storage::task_file(&task.id).display().to_string();
    let markdown_path = storage::task_markdown_file(
        &task.id,
        task.workspace.as_deref(),
        task.folder_name.as_deref(),
    )
    .display()
    .to_string();
    let task_working_directory = task_working_directory(task)
        .ok()
        .flatten()
        .unwrap_or_default();
    template
        .replace("{task_id}", &task.id)
        .replace("{task_full_name}", &task_full_name(task))
        .replace("{workspace}", task.workspace.as_deref().unwrap_or(""))
        .replace("{folder_name}", task.folder_name.as_deref().unwrap_or(""))
        .replace("{agent}", task.agent.as_deref().unwrap_or(""))
        .replace("{prompt}", task.prompt.as_deref().unwrap_or(""))
        .replace("{session_id}", task.session_id.as_deref().unwrap_or(""))
        .replace("{model}", task.model.as_deref().unwrap_or(""))
        .replace("{thinking}", task.thinking.as_deref().unwrap_or("auto"))
        .replace("{status}", task.status.as_str())
        .replace("{task_json_path}", &task_file)
        .replace("{task_markdown_path}", &markdown_path)
        .replace("{task_working_directory}", &task_working_directory)
        .replace("{soul_path}", &storage::soul_path().display().to_string())
        .replace("{task_start_system_prompt_path}", prompt_path)
}

fn task_command_guide(task: &TaskRecord) -> String {
    format!(
        "Task command guide for {id}:\n\
- Show: infynon task show {id}\n\
- Update: infynon task update {id} --mutate --status running\n\
- Resume: infynon task resume {id} --mutate --session-id <session-id> --prompt \"next instruction\"\n\
- Note: infynon task note {id} --mutate --text \"note text\"\n\
- Result: infynon task result {id} --mutate --text \"result text\"\n\
- Complete: infynon task complete {id} --mutate --result \"final result\"\n\
- Fail: infynon task fail {id} --mutate --reason \"failure reason\"\n\
- Kill: infynon task kill {id} --mutate --pid <pid> --reason \"reason\"\n\
- Soul: infynon soul show",
        id = task.id
    )
}

fn task_lifecycle_guide(task: &TaskRecord) -> String {
    format!(
        "Use task id {id} for this entire run. Store the agent session id when available, use task resume for follow-up instructions in the same session, add notes/results for coordination and outputs, and complete the task when the work is verified. Do not leave the task running at the end.",
        id = task.id
    )
}

fn task_hook_error_guide(action: &str, task: &TaskRecord) -> String {
    format!(
        "Agent task hook failed.\nAction: {action}\nTask ID: {id}\n\n{commands}\n\n{lifecycle}",
        action = action,
        id = task.id,
        commands = task_command_guide(task),
        lifecycle = task_lifecycle_guide(task)
    )
}

fn task_working_directory(task: &TaskRecord) -> Result<Option<String>, String> {
    let Some(workspace_name) = task.workspace.as_deref() else {
        return Ok(None);
    };
    let workspace = storage::load_workspace(workspace_name)?;
    if let Some(folder_name) = task.folder_name.as_deref() {
        let Some(folder) = workspace
            .folders
            .iter()
            .find(|folder| folder.folder_name == folder_name)
        else {
            return Err(format!(
                "Folder '{}' was not found in workspace '{}'.",
                folder_name, workspace_name
            ));
        };
        return Ok(Some(folder.path.clone()));
    }
    Ok(workspace.path)
}

fn validate_blocking_reference(
    current_task_id: &str,
    blocked_by: Option<&str>,
) -> Result<(), String> {
    if let Some(blocked_by) = blocked_by {
        if blocked_by == current_task_id {
            return Err("`--blocked-by` cannot reference the current task id.".to_string());
        }
        storage::load_task(blocked_by)
            .map(|_| ())
            .map_err(|_| format!("Blocked-by task '{}' was not found.", blocked_by))?;
    }
    Ok(())
}

fn effective_task_status(status: &str, blocked_by: Option<&String>) -> String {
    if blocked_by.is_some() {
        "blocked".to_string()
    } else {
        status.to_ascii_lowercase()
    }
}

fn is_coding_agent(agent: Option<&str>) -> bool {
    matches!(
        agent.map(|value| value.trim().to_ascii_lowercase()),
        Some(agent) if matches!(agent.as_str(), "codex" | "claude" | "gemini")
    )
}

fn resolve_task_folder_name(
    workspace: Option<&str>,
    folder_name: Option<String>,
) -> Result<Option<String>, String> {
    match workspace {
        Some(workspace_name) => {
            let workspace = storage::load_workspace(workspace_name)?;
            match folder_name {
                Some(folder_name) => {
                    if workspace
                        .folders
                        .iter()
                        .any(|folder| folder.folder_name == folder_name)
                    {
                        Ok(Some(folder_name))
                    } else {
                        Err(format!(
                            "Folder '{}' was not found in workspace '{}'.",
                            folder_name, workspace_name
                        ))
                    }
                }
                None => Ok(workspace.folder_name.clone()),
            }
        }
        None => Ok(folder_name),
    }
}

fn is_finished_status(status: &str) -> bool {
        matches!(
        status.to_ascii_lowercase().as_str(),
        "completed" | "failed" | "killed"
    )
}

fn ensure_not_blocked(task: &TaskRecord, action: &str) -> Result<(), String> {
    if task.status.eq_ignore_ascii_case("blocked") || task.blocked_by.is_some() {
        Err(format!(
            "Cannot {} task '{}' because it is blocked. Clear `blocked_by`/blocked status before starting or resuming it.",
            action, task.id
        ))
    } else {
        Ok(())
    }
}

fn ensure_not_finished(task: &TaskRecord, action: &str) -> Result<(), String> {
    if is_finished_status(&task.status) {
        Err(format!(
            "Cannot {} task '{}' because it is already in terminal status '{}'.",
            action, task.id, task.status
        ))
    } else {
        Ok(())
    }
}

fn restore_task_after_launch_failure(task: &TaskRecord) -> Result<(), String> {
    storage::save_task(task)?;
    let _ = write_task_markdown(task)?;
    Ok(())
}

fn mark_task_failed_after_launch_error(
    manifest: &mut NinjaManifest,
    task: &mut TaskRecord,
    error: &str,
) -> Result<(), String> {
    task.status = "failed".to_string();
    task.notes = Some(append_text(
        task.notes.clone(),
        &format!("launch_error: {}", error),
    ));
    task.result = Some(format!("failed: {}", error));
    task.updated_at = timestamp_now();
    task.ended_at = Some(task.updated_at.clone());
    storage::save_task(task)?;
    let _ = write_task_markdown(task)?;
    storage::upsert_task_summary(manifest, task);
    storage::save_manifest(manifest)
}

fn find_workspace_summary(
    manifest: &NinjaManifest,
    name: &str,
) -> Result<WorkspaceSummary, String> {
    manifest
        .workspaces
        .iter()
        .find(|workspace| workspace.name == name)
        .cloned()
        .ok_or_else(|| format!("Workspace '{}' summary was not found after write.", name))
}

fn find_task_summary(manifest: &NinjaManifest, id: &str) -> Result<TaskSummary, String> {
    manifest
        .tasks
        .iter()
        .find(|task| task.id == id)
        .cloned()
        .ok_or_else(|| format!("Task '{}' summary was not found after write.", id))
}

fn kill_process(pid: u32, force: bool) -> Result<(), String> {
    #[cfg(windows)]
    {
        let mut command = Command::new("taskkill");
        command.arg("/PID").arg(pid.to_string()).arg("/T");
        if force {
            command.arg("/F");
        }
        let output = command.output().map_err(|e| e.to_string())?;
        if output.status.success() {
            return Ok(());
        }
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(if stderr.is_empty() {
            format!("Failed to kill process {}.", pid)
        } else {
            stderr
        })
    }

    #[cfg(not(windows))]
    {
        let signal = if force { "-9" } else { "-15" };
        let output = Command::new("kill")
            .arg(signal)
            .arg(pid.to_string())
            .output()
            .map_err(|e| e.to_string())?;
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(if stderr.is_empty() {
                format!("Failed to kill process {}.", pid)
            } else {
                stderr
            })
        }
    }
}

fn schedule_close_terminal(pid: Option<u32>) -> Option<serde_json::Value> {
    let pid = pid?;
    #[cfg(windows)]
    {
        let script = format!(
            "Start-Sleep -Milliseconds 700; taskkill /PID {} /T /F | Out-Null; Remove-Item -LiteralPath $PSCommandPath -Force -ErrorAction SilentlyContinue",
            pid
        );
        let spawned = spawn_detached_windows_powershell(&script);
        Some(match spawned {
            Ok(child) => json!({
                "scheduled": true,
                "pid": pid,
                "closer_pid": child.id(),
                "method": "taskkill /T /F",
            }),
            Err(err) => json!({
                "scheduled": false,
                "pid": pid,
                "error": err.to_string(),
            }),
        })
    }

    #[cfg(not(windows))]
    {
        let script = format!("sleep 0.5; kill -TERM {}", pid);
        let spawned = Command::new("sh").args(["-lc", &script]).spawn();
        Some(match spawned {
            Ok(child) => json!({
                "scheduled": true,
                "pid": pid,
                "closer_pid": child.id(),
                "method": "kill -TERM",
            }),
            Err(err) => json!({
                "scheduled": false,
                "pid": pid,
                "error": err.to_string(),
            }),
        })
    }
}
