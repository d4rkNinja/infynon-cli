const AGENT_ROOT_WORKSPACE_NAME: &str = "infynon-agent";
const AGENT_ROOT_FOLDER_NAME: &str = "root";

fn workspace_create(
    name: &str,
    folder_name: Option<String>,
    path: Option<String>,
    description: Option<String>,
    default: bool,
    model_inputs: WorkspaceModelInputs,
) -> Result<(), String> {
    let _lock = storage::acquire_storage_lock()?;
    let mut manifest = storage::load_manifest()?;
    if manifest.workspaces.iter().any(|ws| ws.name == name) {
        return Err(format!("Workspace '{}' already exists.", name));
    }
    let now = timestamp_now();
    let folders = workspace_folders_from_inputs(folder_name, path)?;
    let workspace = WorkspaceRecord {
        name: name.to_string(),
        folder_name: folders.first().map(|folder| folder.folder_name.clone()),
        path: folders.first().map(|folder| folder.path.clone()),
        folders,
        models: build_workspace_models(model_inputs),
        description,
        created_at: now.clone(),
        updated_at: now,
    };
    storage::save_workspace(&workspace)?;
    storage::upsert_workspace_summary(&mut manifest, &workspace);
    if default || manifest.default_workspace.is_none() {
        manifest.default_workspace = Some(name.to_string());
    }
    storage::save_manifest(&manifest)?;
    let workspace_summary = find_workspace_summary(&manifest, name)?;
    print_json_pretty(&json!({
        "status": "ok",
        "command": "workspace.create",
        "workspace": workspace_summary,
        "default_workspace": manifest.default_workspace,
        "manifest_path": storage::manifest_path(),
        "file_path": storage::workspace_file(name),
    }));
    Ok(())
}

fn workspace_list() -> Result<(), String> {
    let manifest = storage::load_manifest()?;
    let count = manifest.workspaces.len();
    print_json_pretty(&json!({
        "status": "ok",
        "command": "workspace.list",
        "count": count,
        "default_workspace": manifest.default_workspace,
        "workspaces": manifest.workspaces,
    }));
    Ok(())
}

fn workspace_show(name: &str) -> Result<(), String> {
    let workspace = storage::load_workspace(name)?;
    let manifest = storage::load_manifest()?;
    print_json_pretty(&json!({
        "status": "ok",
        "command": "workspace.show",
        "default": manifest.default_workspace.as_deref() == Some(name),
        "workspace": workspace,
        "file_path": storage::workspace_file(name),
    }));
    Ok(())
}

fn workspace_agent_root_show() -> Result<(), String> {
    let manifest = storage::load_manifest()?;
    let workspace = storage::load_workspace(AGENT_ROOT_WORKSPACE_NAME).ok();
    print_json_pretty(&json!({
        "status": "ok",
        "command": "workspace.agent-root-show",
        "configured": manifest.agent_root_path.is_some(),
        "agent_root_path": manifest.agent_root_path,
        "agent_root_workspace": AGENT_ROOT_WORKSPACE_NAME,
        "agent_root_folder": AGENT_ROOT_FOLDER_NAME,
        "workspace": workspace,
        "manifest_path": storage::manifest_path(),
        "setup_command": "infynon workspace agent-root-set --mutate --path <absolute-directory-path>",
    }));
    Ok(())
}

fn workspace_agent_root_set(path: &str) -> Result<(), String> {
    let _lock = storage::acquire_storage_lock()?;
    let mut manifest = storage::load_manifest()?;
    let path = std::fs::canonicalize(path)
        .map_err(|e| format!("Failed to resolve agent root path '{}': {}", path, e))?
        .display()
        .to_string();
    let now = timestamp_now();
    let mut workspace =
        storage::load_workspace(AGENT_ROOT_WORKSPACE_NAME).unwrap_or_else(|_| WorkspaceRecord {
            name: AGENT_ROOT_WORKSPACE_NAME.to_string(),
            folder_name: Some(AGENT_ROOT_FOLDER_NAME.to_string()),
            path: Some(path.clone()),
            folders: Vec::new(),
            models: WorkspaceModels::default(),
            description: Some("INFYNON agent root workspace.".to_string()),
            created_at: now.clone(),
            updated_at: now.clone(),
        });
    workspace.folder_name = Some(AGENT_ROOT_FOLDER_NAME.to_string());
    workspace.path = Some(path.clone());
    workspace.folders = vec![WorkspaceFolder {
        folder_name: AGENT_ROOT_FOLDER_NAME.to_string(),
        path: path.clone(),
    }];
    workspace.description = Some("INFYNON agent root workspace.".to_string());
    workspace.updated_at = timestamp_now();
    storage::save_workspace(&workspace)?;
    storage::upsert_workspace_summary(&mut manifest, &workspace);
    manifest.agent_root_path = Some(path.clone());
    storage::save_manifest(&manifest)?;
    print_json_pretty(&json!({
        "status": "ok",
        "command": "workspace.agent-root-set",
        "agent_root_path": path,
        "agent_root_workspace": AGENT_ROOT_WORKSPACE_NAME,
        "agent_root_folder": AGENT_ROOT_FOLDER_NAME,
        "default_workspace": manifest.default_workspace,
        "manifest_path": storage::manifest_path(),
        "workspace_file_path": storage::workspace_file(AGENT_ROOT_WORKSPACE_NAME),
    }));
    Ok(())
}

fn workspace_update(
    name: &str,
    folder_name: Option<String>,
    path: Option<String>,
    description: Option<String>,
    default: bool,
    model_inputs: WorkspaceModelInputs,
) -> Result<(), String> {
    let _lock = storage::acquire_storage_lock()?;
    let mut manifest = storage::load_manifest()?;
    let mut workspace = storage::load_workspace(name)?;
    if folder_name.is_some() && path.is_some() {
        workspace.folders = workspace_folders_from_inputs(folder_name, path)?;
    }
    if description.is_some() {
        workspace.description = description;
    }
    apply_workspace_models(&mut workspace.models, model_inputs);
    sync_workspace_primary_fields(&mut workspace);
    workspace.updated_at = timestamp_now();
    storage::save_workspace(&workspace)?;
    storage::upsert_workspace_summary(&mut manifest, &workspace);
    if default {
        manifest.default_workspace = Some(name.to_string());
    }
    storage::save_manifest(&manifest)?;
    let workspace_summary = find_workspace_summary(&manifest, name)?;
    print_json_pretty(&json!({
        "status": "ok",
        "command": "workspace.update",
        "workspace": workspace_summary,
        "default_workspace": manifest.default_workspace,
        "file_path": storage::workspace_file(name),
    }));
    Ok(())
}

fn workspace_add_folder(name: &str, folder_name: &str, path: &str) -> Result<(), String> {
    let _lock = storage::acquire_storage_lock()?;
    let mut manifest = storage::load_manifest()?;
    let mut workspace = storage::load_workspace(name)?;
    if workspace
        .folders
        .iter()
        .any(|folder| folder.folder_name == folder_name)
    {
        return Err(format!(
            "Workspace '{}' already has folder '{}'.",
            name, folder_name
        ));
    }
    if workspace.folders.iter().any(|folder| folder.path == path) {
        return Err(format!("Workspace '{}' already has path '{}'.", name, path));
    }
    workspace.folders.push(WorkspaceFolder {
        folder_name: folder_name.to_string(),
        path: path.to_string(),
    });
    workspace
        .folders
        .sort_by(|a, b| a.folder_name.cmp(&b.folder_name));
    sync_workspace_primary_fields(&mut workspace);
    workspace.updated_at = timestamp_now();
    storage::save_workspace(&workspace)?;
    storage::upsert_workspace_summary(&mut manifest, &workspace);
    storage::save_manifest(&manifest)?;
    let workspace_summary = find_workspace_summary(&manifest, name)?;
    print_json_pretty(&json!({
        "status": "ok",
        "command": "workspace.add-folder",
        "workspace": workspace_summary,
        "added_folder": {
            "folder_name": folder_name,
            "path": path,
        },
        "file_path": storage::workspace_file(name),
    }));
    Ok(())
}

fn workspace_remove_folder(name: &str, folder_name: &str) -> Result<(), String> {
    let _lock = storage::acquire_storage_lock()?;
    let mut manifest = storage::load_manifest()?;
    let mut workspace = storage::load_workspace(name)?;
    if manifest.tasks.iter().any(|task| {
        task.workspace.as_deref() == Some(name) && task.folder_name.as_deref() == Some(folder_name)
    }) {
        return Err(format!(
            "Folder '{}' is still referenced by tasks in workspace '{}'.",
            folder_name, name
        ));
    }
    let before = workspace.folders.len();
    workspace
        .folders
        .retain(|folder| folder.folder_name != folder_name);
    if before == workspace.folders.len() {
        return Err(format!(
            "Workspace '{}' does not contain folder '{}'.",
            name, folder_name
        ));
    }
    sync_workspace_primary_fields(&mut workspace);
    workspace.updated_at = timestamp_now();
    storage::save_workspace(&workspace)?;
    storage::upsert_workspace_summary(&mut manifest, &workspace);
    storage::save_manifest(&manifest)?;
    let workspace_summary = find_workspace_summary(&manifest, name)?;
    print_json_pretty(&json!({
        "status": "ok",
        "command": "workspace.remove-folder",
        "workspace": workspace_summary,
        "removed_folder_name": folder_name,
        "file_path": storage::workspace_file(name),
    }));
    Ok(())
}

fn workspace_remove(name: &str) -> Result<(), String> {
    let _lock = storage::acquire_storage_lock()?;
    let mut manifest = storage::load_manifest()?;
    if manifest
        .tasks
        .iter()
        .any(|task| task.workspace.as_deref() == Some(name))
    {
        return Err(format!(
            "Workspace '{}' is still referenced by tasks. Remove or reassign those tasks first.",
            name
        ));
    }
    let before = manifest.workspaces.len();
    manifest.workspaces.retain(|workspace| workspace.name != name);
    if before == manifest.workspaces.len() {
        return Err(format!("Workspace '{}' was not found.", name));
    }
    if manifest.default_workspace.as_deref() == Some(name) {
        manifest.default_workspace = manifest.workspaces.first().map(|workspace| workspace.name.clone());
    }
    if manifest.agent_root_path.is_some() && name == AGENT_ROOT_WORKSPACE_NAME {
        manifest.agent_root_path = None;
    }
    manifest.updated_at = timestamp_now();
    storage::remove_workspace(name)?;
    storage::save_manifest(&manifest)?;
    print_json_pretty(&json!({
        "status": "ok",
        "command": "workspace.remove",
        "removed_workspace": name,
        "removed_workspace_dir": storage::workspace_dir(name),
        "default_workspace": manifest.default_workspace,
        "manifest_path": storage::manifest_path(),
    }));
    Ok(())
}
