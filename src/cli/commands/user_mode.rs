use crate::cli::args::{NinjaAction, SoulAction, TaskAction, WorkspaceAction};
use crate::ninja::commands;
use crate::utils::print_json_pretty;
use serde_json::json;

pub fn execute_workspace_command(action: WorkspaceAction) {
    if let Err(message) = crate::cli::validate::validate_workspace_action(&action) {
        print_json_pretty(&json!({
            "status": "error",
            "command": "workspace",
            "error": message
        }));
        std::process::exit(2);
    }
    if let Err(err) = commands::execute_workspace(action) {
        print_json_pretty(&json!({
            "status": "error",
            "command": "workspace",
            "error": err
        }));
        std::process::exit(1);
    }
}

pub fn execute_task_command(action: TaskAction) {
    if let Err(message) = crate::cli::validate::validate_task_action(&action) {
        print_json_pretty(&json!({
            "status": "error",
            "command": "task",
            "error": message
        }));
        std::process::exit(2);
    }
    if let Err(err) = commands::execute_task(action) {
        print_json_pretty(&json!({
            "status": "error",
            "command": "task",
            "error": err
        }));
        std::process::exit(1);
    }
}

pub fn execute_soul_command(action: SoulAction) {
    if let Err(err) = commands::execute_soul(action) {
        print_json_pretty(&json!({
            "status": "error",
            "command": "soul",
            "error": err
        }));
        std::process::exit(1);
    }
}

pub fn execute_ninja_command(action: NinjaAction) {
    if let Err(err) = commands::execute_ninja(action) {
        print_json_pretty(&json!({
            "status": "error",
            "command": "ninja",
            "error": err
        }));
        std::process::exit(1);
    }
}

pub fn execute_coding_command(action: NinjaAction) {
    if let Err(err) = commands::execute_coding(action) {
        if err.contains("INFYNON agent root path is not configured") {
            eprintln!("{}", err);
            std::process::exit(1);
        }
        print_json_pretty(&json!({
            "status": "error",
            "command": "coding",
            "error": err
        }));
        std::process::exit(1);
    }
}
