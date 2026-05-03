#[allow(clippy::too_many_arguments)]
fn task_create(
    id: &str,
    workspace: Option<String>,
    folder_name: Option<String>,
    agent: Option<String>,
    model: Option<String>,
    thinking: Option<String>,
    prompt: Option<String>,
    command: Option<String>,
    pid: Option<u32>,
    session_id: Option<String>,
    notes: Option<String>,
    result: Option<String>,
    blocked_by: Option<String>,
    blocked_reason: Option<String>,
    status: &str,
) -> Result<(), String> {
    task_create_from_parent(
        id,
        "",
        workspace,
        folder_name,
        agent,
        model,
        thinking,
        prompt,
        command,
        pid,
        session_id,
        notes,
        result,
        blocked_by,
        blocked_reason,
        status,
    )
}

#[allow(clippy::too_many_arguments)]
fn task_create_from_parent(
    id: &str,
    parent_task_id: &str,
    workspace: Option<String>,
    folder_name: Option<String>,
    agent: Option<String>,
    model: Option<String>,
    thinking: Option<String>,
    prompt: Option<String>,
    command: Option<String>,
    pid: Option<u32>,
    session_id: Option<String>,
    notes: Option<String>,
    result: Option<String>,
    blocked_by: Option<String>,
    blocked_reason: Option<String>,
    status: &str,
) -> Result<(), String> {
    let _lock = storage::acquire_storage_lock()?;
    let mut manifest = storage::load_manifest()?;
    if manifest.tasks.iter().any(|task| task.id == id) {
        return Err(format!("Task '{}' already exists.", id));
    }
    let workspace_name = resolve_workspace_name(&manifest, workspace)?;
    let folder_name = resolve_task_folder_name(workspace_name.as_deref(), folder_name)?;
    let model = match model {
        Some(model) => Some(model),
        None => default_task_model(workspace_name.as_deref(), thinking.as_deref())?,
    };
    validate_blocking_reference(id, blocked_by.as_deref())?;
    let now = timestamp_now();
    let mut normalized_status = effective_task_status(status, blocked_by.as_ref());
    if normalized_status == "draft" && is_coding_agent(agent.as_deref()) {
        normalized_status = "running".to_string();
    }
    let should_launch_on_create = normalized_status == "running";
    let mut task = TaskRecord {
        id: id.to_string(),
        parent_task_id: if parent_task_id.is_empty() {
            None
        } else {
            Some(parent_task_id.to_string())
        },
        blocked_by,
        blocked_reason,
        workspace: workspace_name,
        folder_name,
        agent,
        model,
        thinking: thinking.map(|value| value.to_ascii_lowercase()),
        prompt,
        command,
        pid,
        session_id,
        notes,
        result,
        status: normalized_status,
        created_at: now.clone(),
        updated_at: now.clone(),
        started_at: if should_launch_on_create {
            Some(now.clone())
        } else {
            None
        },
        ended_at: if is_finished_status(status) {
            Some(now)
        } else {
            None
        },
    };
    storage::save_task(&task)?;
    let mut markdown_path = write_task_markdown(&task)?;
    let command_execution = match run_agent_task_action(task.agent.as_deref(), "create", &task) {
        Ok(value) => value,
        Err(err) => {
            let _ = storage::remove_task(id);
            return Err(err);
        }
    };
    let mut task_start_system_prompt_path = None;
    let mut task_start_execution = None;
    if should_launch_on_create {
        let prompt_path = ensure_task_start_system_prompt(&task)?;
        let start_execution = match run_agent_task_action(task.agent.as_deref(), "start", &task) {
            Ok(value) => value,
            Err(err) => {
                let _ = storage::remove_task(id);
                return Err(err);
            }
        };
        if task.pid.is_none() {
            if let Some(pid) = extract_agent_execution_pid(&start_execution) {
                task.pid = Some(pid);
                task.updated_at = timestamp_now();
                storage::save_task(&task)?;
                markdown_path = write_task_markdown(&task)?;
            }
        }
        task_start_system_prompt_path = Some(prompt_path);
        task_start_execution = start_execution;
    }
    storage::upsert_task_summary(&mut manifest, &task);
    storage::save_manifest(&manifest)?;
    let task_summary = find_task_summary(&manifest, id)?;
    print_json_pretty(&json!({
        "status": "ok",
        "command": "task.create",
        "task": task_summary,
        "record": task,
        "task_full_name": task_full_name(&task),
        "task_markdown_path": markdown_path,
        "task_start_system_prompt_path": task_start_system_prompt_path,
        "agent_command_template_path": storage::agent_commands_path(),
        "agent_command_execution": command_execution,
        "task_start_execution": task_start_execution,
        "manifest_path": storage::manifest_path(),
        "file_path": storage::task_file(id),
    }));
    Ok(())
}

fn task_list(
    workspace: Option<String>,
    status: Option<String>,
    agent: Option<String>,
) -> Result<(), String> {
    let manifest = storage::load_manifest()?;
    let workspace_filter = workspace.as_deref();
    let status_filter = status.as_deref().map(|value| value.to_ascii_lowercase());
    let agent_filter = agent.as_deref();
    let tasks: Vec<TaskSummary> = manifest
        .tasks
        .into_iter()
        .filter(|task| workspace_filter.is_none() || task.workspace.as_deref() == workspace_filter)
        .filter(|task| {
            status_filter
                .as_deref()
                .map(|expected| task.status == expected)
                .unwrap_or(true)
        })
        .filter(|task| agent_filter.is_none() || task.agent.as_deref() == agent_filter)
        .collect();
    print_json_pretty(&json!({
        "status": "ok",
        "command": "task.list",
        "count": tasks.len(),
        "filters": {
            "workspace": workspace_filter,
            "status": status_filter,
            "agent": agent_filter,
        },
        "tasks": tasks,
    }));
    Ok(())
}

fn task_show(id: &str) -> Result<(), String> {
    let task = storage::load_task(id)?;
    print_json_pretty(&json!({
        "status": "ok",
        "command": "task.show",
        "task": task,
        "file_path": storage::task_file(id),
    }));
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn task_update(
    id: &str,
    workspace: Option<String>,
    folder_name: Option<String>,
    agent: Option<String>,
    model: Option<String>,
    thinking: Option<String>,
    prompt: Option<String>,
    command: Option<String>,
    pid: Option<u32>,
    session_id: Option<String>,
    notes: Option<String>,
    result: Option<String>,
    blocked_by: Option<String>,
    blocked_reason: Option<String>,
    status: Option<String>,
    parent_task_id: Option<String>,
) -> Result<(), String> {
    let _lock = storage::acquire_storage_lock()?;
    let mut manifest = storage::load_manifest()?;
    let mut task = storage::load_task(id)?;
    let previous = task.clone();
    if let Some(workspace) = workspace {
        ensure_workspace_exists(&manifest, &workspace)?;
        task.workspace = Some(workspace);
    }
    if let Some(folder_name) = folder_name {
        task.folder_name = Some(folder_name);
    }
    if let Some(parent_task_id) = parent_task_id {
        task.parent_task_id = Some(parent_task_id);
    }
    if let Some(blocked_by) = blocked_by {
        validate_blocking_reference(id, Some(blocked_by.as_str()))?;
        task.blocked_by = Some(blocked_by);
    }
    if let Some(blocked_reason) = blocked_reason {
        task.blocked_reason = Some(blocked_reason);
    }
    if task.workspace.is_some() {
        task.folder_name = resolve_task_folder_name(task.workspace.as_deref(), task.folder_name)?;
    }
    if let Some(agent) = agent {
        task.agent = Some(agent);
    }
    if let Some(model) = model {
        task.model = Some(model);
    }
    if let Some(thinking) = thinking {
        task.thinking = Some(thinking.to_ascii_lowercase());
    }
    if task.model.is_none() {
        task.model = default_task_model(task.workspace.as_deref(), task.thinking.as_deref())?;
    }
    if let Some(prompt) = prompt {
        task.prompt = Some(prompt);
    }
    if let Some(command) = command {
        task.command = Some(command);
    }
    if let Some(pid) = pid {
        task.pid = Some(pid);
    }
    if let Some(session_id) = session_id {
        task.session_id = Some(session_id);
    }
    if let Some(notes) = notes {
        task.notes = Some(notes);
    }
    if let Some(result) = result {
        task.result = Some(result);
    }
    if let Some(status) = status {
        task.status = status.to_ascii_lowercase();
        if task.status == "running" && task.started_at.is_none() {
            task.started_at = Some(timestamp_now());
        }
        if is_finished_status(&task.status) {
            task.ended_at = Some(timestamp_now());
        } else {
            task.ended_at = None;
        }
    }
    if task.blocked_by.is_some() {
        task.status = "blocked".to_string();
    }
    task.updated_at = timestamp_now();
    storage::save_task(&task)?;
    let markdown_path = write_task_markdown(&task)?;
    let command_execution = match run_agent_task_action(task.agent.as_deref(), "update", &task) {
        Ok(value) => value,
        Err(err) => {
            restore_task_after_launch_failure(&previous)?;
            return Err(err);
        }
    };
    storage::upsert_task_summary(&mut manifest, &task);
    storage::save_manifest(&manifest)?;
    let task_summary = find_task_summary(&manifest, id)?;
    print_json_pretty(&json!({
        "status": "ok",
        "command": "task.update",
        "task": task_summary,
        "record": task,
        "task_full_name": task_full_name(&task),
        "task_markdown_path": markdown_path,
        "agent_command_template_path": storage::agent_commands_path(),
        "agent_command_execution": command_execution,
        "file_path": storage::task_file(id),
    }));
    Ok(())
}

fn task_start(id: &str, pid: Option<u32>, session_id: Option<String>) -> Result<(), String> {
    let _lock = storage::acquire_storage_lock()?;
    let mut manifest = storage::load_manifest()?;
    let mut task = storage::load_task(id)?;
    ensure_not_finished(&task, "start")?;
    ensure_not_blocked(&task, "start")?;
    let previous = task.clone();
    task.status = "running".to_string();
    if let Some(pid) = pid {
        task.pid = Some(pid);
    }
    if let Some(session_id) = session_id {
        task.session_id = Some(session_id);
    }
    let now = timestamp_now();
    task.started_at = Some(now.clone());
    task.updated_at = now;
    task.ended_at = None;
    storage::save_task(&task)?;
    let mut markdown_path = write_task_markdown(&task)?;
    let task_start_system_prompt_path = ensure_task_start_system_prompt(&task)?;
    let command_execution = match run_agent_task_action(task.agent.as_deref(), "start", &task) {
        Ok(value) => value,
        Err(err) => {
            restore_task_after_launch_failure(&previous)?;
            return Err(err);
        }
    };
    if task.pid.is_none() {
        if let Some(pid) = extract_agent_execution_pid(&command_execution) {
            task.pid = Some(pid);
            task.updated_at = timestamp_now();
            storage::save_task(&task)?;
            markdown_path = write_task_markdown(&task)?;
        }
    }
    storage::upsert_task_summary(&mut manifest, &task);
    storage::save_manifest(&manifest)?;
    let task_summary = find_task_summary(&manifest, id)?;
    print_json_pretty(&json!({
        "status": "ok",
        "command": "task.start",
        "task": task_summary,
        "record": task,
        "task_full_name": task_full_name(&task),
        "task_markdown_path": markdown_path,
        "task_start_system_prompt_path": task_start_system_prompt_path,
        "agent_command_template_path": storage::agent_commands_path(),
        "agent_command_execution": command_execution,
    }));
    Ok(())
}

fn task_resume(id: &str, session_id: Option<String>, prompt: Option<String>) -> Result<(), String> {
    let _lock = storage::acquire_storage_lock()?;
    let mut manifest = storage::load_manifest()?;
    let mut task = storage::load_task(id)?;
    ensure_not_finished(&task, "resume")?;
    ensure_not_blocked(&task, "resume")?;
    let previous = task.clone();
    if let Some(session_id) = session_id {
        task.session_id = Some(session_id);
    }
    if let Some(prompt) = prompt {
        task.prompt = Some(prompt);
    }
    if task.session_id.as_deref().unwrap_or("").trim().is_empty() {
        return Err(format!(
            "Task '{}' has no session id. Pass `--session-id` or update the task first.",
            id
        ));
    }
    task.status = "running".to_string();
    let now = timestamp_now();
    task.started_at.get_or_insert(now.clone());
    task.updated_at = now;
    task.ended_at = None;
    storage::save_task(&task)?;
    let mut markdown_path = write_task_markdown(&task)?;
    let task_start_system_prompt_path = ensure_task_start_system_prompt(&task)?;
    let command_execution = match run_agent_task_action(task.agent.as_deref(), "resume", &task) {
        Ok(value) => value,
        Err(err) => {
            restore_task_after_launch_failure(&previous)?;
            return Err(err);
        }
    };
    if task.pid.is_none() {
        if let Some(pid) = extract_agent_execution_pid(&command_execution) {
            task.pid = Some(pid);
            task.updated_at = timestamp_now();
            storage::save_task(&task)?;
            markdown_path = write_task_markdown(&task)?;
        }
    }
    storage::upsert_task_summary(&mut manifest, &task);
    storage::save_manifest(&manifest)?;
    let task_summary = find_task_summary(&manifest, id)?;
    print_json_pretty(&json!({
        "status": "ok",
        "command": "task.resume",
        "task": task_summary,
        "record": task,
        "task_full_name": task_full_name(&task),
        "task_markdown_path": markdown_path,
        "task_start_system_prompt_path": task_start_system_prompt_path,
        "agent_command_template_path": storage::agent_commands_path(),
        "agent_command_execution": command_execution,
    }));
    Ok(())
}

fn task_append_note(id: &str, text: &str) -> Result<(), String> {
    let _lock = storage::acquire_storage_lock()?;
    let mut manifest = storage::load_manifest()?;
    let mut task = storage::load_task(id)?;
    let previous = task.clone();
    task.notes = Some(append_text(task.notes, text));
    task.updated_at = timestamp_now();
    storage::save_task(&task)?;
    let markdown_path = write_task_markdown(&task)?;
    let command_execution = match run_agent_task_action(task.agent.as_deref(), "note", &task) {
        Ok(value) => value,
        Err(err) => {
            restore_task_after_launch_failure(&previous)?;
            return Err(err);
        }
    };
    storage::upsert_task_summary(&mut manifest, &task);
    storage::save_manifest(&manifest)?;
    let task_summary = find_task_summary(&manifest, id)?;
    print_json_pretty(&json!({
        "status": "ok",
        "command": "task.note",
        "task": task_summary,
        "record": task,
        "task_full_name": task_full_name(&task),
        "task_markdown_path": markdown_path,
        "agent_command_template_path": storage::agent_commands_path(),
        "agent_command_execution": command_execution,
    }));
    Ok(())
}

fn task_append_result(id: &str, text: &str) -> Result<(), String> {
    let _lock = storage::acquire_storage_lock()?;
    let mut manifest = storage::load_manifest()?;
    let mut task = storage::load_task(id)?;
    let previous = task.clone();
    task.result = Some(append_text(task.result, text));
    task.updated_at = timestamp_now();
    storage::save_task(&task)?;
    let markdown_path = write_task_markdown(&task)?;
    let command_execution = match run_agent_task_action(task.agent.as_deref(), "result", &task) {
        Ok(value) => value,
        Err(err) => {
            restore_task_after_launch_failure(&previous)?;
            return Err(err);
        }
    };
    storage::upsert_task_summary(&mut manifest, &task);
    storage::save_manifest(&manifest)?;
    let task_summary = find_task_summary(&manifest, id)?;
    print_json_pretty(&json!({
        "status": "ok",
        "command": "task.result",
        "task": task_summary,
        "record": task,
        "task_full_name": task_full_name(&task),
        "task_markdown_path": markdown_path,
        "agent_command_template_path": storage::agent_commands_path(),
        "agent_command_execution": command_execution,
    }));
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn task_fork(
    new_id: &str,
    from: &str,
    workspace: Option<String>,
    folder_name: Option<String>,
    agent: Option<String>,
    model: Option<String>,
    thinking: Option<String>,
    prompt: Option<String>,
    notes: Option<String>,
    result: Option<String>,
    session_id: Option<String>,
    blocked_by: Option<String>,
    blocked_reason: Option<String>,
    status: &str,
) -> Result<(), String> {
    let parent = storage::load_task(from)?;
    let workspace = workspace.or(parent.workspace.clone());
    let folder_name = folder_name.or(parent.folder_name.clone());
    let agent = agent.or(parent.agent.clone());
    let model = model.or(parent.model.clone());
    let thinking = thinking.or(parent.thinking.clone());
    let prompt = prompt.or(parent.prompt.clone());
    let session_id = session_id.or(parent.session_id.clone());
    let notes = match (parent.notes.clone(), notes) {
        (Some(parent_notes), Some(extra)) => Some(format!("{}\n{}", parent_notes, extra)),
        (Some(parent_notes), None) => Some(parent_notes),
        (None, Some(extra)) => Some(extra),
        (None, None) => None,
    };
    task_create_from_parent(
        new_id,
        from,
        workspace,
        folder_name,
        agent,
        model,
        thinking,
        prompt,
        parent.command.clone(),
        None,
        session_id,
        notes,
        result,
        blocked_by,
        blocked_reason,
        status,
    )
}

fn task_kill(
    id: &str,
    pid: Option<u32>,
    reason: Option<String>,
    force: bool,
) -> Result<(), String> {
    let _lock = storage::acquire_storage_lock()?;
    let mut manifest = storage::load_manifest()?;
    let mut task = storage::load_task(id)?;
    let previous = task.clone();
    ensure_not_finished(&task, "kill")?;
    let effective_pid = pid.or(task.pid);
    let pid = effective_pid.ok_or_else(|| {
        format!(
            "Task '{}' has no recorded pid. Pass `--pid` to kill the process and mark the task.",
            id
        )
    })?;
    kill_process(pid, force)?;
    task.pid = Some(pid);
    task.status = "killed".to_string();
    task.updated_at = timestamp_now();
    task.ended_at = Some(task.updated_at.clone());
    if let Some(reason) = reason {
        task.notes = Some(match task.notes.take() {
            Some(existing) if !existing.is_empty() => {
                format!("{}\nkill_reason: {}", existing, reason)
            }
            _ => format!("kill_reason: {}", reason),
        });
    }
    storage::save_task(&task)?;
    let markdown_path = write_task_markdown(&task)?;
    let command_execution = match run_agent_task_action(task.agent.as_deref(), "kill", &task) {
        Ok(value) => value,
        Err(err) => {
            restore_task_after_launch_failure(&previous)?;
            return Err(err);
        }
    };
    storage::upsert_task_summary(&mut manifest, &task);
    storage::save_manifest(&manifest)?;
    let task_summary = find_task_summary(&manifest, id)?;
    print_json_pretty(&json!({
        "status": "ok",
        "command": "task.kill",
        "task": task_summary,
        "record": task,
        "task_full_name": task_full_name(&task),
        "task_markdown_path": markdown_path,
        "killed_pid": pid,
        "force": force,
        "agent_command_template_path": storage::agent_commands_path(),
        "agent_command_execution": command_execution,
    }));
    Ok(())
}

fn task_complete(
    id: &str,
    notes: Option<String>,
    result: Option<String>,
    close_terminal: bool,
    keep_terminal: bool,
) -> Result<(), String> {
    let _lock = storage::acquire_storage_lock()?;
    let mut manifest = storage::load_manifest()?;
    let mut task = storage::load_task(id)?;
    let previous = task.clone();
    ensure_not_finished(&task, "complete")?;
    if result
        .as_deref()
        .or(task.result.as_deref())
        .map(|value| value.trim().is_empty())
        .unwrap_or(true)
    {
        return Err(format!(
            "Task '{}' cannot be completed without a result. Pass `--result` with the final summary.",
            id
        ));
    }
    task.status = "completed".to_string();
    if let Some(notes) = notes {
        task.notes = Some(notes);
    }
    if let Some(result) = result {
        task.result = Some(result);
    }
    task.updated_at = timestamp_now();
    task.ended_at = Some(task.updated_at.clone());
    storage::save_task(&task)?;
    let markdown_path = write_task_markdown(&task)?;
    let command_execution = match run_agent_task_action(task.agent.as_deref(), "complete", &task) {
        Ok(value) => value,
        Err(err) => {
            restore_task_after_launch_failure(&previous)?;
            return Err(err);
        }
    };
    storage::upsert_task_summary(&mut manifest, &task);
    storage::save_manifest(&manifest)?;
    let task_summary = find_task_summary(&manifest, id)?;
    let should_close_terminal = (close_terminal || task.pid.is_some()) && !keep_terminal;
    let close_terminal_execution = if should_close_terminal {
        schedule_close_terminal(task.pid)
    } else {
        None
    };
    print_json_pretty(&json!({
        "status": "ok",
        "command": "task.complete",
        "task": task_summary,
        "record": task,
        "task_full_name": task_full_name(&task),
        "task_markdown_path": markdown_path,
        "close_terminal_execution": close_terminal_execution,
        "agent_command_template_path": storage::agent_commands_path(),
        "agent_command_execution": command_execution,
    }));
    Ok(())
}

fn task_fail(
    id: &str,
    reason: Option<String>,
    result: Option<String>,
    close_terminal: bool,
    keep_terminal: bool,
) -> Result<(), String> {
    let _lock = storage::acquire_storage_lock()?;
    let mut manifest = storage::load_manifest()?;
    let mut task = storage::load_task(id)?;
    let previous = task.clone();
    ensure_not_finished(&task, "fail")?;
    let final_result = result
        .or_else(|| reason.as_ref().map(|value| format!("failed: {}", value)))
        .ok_or_else(|| {
            format!(
                "Task '{}' cannot be failed without `--reason` or `--result`.",
                id
            )
        })?;
    if let Some(reason) = reason {
        task.notes = Some(append_text(task.notes, &format!("failure_reason: {}", reason)));
    }
    task.result = Some(final_result);
    task.status = "failed".to_string();
    task.updated_at = timestamp_now();
    task.ended_at = Some(task.updated_at.clone());
    storage::save_task(&task)?;
    let markdown_path = write_task_markdown(&task)?;
    let command_execution = match run_agent_task_action(task.agent.as_deref(), "fail", &task) {
        Ok(value) => value,
        Err(err) => {
            restore_task_after_launch_failure(&previous)?;
            return Err(err);
        }
    };
    storage::upsert_task_summary(&mut manifest, &task);
    storage::save_manifest(&manifest)?;
    let task_summary = find_task_summary(&manifest, id)?;
    let should_close_terminal = (close_terminal || task.pid.is_some()) && !keep_terminal;
    let close_terminal_execution = if should_close_terminal {
        schedule_close_terminal(task.pid)
    } else {
        None
    };
    print_json_pretty(&json!({
        "status": "ok",
        "command": "task.fail",
        "task": task_summary,
        "record": task,
        "task_full_name": task_full_name(&task),
        "task_markdown_path": markdown_path,
        "close_terminal_execution": close_terminal_execution,
        "agent_command_template_path": storage::agent_commands_path(),
        "agent_command_execution": command_execution,
    }));
    Ok(())
}

fn task_remove(id: &str) -> Result<(), String> {
    let _lock = storage::acquire_storage_lock()?;
    let mut manifest = storage::load_manifest()?;
    let task = storage::load_task(id)?;
    let command_execution = run_agent_task_action(task.agent.as_deref(), "remove", &task)?;
    let before = manifest.tasks.len();
    manifest.tasks.retain(|task| task.id != id);
    if before == manifest.tasks.len() {
        return Err(format!("Task '{}' was not found.", id));
    }
    manifest.updated_at = timestamp_now();
    storage::remove_task(id)?;
    storage::save_manifest(&manifest)?;
    print_json_pretty(&json!({
        "status": "ok",
        "command": "task.remove",
        "removed_task": id,
        "removed_task_dir": storage::task_dir(id),
        "agent_command_template_path": storage::agent_commands_path(),
        "agent_command_execution": command_execution,
    }));
    Ok(())
}

