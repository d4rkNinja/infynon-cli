pub fn cmd_node_export(id: &str, format: &str, base_url: Option<&str>) {
    let node = match storage::load_node(id) {
        Ok(n) => n,
        Err(e) => {
            Logger::error(&e);
            return;
        }
    };

    let resolved_base = base_url
        .map(|s| s.to_string())
        .or_else(|| super::env::env_base_url())
        .unwrap_or_else(|| "http://localhost:3000".to_string());
    let url = format!("{}{}", resolved_base, node.path);

    match format.to_lowercase().as_str() {
        "curl" => {
            print!("curl -X {}", node.method);
            for (k, v) in &node.headers {
                print!(" \\\n  -H '{}: {}'", k, v);
            }
            if let Some(body) = &node.body_json {
                print!(" \\\n  -d '{}'", body.replace('\'', "\\'"));
            }
            println!(" \\\n  '{}'", url);
        }
        "json" => {
            let json = serde_json::to_string_pretty(&node).unwrap_or_default();
            println!("{}", json);
        }
        _ => {
            Logger::error(&format!(
                "Unknown export format: '{}'. Use: curl, json",
                format
            ));
        }
    }
}

// ── node assertion commands ───────────────────────────────────────────────────

pub fn cmd_node_assertion_list(node_id: &str) {
    println!();
    let node = match storage::load_node(node_id) {
        Ok(n) => n,
        Err(e) => {
            Logger::error(&e);
            return;
        }
    };
    Logger::title(&format!("Assertions: {}", node_id), "cyan");
    println!();
    if node.assertions.is_empty() {
        println!("  No assertions defined.");
        println!();
        return;
    }
    for (i, a) in node.assertions.iter().enumerate() {
        let status = if a.enabled {
            "✔ enabled ".bright_green().to_string()
        } else {
            "✘ disabled".truecolor(120, 120, 120).to_string()
        };
        let fail_label = match a.on_fail {
            OnFail::Stop => "stop".bright_red().to_string(),
            OnFail::Warn => "warn".bright_yellow().to_string(),
        };
        println!(
            "  [{:>2}]  {}  {}  [{}]",
            i,
            status,
            a.check.bright_cyan(),
            fail_label,
        );
    }
    println!();
}

pub fn cmd_node_assertion_enable(node_id: &str, idx: usize) {
    let mut node = match storage::load_node(node_id) {
        Ok(n) => n,
        Err(e) => {
            Logger::error(&e);
            return;
        }
    };
    if !check_index(idx, node.assertions.len()) {
        return;
    }
    node.assertions[idx].enabled = true;
    match storage::save_node(&node) {
        Ok(_) => println!("  {}  Assertion [{}] enabled.", "✔".bright_green(), idx),
        Err(e) => Logger::error(&e),
    }
}

pub fn cmd_node_assertion_disable(node_id: &str, idx: usize) {
    let mut node = match storage::load_node(node_id) {
        Ok(n) => n,
        Err(e) => {
            Logger::error(&e);
            return;
        }
    };
    if !check_index(idx, node.assertions.len()) {
        return;
    }
    node.assertions[idx].enabled = false;
    match storage::save_node(&node) {
        Ok(_) => println!("  {}  Assertion [{}] disabled.", "✔".bright_green(), idx),
        Err(e) => Logger::error(&e),
    }
}

pub fn cmd_node_assertion_toggle(node_id: &str, idx: usize) {
    let mut node = match storage::load_node(node_id) {
        Ok(n) => n,
        Err(e) => {
            Logger::error(&e);
            return;
        }
    };
    if !check_index(idx, node.assertions.len()) {
        return;
    }
    node.assertions[idx].enabled = !node.assertions[idx].enabled;
    let state = if node.assertions[idx].enabled {
        "enabled"
    } else {
        "disabled"
    };
    match storage::save_node(&node) {
        Ok(_) => println!("  {}  Assertion [{}] {}.", "✔".bright_green(), idx, state),
        Err(e) => Logger::error(&e),
    }
}

pub fn cmd_node_assertion_add(node_id: &str, check: &str, on_fail_str: &str) {
    let mut node = match storage::load_node(node_id) {
        Ok(n) => n,
        Err(e) => {
            Logger::error(&e);
            return;
        }
    };
    let on_fail = match on_fail_str.to_lowercase().as_str() {
        "warn" | "continue" => OnFail::Warn,
        _ => OnFail::Stop,
    };
    node.assertions.push(Assertion {
        check: check.to_string(),
        on_fail,
        enabled: true,
    });
    match storage::save_node(&node) {
        Ok(_) => println!(
            "  {}  Assertion added: {}",
            "✔".bright_green(),
            check.bright_cyan()
        ),
        Err(e) => Logger::error(&e),
    }
}

pub fn cmd_node_assertion_remove(node_id: &str, idx: usize) {
    let mut node = match storage::load_node(node_id) {
        Ok(n) => n,
        Err(e) => {
            Logger::error(&e);
            return;
        }
    };
    if !check_index(idx, node.assertions.len()) {
        return;
    }
    let removed = node.assertions.remove(idx);
    match storage::save_node(&node) {
        Ok(_) => println!(
            "  {}  Assertion [{}] removed: {}",
            "✔".bright_green(),
            idx,
            removed.check.bright_cyan()
        ),
        Err(e) => Logger::error(&e),
    }
}

// ── node prompt commands ──────────────────────────────────────────────────────

pub fn cmd_node_prompt_list(node_id: &str) {
    println!();
    let node = match storage::load_node(node_id) {
        Ok(n) => n,
        Err(e) => {
            Logger::error(&e);
            return;
        }
    };
    Logger::title(&format!("Prompt Inputs: {}", node_id), "cyan");
    println!();
    if node.prompt_inputs.is_empty() {
        println!("  No prompt inputs defined.");
        println!();
        return;
    }
    for (i, pi) in node.prompt_inputs.iter().enumerate() {
        let secret_label = if pi.secret {
            " (secret)".bright_yellow().to_string()
        } else {
            String::new()
        };
        let default_label = if let Some(ref d) = pi.default {
            format!(" (default: \"{}\")", d)
                .truecolor(140, 140, 160)
                .to_string()
        } else {
            String::new()
        };
        println!(
            "  [{:>2}]  {}  — \"{}\"{}{}",
            i,
            pi.var.bright_cyan(),
            pi.label.truecolor(200, 200, 220),
            secret_label,
            default_label,
        );
    }
    println!();
}

pub fn cmd_node_prompt_add(
    node_id: &str,
    var: &str,
    label: &str,
    secret: bool,
    default: Option<String>,
    prompt_type: &str,
    options_str: Option<String>,
) {
    use crate::api::types::PromptType;
    let mut node = match storage::load_node(node_id) {
        Ok(n) => n,
        Err(e) => {
            Logger::error(&e);
            return;
        }
    };
    let label_str = if label.is_empty() {
        var.to_string()
    } else {
        label.to_string()
    };
    let pt = match prompt_type {
        "boolean" => PromptType::Boolean,
        "select" => PromptType::Select,
        "multiselect" => PromptType::Multiselect,
        _ => PromptType::Text,
    };
    let options: Vec<String> = options_str
        .map(|s| {
            s.split(',')
                .map(|o| o.trim().to_string())
                .filter(|o| !o.is_empty())
                .collect()
        })
        .unwrap_or_default();
    if (pt == PromptType::Select || pt == PromptType::Multiselect) && options.is_empty() {
        Logger::error(&format!(
            "'--type {}' requires '--options a,b,c' — add options or this prompt will fall back to text at runtime.",
            prompt_type
        ));
        return;
    }
    node.prompt_inputs.push(PromptInput {
        var: var.to_string(),
        label: label_str,
        secret,
        default,
        prompt_type: pt,
        options,
    });
    match storage::save_node(&node) {
        Ok(_) => println!(
            "  {}  Prompt input '{}' added to node '{}'.",
            "✔".bright_green(),
            var.bright_cyan(),
            node_id.bold()
        ),
        Err(e) => Logger::error(&e),
    }
}

pub fn cmd_node_prompt_remove(node_id: &str, index: usize) {
    let mut node = match storage::load_node(node_id) {
        Ok(n) => n,
        Err(e) => {
            Logger::error(&e);
            return;
        }
    };
    if !check_index(index, node.prompt_inputs.len()) {
        return;
    }
    let removed = node.prompt_inputs.remove(index);
    match storage::save_node(&node) {
        Ok(_) => println!(
            "  {}  Prompt input [{}] removed: {}",
            "✔".bright_green(),
            index,
            removed.var.bright_cyan()
        ),
        Err(e) => Logger::error(&e),
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

