pub fn cmd_flow_list() {
    println!();
    Logger::title("Flows", "cyan");

    let flows = storage::list_flows();

    if flows.is_empty() {
        println!();
        println!("  No flows yet. Create one with: infynon weave flow create <name>");
        println!();
        return;
    }

    println!();
    println!(
        "  {:<24} {:<12} {:<24} {}",
        "ID".truecolor(100, 100, 140),
        "Nodes".truecolor(100, 100, 140),
        "Entry".truecolor(100, 100, 140),
        "Base URL".truecolor(100, 100, 140),
    );
    println!("  {}", "─".repeat(72).truecolor(50, 50, 80));

    for flow in &flows {
        let node_count = flow.all_node_ids().len();
        let base = flow.base_url.as_deref().unwrap_or("—");

        println!(
            "  {:<24} {:<12} {:<24} {}",
            flow.id.bold(),
            node_count.to_string().bright_cyan(),
            flow.entry.truecolor(160, 160, 200),
            base.truecolor(120, 120, 150),
        );
    }
    println!();
    println!("  {} flow(s)", flows.len().to_string().bright_cyan());
    println!();
}

// ── flow show ─────────────────────────────────────────────────────────────────

pub fn cmd_flow_show(id: &str) {
    println!();
    let flow = match storage::load_flow(id) {
        Ok(f) => f,
        Err(e) => {
            Logger::error(&e);
            return;
        }
    };

    Logger::title(&format!("Flow: {}", flow.id), "cyan");
    println!();
    println!(
        "  {}    {}",
        "Name".truecolor(100, 100, 140),
        flow.name.bold()
    );
    println!(
        "  {}   {}",
        "Entry".truecolor(100, 100, 140),
        flow.entry.bright_cyan()
    );
    if let Some(url) = &flow.base_url {
        println!(
            "  {}     {}",
            "URL".truecolor(100, 100, 140),
            url.truecolor(160, 160, 200)
        );
    }
    if let Some(desc) = &flow.description {
        println!(
            "  {}    {}",
            "Desc".truecolor(100, 100, 140),
            desc.truecolor(180, 180, 200)
        );
    }

    println!();
    println!("  {}  Graph (BFS order):", "→".truecolor(100, 100, 140));
    println!();

    // Render a simple ASCII tree
    render_flow_ascii(&flow);

    println!();
}

fn render_flow_ascii(flow: &Flow) {
    let nodes_in_order = flow.all_node_ids();

    for (i, node_id) in nodes_in_order.iter().enumerate() {
        let is_entry = node_id == &flow.entry;
        let entry_mark = if is_entry { " ← entry" } else { "" };

        println!(
            "  {}  {}{}",
            format!("{}.", i + 1).truecolor(0, 210, 255),
            node_id.bold(),
            entry_mark.truecolor(100, 100, 140),
        );

        // Show outgoing edges
        let successors: Vec<&Edge> = flow.successors(node_id);
        for edge in successors {
            let carry = if edge.carry.is_empty() {
                "(all context)".to_string()
            } else {
                edge.carry.join(", ")
            };
            let cond = edge
                .condition
                .as_deref()
                .map(|c| format!(" if: {}", c))
                .unwrap_or_default();
            println!(
                "     {}  → {}  [{}]{}",
                "│".truecolor(50, 50, 80),
                edge.to.bright_cyan(),
                carry.truecolor(140, 140, 160),
                cond.truecolor(200, 160, 80),
            );
        }
    }
}

// ── flow run ──────────────────────────────────────────────────────────────────
