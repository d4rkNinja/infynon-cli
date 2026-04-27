fn execute_graph(action: GraphAction) -> i32 {
    if let Err(e) = storage::ensure_kg_layout() {
        Logger::error(&e);
        return EXIT_TRACE_STORAGE_ERROR;
    }
    match action {
        GraphAction::Entity { action } => execute_graph_entity(action),
        GraphAction::Edge { action } => execute_graph_edge(action),
        GraphAction::Show { branch, kind } => cmd_graph_show(branch, kind.as_deref()),
        GraphAction::Build {
            branch,
            all_branches,
        } => cmd_graph_build(branch, all_branches),
        GraphAction::Diff { branch_a, branch_b } => cmd_graph_diff(&branch_a, &branch_b),
        GraphAction::Path { from, to, branch } => cmd_graph_path(&from, &to, branch),
        GraphAction::Impact { entity, branch } => cmd_graph_impact(&entity, branch),
        GraphAction::Orphans { branch } => cmd_graph_orphans(branch),
        GraphAction::Export {
            format,
            branch,
            output,
        } => cmd_graph_export(&format, branch, output.as_deref()),
        GraphAction::Import {
            file,
            format,
            branch,
        } => cmd_graph_import(&file, format.as_deref(), branch),
        GraphAction::Tui { branch } => {
            crate::trace::tui::run_kg(branch);
            0
        }
    }
}

fn resolve_branch(branch: Option<String>) -> String {
    branch.unwrap_or_else(storage::detect_current_branch)
}

fn execute_graph_entity(action: GraphEntityAction) -> i32 {
    match action {
        GraphEntityAction::Add {
            name,
            kind,
            branch,
            meta,
        } => {
            let kind = match EntityKind::from_str(&kind) {
                Ok(v) => v,
                Err(e) => {
                    Logger::error(&e);
                    return EXIT_TRACE_INVALID_INPUT;
                }
            };
            let mut metadata = HashMap::new();
            for pair in &meta {
                if let Some((k, v)) = pair.split_once('=') {
                    metadata.insert(k.to_string(), v.to_string());
                }
            }
            let branch = resolve_branch(branch);
            let now = Utc::now().to_rfc3339();
            let entity = KgEntity {
                id: storage::sanitize(&name),
                kind,
                name: name.clone(),
                metadata,
                branch: branch.clone(),
                created_at: now.clone(),
                updated_at: now,
            };
            match storage::create_entity(entity) {
                Ok(()) => {
                    Logger::success(&format!("Added {} entity '{}'", kind.as_str(), name));
                    Logger::detail("Branch:", &branch);
                    0
                }
                Err(e) => {
                    Logger::error(&e);
                    EXIT_TRACE_STORAGE_ERROR
                }
            }
        }
        GraphEntityAction::Remove { id } => match storage::delete_entity(&id) {
            Ok(()) => {
                Logger::success(&format!("Removed entity '{}'", id));
                0
            }
            Err(e) => {
                Logger::error(&e);
                EXIT_TRACE_STORAGE_ERROR
            }
        },
        GraphEntityAction::List { branch, kind } => {
            let branch = branch.or_else(|| Some(storage::detect_current_branch()));
            let kind_filter = match kind.as_deref() {
                Some(kind) => match EntityKind::from_str(kind) {
                    Ok(parsed) => Some(parsed),
                    Err(e) => {
                        Logger::error(&e);
                        return EXIT_TRACE_INVALID_INPUT;
                    }
                },
                None => None,
            };
            match storage::list_entities(branch.as_deref(), kind_filter) {
                Ok(entities) => {
                    if entities.is_empty() {
                        Logger::info("No entities found.");
                        return 0;
                    }
                    println!("  {:<24} {:<14} {:<16} NAME", "ID", "KIND", "BRANCH");
                    println!("  {}", "-".repeat(80));
                    for e in &entities {
                        println!(
                            "  {:<24} {:<14} {:<16} {}",
                            e.id,
                            e.kind.as_str(),
                            e.branch,
                            e.name
                        );
                    }
                    0
                }
                Err(e) => {
                    Logger::error(&e);
                    EXIT_TRACE_STORAGE_ERROR
                }
            }
        }
    }
}

fn resolve_entity_id(name: &str, branch: &str) -> String {
    match storage::find_entity_by_name(name, branch) {
        Ok(Some(entity)) => entity.id,
        _ => storage::sanitize(name),
    }
}

fn execute_graph_edge(action: GraphEdgeAction) -> i32 {
    match action {
        GraphEdgeAction::Add {
            from,
            to,
            relation,
            weight,
            branch,
            evidence,
        } => {
            let relation = match RelationType::from_str(&relation) {
                Ok(v) => v,
                Err(e) => {
                    Logger::error(&e);
                    return EXIT_TRACE_INVALID_INPUT;
                }
            };
            if !(0.0..=1.0).contains(&weight) {
                Logger::error("Invalid weight. Use a value between 0.0 and 1.0.");
                return EXIT_TRACE_INVALID_INPUT;
            }
            let branch = resolve_branch(branch);
            let source = resolve_entity_id(&from, &branch);
            let target = resolve_entity_id(&to, &branch);
            let id = format!("{}-{}-{}", source, relation.as_str(), target);
            let edge = KgEdge {
                id: id.clone(),
                source: source.clone(),
                target: target.clone(),
                relation,
                weight,
                branch: branch.clone(),
                evidence: evidence.unwrap_or_default(),
                created_at: Utc::now().to_rfc3339(),
            };
            match storage::create_edge(edge) {
                Ok(()) => {
                    Logger::success(&format!(
                        "Added edge {} -> {} ({})",
                        from,
                        to,
                        relation.as_str()
                    ));
                    Logger::detail("Branch:", &branch);
                    0
                }
                Err(e) => {
                    Logger::error(&e);
                    EXIT_TRACE_STORAGE_ERROR
                }
            }
        }
        GraphEdgeAction::Remove { id } => match storage::delete_edge(&id) {
            Ok(()) => {
                Logger::success(&format!("Removed edge '{}'", id));
                0
            }
            Err(e) => {
                Logger::error(&e);
                EXIT_TRACE_STORAGE_ERROR
            }
        },
        GraphEdgeAction::List { branch, relation } => {
            let branch = branch.or_else(|| Some(storage::detect_current_branch()));
            let rel_filter = match relation.as_deref() {
                Some(relation) => match RelationType::from_str(relation) {
                    Ok(parsed) => Some(parsed),
                    Err(e) => {
                        Logger::error(&e);
                        return EXIT_TRACE_INVALID_INPUT;
                    }
                },
                None => None,
            };
            match storage::list_edges(branch.as_deref(), rel_filter) {
                Ok(edges) => {
                    if edges.is_empty() {
                        Logger::info("No edges found.");
                        return 0;
                    }
                    println!(
                        "  {:<36} {:<18} {:<18} {:<16} WEIGHT",
                        "ID", "SOURCE", "TARGET", "RELATION"
                    );
                    println!("  {}", "-".repeat(100));
                    for e in &edges {
                        println!(
                            "  {:<36} {:<18} {:<18} {:<16} {:.2}",
                            e.id,
                            e.source,
                            e.target,
                            e.relation.as_str(),
                            e.weight
                        );
                    }
                    0
                }
                Err(e) => {
                    Logger::error(&e);
                    EXIT_TRACE_STORAGE_ERROR
                }
            }
        }
    }
}

fn cmd_graph_show(branch: Option<String>, kind: Option<&str>) -> i32 {
    let branch = resolve_branch(branch);
    match storage::load_graph(Some(&branch)) {
        Ok(graph) => {
            let kind_filter = match kind {
                Some(kind) => match EntityKind::from_str(kind) {
                    Ok(parsed) => Some(parsed),
                    Err(e) => {
                        Logger::error(&e);
                        return EXIT_TRACE_INVALID_INPUT;
                    }
                },
                None => None,
            };
            let entities: Vec<_> = if let Some(kf) = kind_filter {
                graph.entities.iter().filter(|e| e.kind == kf).collect()
            } else {
                graph.entities.iter().collect()
            };
            Logger::info(&format!(
                "Graph '{}': {} entities, {} edges",
                branch,
                entities.len(),
                graph.edges.len()
            ));
            if !entities.is_empty() {
                println!("\n  {:<24} {:<14} NAME", "ID", "KIND");
                println!("  {}", "-".repeat(60));
                for e in &entities {
                    println!("  {:<24} {:<14} {}", e.id, e.kind.as_str(), e.name);
                }
            }
            if !graph.edges.is_empty() {
                println!("\n  {:<18} {:<16} {:<18}", "SOURCE", "RELATION", "TARGET");
                println!("  {}", "-".repeat(60));
                for e in &graph.edges {
                    println!(
                        "  {:<18} {:<16} {:<18}",
                        e.source,
                        e.relation.as_str(),
                        e.target
                    );
                }
            }
            0
        }
        Err(e) => {
            Logger::error(&e);
            EXIT_TRACE_STORAGE_ERROR
        }
    }
}

fn cmd_graph_build(branch: Option<String>, _all_branches: bool) -> i32 {
    let branch = resolve_branch(branch);
    match storage::auto_build_graph(&branch) {
        Ok((ent_count, edge_count)) => {
            Logger::success(&format!(
                "Built graph for '{}': {} entities, {} edges",
                branch, ent_count, edge_count
            ));
            0
        }
        Err(e) => {
            Logger::error(&e);
            EXIT_TRACE_STORAGE_ERROR
        }
    }
}

fn cmd_graph_diff(branch_a: &str, branch_b: &str) -> i32 {
    let graph_a = match storage::load_graph(Some(branch_a)) {
        Ok(g) => g,
        Err(e) => {
            Logger::error(&e);
            return EXIT_TRACE_STORAGE_ERROR;
        }
    };
    let graph_b = match storage::load_graph(Some(branch_b)) {
        Ok(g) => g,
        Err(e) => {
            Logger::error(&e);
            return EXIT_TRACE_STORAGE_ERROR;
        }
    };

    let ids_a: std::collections::HashSet<_> = graph_a.entities.iter().map(|e| &e.id).collect();
    let ids_b: std::collections::HashSet<_> = graph_b.entities.iter().map(|e| &e.id).collect();
    let only_a: Vec<_> = ids_a.difference(&ids_b).collect();
    let only_b: Vec<_> = ids_b.difference(&ids_a).collect();
    let shared = ids_a.intersection(&ids_b).count();

    let edge_ids_a: std::collections::HashSet<_> = graph_a.edges.iter().map(|e| &e.id).collect();
    let edge_ids_b: std::collections::HashSet<_> = graph_b.edges.iter().map(|e| &e.id).collect();
    let edges_only_a: Vec<_> = edge_ids_a.difference(&edge_ids_b).collect();
    let edges_only_b: Vec<_> = edge_ids_b.difference(&edge_ids_a).collect();

    Logger::info(&format!("Diff: {} vs {}", branch_a, branch_b));
    println!("\n  Entities:");
    println!("    Shared:          {}", shared);
    println!("    Only in {}:  {}", branch_a, only_a.len());
    for id in &only_a {
        println!("      + {}", id);
    }
    println!("    Only in {}:  {}", branch_b, only_b.len());
    for id in &only_b {
        println!("      + {}", id);
    }
    println!("\n  Edges:");
    println!("    Only in {}:  {}", branch_a, edges_only_a.len());
    for id in &edges_only_a {
        println!("      + {}", id);
    }
    println!("    Only in {}:  {}", branch_b, edges_only_b.len());
    for id in &edges_only_b {
        println!("      + {}", id);
    }
    0
}

fn cmd_graph_path(from: &str, to: &str, branch: Option<String>) -> i32 {
    let branch = resolve_branch(branch);
    let graph = match storage::load_graph(Some(&branch)) {
        Ok(g) => g,
        Err(e) => {
            Logger::error(&e);
            return EXIT_TRACE_STORAGE_ERROR;
        }
    };

    let source_id = resolve_entity_id(from, &branch);
    let target_id = resolve_entity_id(to, &branch);

    // Build adjacency list for BFS
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    for edge in &graph.edges {
        adj.entry(edge.source.clone())
            .or_default()
            .push(edge.target.clone());
        adj.entry(edge.target.clone())
            .or_default()
            .push(edge.source.clone());
    }

    // BFS
    let mut visited: HashMap<String, String> = HashMap::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(source_id.clone());
    visited.insert(source_id.clone(), String::new());

    while let Some(current) = queue.pop_front() {
        if current == target_id {
            // Reconstruct path
            let mut path = vec![current.clone()];
            let mut node = current;
            while let Some(prev) = visited.get(&node) {
                if prev.is_empty() {
                    break;
                }
                path.push(prev.clone());
                node = prev.clone();
            }
            path.reverse();
            Logger::success(&format!("Path found ({} hops):", path.len() - 1));
            println!("  {}", path.join(" -> "));
            return 0;
        }
        if let Some(neighbors) = adj.get(&current) {
            for neighbor in neighbors {
                if !visited.contains_key(neighbor) {
                    visited.insert(neighbor.clone(), current.clone());
                    queue.push_back(neighbor.clone());
                }
            }
        }
    }

    Logger::info(&format!("No path found between '{}' and '{}'", from, to));
    0
}

fn cmd_graph_impact(entity: &str, branch: Option<String>) -> i32 {
    let branch = resolve_branch(branch);
    let graph = match storage::load_graph(Some(&branch)) {
        Ok(g) => g,
        Err(e) => {
            Logger::error(&e);
            return EXIT_TRACE_STORAGE_ERROR;
        }
    };

    let start_id = resolve_entity_id(entity, &branch);

    // Build adjacency list for BFS
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    for edge in &graph.edges {
        adj.entry(edge.source.clone())
            .or_default()
            .push(edge.target.clone());
        adj.entry(edge.target.clone())
            .or_default()
            .push(edge.source.clone());
    }

    // BFS outward
    let mut visited: HashMap<String, usize> = HashMap::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back((start_id.clone(), 0usize));
    visited.insert(start_id.clone(), 0);

    while let Some((current, depth)) = queue.pop_front() {
        if let Some(neighbors) = adj.get(&current) {
            for neighbor in neighbors {
                if !visited.contains_key(neighbor) {
                    visited.insert(neighbor.clone(), depth + 1);
                    queue.push_back((neighbor.clone(), depth + 1));
                }
            }
        }
    }

    visited.remove(&start_id);
    if visited.is_empty() {
        Logger::info(&format!("No connected entities for '{}'", entity));
        return 0;
    }

    Logger::success(&format!(
        "Impact from '{}': {} reachable entities",
        entity,
        visited.len()
    ));
    println!("  {:<24} DEPTH", "ENTITY");
    println!("  {}", "-".repeat(40));
    let mut sorted: Vec<_> = visited.into_iter().collect();
    sorted.sort_by_key(|(_, d)| *d);
    for (id, depth) in &sorted {
        println!("  {:<24} {}", id, depth);
    }
    0
}

fn cmd_graph_orphans(branch: Option<String>) -> i32 {
    let branch = resolve_branch(branch);
    let graph = match storage::load_graph(Some(&branch)) {
        Ok(g) => g,
        Err(e) => {
            Logger::error(&e);
            return EXIT_TRACE_STORAGE_ERROR;
        }
    };

    let mut connected: std::collections::HashSet<String> = std::collections::HashSet::new();
    for edge in &graph.edges {
        connected.insert(edge.source.clone());
        connected.insert(edge.target.clone());
    }

    let orphans: Vec<_> = graph
        .entities
        .iter()
        .filter(|e| !connected.contains(&e.id))
        .collect();
    if orphans.is_empty() {
        Logger::info("No orphan entities found.");
        return 0;
    }
    Logger::info(&format!("{} orphan entities:", orphans.len()));
    println!("  {:<24} {:<14} NAME", "ID", "KIND");
    println!("  {}", "-".repeat(60));
    for e in &orphans {
        println!("  {:<24} {:<14} {}", e.id, e.kind.as_str(), e.name);
    }
    0
}

fn cmd_graph_export(format: &str, branch: Option<String>, output: Option<&str>) -> i32 {
    let branch = resolve_branch(branch);
    let graph = match storage::load_graph(Some(&branch)) {
        Ok(g) => g,
        Err(e) => {
            Logger::error(&e);
            return EXIT_TRACE_STORAGE_ERROR;
        }
    };

    let content = match format.to_ascii_lowercase().as_str() {
        "json" => match storage::export_graph_json(&graph) {
            Ok(v) => v,
            Err(e) => {
                Logger::error(&e);
                return EXIT_TRACE_STORAGE_ERROR;
            }
        },
        "dot" => match storage::export_graph_dot(&graph) {
            Ok(v) => v,
            Err(e) => {
                Logger::error(&e);
                return EXIT_TRACE_STORAGE_ERROR;
            }
        },
        other => {
            Logger::error(&format!("Unsupported format '{}'. Use json | dot.", other));
            return EXIT_TRACE_INVALID_INPUT;
        }
    };

    match output {
        Some(path) => match std::fs::write(path, &content) {
            Ok(()) => {
                Logger::success(&format!("Exported graph to '{}'", path));
                0
            }
            Err(e) => {
                Logger::error(&format!("Failed to write '{}': {}", path, e));
                EXIT_TRACE_STORAGE_ERROR
            }
        },
        None => {
            println!("{}", content);
            0
        }
    }
}

fn cmd_graph_import(file: &str, _format: Option<&str>, branch: Option<String>) -> i32 {
    let branch = resolve_branch(branch);
    let content = match std::fs::read_to_string(file) {
        Ok(v) => v,
        Err(e) => {
            Logger::error(&format!("Failed to read '{}': {}", file, e));
            return EXIT_TRACE_STORAGE_ERROR;
        }
    };
    match storage::import_graph_json(&content, Some(&branch)) {
        Ok((ent_count, edge_count)) => {
            Logger::success(&format!(
                "Imported {} entities and {} edges into '{}'",
                ent_count, edge_count, branch
            ));
            0
        }
        Err(e) => {
            Logger::error(&e);
            EXIT_TRACE_STORAGE_ERROR
        }
    }
}
