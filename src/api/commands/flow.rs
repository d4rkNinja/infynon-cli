use std::io::{self, Write};

use owo_colors::OwoColorize;

use crate::api::ai;
use crate::api::executor::{execute_flow, FlowExecuteOptions};
use crate::api::storage;
use crate::api::types::{Edge, Flow};
use crate::tui::logger::Logger;

// ── flow create ───────────────────────────────────────────────────────────────

pub fn cmd_flow_create(name: &str, ai_description: Option<&str>) {
    println!();
    Logger::title("INFYNON API", "cyan");

    let id = name_to_id(name);

    if storage::flow_exists(&id) {
        Logger::error(&format!("Flow '{}' already exists. Choose a different name.", id));
        return;
    }

    let flow = if let Some(desc) = ai_description {
        create_flow_from_ai(&id, name, desc)
    } else {
        create_flow_interactive(&id, name)
    };

    match storage::save_flow(&flow) {
        Ok(path) => {
            println!();
            println!("  {}  Flow saved: {}", "✔".bright_green(), flow.id.bold());
            println!("     Path:  {}", path.display().to_string().truecolor(100, 100, 140));
            println!("     Entry: {}", flow.entry.bright_cyan());
            println!("     Edges: {}", flow.edges.len());
            println!();
        }
        Err(e) => Logger::error(&e),
    }
}

fn create_flow_interactive(id: &str, name: &str) -> Flow {
    Logger::step("Creating flow (interactive)");
    println!();

    let nodes = storage::list_nodes();
    if nodes.is_empty() {
        println!("  {}  No nodes found. Create nodes first with: infynon weave node create", "⚠".bright_yellow());
        println!();
    } else {
        println!("  Available nodes:");
        for n in &nodes {
            println!("    {}  {} {}", "·".truecolor(100, 100, 140), n.id.bright_cyan(), n.path.truecolor(140, 140, 160));
        }
        println!();
    }

    let entry = prompt("  Entry node ID: ");
    let base_url = prompt("  Base URL (e.g. http://localhost:3000): ");
    let description = prompt("  Description (optional): ");

    let mut flow = Flow::new(id, name, &entry);
    flow.base_url = if base_url.is_empty() { None } else { Some(base_url) };
    flow.description = if description.is_empty() { None } else { Some(description) };

    flow
}

fn create_flow_from_ai(id: &str, name: &str, description: &str) -> Flow {
    Logger::step(&format!("Building flow from: \"{}\"", description));

    let nodes = storage::list_nodes();

    if nodes.is_empty() {
        println!("  {}  No nodes found — creating an empty flow.", "⚠".bright_yellow());
        return Flow::new(id, name, "");
    }

    let (entry, edges) = ai::build_flow_edges(&nodes);

    println!();
    println!("  {}  AI-generated flow:", "✔".bright_green());
    println!("     Entry: {}", entry.bright_cyan());
    println!("     Edges: {}", edges.len());

    for edge in &edges {
        println!(
            "     {}  {} → {}{}",
            "·".truecolor(100, 100, 140),
            edge.from.bright_cyan(),
            edge.to.bright_cyan(),
            if edge.carry.is_empty() {
                String::new()
            } else {
                format!("  [carries: {}]", edge.carry.join(", ").truecolor(160, 160, 180).to_string())
            }
        );
    }

    let mut flow = Flow::new(id, name, &entry);
    flow.edges = edges;
    flow.description = Some(description.to_string());

    flow
}

// ── flow list ─────────────────────────────────────────────────────────────────

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
        Err(e) => { Logger::error(&e); return; }
    };

    Logger::title(&format!("Flow: {}", flow.id), "cyan");
    println!();
    println!("  {}    {}", "Name".truecolor(100, 100, 140), flow.name.bold());
    println!("  {}   {}", "Entry".truecolor(100, 100, 140), flow.entry.bright_cyan());
    if let Some(url) = &flow.base_url {
        println!("  {}     {}", "URL".truecolor(100, 100, 140), url.truecolor(160, 160, 200));
    }
    if let Some(desc) = &flow.description {
        println!("  {}    {}", "Desc".truecolor(100, 100, 140), desc.truecolor(180, 180, 200));
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
            let cond = edge.condition.as_deref()
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

pub fn cmd_flow_run(id: &str, base_url_override: Option<&str>, output: Option<&str>) {
    println!();
    Logger::title(&format!("Running flow: {}", id), "cyan");

    let flow = match storage::load_flow(id) {
        Ok(f) => f,
        Err(e) => { Logger::error(&e); return; }
    };

    let nodes = storage::load_nodes_map();

    let base_url = base_url_override
        .map(|s| s.to_string())
        .or_else(|| flow.base_url.clone())
        .unwrap_or_else(|| {
            prompt("  Base URL (e.g. http://localhost:3000): ")
        });

    println!();
    println!("  {}  Target: {}", "→".bright_cyan(), base_url.truecolor(160, 160, 200));
    println!("  {}  Nodes:  {}", "→".bright_cyan(), flow.all_node_ids().len().to_string().bright_cyan());
    println!();

    let result = execute_flow(
        &flow,
        &nodes,
        FlowExecuteOptions {
            base_url: base_url.clone(),
            on_step: Some(Box::new(|step| {
                let icon = if step.passed { "✔".bright_green().to_string() } else { "✘".bright_red().to_string() };
                let status = step.status_code.map(|s| s.to_string()).unwrap_or_else(|| "ERR".to_string());
                println!(
                    "  {}  {} {} {}  {}ms",
                    icon,
                    step.node_id.bold(),
                    step.method.bright_yellow(),
                    step.url.truecolor(100, 100, 160),
                    step.duration_ms.to_string().truecolor(200, 200, 100),
                );
                for ar in &step.assertion_results {
                    if !ar.passed {
                        println!(
                            "     {}  {} → actual: {}",
                            "✘".bright_red(),
                            ar.check.truecolor(200, 100, 100),
                            ar.actual.truecolor(180, 100, 100),
                        );
                    }
                }
                if let Some(err) = &step.error {
                    println!("     {}  {}", "Error:".bright_red(), err.truecolor(220, 100, 100));
                }
            })),
        },
    );

    println!();
    let overall = if result.passed {
        "✔ PASSED".bright_green().to_string()
    } else {
        "✘ FAILED".bright_red().to_string()
    };

    println!("  {}  {}/{} steps passed  ({}ms total)",
        overall,
        result.passed_count(),
        result.steps.len(),
        result.duration_ms(),
    );
    println!();

    // Save run result
    if let Err(e) = storage::save_run_result(&result) {
        Logger::error(&format!("Could not save run result: {}", e));
    }

    // Save report if requested
    if let Some(fmt) = output {
        save_run_report(&result, fmt);
    }
}

pub fn save_run_report_pub(result: &crate::api::types::FlowRunResult, format: &str) {
    save_run_report(result, format);
}

fn save_run_report(result: &crate::api::types::FlowRunResult, format: &str) {
    let md = build_run_markdown(result);
    let ts = result.started_at.format("%Y%m%d-%H%M%S");

    match format.to_lowercase().as_str() {
        "markdown" | "md" => {
            let path = format!("reports/{}-{}.md", result.flow_id, ts);
            std::fs::create_dir_all("reports").ok();
            std::fs::write(&path, &md).ok();
            println!("  {}  Report saved: {}", "✔".bright_green(), path);
        }
        "pdf" => {
            let path = format!("reports/{}-{}.pdf.md", result.flow_id, ts);
            std::fs::create_dir_all("reports").ok();
            std::fs::write(&path, &md).ok();
            println!("  {}  Report saved: {}", "✔".bright_green(), path);
        }
        "both" => {
            let md_path = format!("reports/{}-{}.md", result.flow_id, ts);
            let pdf_path = format!("reports/{}-{}.pdf.md", result.flow_id, ts);
            std::fs::create_dir_all("reports").ok();
            std::fs::write(&md_path, &md).ok();
            std::fs::write(&pdf_path, &md).ok();
            println!("  {}  Reports saved: {} and {}", "✔".bright_green(), md_path, pdf_path);
        }
        _ => {}
    }
}

fn build_run_markdown(result: &crate::api::types::FlowRunResult) -> String {
    let mut md = String::new();
    md.push_str(&format!("# Flow Run: {}\n\n", result.flow_id));
    md.push_str(&format!("- Started: {}\n", result.started_at.format("%Y-%m-%d %H:%M:%S UTC")));
    md.push_str(&format!("- Duration: {}ms\n", result.duration_ms()));
    md.push_str(&format!("- Status: {}\n\n", if result.passed { "PASSED" } else { "FAILED" }));
    md.push_str("## Steps\n\n");

    for step in &result.steps {
        md.push_str(&format!("### {} — {} {}\n\n", step.node_id, step.method, step.url));
        md.push_str(&format!("- Status: {}\n", step.status_code.map(|s| s.to_string()).unwrap_or_else(|| "ERR".to_string())));
        md.push_str(&format!("- Duration: {}ms\n", step.duration_ms));
        md.push_str(&format!("- Result: {}\n", if step.passed { "PASSED" } else { "FAILED" }));

        if !step.assertion_results.is_empty() {
            md.push_str("\n**Assertions:**\n\n");
            for ar in &step.assertion_results {
                let icon = if ar.passed { "✔" } else { "✘" };
                md.push_str(&format!("- {} `{}` (actual: `{}`)\n", icon, ar.check, ar.actual));
            }
        }

        if !step.extracted.is_empty() {
            md.push_str("\n**Extracted:**\n\n");
            for (k, v) in &step.extracted {
                md.push_str(&format!("- `{}` = `{}`\n", k, v));
            }
        }

        if let Some(err) = &step.error {
            md.push_str(&format!("\n**Error:** {}\n", err));
        }

        md.push('\n');
    }

    md
}

// ── flow run all ──────────────────────────────────────────────────────────────

pub fn cmd_flow_run_all(base_url_override: Option<&str>, output: Option<&str>) {
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
        cmd_flow_run(&flow.id, base_url_override, output);
        // Check last run result
        let runs = storage::load_recent_runs(&flow.id, 1);
        if let Some(run) = runs.first() {
            if run.passed { passed += 1; } else { failed += 1; }
        }
    }

    println!();
    println!("  Summary: {} passed, {} failed", passed.to_string().bright_green(), failed.to_string().bright_red());
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
        Err(e) => { Logger::error(&e); return; }
    };
    let flow2 = match storage::load_flow(flow2_id) {
        Ok(f) => f,
        Err(e) => { Logger::error(&e); return; }
    };

    let new_id = name_to_id(new_name);

    let mut merged = Flow::new(&new_id, new_name, &flow1.entry);
    merged.description = Some(format!(
        "Merged from '{}' and '{}'", flow1.name, flow2.name
    ));
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
    merged.edges.dedup_by(|a, b| a.from == b.from && a.to == b.to);

    match storage::save_flow(&merged) {
        Ok(path) => {
            println!();
            println!("  {}  Merged into: {}", "✔".bright_green(), merged.id.bold());
            println!("     Edges: {}", merged.edges.len());
            println!("     Path:  {}", path.display().to_string().truecolor(100, 100, 140));
            println!();
        }
        Err(e) => Logger::error(&e),
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn name_to_id(name: &str) -> String {
    name.to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
}

fn prompt(message: &str) -> String {
    print!("{}", message);
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    input.trim().to_string()
}
