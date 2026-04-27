pub fn save_node_yaml(node: &Node) -> Result<PathBuf, String> {
    let dir = nodes_dir();
    let path = yaml_definition_path(&dir, &node.id)
        .unwrap_or_else(|| definition_path(&dir, &node.id, "yaml"));
    let save = node_to_yaml_save(node);
    let content = serde_yaml::to_string(&save)
        .map_err(|e| format!("Failed to serialize node as YAML: {}", e))?;
    fs::write(&path, content).map_err(|e| format!("Failed to write node YAML file: {}", e))?;
    Ok(path)
}

pub fn save_flow_yaml(flow: &Flow) -> Result<PathBuf, String> {
    let dir = flows_dir();
    let path = yaml_definition_path(&dir, &flow.id)
        .unwrap_or_else(|| definition_path(&dir, &flow.id, "yaml"));
    let save = flow_to_yaml_save(flow);
    let content = serde_yaml::to_string(&save)
        .map_err(|e| format!("Failed to serialize flow as YAML: {}", e))?;
    fs::write(&path, content).map_err(|e| format!("Failed to write flow YAML file: {}", e))?;
    Ok(path)
}

// ── Node I/O ──────────────────────────────────────────────────────────────────

pub fn save_node(node: &Node) -> Result<PathBuf, String> {
    // If the project uses YAML files, save as YAML
    if detect_project_yaml() {
        return save_node_yaml(node);
    }
    let dir = nodes_dir();
    let path = definition_path(&dir, &node.id, "toml");
    let content =
        toml::to_string_pretty(node).map_err(|e| format!("Failed to serialize node: {}", e))?;
    fs::write(&path, content).map_err(|e| format!("Failed to write node file: {}", e))?;
    Ok(path)
}

pub fn load_node(id: &str) -> Result<Node, String> {
    if let Some(path) = existing_definition_path(&nodes_dir(), id) {
        return load_node_from_path(&path);
    }
    load_node_from_path(&definition_path(&nodes_dir(), id, "toml"))
}

pub fn load_node_from_path(path: &Path) -> Result<Node, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Cannot read node file '{}': {}", path.display(), e))?;

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext == "yaml" || ext == "yml" {
        let y: YamlNode = serde_yaml::from_str(&content)
            .map_err(|e| format!("Invalid node YAML in '{}': {}", path.display(), e))?;
        Ok(convert_yaml_node(y))
    } else {
        toml::from_str::<Node>(&content)
            .map_err(|e| format!("Invalid node TOML in '{}': {}", path.display(), e))
    }
}

pub fn list_nodes() -> Vec<Node> {
    let dir = nodes_dir();
    let mut nodes = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext == "toml" || ext == "yaml" || ext == "yml" {
                if let Ok(node) = load_node_from_path(&path) {
                    nodes.push(node);
                }
            }
        }
    }
    nodes.sort_by(|a, b| a.id.cmp(&b.id));
    nodes
}

pub fn delete_node(id: &str) -> Result<(), String> {
    if let Some(path) = existing_definition_path(&nodes_dir(), id) {
        return fs::remove_file(&path).map_err(|e| format!("Cannot delete node '{}': {}", id, e));
    }
    Err(format!("Cannot delete node '{}': not found", id))
}

pub fn node_exists(id: &str) -> bool {
    existing_definition_path(&nodes_dir(), id).is_some()
}

/// Load all nodes as a map for fast lookup during flow execution.
pub fn load_nodes_map() -> HashMap<String, Node> {
    list_nodes()
        .into_iter()
        .map(|n| (n.id.clone(), n))
        .collect()
}

// ── Flow I/O ──────────────────────────────────────────────────────────────────
