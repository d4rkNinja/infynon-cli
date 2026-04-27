fn kg_entity_path(id: &str) -> PathBuf {
    kg_dir()
        .join("entities")
        .join(format!("{}.json", storage_key(id)))
}

fn kg_edge_path(id: &str) -> PathBuf {
    kg_dir()
        .join("edges")
        .join(format!("{}.json", storage_key(id)))
}

fn legacy_kg_entity_path(id: &str) -> PathBuf {
    kg_dir()
        .join("entities")
        .join(format!("{}.json", sanitize(id)))
}

fn legacy_kg_edge_path(id: &str) -> PathBuf {
    kg_dir()
        .join("edges")
        .join(format!("{}.json", sanitize(id)))
}

pub fn create_entity(entity: KgEntity) -> Result<(), String> {
    ensure_kg_layout()?;
    let content = serde_json::to_string_pretty(&entity).map_err(|e| e.to_string())?;
    let path = kg_entity_path(&entity.id);
    fs::write(&path, content).map_err(|e| e.to_string())?;
    remove_legacy_alias(&path, &legacy_kg_entity_path(&entity.id))
}

pub fn delete_entity(id: &str) -> Result<(), String> {
    let mut deleted = false;
    for path in [kg_entity_path(id), legacy_kg_entity_path(id)] {
        match fs::remove_file(&path) {
            Ok(()) => deleted = true,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.to_string()),
        }
    }
    if !deleted {
        return Err(format!("entity '{}' not found", id));
    }
    // Remove all edges referencing this entity
    let edges = list_edges(None, None)?;
    for edge in edges {
        if edge.source == id || edge.target == id {
            let _ = delete_edge(&edge.id);
        }
    }
    Ok(())
}

pub fn list_entities(
    branch: Option<&str>,
    kind: Option<EntityKind>,
) -> Result<Vec<KgEntity>, String> {
    ensure_kg_layout()?;
    let dir = kg_dir().join("entities");
    let entries = match fs::read_dir(&dir) {
        Ok(e) => e,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e.to_string()),
    };
    let mut entities = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        if let Ok(entity) = serde_json::from_str::<KgEntity>(&content) {
            let branch_ok = branch.map(|b| entity.branch == b).unwrap_or(true);
            let kind_ok = kind.map(|k| entity.kind == k).unwrap_or(true);
            if branch_ok && kind_ok {
                entities.push(entity);
            }
        }
    }
    entities.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(entities)
}

pub fn load_entity(id: &str) -> Result<Option<KgEntity>, String> {
    for path in [kg_entity_path(id), legacy_kg_entity_path(id)] {
        match fs::read_to_string(path) {
            Ok(content) => {
                let entity = serde_json::from_str(&content).map_err(|e| e.to_string())?;
                return Ok(Some(entity));
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(None)
}

pub fn find_entity_by_name(name: &str, branch: &str) -> Result<Option<KgEntity>, String> {
    let entities = list_entities(Some(branch), None)?;
    Ok(entities.into_iter().find(|e| e.name == name))
}

pub fn create_edge(edge: KgEdge) -> Result<(), String> {
    ensure_kg_layout()?;
    let content = serde_json::to_string_pretty(&edge).map_err(|e| e.to_string())?;
    let path = kg_edge_path(&edge.id);
    fs::write(&path, content).map_err(|e| e.to_string())?;
    remove_legacy_alias(&path, &legacy_kg_edge_path(&edge.id))
}

pub fn delete_edge(id: &str) -> Result<(), String> {
    let mut deleted = false;
    for path in [kg_edge_path(id), legacy_kg_edge_path(id)] {
        match fs::remove_file(&path) {
            Ok(()) => deleted = true,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.to_string()),
        }
    }
    if deleted {
        Ok(())
    } else {
        Err(format!("edge '{}' not found", id))
    }
}

pub fn list_edges(
    branch: Option<&str>,
    relation: Option<RelationType>,
) -> Result<Vec<KgEdge>, String> {
    ensure_kg_layout()?;
    let dir = kg_dir().join("edges");
    let entries = match fs::read_dir(&dir) {
        Ok(e) => e,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e.to_string()),
    };
    let mut edges = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        if let Ok(edge) = serde_json::from_str::<KgEdge>(&content) {
            let branch_ok = branch.map(|b| edge.branch == b).unwrap_or(true);
            let relation_ok = relation.map(|r| edge.relation == r).unwrap_or(true);
            if branch_ok && relation_ok {
                edges.push(edge);
            }
        }
    }
    edges.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(edges)
}

pub fn load_edge(id: &str) -> Result<Option<KgEdge>, String> {
    for path in [kg_edge_path(id), legacy_kg_edge_path(id)] {
        match fs::read_to_string(path) {
            Ok(content) => {
                let edge = serde_json::from_str(&content).map_err(|e| e.to_string())?;
                return Ok(Some(edge));
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(None)
}

pub fn load_graph(branch: Option<&str>) -> Result<KgGraph, String> {
    Ok(KgGraph {
        entities: list_entities(branch, None)?,
        edges: list_edges(branch, None)?,
    })
}

pub fn export_graph_json(graph: &KgGraph) -> Result<String, String> {
    serde_json::to_string_pretty(graph).map_err(|e| e.to_string())
}

pub fn export_graph_dot(graph: &KgGraph) -> Result<String, String> {
    let mut dot =
        String::from("digraph trace_kg {\n  rankdir=LR;\n  node [fontname=\"Helvetica\"];\n\n");

    // Group entities by branch into subgraph clusters
    let mut branches: HashMap<String, Vec<&KgEntity>> = HashMap::new();
    for entity in &graph.entities {
        branches
            .entry(entity.branch.clone())
            .or_default()
            .push(entity);
    }

    for (branch, entities) in &branches {
        dot.push_str(&format!("  subgraph cluster_{} {{\n", sanitize(branch)));
        dot.push_str(&format!("    label=\"{}\";\n", branch));
        dot.push_str("    style=dashed;\n");
        for entity in entities {
            let shape = match entity.kind {
                EntityKind::File => "box",
                EntityKind::Person => "ellipse",
                EntityKind::Package => "hexagon",
                EntityKind::Decision => "diamond",
                EntityKind::Endpoint => "component",
                EntityKind::Module => "folder",
                EntityKind::Pr => "note",
                EntityKind::Branch => "tab",
                EntityKind::Note => "rect",
                EntityKind::Vulnerability => "octagon",
            };
            dot.push_str(&format!(
                "    \"{}\" [label=\"{}\" shape={}];\n",
                sanitize(&entity.id),
                entity.name.replace('"', "\\\""),
                shape,
            ));
        }
        dot.push_str("  }\n\n");
    }

    for edge in &graph.edges {
        dot.push_str(&format!(
            "  \"{}\" -> \"{}\" [label=\"{}\"];\n",
            sanitize(&edge.source),
            sanitize(&edge.target),
            edge.relation.as_str(),
        ));
    }

    dot.push_str("}\n");
    Ok(dot)
}

pub fn import_graph_json(
    content: &str,
    target_branch: Option<&str>,
) -> Result<(usize, usize), String> {
    let mut graph: KgGraph = serde_json::from_str(content).map_err(|e| e.to_string())?;
    if let Some(branch) = target_branch {
        for entity in &mut graph.entities {
            entity.branch = branch.to_string();
        }
        for edge in &mut graph.edges {
            edge.branch = branch.to_string();
        }
    }
    let entity_count = graph.entities.len();
    let edge_count = graph.edges.len();
    for entity in graph.entities {
        create_entity(entity)?;
    }
    for edge in graph.edges {
        create_edge(edge)?;
    }
    Ok((entity_count, edge_count))
}

pub fn detect_current_branch() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "main".to_string())
}

pub fn auto_build_graph(branch: &str) -> Result<(usize, usize), String> {
    ensure_kg_layout()?;
    let now = Utc::now().to_rfc3339();
    let mut entities_created = 0usize;
    let mut edges_created = 0usize;

    // Pre-load all existing entities and edges into HashSets to avoid N+1 filesystem scans
    let existing = list_entities(Some(branch), None).unwrap_or_default();
    let mut known_entities: std::collections::HashSet<String> =
        existing.iter().map(|e| e.name.clone()).collect();
    let existing_edges = list_edges(Some(branch), None).unwrap_or_default();
    let mut known_edges: std::collections::HashSet<String> = existing_edges
        .iter()
        .map(|e| edge_dedupe_key(&e.source, &e.target, e.relation))
        .collect();

    // Run git log to extract person->file relationships
    let output = std::process::Command::new("git")
        .args([
            "log",
            "--name-only",
            "--format=%an",
            "--no-merges",
            "-100",
            branch,
            "--",
        ])
        .output()
        .map_err(|e| format!("failed to run git log: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "git log failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut current_person: Option<String> = None;

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // If the line doesn't look like a file path, treat it as a person name
        if !line.contains('/') && !line.contains('.') {
            current_person = Some(line.to_string());
            // Create person entity if not exists
            let person_id = stable_trace_id("person", line);
            if !known_entities.contains(line) {
                create_entity(KgEntity {
                    id: person_id,
                    kind: EntityKind::Person,
                    name: line.to_string(),
                    metadata: HashMap::new(),
                    branch: branch.to_string(),
                    created_at: now.clone(),
                    updated_at: now.clone(),
                })?;
                known_entities.insert(line.to_string());
                entities_created += 1;
            }
            continue;
        }

        // It's a file path
        if let Some(ref person) = current_person {
            let file_id = stable_trace_id("file", line);
            if !known_entities.contains(line) {
                create_entity(KgEntity {
                    id: file_id.clone(),
                    kind: EntityKind::File,
                    name: line.to_string(),
                    metadata: HashMap::new(),
                    branch: branch.to_string(),
                    created_at: now.clone(),
                    updated_at: now.clone(),
                })?;
                known_entities.insert(line.to_string());
                entities_created += 1;
            }

            let person_id = stable_trace_id("person", person);
            let edge_id = stable_trace_id("edge", &format!("{}|{}|modified_by", line, person));
            let edge_key = edge_dedupe_key(&file_id, &person_id, RelationType::ModifiedBy);
            if !known_edges.contains(&edge_key) {
                create_edge(KgEdge {
                    id: edge_id.clone(),
                    source: file_id,
                    target: person_id,
                    relation: RelationType::ModifiedBy,
                    weight: 1.0,
                    branch: branch.to_string(),
                    evidence: format!("git log on {}", branch),
                    created_at: now.clone(),
                })?;
                known_edges.insert(edge_key);
                edges_created += 1;
            }
        }
    }

    // Process existing trace notes
    let notes = list_notes().unwrap_or_default();
    for note in &notes {
        let note_entity_id = stable_trace_id("note", &note.id);
        if !known_entities.contains(&note.title) {
            create_entity(KgEntity {
                id: note_entity_id.clone(),
                kind: EntityKind::Note,
                name: note.title.clone(),
                metadata: {
                    let mut m = HashMap::new();
                    m.insert("scope".to_string(), note.scope.as_str().to_string());
                    m.insert("status".to_string(), note.status.as_str().to_string());
                    m
                },
                branch: branch.to_string(),
                created_at: note.created_at.clone(),
                updated_at: note.updated_at.clone(),
            })?;
            known_entities.insert(note.title.clone());
            entities_created += 1;
        }

        // Create Documents edge from note to its target
        if !note.target.is_empty() {
            let target_id = stable_trace_id("file", &note.target);
            if load_entity(&target_id)?.is_none() {
                create_entity(KgEntity {
                    id: target_id.clone(),
                    kind: EntityKind::File,
                    name: note.target.clone(),
                    metadata: HashMap::new(),
                    branch: branch.to_string(),
                    created_at: note.created_at.clone(),
                    updated_at: note.updated_at.clone(),
                })?;
                known_entities.insert(note.target.clone());
                entities_created += 1;
            }
            let edge_id =
                stable_trace_id("edge", &format!("{}|{}|documents", note.id, note.target));
            let edge_key = edge_dedupe_key(&note_entity_id, &target_id, RelationType::Documents);
            if !known_edges.contains(&edge_key) {
                create_edge(KgEdge {
                    id: edge_id.clone(),
                    source: note_entity_id,
                    target: target_id,
                    relation: RelationType::Documents,
                    weight: 1.0,
                    branch: branch.to_string(),
                    evidence: format!("trace note {}", note.id),
                    created_at: note.created_at.clone(),
                })?;
                known_edges.insert(edge_key);
                edges_created += 1;
            }
        }
    }

    Ok((entities_created, edges_created))
}
