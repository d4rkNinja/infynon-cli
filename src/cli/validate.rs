use crate::cli::args::{
    AiAction, ApiCommands, AssertionAction, EnvAction, FlowAction, NodeAction, PkgArgs,
    PkgCommands, PromptAction,
};
use std::path::Path;

const KNOWN_ECOSYSTEMS: &[&str] = &[
    "npm", "yarn", "pnpm", "bun",
    "pip", "pip3", "pypi", "uv", "poetry",
    "cargo", "crates.io",
    "go", "golang",
    "gem", "rubygems",
    "composer", "packagist",
    "nuget", "dotnet",
    "hex", "mix",
    "pub", "pub.dev", "dart",
    "postgres", "mysql", "sqlite",
];

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
            output,
            ..
        } => {
            validate_id(id, "flow id")?;
            validate_optional_url(base_url.as_deref(), "--base-url")?;
            validate_optional_output_format(output.as_deref())
        }
        FlowAction::RunAll {
            base_url, output, ..
        } => {
            validate_optional_url(base_url.as_deref(), "--base-url")?;
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
    if KNOWN_ECOSYSTEMS.contains(&trimmed.as_str()) {
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
