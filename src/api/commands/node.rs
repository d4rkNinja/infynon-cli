use std::collections::{HashMap, HashSet};
use std::io::{self, Write};
use std::sync::OnceLock;

use owo_colors::OwoColorize;
use regex::Regex;
use serde_json::Value;

use crate::api::ai;
use crate::api::executor;
use crate::api::storage;
use crate::api::types::{Assertion, Edge, Extraction, Node, OnFail, PromptInput};
use crate::api::variables;
use crate::tui::logger::Logger;

static PLACEHOLDER_RE: OnceLock<Regex> = OnceLock::new();

// ── node create ───────────────────────────────────────────────────────────────

pub fn cmd_node_create(ai_description: Option<&str>) {
    println!();
    Logger::title("INFYNON API", "cyan");

    let node = if let Some(desc) = ai_description {
        create_node_from_ai(desc)
    } else {
        create_node_interactive()
    };

    match storage::save_node(&node) {
        Ok(path) => {
            println!();
            println!("  {}  Node saved: {}", "✔".bright_green(), node.id.bold());
            println!("     Path:   {}", path.display().to_string().truecolor(100, 100, 140));
            println!("     Method: {} {}", node.method.bright_cyan(), node.path);
            println!();
        }
        Err(e) => {
            Logger::error(&format!("Failed to save node: {}", e));
        }
    }
}

fn create_node_interactive() -> Node {
    Logger::step("Creating new node (interactive)");
    println!();

    let id = prompt("  Node ID (e.g. login, create-cart): ");
    let name = prompt("  Display name: ");
    let method = prompt("  HTTP method [GET/POST/PUT/PATCH/DELETE]: ").to_uppercase();
    let method = if method.is_empty() { "GET".to_string() } else { method };
    let path = prompt("  Path (e.g. /users/{user_id}): ");

    let mut node = Node::new(&id, &name, &method, &path);

    // Headers
    println!();
    println!("  {}  Add headers? (leave blank to skip)", "→".truecolor(100, 100, 140));
    loop {
        let key = prompt("    Header name (or Enter to finish): ");
        if key.is_empty() { break; }
        let val = prompt(&format!("    {} value: ", key));
        node.headers.insert(key, val);
    }

    // Body
    if method == "POST" || method == "PUT" || method == "PATCH" {
        println!();
        let body = prompt("  JSON body (or Enter to skip, use {var} for placeholders): ");
        if !body.is_empty() {
            node.body_json = Some(body);
        }
    }

    // Extractions
    println!();
    println!("  {}  Add extractions? (pull values from response into context)", "→".truecolor(100, 100, 140));
    let ai_ext = prompt("    Auto-generate extractions? [Y/n]: ");
    if ai_ext.trim().to_lowercase() != "n" {
        node.extractions = ai::generate_extractions(&node);
        println!("    Generated {} extraction(s):", node.extractions.len());
        for e in &node.extractions {
            println!("      {}  {} ← {}", "·".truecolor(100, 100, 140), e.name.bright_cyan(), e.from);
        }
    } else {
        loop {
            let name = prompt("    Variable name (or Enter to finish): ");
            if name.is_empty() { break; }
            let from = prompt(&format!("    Extract '{}' from (e.g. body.token, header.location): ", name));
            node.extractions.push(Extraction { name, from });
        }
    }

    // Assertions
    println!();
    println!("  {}  Add assertions?", "→".truecolor(100, 100, 140));
    let ai_assert = prompt("    Auto-generate assertions? [Y/n]: ");
    if ai_assert.trim().to_lowercase() != "n" {
        node.assertions = ai::generate_assertions(&node);
        println!("    Generated {} assertion(s):", node.assertions.len());
        for a in &node.assertions {
            println!("      {}  {}", "·".truecolor(100, 100, 140), a.check.bright_cyan());
        }
    } else {
        loop {
            let check = prompt("    Assertion (e.g. 'status == 201', or Enter to finish): ");
            if check.is_empty() { break; }
            node.assertions.push(Assertion { check, on_fail: OnFail::Stop, enabled: true });
        }
    }

    node
}

fn create_node_from_ai(description: &str) -> Node {
    Logger::step(&format!("Generating node from: \"{}\"", description));

    // Parse description heuristically
    // Expect format: "METHOD /path" or just a description
    // Try to parse "METHOD /path [rest...]"
    let words: Vec<&str> = description.splitn(3, ' ').collect();
    let (method, path) = match words.as_slice() {
        [m, p, ..] => {
            let mu = m.to_uppercase();
            if ["GET", "POST", "PUT", "PATCH", "DELETE"].contains(&mu.as_str()) && p.starts_with('/') {
                // Take only the path token (stop at first space)
                let clean_path = p.split_whitespace().next().unwrap_or(p).to_string();
                (mu, clean_path)
            } else {
                (infer_method(description), infer_path(description))
            }
        }
        _ => (infer_method(description), infer_path(description)),
    };

    // Build id from path
    let id = path_to_id(&path);
    let name = format!("{} {}", title_case(&method.to_lowercase()), path.trim_matches('/').replace('/', " ").replace('{', "").replace('}', ""));

    let mut node = Node::new(&id, &name, &method, &path);

    // Auto-generate extractions and assertions
    node.extractions = ai::generate_extractions(&node);
    node.assertions = ai::generate_assertions(&node);

    // Add default JSON content-type for POST/PUT/PATCH
    if ["POST", "PUT", "PATCH"].contains(&method.as_str()) {
        node.headers.insert("Content-Type".to_string(), "application/json".to_string());
    }

    // Add auth header placeholder if path doesn't look like auth itself
    let path_lower = path.to_lowercase();
    if !path_lower.contains("login") && !path_lower.contains("auth") {
        node.headers.insert("Authorization".to_string(), "Bearer {token}".to_string());
    }

    println!();
    println!("  {}  Generated node:", "✔".bright_green());
    println!("     ID:     {}", node.id.bright_cyan());
    println!("     Method: {} {}", node.method.bright_yellow(), node.path);
    println!("     Extractions: {}", node.extractions.len());
    println!("     Assertions:  {}", node.assertions.len());

    node
}

fn has_any_keyword(text: &str, keywords: &[&str]) -> bool {
    keywords.iter().any(|k| text.contains(k))
}

fn infer_method(desc: &str) -> String {
    let d = desc.to_lowercase();
    if has_any_keyword(&d, &["create", "add", "register", "login", "post"]) {
        "POST".to_string()
    } else if has_any_keyword(&d, &["update", "edit", "change"]) {
        "PATCH".to_string()
    } else if has_any_keyword(&d, &["delete", "remove"]) {
        "DELETE".to_string()
    } else {
        "GET".to_string()
    }
}

fn infer_path(desc: &str) -> String {
    // Look for /path-like pattern
    if let Some(pos) = desc.find('/') {
        let end = desc[pos..].find(|c: char| c == ' ' || c == '"' || c == '\'').unwrap_or(desc.len() - pos);
        return desc[pos..pos + end].to_string();
    }
    // Build from description keywords
    let keywords: Vec<&str> = desc.split_whitespace()
        .filter(|w| !["GET", "POST", "PUT", "PATCH", "DELETE", "a", "an", "the", "to", "from", "with"].contains(w))
        .take(2)
        .collect();
    format!("/{}", keywords.join("/").to_lowercase())
}

fn path_to_id(path: &str) -> String {
    path.trim_matches('/')
        .replace('/', "-")
        .replace('{', "")
        .replace('}', "")
        .to_lowercase()
}

fn title_case(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

// ── node get ──────────────────────────────────────────────────────────────────

pub fn cmd_node_get(id: &str) {
    println!();
    match storage::load_node(id) {
        Ok(node) => print_node_detail(&node),
        Err(e)   => Logger::error(&e),
    }
}

fn print_node_detail(node: &Node) {
    Logger::title(&format!("Node: {}", node.id), "cyan");
    println!();
    println!("  {}     {}", "Name".truecolor(100, 100, 140), node.name.bold());
    println!("  {}   {} {}", "Method".truecolor(100, 100, 140), node.method.bright_yellow(), node.path.bright_cyan());
    if let Some(desc) = &node.description {
        println!("  {}    {}", "Desc".truecolor(100, 100, 140), desc.truecolor(180, 180, 200));
    }

    if !node.headers.is_empty() {
        println!();
        println!("  {}  Headers:", "→".truecolor(100, 100, 140));
        for (k, v) in &node.headers {
            println!("     {}: {}", k.bright_cyan(), v);
        }
    }

    if let Some(body) = &node.body_json {
        println!();
        println!("  {}  Body:", "→".truecolor(100, 100, 140));
        println!("     {}", body.truecolor(180, 180, 200));
    }

    if !node.extractions.is_empty() {
        println!();
        println!("  {}  Extractions:", "→".truecolor(100, 100, 140));
        for e in &node.extractions {
            println!("     {}  {} ← {}", "·".truecolor(100, 100, 140), e.name.bright_cyan(), e.from);
        }
    }

    if !node.assertions.is_empty() {
        println!();
        println!("  {}  Assertions:", "→".truecolor(100, 100, 140));
        for a in &node.assertions {
            let fail_label = match a.on_fail {
                OnFail::Stop => "stop".bright_red().to_string(),
                OnFail::Warn => "warn".bright_yellow().to_string(),
            };
            println!("     {}  {} [{}]", "·".truecolor(100, 100, 140), a.check.bright_cyan(), fail_label);
        }
    }
    println!();
}

// ── node list ─────────────────────────────────────────────────────────────────

pub fn cmd_node_list() {
    println!();
    Logger::title("Node Library", "cyan");

    let nodes = storage::list_nodes();
    let flows = storage::list_flows();

    if nodes.is_empty() {
        println!();
        println!("  No nodes yet. Create one with: infynon weave node create");
        println!();
        return;
    }

    // Build a map of node_id → flow names it appears in
    let mut node_flows: HashMap<String, Vec<String>> = HashMap::new();
    for flow in &flows {
        for node_id in flow.all_node_ids() {
            node_flows.entry(node_id).or_default().push(flow.name.clone());
        }
    }

    println!();
    println!(
        "  {:<20} {:<8} {:<30} {:<20}",
        "ID".truecolor(100, 100, 140),
        "Method".truecolor(100, 100, 140),
        "Path".truecolor(100, 100, 140),
        "Used in".truecolor(100, 100, 140),
    );
    println!("  {}", "─".repeat(80).truecolor(50, 50, 80));

    for node in &nodes {
        let flow_names = node_flows.get(&node.id)
            .map(|names| names.join(", "))
            .unwrap_or_else(|| "—".truecolor(60, 60, 80).to_string());

        let method_colored = match node.method.as_str() {
            "GET"    => node.method.bright_green().to_string(),
            "POST"   => node.method.bright_cyan().to_string(),
            "PUT"    => node.method.bright_yellow().to_string(),
            "PATCH"  => node.method.truecolor(255, 140, 50).to_string(),
            "DELETE" => node.method.bright_red().to_string(),
            other    => other.to_string(),
        };

        println!(
            "  {:<20} {:<8} {:<30} {}",
            node.id.bold(),
            method_colored,
            node.path.bright_cyan(),
            flow_names.truecolor(160, 160, 180),
        );
    }
    println!();
    println!("  {} nodes total", nodes.len().to_string().bright_cyan());
    println!();
}

// ── node remove ───────────────────────────────────────────────────────────────

pub fn cmd_node_remove(id: &str) {
    // Check if node is used in any flows
    let flows = storage::list_flows();
    let using_flows: Vec<&str> = flows.iter()
        .filter(|f| f.all_node_ids().contains(&id.to_string()))
        .map(|f| f.name.as_str())
        .collect();

    if !using_flows.is_empty() {
        println!();
        println!("  {}  Node '{}' is used in: {}", "⚠".bright_yellow(), id.bold(), using_flows.join(", "));
        let confirm = prompt("  Remove anyway? [y/N]: ");
        if confirm.trim().to_lowercase() != "y" {
            println!("  Cancelled.");
            println!();
            return;
        }
    }

    match storage::delete_node(id) {
        Ok(()) => {
            println!();
            println!("  {}  Node '{}' removed.", "✔".bright_green(), id.bold());
            println!();
        }
        Err(e) => Logger::error(&e),
    }
}

// ── node clone ────────────────────────────────────────────────────────────────

pub fn cmd_node_clone(id: &str, new_id: &str) {
    println!();
    match storage::load_node(id) {
        Ok(mut node) => {
            node.id = new_id.to_string();
            node.name = format!("{} (copy)", node.name);
            match storage::save_node(&node) {
                Ok(_) => {
                    println!("  {}  Cloned '{}' → '{}'", "✔".bright_green(), id.bold(), new_id.bold());
                    println!();
                }
                Err(e) => Logger::error(&e),
            }
        }
        Err(e) => Logger::error(&e),
    }
}

// ── node run ──────────────────────────────────────────────────────────────────

pub fn cmd_node_run(id: &str, base_url: &str, set_vars: &[(String, String)], prompt_flag: bool) {
    println!();
    Logger::title(&format!("Running node: {}", id), "cyan");

    let node = match storage::load_node(id) {
        Ok(n) => n,
        Err(e) => { Logger::error(&e); return; }
    };

    let mut context = variables::parse_set_vars(set_vars);

    if prompt_flag {
        let unresolved = collect_unresolved_placeholders(&node, &context);
        if !unresolved.is_empty() {
            use dialoguer::Input;
            println!();
            println!("  {}  Prompting for {} unresolved variable(s):", "→".bright_cyan(), unresolved.len());
            for var in &unresolved {
                let val: String = Input::<String>::new()
                    .with_prompt(format!("  {}", var))
                    .interact_text()
                    .unwrap_or_default();
                context.insert(var.clone(), Value::String(val));
            }
        }
    }

    println!();
    println!("  {}  {} {}{}", "→".bright_cyan(), node.method.bright_yellow(), base_url, node.path);
    println!();

    let on_prompt = make_cli_prompt();
    let result = executor::execute_node(&node, &context, base_url, Some(&on_prompt));
    print_step_result(&result);
}

/// Collect placeholder variable names from node path/headers/body that are not already
/// provided in context, environment, or declared as prompt_inputs.
fn collect_unresolved_placeholders(node: &Node, context: &HashMap<String, Value>) -> Vec<String> {
    let re = PLACEHOLDER_RE.get_or_init(|| {
        Regex::new(r"\{([A-Za-z_][A-Za-z0-9_]*)\}").unwrap()
    });
    let prompt_vars: HashSet<&str> = node.prompt_inputs.iter().map(|pi| pi.var.as_str()).collect();
    let mut seen: HashSet<String> = HashSet::new();
    let mut result = Vec::new();

    let mut check = |text: &str| {
        for cap in re.captures_iter(text) {
            let var = cap[1].to_string();
            if seen.contains(&var) { continue; }
            seen.insert(var.clone());
            if context.contains_key(&var) { continue; }
            if std::env::var(&var).is_ok() { continue; }
            if prompt_vars.contains(var.as_str()) { continue; }
            result.push(var);
        }
    };

    check(&node.path);
    for v in node.headers.values() { check(v); }
    if let Some(body) = &node.body_json { check(body); }

    result
}

/// Build a CLI on_prompt callback using dialoguer for interactive input.
pub fn make_cli_prompt() -> impl Fn(&str, &[PromptInput]) -> HashMap<String, Value> {
    |node_id: &str, inputs: &[PromptInput]| -> HashMap<String, Value> {
        use dialoguer::{Input, Password};
        println!("\n  Node '{}' needs input:", node_id);
        let mut map = HashMap::new();
        for pi in inputs {
            let label = if pi.label.is_empty() { pi.var.clone() } else { pi.label.clone() };
            let val: String = if pi.secret {
                let pw = Password::new().with_prompt(format!("  {}", label));
                pw.interact().unwrap_or_default()
            } else {
                let mut inp = Input::<String>::new().with_prompt(format!("  {}", label));
                if let Some(ref d) = pi.default {
                    inp = inp.default(d.clone());
                }
                inp.interact_text().unwrap_or_default()
            };
            map.insert(pi.var.clone(), Value::String(val));
        }
        map
    }
}

fn print_step_result(step: &crate::api::types::StepResult) {
    let status_icon = if step.passed { "✔".bright_green().to_string() } else { "✘".bright_red().to_string() };
    let status_str = step.status_code.map(|s| s.to_string()).unwrap_or_else(|| "—".to_string());

    println!(
        "  {}  {} {}  {}ms",
        status_icon,
        status_str.bold(),
        step.url.truecolor(100, 100, 160),
        step.duration_ms.to_string().bright_yellow(),
    );

    if let Some(err) = &step.error {
        println!("     {}  {}", "Error:".bright_red(), err);
    }

    for ar in &step.assertion_results {
        let icon = if ar.passed { "✔".bright_green().to_string() } else { "✘".bright_red().to_string() };
        println!("     {}  {} (actual: {})", icon, ar.check.truecolor(200, 200, 220), ar.actual.truecolor(160, 160, 180));
    }

    if !step.extracted.is_empty() {
        println!();
        println!("     {}  Extracted:", "→".truecolor(100, 100, 140));
        for (k, v) in &step.extracted {
            let display = match v {
                Value::String(s) => if s.len() > 40 { format!("{}...", &s[..40]) } else { s.clone() },
                other => other.to_string(),
            };
            println!("        {}  {} = {}", "·".truecolor(100, 100, 140), k.bright_cyan(), display.truecolor(180, 180, 200));
        }
    }
    println!();
}

// ── node export ───────────────────────────────────────────────────────────────

pub fn cmd_node_export(id: &str, format: &str, base_url: Option<&str>) {
    let node = match storage::load_node(id) {
        Ok(n) => n,
        Err(e) => { Logger::error(&e); return; }
    };

    let resolved_base = base_url.map(|s| s.to_string())
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
            Logger::error(&format!("Unknown export format: '{}'. Use: curl, json", format));
        }
    }
}

// ── node assertion commands ───────────────────────────────────────────────────

pub fn cmd_node_assertion_list(node_id: &str) {
    println!();
    let node = match storage::load_node(node_id) {
        Ok(n) => n,
        Err(e) => { Logger::error(&e); return; }
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
        Err(e) => { Logger::error(&e); return; }
    };
    if idx >= node.assertions.len() {
        Logger::error(&format!("Index {} out of range (0..{})", idx, node.assertions.len()));
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
        Err(e) => { Logger::error(&e); return; }
    };
    if idx >= node.assertions.len() {
        Logger::error(&format!("Index {} out of range (0..{})", idx, node.assertions.len()));
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
        Err(e) => { Logger::error(&e); return; }
    };
    if idx >= node.assertions.len() {
        Logger::error(&format!("Index {} out of range (0..{})", idx, node.assertions.len()));
        return;
    }
    node.assertions[idx].enabled = !node.assertions[idx].enabled;
    let state = if node.assertions[idx].enabled { "enabled" } else { "disabled" };
    match storage::save_node(&node) {
        Ok(_) => println!("  {}  Assertion [{}] {}.", "✔".bright_green(), idx, state),
        Err(e) => Logger::error(&e),
    }
}

pub fn cmd_node_assertion_add(node_id: &str, check: &str, on_fail_str: &str) {
    let mut node = match storage::load_node(node_id) {
        Ok(n) => n,
        Err(e) => { Logger::error(&e); return; }
    };
    let on_fail = match on_fail_str.to_lowercase().as_str() {
        "warn" | "continue" => OnFail::Warn,
        _ => OnFail::Stop,
    };
    node.assertions.push(Assertion { check: check.to_string(), on_fail, enabled: true });
    match storage::save_node(&node) {
        Ok(_) => println!("  {}  Assertion added: {}", "✔".bright_green(), check.bright_cyan()),
        Err(e) => Logger::error(&e),
    }
}

pub fn cmd_node_assertion_remove(node_id: &str, idx: usize) {
    let mut node = match storage::load_node(node_id) {
        Ok(n) => n,
        Err(e) => { Logger::error(&e); return; }
    };
    if idx >= node.assertions.len() {
        Logger::error(&format!("Index {} out of range (0..{})", idx, node.assertions.len()));
        return;
    }
    let removed = node.assertions.remove(idx);
    match storage::save_node(&node) {
        Ok(_) => println!("  {}  Assertion [{}] removed: {}", "✔".bright_green(), idx, removed.check.bright_cyan()),
        Err(e) => Logger::error(&e),
    }
}

// ── node prompt commands ──────────────────────────────────────────────────────

pub fn cmd_node_prompt_list(node_id: &str) {
    println!();
    let node = match storage::load_node(node_id) {
        Ok(n) => n,
        Err(e) => { Logger::error(&e); return; }
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
            format!(" (default: \"{}\")", d).truecolor(140, 140, 160).to_string()
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
) {
    let mut node = match storage::load_node(node_id) {
        Ok(n) => n,
        Err(e) => { Logger::error(&e); return; }
    };
    let label_str = if label.is_empty() { var.to_string() } else { label.to_string() };
    node.prompt_inputs.push(PromptInput {
        var: var.to_string(),
        label: label_str,
        secret,
        default,
    });
    match storage::save_node(&node) {
        Ok(_) => println!("  {}  Prompt input '{}' added to node '{}'.", "✔".bright_green(), var.bright_cyan(), node_id.bold()),
        Err(e) => Logger::error(&e),
    }
}

pub fn cmd_node_prompt_remove(node_id: &str, index: usize) {
    let mut node = match storage::load_node(node_id) {
        Ok(n) => n,
        Err(e) => { Logger::error(&e); return; }
    };
    if index >= node.prompt_inputs.len() {
        Logger::error(&format!("Index {} out of range (0..{})", index, node.prompt_inputs.len()));
        return;
    }
    let removed = node.prompt_inputs.remove(index);
    match storage::save_node(&node) {
        Ok(_) => println!("  {}  Prompt input [{}] removed: {}", "✔".bright_green(), index, removed.var.bright_cyan()),
        Err(e) => Logger::error(&e),
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn prompt(message: &str) -> String {
    print!("{}", message);
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    input.trim().to_string()
}

pub fn print_step_result_pub(step: &crate::api::types::StepResult) {
    print_step_result(step);
}
