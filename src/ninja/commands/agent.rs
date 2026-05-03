
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
            Ok(Some(run_agent_task_command(&template, action, task)?))
        }
    }
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
        .map_err(|err| format!("{}\n\n{}", err, task_hook_error_guide(action, task)))
}

fn wrap_foreground_task_command(command: &str, action: &str, task: &TaskRecord) -> String {
    let reason = format!(
        "Agent command exited with non-zero status during task {}.",
        action
    );
    let result = format!("failed: {}", reason);
    if cfg!(windows) {
        format!(
            "& {{ {} }}; if ($LASTEXITCODE -ne 0) {{ infynon task fail {} --mutate --reason {} --result {} }}",
            command,
            task.id,
            shell_quote(&reason),
            shell_quote(&result)
        )
    } else {
        format!(
            "( {} ); code=$?; if [ \"$code\" -ne 0 ]; then infynon task fail {} --mutate --reason {} --result {}; fi; exit \"$code\"",
            command,
            shell_quote(&task.id),
            shell_quote(&reason),
            shell_quote(&result)
        )
    }
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
    let close_invoking_terminal = if request.background {
        None
    } else {
        Some(schedule_close_invoking_terminal())
    };
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
    let gemini_task_system_prompt_env = if cfg!(windows) {
        format!(
            "$env:GEMINI_SYSTEM_MD = \"{}\";",
            task_start_system_prompt_path
        )
    } else {
        format!("GEMINI_SYSTEM_MD=\"{}\"", task_start_system_prompt_path)
    };
    let task_command_guide = task_command_guide(task);
    let task_lifecycle_guide = task_lifecycle_guide(task);
    let task_start_system_prompt =
        render_task_start_system_prompt(task, &task_start_system_prompt_path);
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
        .replace("{task_start_system_prompt}", &task_start_system_prompt)
        .replace(
            "{quoted_task_start_system_prompt}",
            &shell_quote(&task_start_system_prompt),
        )
        .replace("{task_command_guide}", &task_command_guide)
        .replace("{task_lifecycle_guide}", &task_lifecycle_guide)
        .replace("{task_working_directory}", &task_working_directory)
        .replace(
            "{gemini_task_system_prompt_env}",
            &gemini_task_system_prompt_env,
        )
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
