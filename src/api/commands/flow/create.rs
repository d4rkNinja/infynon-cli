pub fn cmd_flow_create(name: &str, ai_description: Option<&str>) {
    println!();
    Logger::title("INFYNON API", "cyan");

    let id = name_to_id(name);

    if storage::flow_exists(&id) {
        Logger::error(&format!(
            "Flow '{}' already exists. Choose a different name.",
            id
        ));
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
            println!(
                "     Path:  {}",
                path.display().to_string().truecolor(100, 100, 140)
            );
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
        println!(
            "  {}  No nodes found. Create nodes first with: infynon weave node create",
            "⚠".bright_yellow()
        );
        println!();
    } else {
        println!("  Available nodes:");
        for n in &nodes {
            println!(
                "    {}  {} {}",
                "·".truecolor(100, 100, 140),
                n.id.bright_cyan(),
                n.path.truecolor(140, 140, 160)
            );
        }
        println!();
    }

    let entry = prompt("  Entry node ID: ");
    let base_url = prompt("  Base URL (e.g. http://localhost:3000): ");
    let description = prompt("  Description (optional): ");

    let mut flow = Flow::new(id, name, &entry);
    flow.base_url = if base_url.is_empty() {
        None
    } else {
        Some(base_url)
    };
    flow.description = if description.is_empty() {
        None
    } else {
        Some(description)
    };

    flow
}

fn create_flow_from_ai(id: &str, name: &str, description: &str) -> Flow {
    Logger::step(&format!("Building flow from: \"{}\"", description));

    let nodes = storage::list_nodes();

    if nodes.is_empty() {
        println!(
            "  {}  No nodes found — creating an empty flow.",
            "⚠".bright_yellow()
        );
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
                format!(
                    "  [carries: {}]",
                    edge.carry.join(", ").truecolor(160, 160, 180)
                )
            }
        );
    }

    let mut flow = Flow::new(id, name, &entry);
    flow.edges = edges;
    flow.description = Some(description.to_string());

    flow
}

// ── flow list ─────────────────────────────────────────────────────────────────
