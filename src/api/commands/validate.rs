use owo_colors::OwoColorize;

use crate::api::storage;

pub fn cmd_validate() {
    let nodes = storage::list_nodes();
    let flows = storage::list_flows();

    println!();
    println!("  {}  Weave Validation", "◆".bright_cyan());
    println!("  {}", "─".repeat(42).truecolor(50, 50, 80));
    println!("  Nodes: {} checked", nodes.len());
    println!("  Flows: {} checked", flows.len());
    println!();

    let valid_methods = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD"];
    let node_ids: std::collections::HashSet<String> = nodes.iter().map(|n| n.id.clone()).collect();

    let mut node_errors = 0usize;
    let mut node_warnings = 0usize;
    let mut node_valid = 0usize;

    // ── Validate nodes ────────────────────────────────────────────────────────
    for node in &nodes {
        let mut errors: Vec<String> = Vec::new();
        let mut warnings: Vec<String> = Vec::new();

        if node.id.is_empty() {
            errors.push("id is empty".to_string());
        }
        if !valid_methods.contains(&node.method.to_uppercase().as_str()) {
            errors.push(format!(
                "method '{}' is not valid (must be GET POST PUT PATCH DELETE HEAD)",
                node.method
            ));
        }
        if !node.path.starts_with('/') {
            errors.push(format!("path '{}' does not start with /", node.path));
        }
        if let Some(ref body) = node.body_json {
            if serde_json::from_str::<serde_json::Value>(body).is_err() {
                errors.push("body_json is not valid JSON".to_string());
            }
        }
        if node.assertions.is_empty() {
            warnings.push("no assertions defined".to_string());
        }
        for ext in &node.extractions {
            if !ext.from.starts_with("body.")
                && ext.from != "body"
                && !ext.from.starts_with("header.")
                && ext.from != "status"
            {
                warnings.push(format!(
                    "extraction '{}': from '{}' should start with body., header., or be status",
                    ext.name, ext.from
                ));
            }
        }

        if !errors.is_empty() {
            node_errors += 1;
            println!(
                "  {}  {:<36}  ERROR: {}",
                "✘".bright_red(),
                node.id.bold(),
                errors.join("; ").bright_red(),
            );
        } else if !warnings.is_empty() {
            node_warnings += 1;
            println!(
                "  {}  {:<36}  WARNING: {}",
                "⚠".bright_yellow(),
                node.id.bold(),
                warnings.join("; ").bright_yellow(),
            );
        } else {
            node_valid += 1;
            println!("  {}  {:<36}  valid", "✔".bright_green(), node.id.bold(),);
        }
    }

    if nodes.is_empty() {
        println!("  (no nodes)");
    }

    println!();

    // ── Validate flows ────────────────────────────────────────────────────────
    let mut flow_errors = 0usize;
    let mut flow_valid = 0usize;

    for flow in &flows {
        let mut errors: Vec<String> = Vec::new();

        if flow.id.is_empty() {
            errors.push("id is empty".to_string());
        }
        if flow.entry.is_empty() {
            errors.push("entry is empty".to_string());
        } else if !node_ids.contains(&flow.entry) {
            errors.push(format!("entry node '{}' not found in library", flow.entry));
        }

        for edge in &flow.edges {
            if !node_ids.contains(&edge.from) {
                errors.push(format!(
                    "edge from '{}': node not found in library",
                    edge.from
                ));
            }
            if !node_ids.contains(&edge.to) {
                errors.push(format!("edge to '{}': node not found in library", edge.to));
            }
        }

        // Check for cycles via DFS
        if errors.is_empty() {
            if has_cycle(flow) {
                errors.push("circular dependency detected".to_string());
            }
        }

        let node_count = flow.all_node_ids().len();
        let edge_count = flow.edges.len();

        if !errors.is_empty() {
            flow_errors += 1;
            println!(
                "  {}  {:<36}  ERROR: {}",
                "✘".bright_red(),
                flow.id.bold(),
                errors.join("; ").bright_red(),
            );
        } else {
            flow_valid += 1;
            println!(
                "  {}  {:<36}  valid  ({} nodes, {} edges)",
                "✔".bright_green(),
                flow.id.bold(),
                node_count,
                edge_count,
            );
        }
    }

    if flows.is_empty() {
        println!("  (no flows)");
    }

    println!();
    println!(
        "  Summary: {} nodes ({} valid, {} warning)  |  {} flows ({} valid, {} error)",
        nodes.len().to_string().bright_cyan(),
        node_valid.to_string().bright_green(),
        node_warnings.to_string().bright_yellow(),
        flows.len().to_string().bright_cyan(),
        flow_valid.to_string().bright_green(),
        flow_errors.to_string().bright_red(),
    );
    println!();

    if flow_errors > 0 || node_errors > 0 {
        std::process::exit(1);
    }
}

fn has_cycle(flow: &crate::api::types::Flow) -> bool {
    // DFS cycle detection
    let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut rec_stack: std::collections::HashSet<String> = std::collections::HashSet::new();

    fn dfs(
        node_id: &str,
        flow: &crate::api::types::Flow,
        visited: &mut std::collections::HashSet<String>,
        rec_stack: &mut std::collections::HashSet<String>,
    ) -> bool {
        visited.insert(node_id.to_string());
        rec_stack.insert(node_id.to_string());

        for edge in flow.successors(node_id) {
            if !visited.contains(&edge.to) {
                if dfs(&edge.to, flow, visited, rec_stack) {
                    return true;
                }
            } else if rec_stack.contains(&edge.to) {
                return true;
            }
        }

        rec_stack.remove(node_id);
        false
    }

    let all_ids: Vec<String> = flow.all_node_ids();
    for node_id in &all_ids {
        if !visited.contains(node_id) {
            if dfs(node_id, flow, &mut visited, &mut rec_stack) {
                return true;
            }
        }
    }
    false
}
