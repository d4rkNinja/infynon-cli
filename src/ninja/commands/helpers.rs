fn resolve_workspace_name(
    manifest: &crate::ninja::types::NinjaManifest,
    workspace: Option<String>,
) -> Result<Option<String>, String> {
    match workspace {
        Some(name) => {
            ensure_workspace_exists(manifest, &name)?;
            Ok(Some(name))
        }
        None => Ok(manifest.default_workspace.clone()),
    }
}

fn ensure_workspace_exists(
    manifest: &crate::ninja::types::NinjaManifest,
    name: &str,
) -> Result<(), String> {
    if manifest
        .workspaces
        .iter()
        .any(|workspace| workspace.name == name)
    {
        Ok(())
    } else {
        Err(format!("Workspace '{}' was not found.", name))
    }
}

fn sync_workspace_primary_fields(workspace: &mut WorkspaceRecord) {
    if workspace.folders.is_empty() {
        workspace.folder_name = None;
        workspace.path = None;
    } else {
        workspace.folder_name = workspace
            .folders
            .first()
            .map(|folder| folder.folder_name.clone());
        workspace.path = workspace.folders.first().map(|folder| folder.path.clone());
    }
}

fn workspace_folders_from_inputs(
    folder_name: Option<String>,
    path: Option<String>,
) -> Result<Vec<WorkspaceFolder>, String> {
    match (folder_name, path) {
        (Some(folder_name), Some(path)) => Ok(vec![WorkspaceFolder { folder_name, path }]),
        (None, None) => Ok(Vec::new()),
        _ => Err("`--folder-name` and `--path` must be provided together.".to_string()),
    }
}

#[derive(Debug)]
struct WorkspaceModelInputs {
    lite_model: Option<String>,
    lite_thinking: Option<String>,
    frontier_model: Option<String>,
    frontier_thinking: Option<String>,
    highest_frontier_model: Option<String>,
    highest_frontier_thinking: Option<String>,
    super_lite_model: Option<String>,
    super_lite_thinking: Option<String>,
}

fn build_workspace_models(inputs: WorkspaceModelInputs) -> WorkspaceModels {
    let mut models = WorkspaceModels::default();
    apply_workspace_models(&mut models, inputs);
    models
}

fn apply_workspace_models(models: &mut WorkspaceModels, inputs: WorkspaceModelInputs) {
    apply_workspace_model_slot(
        &mut models.lite_model,
        inputs.lite_model,
        inputs.lite_thinking,
    );
    apply_workspace_model_slot(
        &mut models.frontier_model,
        inputs.frontier_model,
        inputs.frontier_thinking,
    );
    apply_workspace_model_slot(
        &mut models.highest_frontier_model,
        inputs.highest_frontier_model,
        inputs.highest_frontier_thinking,
    );
    apply_workspace_model_slot(
        &mut models.super_lite_model,
        inputs.super_lite_model,
        inputs.super_lite_thinking,
    );
}

fn apply_workspace_model_slot(
    slot: &mut WorkspaceModelSlot,
    model: Option<String>,
    thinking: Option<String>,
) {
    if let Some(model) = model {
        slot.model = Some(model);
    }
    if let Some(thinking) = thinking {
        slot.thinking = thinking.to_ascii_lowercase();
    }
}

fn task_full_name(task: &TaskRecord) -> String {
    format!(
        "{}-{}-{}",
        task.id,
        task.workspace.as_deref().unwrap_or("no-workspace"),
        task.folder_name.as_deref().unwrap_or("no-folder")
    )
}

fn write_task_markdown(task: &TaskRecord) -> Result<String, String> {
    let markdown = format_task_markdown(task);
    let path = storage::replace_task_markdown(task, &markdown)?;
    Ok(path.display().to_string())
}

fn format_task_markdown(task: &TaskRecord) -> String {
    let description = task
        .prompt
        .as_deref()
        .or(task.notes.as_deref())
        .unwrap_or("");
    let result = task.result.as_deref().unwrap_or("");
    let start_time = task
        .started_at
        .as_deref()
        .unwrap_or(task.created_at.as_str());
    let end_time = task.ended_at.as_deref().unwrap_or("");
    let command = task.command.as_deref().unwrap_or("");
    let agent = task.agent.as_deref().unwrap_or("");
    let model = task.model.as_deref().unwrap_or("");
    let thinking = task.thinking.as_deref().unwrap_or("auto");
    let workspace = task.workspace.as_deref().unwrap_or("");
    let folder = task.folder_name.as_deref().unwrap_or("");
    let pid = task.pid.map(|value| value.to_string()).unwrap_or_default();
    let session_id = task.session_id.as_deref().unwrap_or("");
    let notes = task.notes.as_deref().unwrap_or("");
    let blocked_by = task.blocked_by.as_deref().unwrap_or("");
    let blocked_reason = task.blocked_reason.as_deref().unwrap_or("");

    format!(
        "# Task Tracking\n\n\
Task ID: {task_id}\n\
Parent Task ID: {parent_task_id}\n\
Blocked By Task ID: {blocked_by}\n\
Blocked Reason: {blocked_reason}\n\
Task Full Name: {task_full_name}\n\
Task Name: {task_name}\n\
Workspace Name: {workspace}\n\
Folder Name: {folder}\n\
Model Name: {model}\n\
Thinking Power: {thinking}\n\
Agent Name: {agent}\n\
Task Status: {status}\n\
Start Time: {start_time}\n\
End Time: {end_time}\n\
PID: {pid}\n\
Session ID: {session_id}\n\
Command: {command}\n\n\
## Task Description\n\n\
{description}\n\n\
## Task Notes\n\n\
{notes}\n\n\
## Task Results\n\n\
{result}\n",
        task_id = task.id,
        parent_task_id = task.parent_task_id.as_deref().unwrap_or(""),
        blocked_by = blocked_by,
        blocked_reason = blocked_reason,
        task_full_name = task_full_name(task),
        task_name = task.id,
        workspace = workspace,
        folder = folder,
        model = model,
        thinking = thinking,
        agent = agent,
        status = task.status,
        start_time = start_time,
        end_time = end_time,
        pid = pid,
        session_id = session_id,
        command = command,
        description = description,
        notes = notes,
        result = result,
    )
}

fn append_text(current: Option<String>, text: &str) -> String {
    match current {
        Some(existing) if !existing.trim().is_empty() => format!("{}\n{}", existing, text),
        _ => text.to_string(),
    }
}
