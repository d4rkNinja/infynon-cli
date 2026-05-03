use crate::cli::args::{NinjaAction, SoulAction, TaskAction, WorkspaceAction};
use crate::ninja::storage;
use crate::ninja::types::{
    timestamp_now, NinjaManifest, TaskRecord, TaskSummary, WorkspaceFolder, WorkspaceModelSlot,
    WorkspaceModels, WorkspaceRecord, WorkspaceSummary,
};
use crate::utils::print_json_pretty;
use serde_json::json;
use std::io::{self, Read};
use std::process::Command;

pub fn execute_workspace(action: WorkspaceAction) -> Result<(), String> {
    match action {
        WorkspaceAction::Create {
            name,
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
        } => workspace_create(
            &name,
            folder_name,
            path,
            description,
            default,
            WorkspaceModelInputs {
                lite_model,
                lite_thinking,
                frontier_model,
                frontier_thinking,
                highest_frontier_model,
                highest_frontier_thinking,
                super_lite_model,
                super_lite_thinking,
            },
        ),
        WorkspaceAction::List => workspace_list(),
        WorkspaceAction::Show { name } => workspace_show(&name),
        WorkspaceAction::AgentRootShow => workspace_agent_root_show(),
        WorkspaceAction::AgentRootSet { path, .. } => workspace_agent_root_set(&path),
        WorkspaceAction::Update {
            name,
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
        } => workspace_update(
            &name,
            folder_name,
            path,
            description,
            default,
            WorkspaceModelInputs {
                lite_model,
                lite_thinking,
                frontier_model,
                frontier_thinking,
                highest_frontier_model,
                highest_frontier_thinking,
                super_lite_model,
                super_lite_thinking,
            },
        ),
        WorkspaceAction::AddFolder {
            name,
            folder_name,
            path,
            ..
        } => workspace_add_folder(&name, &folder_name, &path),
        WorkspaceAction::RemoveFolder {
            name, folder_name, ..
        } => workspace_remove_folder(&name, &folder_name),
        WorkspaceAction::Remove { name, .. } => workspace_remove(&name),
    }
}

pub fn execute_task(action: TaskAction) -> Result<(), String> {
    match action {
        TaskAction::Create {
            id,
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
        } => task_create(
            &id,
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
            &status,
        ),
        TaskAction::List {
            workspace,
            status,
            agent,
        } => task_list(workspace, status, agent),
        TaskAction::Show { id } => task_show(&id),
        TaskAction::Update {
            id,
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
        } => task_update(
            &id,
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
        ),
        TaskAction::Start {
            id,
            pid,
            session_id,
            ..
        } => task_start(&id, pid, session_id),
        TaskAction::Resume {
            id,
            session_id,
            prompt,
            ..
        } => task_resume(&id, session_id, prompt),
        TaskAction::Kill {
            id,
            pid,
            reason,
            force,
            ..
        } => task_kill(&id, pid, reason, force),
        TaskAction::Complete {
            id,
            notes,
            result,
            close_terminal,
            keep_terminal,
            ..
        } => task_complete(&id, notes, result, close_terminal, keep_terminal),
        TaskAction::Fail {
            id,
            reason,
            result,
            close_terminal,
            keep_terminal,
            ..
        } => task_fail(&id, reason, result, close_terminal, keep_terminal),
        TaskAction::Note { id, text, .. } => task_append_note(&id, &text),
        TaskAction::Result { id, text, .. } => task_append_result(&id, &text),
        TaskAction::Fork {
            new_id,
            from,
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
            ..
        } => task_fork(
            &new_id,
            &from,
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
            &status,
        ),
        TaskAction::Remove { id, .. } => task_remove(&id),
    }
}

pub fn execute_soul(action: SoulAction) -> Result<(), String> {
    match action {
        SoulAction::Show => soul_show(),
        SoulAction::Update { text, file } => soul_update(text, file),
    }
}

pub fn execute_ninja(action: NinjaAction) -> Result<(), String> {
    if matches!(action, NinjaAction::Tui) {
        return crate::ninja::tui::run();
    }
    let request = AgentLaunchRequest::from_action(action);
    run_project_agent_open(request)
}

pub fn ensure_first_run_identity_prompt() {
    let raw_args: Vec<String> = std::env::args().collect();
    if should_skip_identity_prompt(&raw_args) {
        return;
    }
    let Ok(identity) = storage::load_user_identity() else {
        return;
    };
    if identity
        .as_ref()
        .and_then(|value| value.name.as_deref())
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
    {
        return;
    }
    let Ok(name) = dialoguer::Input::<String>::new()
        .with_prompt("INFYNON username")
        .allow_empty(false)
        .interact_text()
    else {
        return;
    };
    let name = name.trim();
    if name.is_empty() {
        return;
    }
    let now = timestamp_now();
    let created_at = identity
        .as_ref()
        .map(|value| value.created_at.clone())
        .unwrap_or_else(|| now.clone());
    let updated = crate::ninja::types::UserIdentity {
        name: Some(name.to_string()),
        created_at,
        updated_at: now,
    };
    let _ = storage::save_user_identity(&updated);
    let _ = storage::ensure_soul_file();
}

fn should_skip_identity_prompt(args: &[String]) -> bool {
    if args.len() > 1 {
        return true;
    }
    if args.iter().any(|arg| {
        matches!(
            arg.as_str(),
            "-h" | "--help" | "-V" | "--version" | "help" | "completion"
        )
    }) {
        return true;
    }
    false
}

pub fn execute_coding(action: NinjaAction) -> Result<(), String> {
    if matches!(action, NinjaAction::Tui) {
        return crate::ninja::tui::run();
    }
    let request = AgentLaunchRequest::from_action(action);
    run_agent_bootstrap(request)
}

fn soul_show() -> Result<(), String> {
    let soul_path = storage::ensure_soul_file()?;
    let content = storage::read_soul()?;
    let is_blank = content.trim().is_empty();
    print_json_pretty(&json!({
        "status": "ok",
        "command": "soul.show",
        "soul_path": soul_path,
        "is_blank": is_blank,
        "content": content,
        "suggested_structure": soul_suggested_structure(),
    }));
    Ok(())
}

fn soul_update(text: Option<String>, file: Option<String>) -> Result<(), String> {
    if text.is_some() && file.is_some() {
        return Err("Use either `--text` or `--file`, not both.".to_string());
    }
    let content = if let Some(text) = text {
        text
    } else if let Some(file) = file {
        std::fs::read_to_string(&file)
            .map_err(|e| format!("Failed to read soul update file '{}': {}", file, e))?
    } else {
        let mut input = String::new();
        io::stdin()
            .read_to_string(&mut input)
            .map_err(|e| e.to_string())?;
        if input.is_empty() {
            return Err("Pass `--text`, `--file`, or pipe content on stdin.".to_string());
        }
        input
    };
    let path = storage::write_soul(&content)?;
    print_json_pretty(&json!({
        "status": "ok",
        "command": "soul.update",
        "soul_path": path,
        "bytes": content.len(),
    }));
    Ok(())
}

fn soul_suggested_structure() -> serde_json::Value {
    json!({
        "title": "Soul Profile",
        "instruction": "When the soul profile is blank, collect only stable global user context. Do not invent details.",
        "sections": [
            {
                "name": "Name",
                "questions": [
                    "What is your name?",
                    "How should INFYNON address you?"
                ]
            },
            {
                "name": "Purpose",
                "questions": [
                    "What are you trying to achieve with INFYNON?",
                    "What kind of work should agents help you complete?"
                ]
            },
            {
                "name": "Profession",
                "questions": [
                    "What is your profession or role?",
                    "What domain or industry do you usually work in?"
                ]
            },
            {
                "name": "Current Projects",
                "questions": [
                    "What projects matter right now?",
                    "Which projects should agents understand first?"
                ]
            },
            {
                "name": "Skills",
                "questions": [
                    "What technical skills do you already have?",
                    "Which areas should agents explain carefully?"
                ]
            },
            {
                "name": "Goals",
                "questions": [
                    "What short-term goals should agents help with?",
                    "What long-term goals should agents remember?"
                ]
            },
            {
                "name": "Communication Style",
                "questions": [
                    "How direct should agents be?",
                    "Should agents ask questions often or make reasonable assumptions?"
                ]
            },
            {
                "name": "Answer Style",
                "questions": [
                    "Do you prefer short answers, detailed explanations, or step-by-step reports?",
                    "Should agents include commands, examples, or summaries by default?"
                ]
            },
            {
                "name": "Decision Preferences",
                "questions": [
                    "When should agents choose for you?",
                    "When should agents stop and ask for confirmation?"
                ]
            },
            {
                "name": "Coding Preferences",
                "questions": [
                    "What coding style should agents follow globally?",
                    "What testing or validation habits should agents use?"
                ]
            },
            {
                "name": "Global Constraints",
                "questions": [
                    "What constraints apply across all workspaces?",
                    "What should agents avoid doing globally?"
                ]
            }
        ],
        "markdown_template": "# Soul Profile\n\n## Name\n\n## Purpose\n\n## Profession\n\n## Current Projects\n\n## Skills\n\n## Goals\n\n## Communication Style\n\n## Answer Style\n\n## Decision Preferences\n\n## Coding Preferences\n\n## Global Constraints\n"
    })
}

