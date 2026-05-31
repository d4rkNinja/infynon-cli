pub fn cmd_flow_run_all(
    base_url_override: Option<&str>,
    set_vars: &[(String, String)],
    format: Option<&str>,
    output: Option<&str>,
    no_input: bool,
) -> i32 {
    let flows = storage::list_flows();
    if flows.is_empty() {
        if let Some(stdout_format) = format {
            render_run_suite(&[], stdout_format);
        } else {
            println!();
            println!("  No flows to run.");
            println!();
        }
        return 0;
    }

    let human_output = format.is_none();
    if human_output {
        println!();
        Logger::title(&format!("Running all {} flow(s)", flows.len()), "cyan");
    }

    let mut records = Vec::new();

    for flow in &flows {
        match run_flow_once(
            &flow.id,
            base_url_override,
            set_vars,
            no_input,
            human_output,
        ) {
            Ok(result) => {
                if let Err(e) = storage::save_run_result(&result) {
                    Logger::error(&format!("Could not save run result: {}", e));
                }
                if let Some(report_format) = output {
                    save_run_report_pub(&result, report_format);
                }
                let exit_code = classify_flow_result(&result);
                records.push(FlowRunRecord {
                    flow_id: result.flow_id.clone(),
                    flow_name: result.flow_name.clone(),
                    exit_code,
                    result: Some(result),
                    error: None,
                });
            }
            Err(failure) => {
                if human_output {
                    Logger::error(&format!("Flow '{}': {}", flow.id, failure.message));
                }
                records.push(FlowRunRecord {
                    flow_id: failure.flow_id,
                    flow_name: failure.flow_name,
                    exit_code: failure.code,
                    result: None,
                    error: Some(failure.message),
                });
            }
        }
    }

    if let Some(stdout_format) = format {
        render_run_suite(&records, stdout_format);
    } else {
        let passed = records
            .iter()
            .filter(|record| record.exit_code == 0)
            .count();
        let failed = records.len().saturating_sub(passed);
        println!();
        println!("  Summary: {} passed, {} failed", passed, failed);
        println!();
    }

    suite_exit_code(&records)
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
