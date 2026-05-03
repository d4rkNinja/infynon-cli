fn render_project_agent_command_template(
    template: &str,
    agent: &str,
    system_prompt_path: Option<&str>,
    model: Option<&str>,
) -> String {
    let project_path = std::env::current_dir()
        .map(|path| path.display().to_string())
        .unwrap_or_default();
    let system_prompt_path = system_prompt_path.unwrap_or("");
    let system_prompt = if system_prompt_path.trim().is_empty() {
        String::new()
    } else {
        std::fs::read_to_string(system_prompt_path).unwrap_or_default()
    };
    let model_arg = model_arg(model.unwrap_or(""));
    let gemini_system_prompt_env = if cfg!(windows) {
        format!("$env:GEMINI_SYSTEM_MD = \"{}\";", system_prompt_path)
    } else {
        format!("GEMINI_SYSTEM_MD=\"{}\"", system_prompt_path)
    };
    template
        .replace("{agent}", agent)
        .replace("{project_path}", &project_path)
        .replace("{system_prompt_path}", system_prompt_path)
        .replace("{system_prompt}", &system_prompt)
        .replace("{quoted_system_prompt}", &shell_quote(&system_prompt))
        .replace("{model}", model.unwrap_or(""))
        .replace("{model_arg}", &model_arg)
        .replace("{gemini_system_prompt_env}", &gemini_system_prompt_env)
}

fn model_arg(model: &str) -> String {
    let model = model.trim();
    if model.is_empty() {
        String::new()
    } else {
        format!("--model {}", shell_quote(model))
    }
}

fn default_launch_context() -> Result<(Option<String>, Option<String>), String> {
    let manifest = storage::load_manifest()?;
    let Some(agent_root_path) = manifest
        .agent_root_path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Err(agent_root_missing_message());
    };
    let agent_root = std::path::Path::new(agent_root_path);
    if !agent_root.exists() || !agent_root.is_dir() {
        return Err(format!(
            "INFYNON agent root path '{}' is missing or is not a directory.\nAsk Stark for the correct absolute folder path, then run:\ninfynon workspace agent-root-set --mutate --path <absolute-directory-path>",
            agent_root_path
        ));
    }
    let Some(workspace_name) = manifest.default_workspace.as_deref() else {
        return Ok((Some(agent_root_path.to_string()), None));
    };
    let model = storage::load_workspace(workspace_name)
        .ok()
        .and_then(|workspace| select_workspace_model(&workspace, Some("medium")));
    Ok((Some(agent_root_path.to_string()), model))
}

fn agent_root_missing_message() -> String {
    "INFYNON agent root path is not configured.\nAsk Stark for the absolute folder path that INFYNON agents should use as their root workspace, then save it with:\ninfynon workspace agent-root-set --mutate --path <absolute-directory-path>\nAfter that, rerun the coding command.".to_string()
}

fn default_task_model(
    workspace: Option<&str>,
    thinking: Option<&str>,
) -> Result<Option<String>, String> {
    let Some(workspace_name) = workspace else {
        return Ok(None);
    };
    let workspace = storage::load_workspace(workspace_name)?;
    Ok(select_workspace_model(&workspace, thinking))
}

fn select_workspace_model(
    workspace: &crate::ninja::types::WorkspaceRecord,
    thinking: Option<&str>,
) -> Option<String> {
    let preferred = match thinking.unwrap_or("medium").to_ascii_lowercase().as_str() {
        "low" => workspace.models.lite_model.model.as_ref(),
        "high" | "xhigh" => workspace.models.highest_frontier_model.model.as_ref(),
        _ => workspace.models.frontier_model.model.as_ref(),
    };
    preferred
        .or(workspace.models.frontier_model.model.as_ref())
        .or(workspace.models.highest_frontier_model.model.as_ref())
        .or(workspace.models.lite_model.model.as_ref())
        .or(workspace.models.super_lite_model.model.as_ref())
        .cloned()
}
