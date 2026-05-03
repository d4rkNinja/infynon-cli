use crate::ninja::types::{
    timestamp_now, AgentCommandTemplates, NinjaManifest, TaskRecord, TaskSummary, UserIdentity,
    WorkspaceRecord, WorkspaceSummary,
};
use crate::utils::{ensure_dir, ensure_parent_dir, home_infynon_dir, safe_file_stem};
use std::fs;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const AGENT_ROOT_WORKSPACE_NAME: &str = "infynon-agent";

pub fn manifest_path() -> PathBuf {
    home_infynon_dir().join("ninja.yml")
}

pub fn agent_commands_path() -> PathBuf {
    home_infynon_dir().join("agent-commands.json")
}

pub fn user_identity_path() -> PathBuf {
    home_infynon_dir().join("user.json")
}

pub fn soul_path() -> PathBuf {
    home_infynon_dir().join("soul.md")
}

pub fn hidden_system_prompt_path() -> PathBuf {
    home_infynon_dir().join("ninja").join("systemprompt.md")
}

pub fn hidden_onboard_user_prompt_path() -> PathBuf {
    home_infynon_dir()
        .join("ninja")
        .join("onboarduser-prompt.md")
}

pub fn task_start_system_prompt_path(task_id: &str) -> PathBuf {
    home_infynon_dir().join("ninja").join(format!(
        "task-start-systemprompt-{}.md",
        safe_file_stem(task_id)
    ))
}

pub fn workspaces_dir() -> PathBuf {
    home_infynon_dir().join("workspaces")
}

pub fn tasks_dir() -> PathBuf {
    home_infynon_dir().join("tasks")
}

pub fn lock_path() -> PathBuf {
    home_infynon_dir().join("ninja.lock")
}

pub struct StorageLock {
    path: PathBuf,
}

impl Drop for StorageLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

pub fn acquire_storage_lock() -> Result<StorageLock, String> {
    ensure_dir(&home_infynon_dir()).map_err(|e| e.to_string())?;
    let path = lock_path();
    let pid = std::process::id();
    for _ in 0..600 {
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(_) => {
                let _ = fs::write(&path, pid.to_string());
                return Ok(StorageLock { path });
            }
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                thread::sleep(Duration::from_millis(50));
            }
            Err(err) => return Err(format!("Failed to acquire INFYNON storage lock: {}", err)),
        }
    }
    Err(format!(
        "Timed out waiting for INFYNON storage lock at {}.",
        path.display()
    ))
}

pub fn workspace_dir(name: &str) -> PathBuf {
    workspaces_dir().join(safe_file_stem(name))
}

pub fn workspace_file(name: &str) -> PathBuf {
    workspace_dir(name).join("config.json")
}

pub fn task_dir(id: &str) -> PathBuf {
    tasks_dir().join(safe_file_stem(id))
}

pub fn task_file(id: &str) -> PathBuf {
    task_dir(id).join("task.json")
}

pub fn task_markdown_file(id: &str, workspace: Option<&str>, folder_name: Option<&str>) -> PathBuf {
    let workspace = workspace.unwrap_or("no-workspace");
    let folder_name = folder_name.unwrap_or("no-folder");
    let file_name = format!(
        "{}-{}-{}.md",
        safe_file_stem(id),
        safe_file_stem(workspace),
        safe_file_stem(folder_name)
    );
    task_dir(id).join(file_name)
}

pub fn ensure_layout() -> Result<(), String> {
    ensure_dir(&home_infynon_dir()).map_err(|e| e.to_string())?;
    ensure_dir(&workspaces_dir()).map_err(|e| e.to_string())?;
    ensure_dir(&tasks_dir()).map_err(|e| e.to_string())?;
    ensure_agent_commands_file()?;
    Ok(())
}

pub fn load_user_identity() -> Result<Option<UserIdentity>, String> {
    ensure_dir(&home_infynon_dir()).map_err(|e| e.to_string())?;
    let path = user_identity_path();
    if !path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str::<UserIdentity>(&text)
        .map(Some)
        .map_err(|e| e.to_string())
}

pub fn save_user_identity(identity: &UserIdentity) -> Result<(), String> {
    let path = user_identity_path();
    ensure_parent_dir(&path).map_err(|e| e.to_string())?;
    let text = serde_json::to_string_pretty(identity).map_err(|e| e.to_string())?;
    atomic_write_text(&path, &text)
}

pub fn ensure_soul_file() -> Result<PathBuf, String> {
    let path = soul_path();
    ensure_parent_dir(&path).map_err(|e| e.to_string())?;
    if !path.exists() {
        fs::write(&path, "").map_err(|e| e.to_string())?;
    }
    Ok(path)
}

pub fn read_soul() -> Result<String, String> {
    let path = ensure_soul_file()?;
    fs::read_to_string(path).map_err(|e| e.to_string())
}

pub fn write_soul(content: &str) -> Result<PathBuf, String> {
    let path = soul_path();
    ensure_parent_dir(&path).map_err(|e| e.to_string())?;
    atomic_write_text(&path, content)?;
    Ok(path)
}

fn ensure_agent_commands_file() -> Result<(), String> {
    let path = agent_commands_path();
    let internal = internal_agent_command_templates()?;
    if !path.exists() {
        let text = serde_json::to_string_pretty(&internal).map_err(|e| e.to_string())?;
        atomic_write_text(&path, &text)?;
    } else {
        let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let mut existing: AgentCommandTemplates =
            serde_json::from_str(&text).map_err(|e| e.to_string())?;
        if merge_agent_command_templates(&mut existing, &internal) {
            let text = serde_json::to_string_pretty(&existing).map_err(|e| e.to_string())?;
            atomic_write_text(&path, &text)?;
        }
    }
    Ok(())
}

fn internal_agent_command_templates() -> Result<AgentCommandTemplates, String> {
    let text = include_str!("agent-commands.json");
    serde_json::from_str::<AgentCommandTemplates>(text).map_err(|e| e.to_string())
}

fn merge_agent_command_templates(
    target: &mut AgentCommandTemplates,
    source: &AgentCommandTemplates,
) -> bool {
    merge_agent_command_group(&mut target.codex, &source.codex)
        | merge_agent_command_group(&mut target.claude, &source.claude)
        | merge_agent_command_group(&mut target.gemini, &source.gemini)
}

fn merge_string(target: &mut String, source: &str) -> bool {
    if target.trim().is_empty() && !source.trim().is_empty() {
        *target = source.to_string();
        true
    } else {
        false
    }
}

fn clear_legacy_task_show_command(target: &mut String) -> bool {
    if target.trim() == "infynon task show {task_id}" {
        target.clear();
        true
    } else {
        false
    }
}

fn normalize_agent_command(target: &mut String) -> bool {
    let original = target.clone();
    if target.contains("--append-system-prompt-file \"{system_prompt_path}\"") {
        *target = target.replace(
            "--append-system-prompt-file \"{system_prompt_path}\"",
            "--append-system-prompt {quoted_system_prompt}",
        );
    }
    if target.contains("--append-system-prompt-file \"{task_start_system_prompt_path}\"") {
        *target = target.replace(
            "--append-system-prompt-file \"{task_start_system_prompt_path}\"",
            "--append-system-prompt {quoted_task_start_system_prompt}",
        );
    }
    if target.contains("codex resume")
        && target.contains("--no-alt-screen")
        && !target.contains("--yolo")
    {
        *target = target.replace("--no-alt-screen", "--yolo --no-alt-screen");
    }
    *target != original
}

fn merge_agent_command_group(
    target: &mut crate::ninja::types::AgentCommandGroup,
    source: &crate::ninja::types::AgentCommandGroup,
) -> bool {
    let mut changed = false;
    changed |= merge_string(&mut target.open, &source.open);
    changed |= merge_string(&mut target.bootstrap, &source.bootstrap);
    changed |= merge_string(
        &mut target.bootstrap_background,
        &source.bootstrap_background,
    );
    changed |= merge_string(&mut target.task.create, &source.task.create);
    changed |= merge_string(&mut target.task.start, &source.task.start);
    changed |= merge_string(&mut target.task.resume, &source.task.resume);
    changed |= merge_string(&mut target.task.note, &source.task.note);
    changed |= merge_string(&mut target.task.update, &source.task.update);
    changed |= merge_string(&mut target.task.result, &source.task.result);
    changed |= merge_string(&mut target.task.complete, &source.task.complete);
    changed |= merge_string(&mut target.task.fail, &source.task.fail);
    changed |= merge_string(&mut target.task.kill, &source.task.kill);
    changed |= merge_string(&mut target.task.remove, &source.task.remove);
    changed |= clear_legacy_task_show_command(&mut target.task.create);
    changed |= clear_legacy_task_show_command(&mut target.task.note);
    changed |= clear_legacy_task_show_command(&mut target.task.update);
    changed |= clear_legacy_task_show_command(&mut target.task.result);
    changed |= clear_legacy_task_show_command(&mut target.task.complete);
    changed |= clear_legacy_task_show_command(&mut target.task.fail);
    changed |= clear_legacy_task_show_command(&mut target.task.kill);
    changed |= clear_legacy_task_show_command(&mut target.task.remove);
    changed |= normalize_agent_command(&mut target.open);
    changed |= normalize_agent_command(&mut target.bootstrap);
    changed |= normalize_agent_command(&mut target.bootstrap_background);
    changed |= normalize_agent_command(&mut target.task.start);
    changed |= normalize_agent_command(&mut target.task.resume);
    changed
}

pub fn load_manifest() -> Result<NinjaManifest, String> {
    ensure_layout()?;
    let path = manifest_path();
    if !path.exists() {
        let manifest = NinjaManifest::default();
        save_manifest(&manifest)?;
        return Ok(manifest);
    }
    let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut manifest = serde_yaml::from_str::<NinjaManifest>(&text).map_err(|e| e.to_string())?;
    if repair_agent_root_default(&mut manifest) {
        save_manifest(&manifest)?;
    }
    Ok(manifest)
}

pub fn save_manifest(manifest: &NinjaManifest) -> Result<(), String> {
    ensure_layout()?;
    let path = manifest_path();
    ensure_parent_dir(&path).map_err(|e| e.to_string())?;
    let text = serde_yaml::to_string(manifest).map_err(|e| e.to_string())?;
    atomic_write_text(&path, &text)
}

pub fn load_agent_commands() -> Result<AgentCommandTemplates, String> {
    ensure_layout()?;
    let path = agent_commands_path();
    let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str::<AgentCommandTemplates>(&text).map_err(|e| e.to_string())
}

pub fn load_internal_agent_commands() -> Result<AgentCommandTemplates, String> {
    internal_agent_command_templates()
}

pub fn ensure_hidden_system_prompt_file() -> Result<PathBuf, String> {
    let path = hidden_system_prompt_path();
    ensure_parent_dir(&path).map_err(|e| e.to_string())?;
    let text = rendered_bootstrap_system_prompt()?;
    let should_write = match fs::read_to_string(&path) {
        Ok(existing) => existing != text,
        Err(_) => true,
    };
    if should_write {
        atomic_write_text(&path, &text)?;
    }
    ensure_hidden_onboard_user_prompt_file()?;
    mark_hidden_best_effort(&home_infynon_dir());
    if let Some(parent) = path.parent() {
        mark_hidden_best_effort(parent);
    }
    mark_hidden_best_effort(&path);
    Ok(path)
}

pub fn ensure_hidden_onboard_user_prompt_file() -> Result<PathBuf, String> {
    let path = hidden_onboard_user_prompt_path();
    ensure_parent_dir(&path).map_err(|e| e.to_string())?;
    let text = include_str!("onboarduser-prompt.md");
    let should_write = match fs::read_to_string(&path) {
        Ok(existing) => existing != text,
        Err(_) => true,
    };
    if should_write {
        atomic_write_text(&path, text)?;
    }
    mark_hidden_best_effort(&home_infynon_dir());
    if let Some(parent) = path.parent() {
        mark_hidden_best_effort(parent);
    }
    mark_hidden_best_effort(&path);
    Ok(path)
}

fn rendered_bootstrap_system_prompt() -> Result<String, String> {
    let mut prompt = include_str!("systemprompt.md").to_string();
    if read_soul()?.trim().is_empty() {
        prompt.push_str("\n\n");
        prompt.push_str(include_str!("onboarduser-prompt.md"));
    }
    Ok(prompt)
}

pub fn write_task_start_system_prompt(task_id: &str, content: &str) -> Result<PathBuf, String> {
    let path = task_start_system_prompt_path(task_id);
    ensure_parent_dir(&path).map_err(|e| e.to_string())?;
    atomic_write_text(&path, content)?;
    mark_hidden_best_effort(&home_infynon_dir());
    if let Some(parent) = path.parent() {
        mark_hidden_best_effort(parent);
    }
    mark_hidden_best_effort(&path);
    Ok(path)
}

fn mark_hidden_best_effort(path: &std::path::Path) {
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("attrib")
            .arg("+h")
            .arg(path)
            .status();
    }

    #[cfg(not(windows))]
    {
        let _ = path;
    }
}

pub fn load_workspace(name: &str) -> Result<WorkspaceRecord, String> {
    let path = workspace_file(name);
    let text =
        fs::read_to_string(&path).map_err(|_| format!("Workspace '{}' was not found.", name))?;
    serde_json::from_str::<WorkspaceRecord>(&text).map_err(|e| e.to_string())
}

pub fn save_workspace(workspace: &WorkspaceRecord) -> Result<(), String> {
    let path = workspace_file(&workspace.name);
    ensure_parent_dir(&path).map_err(|e| e.to_string())?;
    let text = serde_json::to_string_pretty(workspace).map_err(|e| e.to_string())?;
    atomic_write_text(&path, &text)
}

pub fn remove_workspace(name: &str) -> Result<(), String> {
    let path = workspace_dir(name);
    if path.exists() {
        fs::remove_dir_all(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn load_task(id: &str) -> Result<TaskRecord, String> {
    let path = task_file(id);
    let text = fs::read_to_string(&path).map_err(|_| format!("Task '{}' was not found.", id))?;
    serde_json::from_str::<TaskRecord>(&text).map_err(|e| e.to_string())
}

pub fn save_task(task: &TaskRecord) -> Result<(), String> {
    let path = task_file(&task.id);
    ensure_parent_dir(&path).map_err(|e| e.to_string())?;
    let text = serde_json::to_string_pretty(task).map_err(|e| e.to_string())?;
    atomic_write_text(&path, &text)
}

pub fn remove_task(id: &str) -> Result<(), String> {
    let path = task_dir(id);
    if path.exists() {
        fs::remove_dir_all(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn replace_task_markdown(task: &TaskRecord, content: &str) -> Result<PathBuf, String> {
    let dir = task_dir(&task.id);
    ensure_dir(&dir).map_err(|e| e.to_string())?;
    let path = task_markdown_file(
        &task.id,
        task.workspace.as_deref(),
        task.folder_name.as_deref(),
    );
    atomic_write_text(&path, content)?;
    let existing = fs::read_dir(&dir).map_err(|e| e.to_string())?;
    for entry in existing {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| value.eq_ignore_ascii_case("md"))
            .unwrap_or(false)
            && path
                != task_markdown_file(
                    &task.id,
                    task.workspace.as_deref(),
                    task.folder_name.as_deref(),
                )
        {
            fs::remove_file(path).map_err(|e| e.to_string())?;
        }
    }
    Ok(path)
}

fn atomic_write_text(path: &std::path::Path, content: &str) -> Result<(), String> {
    ensure_parent_dir(path).map_err(|e| e.to_string())?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or_default();
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("write");
    let temp_path = path.with_file_name(format!(
        ".{}.{}.{}.tmp",
        file_name,
        std::process::id(),
        stamp
    ));
    fs::write(&temp_path, content).map_err(|e| e.to_string())?;
    #[cfg(windows)]
    {
        if path.exists() {
            fs::remove_file(path).map_err(|e| e.to_string())?;
        }
    }
    fs::rename(&temp_path, path).map_err(|e| {
        let _ = fs::remove_file(&temp_path);
        e.to_string()
    })
}

pub fn upsert_workspace_summary(manifest: &mut NinjaManifest, workspace: &WorkspaceRecord) {
    let summary = WorkspaceSummary {
        name: workspace.name.clone(),
        folder_name: workspace.folder_name.clone(),
        path: workspace.path.clone(),
        folders: workspace.folders.clone(),
        models: workspace.models.clone(),
        description: workspace.description.clone(),
        created_at: workspace.created_at.clone(),
        updated_at: workspace.updated_at.clone(),
    };
    if let Some(existing) = manifest
        .workspaces
        .iter_mut()
        .find(|item| item.name == workspace.name)
    {
        *existing = summary;
    } else {
        manifest.workspaces.push(summary);
    }
    manifest.workspaces.sort_by(|a, b| a.name.cmp(&b.name));
    manifest.updated_at = timestamp_now();
}

pub fn upsert_task_summary(manifest: &mut NinjaManifest, task: &TaskRecord) {
    let summary = TaskSummary {
        id: task.id.clone(),
        parent_task_id: task.parent_task_id.clone(),
        blocked_by: task.blocked_by.clone(),
        blocked_reason: task.blocked_reason.clone(),
        workspace: task.workspace.clone(),
        folder_name: task.folder_name.clone(),
        status: task.status.clone(),
        agent: task.agent.clone(),
        model: task.model.clone(),
        thinking: task.thinking.clone(),
        pid: task.pid,
        session_id: task.session_id.clone(),
        markdown_path: Some(
            task_markdown_file(
                &task.id,
                task.workspace.as_deref(),
                task.folder_name.as_deref(),
            )
            .display()
            .to_string(),
        ),
        created_at: task.created_at.clone(),
        updated_at: task.updated_at.clone(),
    };
    if let Some(existing) = manifest.tasks.iter_mut().find(|item| item.id == task.id) {
        *existing = summary;
    } else {
        manifest.tasks.push(summary);
    }
    manifest.tasks.sort_by(|a, b| a.id.cmp(&b.id));
    manifest.updated_at = timestamp_now();
}

fn repair_agent_root_default(manifest: &mut NinjaManifest) -> bool {
    if manifest.default_workspace.as_deref() != Some(AGENT_ROOT_WORKSPACE_NAME) {
        return false;
    }
    let Some(project_workspace) = manifest
        .workspaces
        .iter()
        .find(|workspace| workspace.name != AGENT_ROOT_WORKSPACE_NAME)
        .map(|workspace| workspace.name.clone())
    else {
        return false;
    };
    manifest.default_workspace = Some(project_workspace);
    manifest.updated_at = timestamp_now();
    true
}
