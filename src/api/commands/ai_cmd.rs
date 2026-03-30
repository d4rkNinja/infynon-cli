use std::collections::HashMap;

use owo_colors::OwoColorize;

use crate::api::ai;
use crate::api::executor::{execute_flow, FlowExecuteOptions};
use crate::api::storage;
use crate::api::types::{Edge, Node};
use crate::tui::logger::Logger;

// ── ai suggest ────────────────────────────────────────────────────────────────

pub fn cmd_ai_suggest(after_node_id: &str) {
    println!();
    Logger::title("AI Suggest", "cyan");

    let current = match storage::load_node(after_node_id) {
        Ok(n) => n,
        Err(_) => {
            Logger::error(&format!("Node '{}' not found.", after_node_id));
            return;
        }
    };

    let all_nodes = storage::list_nodes();
    let candidates: Vec<_> = all_nodes.iter().filter(|n| n.id != after_node_id).cloned().collect();
    let suggestions = ai::suggest_next_nodes(&current, &candidates);

    if suggestions.is_empty() {
        println!();
        println!("  No suggestions found. Try adding more nodes to the library.");
        println!();
        return;
    }

    println!();
    println!(
        "  Suggested next nodes after {} {} {}:",
        current.method.bright_yellow(),
        current.path.bright_cyan(),
        format!("({})", after_node_id).truecolor(100, 100, 140),
    );
    println!();

    for (i, s) in suggestions.iter().enumerate() {
        let icon = if i == 0 { "★".bright_yellow().to_string() } else { "·".truecolor(100, 100, 140).to_string() };
        println!(
            "  {}  {} {} {}  — {}",
            icon,
            s.node.id.bold(),
            s.node.method.bright_yellow(),
            s.node.path.bright_cyan(),
            format!("{:.0}% confidence", s.confidence * 100.0).truecolor(140, 140, 160),
        );
        println!(
            "     Reason: {}",
            s.reason.truecolor(180, 180, 200),
        );
        if !s.edge.carry.is_empty() {
            println!(
                "     Carries: {}",
                s.edge.carry.join(", ").bright_yellow(),
            );
        }
        println!();
    }

    println!("  Attach best match: infynon api attach {} {} --ai", after_node_id, suggestions[0].node.id);
    println!();
}

// ── ai attach ────────────────────────────────────────────────────────────────

pub fn cmd_ai_attach(after_node_id: &str, flow_id: Option<&str>) {
    super::attach::cmd_attach_ai(after_node_id, None, flow_id);
}

// ── ai complete ───────────────────────────────────────────────────────────────

pub fn cmd_ai_complete(flow_id: &str) {
    println!();
    Logger::title("AI Complete Flow", "cyan");

    let mut flow = match storage::load_flow(flow_id) {
        Ok(f) => f,
        Err(e) => { Logger::error(&e); return; }
    };

    let all_nodes = storage::list_nodes();
    let flow_node_ids = flow.all_node_ids();

    // Find nodes not yet in the flow
    let orphan_nodes: Vec<_> = all_nodes.iter()
        .filter(|n| !flow_node_ids.contains(&n.id))
        .cloned()
        .collect();

    if orphan_nodes.is_empty() {
        println!();
        println!("  {}  Flow is already complete — all nodes are connected.", "✔".bright_green());
        println!();
        return;
    }

    println!();
    println!(
        "  {}  Found {} unconnected node(s). Analyzing...",
        "→".bright_cyan(),
        orphan_nodes.len(),
    );

    let mut new_edges: Vec<Edge> = Vec::new();

    // Build a lookup map once to avoid O(n²) per-orphan searches
    let node_map: HashMap<&str, &Node> = all_nodes.iter().map(|n| (n.id.as_str(), n)).collect();

    // For each orphan, find the best node in the flow to connect from
    for orphan in &orphan_nodes {
        let flow_nodes: Vec<_> = flow_node_ids.iter()
            .filter_map(|id| node_map.get(id.as_str()).copied())
            .collect();

        let mut best_score: f32 = 0.0;
        let mut best_edge: Option<Edge> = None;

        for flow_node in &flow_nodes {
            let suggestions = ai::suggest_next_nodes(flow_node, &[orphan.clone()]);
            if let Some(s) = suggestions.into_iter().next() {
                if s.confidence > best_score {
                    best_score = s.confidence;
                    best_edge = Some(s.edge);
                }
            }
        }

        if let Some(edge) = best_edge {
            if best_score > 0.1 {
                println!(
                    "  {}  {} → {}  ({:.0}% confidence)",
                    "✔".bright_green(),
                    edge.from.bold(),
                    orphan.id.bold().bright_cyan(),
                    best_score * 100.0,
                );
                new_edges.push(edge);
            } else {
                println!(
                    "  {}  Cannot find a good place for '{}' (low confidence)",
                    "⚠".bright_yellow(),
                    orphan.id,
                );
            }
        }
    }

    if new_edges.is_empty() {
        println!();
        println!("  No edges could be inferred. Add nodes manually with: infynon api attach");
        println!();
        return;
    }

    flow.edges.extend(new_edges);

    match storage::save_flow(&flow) {
        Ok(_) => {
            println!();
            println!("  {}  Flow updated with {} new edge(s)", "✔".bright_green(), orphan_nodes.len());
            println!();
        }
        Err(e) => Logger::error(&e),
    }
}

// ── ai probe ─────────────────────────────────────────────────────────────────

pub fn cmd_ai_probe(flow_id: &str, base_url_override: Option<&str>) {
    println!();
    Logger::title("AI Security Probes", "cyan");

    let flow = match storage::load_flow(flow_id) {
        Ok(f) => f,
        Err(e) => { Logger::error(&e); return; }
    };

    let nodes = storage::load_nodes_map();

    let base_url = base_url_override
        .map(|s| s.to_string())
        .or_else(|| flow.base_url.clone())
        .unwrap_or_else(|| {
            use std::io::{self, Write};
            print!("  Base URL: ");
            io::stdout().flush().ok();
            let mut s = String::new();
            io::stdin().read_line(&mut s).ok();
            s.trim().to_string()
        });

    println!();
    println!("  {}  Running baseline flow first...", "→".bright_cyan());

    // Run the flow first to get a baseline
    let run_result = execute_flow(
        &flow,
        &nodes,
        FlowExecuteOptions {
            base_url: base_url.clone(),
            initial_context: std::collections::HashMap::new(),
            on_step: None,
            on_prompt: None,
        },
    );

    println!("  {}  Baseline: {}/{} steps passed",
        if run_result.passed { "✔".bright_green().to_string() } else { "⚠".bright_yellow().to_string() },
        run_result.passed_count(), run_result.steps.len());
    println!();
    println!("  {}  Running security probes...", "→".bright_cyan());
    println!();

    let probes = ai::run_security_probes(&flow, &nodes, &run_result, &base_url);

    let critical_count = probes.iter().filter(|p| !p.passed && p.severity == crate::api::types::ProbeSeverity::Critical).count();
    let high_count = probes.iter().filter(|p| !p.passed && p.severity == crate::api::types::ProbeSeverity::High).count();

    for probe in &probes {
        let icon = if probe.passed { "✔".bright_green().to_string() } else {
            match probe.severity {
                crate::api::types::ProbeSeverity::Critical => "✘".bright_red().to_string(),
                crate::api::types::ProbeSeverity::High => "✘".bright_red().to_string(),
                crate::api::types::ProbeSeverity::Medium => "⚠".bright_yellow().to_string(),
                crate::api::types::ProbeSeverity::Low => "ℹ".bright_cyan().to_string(),
            }
        };

        let severity_label = if !probe.passed {
            format!("[{}]", probe.severity.label())
        } else {
            String::new()
        };

        println!(
            "  {}  {}  {} {}",
            icon,
            probe.probe_type.label().bold(),
            probe.description.truecolor(180, 180, 200),
            severity_label.bright_red(),
        );

        if !probe.passed {
            if let Some(details) = &probe.details {
                println!("     {}  {}", "→".truecolor(100, 100, 140), details.truecolor(220, 160, 100));
            }
            if let Some(repro) = &probe.reproduction {
                println!("     {}  {}", "curl:".truecolor(100, 100, 140), repro.truecolor(140, 140, 160));
            }
        }
        println!();
    }

    println!("  ─────────────────────────────────────────");
    if critical_count > 0 || high_count > 0 {
        println!(
            "  {}  {} critical, {} high findings",
            "⚠".bright_red(),
            critical_count.to_string().bright_red(),
            high_count.to_string().bright_yellow(),
        );
    } else {
        println!("  {}  No critical or high findings", "✔".bright_green());
    }
    println!();

    // Save probe results to run
    if let Err(e) = storage::save_run_result(&run_result) {
        Logger::error(&format!("Could not save run: {}", e));
    }
}

// ── ai build-flow ─────────────────────────────────────────────────────────────

pub fn cmd_ai_build_flow(node_ids: &[String], name: &str) {
    println!();
    Logger::title("AI Build Flow", "cyan");

    if node_ids.is_empty() {
        Logger::error("No node IDs provided. Usage: infynon api ai build-flow --nodes login,create-cart,checkout");
        return;
    }

    // Load specified nodes
    let mut nodes: Vec<crate::api::types::Node> = Vec::new();
    for id in node_ids {
        match storage::load_node(id) {
            Ok(n) => nodes.push(n),
            Err(_) => {
                println!("  {}  Node '{}' not found — skipping", "⚠".bright_yellow(), id);
            }
        }
    }

    if nodes.is_empty() {
        Logger::error("No valid nodes found.");
        return;
    }

    println!();
    println!("  {}  Analyzing {} nodes...", "→".bright_cyan(), nodes.len());

    let (entry, edges) = ai::build_flow_edges(&nodes);

    println!();
    println!("  {}  Proposed flow:", "✔".bright_green());
    println!("     Entry: {}", entry.bright_cyan());
    for edge in &edges {
        println!(
            "     {}  {} → {}  [{}]",
            "·".truecolor(100, 100, 140),
            edge.from.bold(),
            edge.to.bright_cyan(),
            if edge.carry.is_empty() { "all context".to_string() } else { edge.carry.join(", ") }
        );
    }

    println!();
    let id = name.to_lowercase().replace(' ', "-");

    use std::io::{self, Write};
    print!("  Save as flow '{}'? [Y/n]: ", id.bold());
    io::stdout().flush().ok();
    let mut answer = String::new();
    io::stdin().read_line(&mut answer).ok();

    if answer.trim().to_lowercase() == "n" {
        println!("  Cancelled.");
        return;
    }

    let mut flow = crate::api::types::Flow::new(&id, name, &entry);
    flow.edges = edges;

    match storage::save_flow(&flow) {
        Ok(path) => {
            println!("  {}  Flow saved: {}", "✔".bright_green(), path.display().to_string().truecolor(100, 100, 140));
        }
        Err(e) => Logger::error(&e),
    }
    println!();
}

// ── ai explain ────────────────────────────────────────────────────────────────

pub fn cmd_ai_explain(flow_id: &str, run_index: usize) {
    println!();
    Logger::title("AI Explain", "cyan");

    let runs = storage::load_recent_runs(flow_id, run_index + 1);

    if runs.is_empty() {
        println!();
        println!("  No run history found for flow '{}'.", flow_id);
        println!("  Run the flow first: infynon api flow run {}", flow_id);
        println!();
        return;
    }

    let run = &runs[run_index.min(runs.len() - 1)];
    let explanation = ai::explain_failure(run);

    println!();
    println!("{}", explanation);
    println!();
}

// ── ai assert ─────────────────────────────────────────────────────────────────

pub fn cmd_ai_assert(node_id: &str) {
    println!();
    Logger::title("AI Generate Assertions", "cyan");

    let mut node = match storage::load_node(node_id) {
        Ok(n) => n,
        Err(e) => { Logger::error(&e); return; }
    };

    let new_assertions = ai::generate_assertions(&node);

    println!();
    println!("  {}  Generated {} assertion(s) for '{}':", "→".bright_cyan(), new_assertions.len(), node_id.bold());
    for a in &new_assertions {
        println!("     {}  {}", "·".truecolor(100, 100, 140), a.check.bright_cyan());
    }

    println!();

    use std::io::{self, Write};
    print!("  Replace existing assertions? [Y/n]: ");
    io::stdout().flush().ok();
    let mut answer = String::new();
    io::stdin().read_line(&mut answer).ok();

    if answer.trim().to_lowercase() != "n" {
        node.assertions = new_assertions;
        match storage::save_node(&node) {
            Ok(_) => println!("  {}  Node updated.", "✔".bright_green()),
            Err(e) => Logger::error(&e),
        }
    }
    println!();
}

// ── ai branch ────────────────────────────────────────────────────────────────

pub fn cmd_ai_branch(node_id: &str) {
    println!();
    Logger::title("AI Branch", "cyan");

    let node = match storage::load_node(node_id) {
        Ok(n) => n,
        Err(e) => { Logger::error(&e); return; }
    };

    // Generate conditional branches based on common status codes
    let branches = vec![
        (format!("status == {}", if node.method == "POST" { 201 } else { 200 }), "success path"),
        ("status == 400".to_string(), "validation error path"),
        ("status == 401".to_string(), "unauthorized path"),
        ("status == 404".to_string(), "not found path"),
        ("status == 500".to_string(), "server error path"),
    ];

    println!();
    println!("  {}  Suggested conditional branches for '{}' {} {}:", "→".bright_cyan(), node_id.bold(), node.method.bright_yellow(), node.path.bright_cyan());
    println!();

    for (i, (condition, label)) in branches.iter().enumerate() {
        println!(
            "  {}  {} → ?  [if: {}]",
            format!("{}.", i + 1).truecolor(0, 210, 255),
            node_id.bold(),
            condition.bright_yellow(),
        );
        println!("     ({} — connect to appropriate handler node)", label.truecolor(140, 140, 160));
        println!("     infynon api attach {} <handler-node> --if \"{}\"", node_id, condition);
        println!();
    }

    println!("  Use: infynon api attach <from> <to> --if \"<condition>\"");
    println!();
}
