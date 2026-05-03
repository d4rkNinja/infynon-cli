use crate::cli::args::{
    AiAction, ApiCommands, AssertionAction, EnvAction, FlowAction, NodeAction, PkgArgs,
    PkgCommands, PromptAction, TaskAction, WorkspaceAction,
};
use regex::Regex;
use std::path::Path;
use std::sync::OnceLock;

pub fn validate_pkg_args(args: &PkgArgs) -> Result<(), String> {
    validate_optional_severity(args.strict.as_deref(), "--strict")?;
    validate_optional_path(args.pkg_file.as_deref(), "--pkg-file")?;

    if args.yes && args.skip_vulnerable {
        return Err("Use either `--yes` or `--skip-vulnerable`, not both.".to_string());
    }
    if args.yes && args.auto_fix {
        return Err("Use either `--yes` or `--auto-fix`, not both.".to_string());
    }

    if let Some(command) = &args.command {
        validate_pkg_command(command)?;
    }

    if !args.passthrough_args.is_empty()
        && args
            .passthrough_args
            .iter()
            .any(|arg| arg.trim().is_empty())
    {
        return Err("Passthrough package-manager arguments cannot be empty.".to_string());
    }

    Ok(())
}

pub fn validate_api_command(action: &ApiCommands) -> Result<(), String> {
    match action {
        ApiCommands::Tui { flow_id } => validate_optional_id(flow_id.as_deref(), "flow id"),
        ApiCommands::Node { action } => validate_node_action(action),
        ApiCommands::Flow { action } => validate_flow_action(action),
        ApiCommands::Attach { from, to, .. } | ApiCommands::Detach { from, to } => {
            validate_id(from, "source node id")?;
            validate_id(to, "target node id")?;
            if from == to {
                return Err("Source and target nodes must be different.".to_string());
            }
            Ok(())
        }
        ApiCommands::Ai { action } => validate_ai_action(action),
        ApiCommands::Env { action } => validate_env_action(action),
        ApiCommands::Validate => Ok(()),
        ApiCommands::Import {
            spec,
            base_url,
            prefix,
            ..
        } => {
            validate_existing_file(spec, "spec path")?;
            validate_spec_extension(spec)?;
            validate_optional_url(base_url.as_deref(), "--base-url")?;
            validate_optional_non_empty(prefix.as_deref(), "--prefix")
        }
    }
}

pub fn validate_workspace_action(action: &WorkspaceAction) -> Result<(), String> {
    match action {
        WorkspaceAction::Create {
            name,
            mutate,
            folder_name,
            path,
            description,
            lite_model,
            lite_thinking,
            frontier_model,
            frontier_thinking,
            highest_frontier_model,
            highest_frontier_thinking,
            super_lite_model,
            super_lite_thinking,
            ..
        } => {
            validate_portable_name(name, "workspace name")?;
            validate_mutate(*mutate)?;
            validate_optional_portable_name(folder_name.as_deref(), "folder name")?;
            validate_optional_directory_path(path.as_deref(), "--path")?;
            validate_optional_non_empty(description.as_deref(), "--description")?;
            validate_workspace_folder_pair(folder_name.as_deref(), path.as_deref())?;
            validate_workspace_models([
                (
                    "--lite-model",
                    lite_model.as_deref(),
                    "--lite-thinking",
                    lite_thinking.as_deref(),
                ),
                (
                    "--frontier-model",
                    frontier_model.as_deref(),
                    "--frontier-thinking",
                    frontier_thinking.as_deref(),
                ),
                (
                    "--highest-frontier-model",
                    highest_frontier_model.as_deref(),
                    "--highest-frontier-thinking",
                    highest_frontier_thinking.as_deref(),
                ),
                (
                    "--super-lite-model",
                    super_lite_model.as_deref(),
                    "--super-lite-thinking",
                    super_lite_thinking.as_deref(),
                ),
            ])
        }
        WorkspaceAction::List => Ok(()),
        WorkspaceAction::Show { name } => validate_portable_name(name, "workspace name"),
        WorkspaceAction::AgentRootShow => Ok(()),
        WorkspaceAction::AgentRootSet { mutate, path } => {
            validate_mutate(*mutate)?;
            validate_directory_path(path, "--path")
        }
        WorkspaceAction::Update {
            name,
            mutate,
            folder_name,
            path,
            description,
            default,
            lite_model,
            lite_thinking,
            frontier_model,
            frontier_thinking,
            highest_frontier_model,
            highest_frontier_thinking,
            super_lite_model,
            super_lite_thinking,
            ..
        } => {
            validate_portable_name(name, "workspace name")?;
            validate_mutate(*mutate)?;
            validate_optional_portable_name(folder_name.as_deref(), "folder name")?;
            validate_optional_directory_path(path.as_deref(), "--path")?;
            validate_optional_non_empty(description.as_deref(), "--description")?;
            validate_workspace_folder_pair(folder_name.as_deref(), path.as_deref())?;
            validate_workspace_models([
                (
                    "--lite-model",
                    lite_model.as_deref(),
                    "--lite-thinking",
                    lite_thinking.as_deref(),
                ),
                (
                    "--frontier-model",
                    frontier_model.as_deref(),
                    "--frontier-thinking",
                    frontier_thinking.as_deref(),
                ),
                (
                    "--highest-frontier-model",
                    highest_frontier_model.as_deref(),
                    "--highest-frontier-thinking",
                    highest_frontier_thinking.as_deref(),
                ),
                (
                    "--super-lite-model",
                    super_lite_model.as_deref(),
                    "--super-lite-thinking",
                    super_lite_thinking.as_deref(),
                ),
            ])?;
            if folder_name.is_none()
                && path.is_none()
                && description.is_none()
                && !default
                && lite_model.is_none()
                && lite_thinking.is_none()
                && frontier_model.is_none()
                && frontier_thinking.is_none()
                && highest_frontier_model.is_none()
                && highest_frontier_thinking.is_none()
                && super_lite_model.is_none()
                && super_lite_thinking.is_none()
            {
                return Err(
                    "Workspace update requires at least one change flag or `--default`."
                        .to_string(),
                );
            }
            Ok(())
        }
        WorkspaceAction::AddFolder {
            name,
            mutate,
            folder_name,
            path,
        } => {
            validate_portable_name(name, "workspace name")?;
            validate_mutate(*mutate)?;
            validate_portable_name(folder_name, "folder name")?;
            validate_directory_path(path, "--path")
        }
        WorkspaceAction::RemoveFolder {
            name,
            mutate,
            folder_name,
        } => {
            validate_portable_name(name, "workspace name")?;
            validate_mutate(*mutate)?;
            validate_portable_name(folder_name, "folder name")
        }
        WorkspaceAction::Remove { name, mutate } => {
            validate_portable_name(name, "workspace name")?;
            validate_mutate(*mutate)
        }
    }
}

pub fn validate_task_action(action: &TaskAction) -> Result<(), String> {
    match action {
        TaskAction::Create {
            id,
            mutate,
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
            ..
        } => {
            validate_uuid_v4(id, "task id")?;
            validate_mutate(*mutate)?;
            validate_optional_portable_name(workspace.as_deref(), "workspace name")?;
            validate_optional_portable_name(folder_name.as_deref(), "folder name")?;
            validate_optional_non_empty(agent.as_deref(), "--agent")?;
            validate_optional_non_empty(model.as_deref(), "--model")?;
            validate_optional_workspace_thinking(thinking.as_deref(), "--thinking")?;
            validate_optional_non_empty(prompt.as_deref(), "--prompt")?;
            validate_optional_non_empty(command.as_deref(), "--command")?;
            validate_optional_non_empty(session_id.as_deref(), "--session-id")?;
            validate_optional_non_empty(notes.as_deref(), "--notes")?;
            validate_optional_non_empty(result.as_deref(), "--result")?;
            validate_optional_uuid_v4(blocked_by.as_deref(), "--blocked-by")?;
            validate_optional_non_empty(blocked_reason.as_deref(), "--blocked-reason")?;
            validate_blocked_pair(blocked_by.as_deref(), blocked_reason.as_deref())?;
            validate_optional_pid(*pid)?;
            validate_task_status(status)
        }
        TaskAction::List {
            workspace,
            status,
            agent,
        } => {
            validate_optional_portable_name(workspace.as_deref(), "workspace name")?;
            validate_optional_task_status(status.as_deref())?;
            validate_optional_non_empty(agent.as_deref(), "--agent")
        }
        TaskAction::Show { id } | TaskAction::Remove { id, .. } => validate_uuid_v4(id, "task id"),
        TaskAction::Update {
            id,
            mutate,
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
            parent_task_id,
            ..
        } => {
            validate_uuid_v4(id, "task id")?;
            validate_mutate(*mutate)?;
            validate_optional_portable_name(workspace.as_deref(), "workspace name")?;
            validate_optional_portable_name(folder_name.as_deref(), "folder name")?;
            validate_optional_non_empty(agent.as_deref(), "--agent")?;
            validate_optional_non_empty(model.as_deref(), "--model")?;
            validate_optional_workspace_thinking(thinking.as_deref(), "--thinking")?;
            validate_optional_non_empty(prompt.as_deref(), "--prompt")?;
            validate_optional_non_empty(command.as_deref(), "--command")?;
            validate_optional_non_empty(session_id.as_deref(), "--session-id")?;
            validate_optional_non_empty(notes.as_deref(), "--notes")?;
            validate_optional_non_empty(result.as_deref(), "--result")?;
            validate_optional_uuid_v4(blocked_by.as_deref(), "--blocked-by")?;
            validate_optional_non_empty(blocked_reason.as_deref(), "--blocked-reason")?;
            validate_blocked_pair(blocked_by.as_deref(), blocked_reason.as_deref())?;
            validate_optional_pid(*pid)?;
            validate_optional_task_status(status.as_deref())?;
            validate_optional_uuid_v4(parent_task_id.as_deref(), "parent task id")?;
            if workspace.is_none()
                && folder_name.is_none()
                && agent.is_none()
                && model.is_none()
                && thinking.is_none()
                && prompt.is_none()
                && command.is_none()
                && pid.is_none()
                && session_id.is_none()
                && notes.is_none()
                && result.is_none()
                && blocked_by.is_none()
                && blocked_reason.is_none()
                && status.is_none()
                && parent_task_id.is_none()
            {
                return Err("Task update requires at least one change flag.".to_string());
            }
            Ok(())
        }
        TaskAction::Start {
            id,
            mutate,
            pid,
            session_id,
        } => {
            validate_uuid_v4(id, "task id")?;
            validate_mutate(*mutate)?;
            validate_optional_pid(*pid)?;
            validate_optional_non_empty(session_id.as_deref(), "--session-id")
        }
        TaskAction::Resume {
            id,
            mutate,
            session_id,
            prompt,
        } => {
            validate_uuid_v4(id, "task id")?;
            validate_mutate(*mutate)?;
            validate_optional_non_empty(session_id.as_deref(), "--session-id")?;
            validate_optional_non_empty(prompt.as_deref(), "--prompt")
        }
        TaskAction::Kill {
            id,
            mutate,
            pid,
            reason,
            ..
        } => {
            validate_uuid_v4(id, "task id")?;
            validate_mutate(*mutate)?;
            validate_optional_pid(*pid)?;
            validate_optional_non_empty(reason.as_deref(), "--reason")
        }
        TaskAction::Complete {
            id,
            mutate,
            notes,
            result,
            close_terminal,
            keep_terminal,
            ..
        } => {
            validate_uuid_v4(id, "task id")?;
            validate_mutate(*mutate)?;
            validate_optional_non_empty(notes.as_deref(), "--notes")?;
            validate_optional_non_empty(result.as_deref(), "--result")?;
            validate_terminal_close_flags(*close_terminal, *keep_terminal)
        }
        TaskAction::Fail {
            id,
            mutate,
            reason,
            result,
            close_terminal,
            keep_terminal,
            ..
        } => {
            validate_uuid_v4(id, "task id")?;
            validate_mutate(*mutate)?;
            validate_optional_non_empty(reason.as_deref(), "--reason")?;
            validate_optional_non_empty(result.as_deref(), "--result")?;
            validate_terminal_close_flags(*close_terminal, *keep_terminal)?;
            if reason.is_none() && result.is_none() {
                return Err("Task fail requires `--reason` or `--result`.".to_string());
            }
            Ok(())
        }
        TaskAction::Note { id, mutate, text } | TaskAction::Result { id, mutate, text } => {
            validate_uuid_v4(id, "task id")?;
            validate_mutate(*mutate)?;
            validate_non_empty(text, "--text")
        }
        TaskAction::Fork {
            new_id,
            from,
            mutate,
            workspace,
            folder_name,
            agent,
            model,
            thinking,
            prompt,
            notes,
            result,
            session_id,
            blocked_by,
            blocked_reason,
            status,
        } => {
            validate_uuid_v4(new_id, "new task id")?;
            validate_uuid_v4(from, "source task id")?;
            if new_id == from {
                return Err("Fork target id must be different from source task id.".to_string());
            }
            validate_mutate(*mutate)?;
            validate_optional_portable_name(workspace.as_deref(), "workspace name")?;
            validate_optional_portable_name(folder_name.as_deref(), "folder name")?;
            validate_optional_non_empty(agent.as_deref(), "--agent")?;
            validate_optional_non_empty(model.as_deref(), "--model")?;
            validate_optional_workspace_thinking(thinking.as_deref(), "--thinking")?;
            validate_optional_non_empty(prompt.as_deref(), "--prompt")?;
            validate_optional_non_empty(notes.as_deref(), "--notes")?;
            validate_optional_non_empty(result.as_deref(), "--result")?;
            validate_optional_non_empty(session_id.as_deref(), "--session-id")?;
            validate_optional_uuid_v4(blocked_by.as_deref(), "--blocked-by")?;
            validate_optional_non_empty(blocked_reason.as_deref(), "--blocked-reason")?;
            validate_blocked_pair(blocked_by.as_deref(), blocked_reason.as_deref())?;
            validate_task_status(status)
        }
    }
}

fn validate_pkg_command(command: &PkgCommands) -> Result<(), String> {
    match command {
        PkgCommands::Scan {
            output,
            fix,
            pkg_file,
        } => {
            validate_optional_output_format(output.as_deref())?;
            validate_optional_severity(fix.as_deref(), "--fix")?;
            validate_optional_path(pkg_file.as_deref(), "--pkg-file")
        }
        PkgCommands::Audit { pkg_file }
        | PkgCommands::Outdated { pkg_file }
        | PkgCommands::Doctor { pkg_file }
        | PkgCommands::Fix { pkg_file, .. }
        | PkgCommands::Clean { pkg_file } => {
            validate_optional_path(pkg_file.as_deref(), "--pkg-file")
        }
        PkgCommands::Why { package, pkg_file } => {
            validate_non_empty(package, "package")?;
            validate_optional_path(pkg_file.as_deref(), "--pkg-file")
        }
        PkgCommands::Explain {
            package,
            ecosystem,
            pkg_file,
        } => {
            validate_non_empty(package, "package")?;
            validate_optional_ecosystem(ecosystem.as_deref(), "--ecosystem")?;
            validate_optional_path(pkg_file.as_deref(), "--pkg-file")
        }
        PkgCommands::Diff {
            package,
            v1,
            v2,
            ecosystem,
        } => {
            validate_non_empty(package, "package")?;
            validate_non_empty(v1, "v1")?;
            validate_non_empty(v2, "v2")?;
            validate_optional_ecosystem(ecosystem.as_deref(), "--ecosystem")
        }
        PkgCommands::Size {
            packages,
            ecosystem,
        } => {
            if packages.is_empty() {
                return Err("Provide at least one package for `pkg size`.".to_string());
            }
            for package in packages {
                validate_non_empty(package, "package")?;
            }
            validate_optional_ecosystem(ecosystem.as_deref(), "--ecosystem")
        }
        PkgCommands::Search { query, ecosystem } => {
            validate_non_empty(query, "query")?;
            validate_optional_ecosystem(ecosystem.as_deref(), "--ecosystem")
        }
        PkgCommands::Migrate { from, to } => {
            validate_ecosystem(from, "`from` ecosystem")?;
            validate_ecosystem(to, "`to` ecosystem")?;
            if from.eq_ignore_ascii_case(to) {
                return Err("Migration source and target ecosystems must be different.".to_string());
            }
            Ok(())
        }
        PkgCommands::EagleEye { .. } => Ok(()),
    }
}

fn validate_node_action(action: &NodeAction) -> Result<(), String> {
    match action {
        NodeAction::Create { ai } => validate_optional_non_empty(ai.as_deref(), "--ai"),
        NodeAction::Get { id }
        | NodeAction::Remove { id }
        | NodeAction::Run { id, .. }
        | NodeAction::Export { id, .. } => validate_id(id, "node id"),
        NodeAction::List => Ok(()),
        NodeAction::Clone { id, new_id } => {
            validate_id(id, "node id")?;
            validate_id(new_id, "new node id")?;
            if id == new_id {
                return Err("Clone target id must be different from the source id.".to_string());
            }
            Ok(())
        }
        NodeAction::Assertion { node_id, action } => {
            validate_id(node_id, "node id")?;
            validate_assertion_action(action)
        }
        NodeAction::Prompt { node_id, action } => {
            validate_id(node_id, "node id")?;
            validate_prompt_action(action)
        }
    }
}

fn validate_flow_action(action: &FlowAction) -> Result<(), String> {
    match action {
        FlowAction::Create { name, ai } => {
            validate_non_empty(name, "flow name")?;
            validate_optional_non_empty(ai.as_deref(), "--ai")
        }
        FlowAction::List => Ok(()),
        FlowAction::Show { id } | FlowAction::Remove { id } => validate_id(id, "flow id"),
        FlowAction::Run {
            id,
            base_url,
            format,
            output,
            ..
        } => {
            validate_id(id, "flow id")?;
            validate_optional_url(base_url.as_deref(), "--base-url")?;
            validate_optional_flow_format(format.as_deref(), "--format")?;
            validate_optional_output_format(output.as_deref())
        }
        FlowAction::RunAll {
            base_url,
            format,
            output,
            ..
        } => {
            validate_optional_url(base_url.as_deref(), "--base-url")?;
            validate_optional_flow_format(format.as_deref(), "--format")?;
            validate_optional_output_format(output.as_deref())
        }
        FlowAction::Merge {
            flow1,
            flow2,
            join_at,
            name,
        } => {
            validate_id(flow1, "flow1 id")?;
            validate_id(flow2, "flow2 id")?;
            validate_id(join_at, "join-at node id")?;
            validate_non_empty(name, "merged flow name")?;
            if flow1 == flow2 {
                return Err("flow1 and flow2 must be different.".to_string());
            }
            Ok(())
        }
    }
}

fn validate_ai_action(action: &AiAction) -> Result<(), String> {
    match action {
        AiAction::Suggest { after } | AiAction::Attach { after, .. } => {
            validate_id(after, "node id")
        }
        AiAction::Complete { flow_id } | AiAction::Explain { flow_id, .. } => {
            validate_id(flow_id, "flow id")
        }
        AiAction::Probe { flow_id, base_url } => {
            validate_id(flow_id, "flow id")?;
            validate_optional_url(base_url.as_deref(), "--base-url")
        }
        AiAction::BuildFlow { nodes, name } => {
            if nodes.is_empty() {
                return Err("Provide at least one node id to build a flow.".to_string());
            }
            for node in nodes {
                validate_id(node, "node id")?;
            }
            validate_non_empty(name, "flow name")
        }
        AiAction::Assert { node_id } | AiAction::Branch { node_id } => {
            validate_id(node_id, "node id")
        }
    }
}

fn validate_env_action(action: &EnvAction) -> Result<(), String> {
    match action {
        EnvAction::List => Ok(()),
        EnvAction::Set { key, value } => {
            validate_non_empty(key, "key")?;
            validate_non_empty(value, "value")
        }
        EnvAction::Delete { key } | EnvAction::Get { key, .. } => validate_non_empty(key, "key"),
    }
}

fn validate_assertion_action(action: &AssertionAction) -> Result<(), String> {
    match action {
        AssertionAction::List
        | AssertionAction::Enable { .. }
        | AssertionAction::Disable { .. }
        | AssertionAction::Toggle { .. }
        | AssertionAction::Remove { .. } => Ok(()),
        AssertionAction::Add { check, on_fail } => {
            validate_non_empty(check, "assertion check")?;
            match on_fail.trim().to_ascii_lowercase().as_str() {
                "stop" | "warn" => Ok(()),
                _ => Err("`--on-fail` must be `stop` or `warn`.".to_string()),
            }
        }
    }
}

fn validate_prompt_action(action: &PromptAction) -> Result<(), String> {
    match action {
        PromptAction::List | PromptAction::Remove { .. } => Ok(()),
        PromptAction::Add {
            var,
            prompt_type,
            options,
            ..
        } => {
            validate_non_empty(var, "prompt variable")?;
            match prompt_type.trim().to_ascii_lowercase().as_str() {
                "text" | "boolean" => Ok(()),
                "select" | "multiselect" => {
                    let values = options
                        .as_deref()
                        .map(|value| {
                            value
                                .split(',')
                                .filter(|item| !item.trim().is_empty())
                                .count()
                        })
                        .unwrap_or(0);
                    if values == 0 {
                        Err("Select and multiselect prompt types require `--options`.".to_string())
                    } else {
                        Ok(())
                    }
                }
                _ => Err(
                    "Prompt type must be one of: text | boolean | select | multiselect."
                        .to_string(),
                ),
            }
        }
    }
}

fn validate_optional_severity(value: Option<&str>, flag: &str) -> Result<(), String> {
    if let Some(value) = value {
        match value.trim().to_ascii_lowercase().as_str() {
            "critical" | "high" | "medium" | "low" | "informational" | "info" | "all" => Ok(()),
            _ => Err(format!(
                "{} must be one of: critical | high | medium | low | informational | all.",
                flag
            )),
        }
    } else {
        Ok(())
    }
}

fn validate_optional_output_format(value: Option<&str>) -> Result<(), String> {
    if let Some(value) = value {
        match value.trim().to_ascii_lowercase().as_str() {
            "markdown" | "pdf" | "both" => Ok(()),
            _ => Err("Output format must be one of: markdown | pdf | both.".to_string()),
        }
    } else {
        Ok(())
    }
}

fn validate_optional_flow_format(value: Option<&str>, flag: &str) -> Result<(), String> {
    if let Some(value) = value {
        match value.trim().to_ascii_lowercase().as_str() {
            "json" | "markdown" | "junit" => Ok(()),
            _ => Err(format!("{} must be one of: json | markdown | junit.", flag)),
        }
    } else {
        Ok(())
    }
}

fn validate_optional_path(path: Option<&str>, label: &str) -> Result<(), String> {
    if let Some(path) = path {
        validate_non_empty(path, label)?;
        if !Path::new(path).exists() {
            return Err(format!("{} '{}' does not exist.", label, path));
        }
    }
    Ok(())
}

fn validate_existing_file(path: &str, label: &str) -> Result<(), String> {
    validate_non_empty(path, label)?;
    let file = Path::new(path);
    if !file.exists() {
        return Err(format!("{} '{}' does not exist.", label, path));
    }
    if !file.is_file() {
        return Err(format!("{} '{}' must be a file.", label, path));
    }
    Ok(())
}

fn validate_spec_extension(path: &str) -> Result<(), String> {
    match Path::new(path).extension().and_then(|value| value.to_str()) {
        Some("yaml") | Some("yml") | Some("json") => Ok(()),
        _ => Err("Spec path must point to a .yaml, .yml, or .json file.".to_string()),
    }
}

fn validate_optional_url(value: Option<&str>, label: &str) -> Result<(), String> {
    if let Some(value) = value {
        validate_url(value, label)?;
    }
    Ok(())
}

fn validate_url(value: &str, label: &str) -> Result<(), String> {
    let trimmed = value.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        Ok(())
    } else {
        Err(format!("{} must start with http:// or https://.", label))
    }
}

fn validate_optional_ecosystem(value: Option<&str>, label: &str) -> Result<(), String> {
    if let Some(value) = value {
        validate_ecosystem(value, label)?;
    }
    Ok(())
}

fn validate_ecosystem(value: &str, label: &str) -> Result<(), String> {
    let trimmed = value.trim().to_ascii_lowercase();
    if crate::ecosystems::catalog::is_known_ecosystem(&trimmed) {
        Ok(())
    } else {
        Err(format!("{} '{}' is not supported.", label, value))
    }
}

fn validate_optional_non_empty(value: Option<&str>, label: &str) -> Result<(), String> {
    if let Some(value) = value {
        validate_non_empty(value, label)?;
    }
    Ok(())
}

fn validate_optional_id(value: Option<&str>, label: &str) -> Result<(), String> {
    if let Some(value) = value {
        validate_id(value, label)?;
    }
    Ok(())
}

fn validate_id(value: &str, label: &str) -> Result<(), String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!("{} cannot be empty.", label));
    }
    if trimmed.contains(char::is_whitespace) {
        return Err(format!("{} cannot contain whitespace.", label));
    }
    Ok(())
}

fn validate_non_empty(value: &str, label: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("{} cannot be empty.", label))
    } else {
        Ok(())
    }
}

fn validate_mutate(mutate: bool) -> Result<(), String> {
    if mutate {
        Ok(())
    } else {
        Err("Mutating commands require `--mutate`.".to_string())
    }
}

fn validate_optional_task_status(value: Option<&str>) -> Result<(), String> {
    if let Some(value) = value {
        validate_task_status(value)
    } else {
        Ok(())
    }
}

fn validate_task_status(value: &str) -> Result<(), String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "draft" | "queued" | "running" | "blocked" | "completed" | "failed" | "killed" => {
            Ok(())
        }
        _ => Err(
            "Task status must be one of: draft | queued | running | blocked | completed | failed | killed."
                .to_string(),
        ),
    }
}

fn validate_optional_pid(pid: Option<u32>) -> Result<(), String> {
    if let Some(pid) = pid {
        if pid == 0 {
            return Err("`--pid` must be greater than zero.".to_string());
        }
    }
    Ok(())
}

fn validate_terminal_close_flags(close_terminal: bool, keep_terminal: bool) -> Result<(), String> {
    if close_terminal && keep_terminal {
        Err("Use either `--close-terminal` or `--keep-terminal`, not both.".to_string())
    } else {
        Ok(())
    }
}

fn validate_optional_directory_path(value: Option<&str>, label: &str) -> Result<(), String> {
    if let Some(value) = value {
        validate_directory_path(value, label)?;
    }
    Ok(())
}

fn validate_optional_portable_name(value: Option<&str>, label: &str) -> Result<(), String> {
    if let Some(value) = value {
        validate_portable_name(value, label)?;
    }
    Ok(())
}

fn validate_portable_name(value: &str, label: &str) -> Result<(), String> {
    validate_id(value, label)?;
    if !crate::utils::is_portable_file_stem(value) {
        return Err(format!(
            "{} must use only ASCII letters, digits, '-' or '_'.",
            label
        ));
    }
    Ok(())
}

fn validate_directory_path(value: &str, label: &str) -> Result<(), String> {
    validate_non_empty(value, label)?;
    let path = Path::new(value);
    if !path.is_absolute() {
        return Err(format!("{} must be an absolute path.", label));
    }
    if !path.exists() {
        return Err(format!("{} '{}' does not exist.", label, value));
    }
    if !path.is_dir() {
        return Err(format!("{} '{}' must be a directory.", label, value));
    }
    Ok(())
}

fn validate_workspace_folder_pair(
    folder_name: Option<&str>,
    path: Option<&str>,
) -> Result<(), String> {
    match (folder_name, path) {
        (Some(_), Some(_)) | (None, None) => Ok(()),
        _ => Err("`--folder-name` and `--path` must be provided together.".to_string()),
    }
}

fn validate_workspace_models<const N: usize>(
    entries: [(&str, Option<&str>, &str, Option<&str>); N],
) -> Result<(), String> {
    for (model_flag, model_value, thinking_flag, thinking_value) in entries {
        validate_optional_non_empty(model_value, model_flag)?;
        validate_optional_workspace_thinking(thinking_value, thinking_flag)?;
        if model_value.is_none() && thinking_value.is_some() {
            return Err(format!(
                "{} requires {} to be provided as well.",
                thinking_flag, model_flag
            ));
        }
    }
    Ok(())
}

fn validate_optional_workspace_thinking(value: Option<&str>, flag: &str) -> Result<(), String> {
    if let Some(value) = value {
        match value.trim().to_ascii_lowercase().as_str() {
            "auto" | "low" | "medium" | "high" | "xhigh" => Ok(()),
            _ => Err(format!(
                "{} must be one of: auto | low | medium | high | xhigh.",
                flag
            )),
        }
    } else {
        Ok(())
    }
}

fn validate_optional_uuid_v4(value: Option<&str>, label: &str) -> Result<(), String> {
    if let Some(value) = value {
        validate_uuid_v4(value, label)?;
    }
    Ok(())
}

fn validate_uuid_v4(value: &str, label: &str) -> Result<(), String> {
    static UUID_V4_RE: OnceLock<Regex> = OnceLock::new();
    let trimmed = value.trim();
    let re = UUID_V4_RE.get_or_init(|| {
        Regex::new(r"(?i)^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$")
            .expect("valid uuid v4 regex")
    });
    if re.is_match(trimmed) {
        Ok(())
    } else {
        Err(format!("{} must be a valid UUIDv4.", label))
    }
}

fn validate_blocked_pair(
    blocked_by: Option<&str>,
    blocked_reason: Option<&str>,
) -> Result<(), String> {
    match (blocked_by, blocked_reason) {
        (Some(_), Some(_)) | (None, None) => Ok(()),
        _ => Err("`--blocked-by` and `--blocked-reason` must be provided together.".to_string()),
    }
}
