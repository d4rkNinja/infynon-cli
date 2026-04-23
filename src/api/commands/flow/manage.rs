pub fn cmd_flow_run_all(
    base_url_override: Option<&str>,
    set_vars: &[(String, String)],
    output: Option<&str>,
) {
    let flows = storage::list_flows();
    if flows.is_empty() {
        println!();
        println!("  No flows to run.");
        println!();
        return;
    }

    println!();
    Logger::title(&format!("Running all {} flow(s)", flows.len()), "cyan");

    let mut passed = 0;
    let mut failed = 0;

    for flow in &flows {
        cmd_flow_run(&flow.id, base_url_override, set_vars, output);
        let runs = storage::load_recent_runs(&flow.id, 1);
        if let Some(run) = runs.first() {
            if run.passed {
                passed += 1;
            } else {
                failed += 1;
            }
        }
    }

    println!();
    println!(
        "  Summary: {} passed, {} failed",
        passed.to_string().bright_green(),
        failed.to_string().bright_red()
    );
    println!();
}

// ── flow remove ───────────────────────────────────────────────────────────────

pub fn cmd_flow_remove(id: &str) {
    println!();
    let confirm = prompt(&format!("  Remove flow '{}'? [y/N]: ", id.bold()));
    if confirm.trim().to_lowercase() != "y" {
        println!("  Cancelled.");
        println!();
        return;
    }

    match storage::delete_flow(id) {
        Ok(()) => {
            println!("  {}  Flow '{}' removed.", "✔".bright_green(), id.bold());
            println!();
        }
        Err(e) => Logger::error(&e),
    }
}

// ── flow merge ────────────────────────────────────────────────────────────────

pub fn cmd_flow_merge(flow1_id: &str, flow2_id: &str, join_at: &str, new_name: &str) {
    println!();
    Logger::title("Flow Merge", "cyan");

    let flow1 = match storage::load_flow(flow1_id) {
        Ok(f) => f,
        Err(e) => {
            Logger::error(&e);
            return;
        }
    };
    let flow2 = match storage::load_flow(flow2_id) {
        Ok(f) => f,
        Err(e) => {
            Logger::error(&e);
            return;
        }
    };

    let new_id = name_to_id(new_name);

    let mut merged = Flow::new(&new_id, new_name, &flow1.entry);
    merged.description = Some(format!("Merged from '{}' and '{}'", flow1.name, flow2.name));
    merged.base_url = flow1.base_url.clone();

    // Add all edges from both flows
    merged.edges.extend(flow1.edges.clone());
    merged.edges.extend(flow2.edges.clone());

    // Connect the join point: link flow1's end to flow2's entry via join_at
    // Find the node in flow1 that connects to join_at (or use join_at as the bridge)
    merged.edges.push(Edge {
        from: join_at.to_string(),
        to: flow2.entry.clone(),
        carry: vec![],
        condition: None,
    });

    // Deduplicate edges
    merged
        .edges
        .dedup_by(|a, b| a.from == b.from && a.to == b.to);

    match storage::save_flow(&merged) {
        Ok(path) => {
            println!();
            println!(
                "  {}  Merged into: {}",
                "✔".bright_green(),
                merged.id.bold()
            );
            println!("     Edges: {}", merged.edges.len());
            println!(
                "     Path:  {}",
                path.display().to_string().truecolor(100, 100, 140)
            );
            println!();
        }
        Err(e) => Logger::error(&e),
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

