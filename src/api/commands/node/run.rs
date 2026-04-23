pub fn cmd_node_run(id: &str, base_url: &str, set_vars: &[(String, String)], prompt_flag: bool) {
    println!();
    Logger::title(&format!("Running node: {}", id), "cyan");

    let node = match storage::load_node(id) {
        Ok(n) => n,
        Err(e) => {
            Logger::error(&e);
            return;
        }
    };

    let mut context = variables::parse_set_vars(set_vars);

    if prompt_flag {
        let unresolved = collect_unresolved_placeholders(&node, &context);
        if !unresolved.is_empty() {
            use dialoguer::Input;
            println!();
            println!(
                "  {}  Prompting for {} unresolved variable(s):",
                "→".bright_cyan(),
                unresolved.len()
            );
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
    println!(
        "  {}  {} {}{}",
        "→".bright_cyan(),
        node.method.bright_yellow(),
        base_url,
        node.path
    );
    println!();

    let on_prompt = make_cli_prompt();
    let result = executor::execute_node(&node, &context, base_url, Some(&on_prompt));
    print_step_result(&result);
}

/// Collect placeholder variable names from node path/headers/body that are not already
/// provided in context, environment, or declared as prompt_inputs.
fn collect_unresolved_placeholders(node: &Node, context: &HashMap<String, Value>) -> Vec<String> {
    let re = crate::api::variables::get_placeholder_regex();
    let prompt_vars: HashSet<&str> = node
        .prompt_inputs
        .iter()
        .map(|pi| pi.var.as_str())
        .collect();
    let mut seen: HashSet<String> = HashSet::new();
    let mut result = Vec::new();

    let mut check = |text: &str| {
        for cap in re.captures_iter(text) {
            let var = cap[1].to_string();
            if seen.contains(&var) {
                continue;
            }
            seen.insert(var.clone());
            if context.contains_key(&var) {
                continue;
            }
            if std::env::var(&var).is_ok() {
                continue;
            }
            if prompt_vars.contains(var.as_str()) {
                continue;
            }
            result.push(var);
        }
    };

    check(&node.path);
    for v in node.headers.values() {
        check(v);
    }
    if let Some(body) = &node.body_json {
        check(body);
    }

    result
}

/// Build a non-interactive on_prompt callback for AI/CI/probe mode.
/// Uses only explicit `default` values and never blocks on stdin.
pub fn make_noninteractive_prompt() -> impl Fn(&str, &[PromptInput]) -> HashMap<String, Value> {
    use crate::api::types::PromptType;
    |_node_id: &str, inputs: &[PromptInput]| -> HashMap<String, Value> {
        inputs
            .iter()
            .filter_map(|pi| {
                let val = match pi.prompt_type {
                    PromptType::Boolean => pi.default.as_deref().map(|d| {
                        if d == "true" || d == "yes" || d == "1" {
                            "true".to_string()
                        } else {
                            "false".to_string()
                        }
                    }),
                    PromptType::Select | PromptType::Multiselect | PromptType::Text => {
                        pi.default.clone()
                    }
                }?;
                Some((pi.var.clone(), Value::String(val)))
            })
            .collect()
    }
}

/// Build a CLI on_prompt callback using dialoguer for interactive input.
pub fn make_cli_prompt() -> impl Fn(&str, &[PromptInput]) -> HashMap<String, Value> {
    |node_id: &str, inputs: &[PromptInput]| -> HashMap<String, Value> {
        use crate::api::types::PromptType;
        use dialoguer::{Confirm, Input, MultiSelect, Password, Select};
        println!("\n  Node '{}' needs input:", node_id);
        let mut map = HashMap::new();
        for pi in inputs {
            let raw_label = if pi.label.is_empty() {
                pi.var.clone()
            } else {
                pi.label.clone()
            };
            let label = if raw_label.contains("{$") {
                crate::api::variables::substitute_env_placeholders(&raw_label)
            } else {
                raw_label
            };
            let val: String = match pi.prompt_type {
                PromptType::Boolean => {
                    let default_bool = pi
                        .default
                        .as_deref()
                        .map(|d| d == "true" || d == "yes" || d == "1")
                        .unwrap_or(false);
                    let answer = Confirm::new()
                        .with_prompt(format!("  {}", label))
                        .default(default_bool)
                        .interact()
                        .unwrap_or(default_bool);
                    if answer {
                        "true".to_string()
                    } else {
                        "false".to_string()
                    }
                }
                PromptType::Select => {
                    if pi.options.is_empty() {
                        // Fallback to text if no options defined
                        let mut inp = Input::<String>::new().with_prompt(format!("  {}", label));
                        if let Some(ref d) = pi.default {
                            inp = inp.default(d.clone());
                        }
                        inp.interact_text().unwrap_or_default()
                    } else {
                        let default_idx = pi
                            .default
                            .as_deref()
                            .and_then(|d| pi.options.iter().position(|o| o == d))
                            .unwrap_or(0);
                        let idx = Select::new()
                            .with_prompt(format!("  {}", label))
                            .items(&pi.options)
                            .default(default_idx)
                            .interact()
                            .unwrap_or(default_idx);
                        pi.options.get(idx).cloned().unwrap_or_default()
                    }
                }
                PromptType::Multiselect => {
                    if pi.options.is_empty() {
                        let mut inp = Input::<String>::new().with_prompt(format!("  {}", label));
                        if let Some(ref d) = pi.default {
                            inp = inp.default(d.clone());
                        }
                        inp.interact_text().unwrap_or_default()
                    } else {
                        let defaults: Vec<bool> = {
                            let selected: std::collections::HashSet<&str> = pi
                                .default
                                .as_deref()
                                .map(|d| d.split(',').map(|s| s.trim()).collect())
                                .unwrap_or_default();
                            pi.options
                                .iter()
                                .map(|o| selected.contains(o.as_str()))
                                .collect()
                        };
                        let idxs = MultiSelect::new()
                            .with_prompt(format!("  {}", label))
                            .items(&pi.options)
                            .defaults(&defaults)
                            .interact()
                            .unwrap_or_default();
                        idxs.into_iter()
                            .filter_map(|i| pi.options.get(i).cloned())
                            .collect::<Vec<_>>()
                            .join(",")
                    }
                }
                PromptType::Text => {
                    if pi.secret {
                        Password::new()
                            .with_prompt(format!("  {}", label))
                            .interact()
                            .unwrap_or_default()
                    } else {
                        let mut inp = Input::<String>::new().with_prompt(format!("  {}", label));
                        if let Some(ref d) = pi.default {
                            inp = inp.default(d.clone());
                        }
                        inp.interact_text().unwrap_or_default()
                    }
                }
            };
            map.insert(pi.var.clone(), Value::String(val));
        }
        map
    }
}

fn print_step_result(step: &crate::api::types::StepResult) {
    let status_icon = if step.passed {
        "✔".bright_green().to_string()
    } else {
        "✘".bright_red().to_string()
    };
    let status_str = step
        .status_code
        .map(|s| s.to_string())
        .unwrap_or_else(|| "—".to_string());

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
        let icon = if ar.passed {
            "✔".bright_green().to_string()
        } else {
            "✘".bright_red().to_string()
        };
        println!(
            "     {}  {} (actual: {})",
            icon,
            ar.check.truecolor(200, 200, 220),
            ar.actual.truecolor(160, 160, 180)
        );
    }

    if !step.extracted.is_empty() {
        println!();
        println!("     {}  Extracted:", "→".truecolor(100, 100, 140));
        for (k, v) in &step.extracted {
            let display = match v {
                Value::String(s) => {
                    if s.len() > 40 {
                        format!("{}...", &s[..40])
                    } else {
                        s.clone()
                    }
                }
                other => other.to_string(),
            };
            println!(
                "        {}  {} = {}",
                "·".truecolor(100, 100, 140),
                k.bright_cyan(),
                display.truecolor(180, 180, 200)
            );
        }
    }
    println!();
}

// ── node export ───────────────────────────────────────────────────────────────
