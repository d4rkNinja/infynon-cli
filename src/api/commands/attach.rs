use owo_colors::OwoColorize;

use crate::api::ai;
use crate::api::storage;
use crate::api::types::Edge;
use crate::tui::logger::Logger;

// ── attach ────────────────────────────────────────────────────────────────────

pub fn cmd_attach(
    from_id: &str,
    to_id: &str,
    carry: &[String],
    condition: Option<&str>,
    use_ai: bool,
) {
    println!();

    // Validate nodes exist
    let from_node = match storage::load_node(from_id) {
        Ok(n) => n,
        Err(_) => {
            Logger::error(&format!("Node '{}' not found. Create it first.", from_id));
            return;
        }
    };
    let to_node = match storage::load_node(to_id) {
        Ok(n) => n,
        Err(_) => {
            Logger::error(&format!("Node '{}' not found. Create it first.", to_id));
            return;
        }
    };

    // Infer carry list
    let resolved_carry = if use_ai || carry.is_empty() {
        let inferred = ai::infer_carry(&from_node, &to_node);
        if !inferred.is_empty() {
            println!(
                "  {}  AI inferred carry: {}",
                "→".bright_cyan(),
                inferred.join(", ").bright_yellow()
            );
        }
        inferred
    } else {
        carry.to_vec()
    };

    let edge = Edge {
        from: from_id.to_string(),
        to: to_id.to_string(),
        carry: resolved_carry.clone(),
        condition: condition.map(|s| s.to_string()),
    };

    // Attach to every flow that contains from_id or create a standalone edge record
    let flows = storage::list_flows();
    let mut attached_to: Vec<String> = Vec::new();

    for mut flow in flows {
        if flow.all_node_ids().contains(&from_id.to_string()) {
            // Check not already attached
            let already = flow.edges.iter().any(|e| e.from == from_id && e.to == to_id);
            if !already {
                flow.edges.push(edge.clone());
                if let Ok(_) = storage::save_flow(&flow) {
                    attached_to.push(flow.name.clone());
                }
            }
        }
    }

    Logger::title("Attach", "cyan");
    println!();
    println!(
        "  {}  {} → {}",
        "✔".bright_green(),
        from_id.bold(),
        to_id.bold().bright_cyan(),
    );

    if !resolved_carry.is_empty() {
        println!(
            "     Carries: {}",
            resolved_carry.join(", ").bright_yellow()
        );
    }

    if let Some(cond) = condition {
        println!("     Condition: {}", cond.truecolor(200, 160, 80));
    }

    if attached_to.is_empty() {
        println!();
        println!(
            "  {}  Node '{}' is not yet in any flow. Add it to a flow with:",
            "ℹ".bright_cyan(),
            from_id
        );
        println!("     infynon weave flow create <name>  (then attach nodes)");
    } else {
        println!(
            "     Updated flow(s): {}",
            attached_to.join(", ").truecolor(160, 160, 200)
        );
    }
    println!();
}

// ── attach with AI-generated next node ───────────────────────────────────────

pub fn cmd_attach_ai(from_id: &str, description: Option<&str>, flow_id: Option<&str>) {
    println!();
    Logger::title("AI Attach", "cyan");

    let from_node = match storage::load_node(from_id) {
        Ok(n) => n,
        Err(_) => {
            Logger::error(&format!("Node '{}' not found.", from_id));
            return;
        }
    };

    let all_nodes = storage::list_nodes();

    // Get suggestions
    let suggestions = ai::suggest_next_nodes(
        &from_node,
        &all_nodes.iter().filter(|n| n.id != from_id).cloned().collect::<Vec<_>>(),
    );

    if suggestions.is_empty() {
        println!();
        if let Some(desc) = description {
            // No existing nodes match — offer to create one
            println!("  {}  No matching node found. Creating from description...", "→".bright_cyan());
            super::node::cmd_node_create(Some(desc));

            // Reload and try to attach
            let new_nodes = storage::list_nodes();
            let new_suggestions = ai::suggest_next_nodes(&from_node, &new_nodes);
            if let Some(best) = new_suggestions.into_iter().next() {
                attach_edge_to_flows(best.edge, flow_id);
            }
        } else {
            println!("  No suitable next nodes found.");
            println!("  Try: infynon weave node create --ai \"describe what comes next\"");
        }
        println!();
        return;
    }

    let best = &suggestions[0];

    println!();
    println!("  {}  Best match: {}", "→".bright_cyan(), best.node.id.bold());
    println!("     {} {} {}", best.node.method.bright_yellow(), best.node.path.bright_cyan(), format!("(confidence: {:.0}%)", best.confidence * 100.0).truecolor(140, 140, 160));
    println!("     Reason: {}", best.reason.truecolor(180, 180, 200));
    println!("     Carries: {}", if best.edge.carry.is_empty() { "all context".to_string() } else { best.edge.carry.join(", ") }.bright_yellow());

    if suggestions.len() > 1 {
        println!();
        println!("  {}  Other candidates:", "ℹ".bright_cyan());
        for s in suggestions.iter().skip(1).take(3) {
            println!(
                "     {}  {} {} ({:.0}%)",
                "·".truecolor(100, 100, 140),
                s.node.id.truecolor(160, 160, 200),
                s.node.path.truecolor(120, 120, 160),
                s.confidence * 100.0,
            );
        }
    }

    attach_edge_to_flows(best.edge.clone(), flow_id);
    println!();
    println!("  {}  Attached: {} → {}", "✔".bright_green(), from_id.bold(), best.node.id.bold().bright_cyan());
    println!();
}

fn attach_edge_to_flows(edge: Edge, flow_id_filter: Option<&str>) {
    let flows = storage::list_flows();
    for mut flow in flows {
        if let Some(fid) = flow_id_filter {
            if flow.id != fid { continue; }
        }
        if flow.all_node_ids().contains(&edge.from) {
            let already = flow.edges.iter().any(|e| e.from == edge.from && e.to == edge.to);
            if !already {
                flow.edges.push(edge.clone());
                storage::save_flow(&flow).ok();
            }
        }
    }
}

// ── detach ────────────────────────────────────────────────────────────────────

pub fn cmd_detach(from_id: &str, to_id: &str) {
    println!();
    Logger::title("Detach", "cyan");

    let flows = storage::list_flows();
    let mut updated = 0;

    for mut flow in flows {
        let before = flow.edges.len();
        flow.edges.retain(|e| !(e.from == from_id && e.to == to_id));
        if flow.edges.len() != before {
            storage::save_flow(&flow).ok();
            updated += 1;
        }
    }

    if updated > 0 {
        println!();
        println!(
            "  {}  Detached: {} → {}  (updated {} flow(s))",
            "✔".bright_green(),
            from_id.bold(),
            to_id.bold(),
            updated.to_string().bright_cyan(),
        );
    } else {
        println!();
        println!("  {}  No edge found from '{}' to '{}'", "ℹ".bright_yellow(), from_id, to_id);
    }
    println!();
}
