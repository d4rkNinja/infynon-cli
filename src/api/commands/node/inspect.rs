pub fn cmd_node_get(id: &str) {
    println!();
    match storage::load_node(id) {
        Ok(node) => print_node_detail(&node),
        Err(e) => Logger::error(&e),
    }
}

fn print_node_detail(node: &Node) {
    Logger::title(&format!("Node: {}", node.id), "cyan");
    println!();
    println!(
        "  {}     {}",
        "Name".truecolor(100, 100, 140),
        node.name.bold()
    );
    println!(
        "  {}   {} {}",
        "Method".truecolor(100, 100, 140),
        node.method.bright_yellow(),
        node.path.bright_cyan()
    );
    if let Some(desc) = &node.description {
        println!(
            "  {}    {}",
            "Desc".truecolor(100, 100, 140),
            desc.truecolor(180, 180, 200)
        );
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
            println!(
                "     {}  {} ← {}",
                "·".truecolor(100, 100, 140),
                e.name.bright_cyan(),
                e.from
            );
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
            println!(
                "     {}  {} [{}]",
                "·".truecolor(100, 100, 140),
                a.check.bright_cyan(),
                fail_label
            );
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
            node_flows
                .entry(node_id)
                .or_default()
                .push(flow.name.clone());
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
        let flow_names = node_flows
            .get(&node.id)
            .map(|names| names.join(", "))
            .unwrap_or_else(|| "—".truecolor(60, 60, 80).to_string());

        let method_colored = match node.method.as_str() {
            "GET" => node.method.bright_green().to_string(),
            "POST" => node.method.bright_cyan().to_string(),
            "PUT" => node.method.bright_yellow().to_string(),
            "PATCH" => node.method.truecolor(255, 140, 50).to_string(),
            "DELETE" => node.method.bright_red().to_string(),
            other => other.to_string(),
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
    let using_flows: Vec<&str> = flows
        .iter()
        .filter(|f| f.all_node_ids().contains(&id.to_string()))
        .map(|f| f.name.as_str())
        .collect();

    if !using_flows.is_empty() {
        println!();
        println!(
            "  {}  Node '{}' is used in: {}",
            "⚠".bright_yellow(),
            id.bold(),
            using_flows.join(", ")
        );
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
                    println!(
                        "  {}  Cloned '{}' → '{}'",
                        "✔".bright_green(),
                        id.bold(),
                        new_id.bold()
                    );
                    println!();
                }
                Err(e) => Logger::error(&e),
            }
        }
        Err(e) => Logger::error(&e),
    }
}

// ── node run ──────────────────────────────────────────────────────────────────
