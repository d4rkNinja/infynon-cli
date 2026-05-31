pub fn save_flow(flow: &Flow) -> Result<PathBuf, String> {
    // If the project uses YAML files, save as YAML
    if detect_project_yaml() {
        return save_flow_yaml(flow);
    }
    let dir = flows_dir();
    let path = definition_path(&dir, &flow.id, "toml");
    let content =
        toml::to_string_pretty(flow).map_err(|e| format!("Failed to serialize flow: {}", e))?;
    fs::write(&path, content).map_err(|e| format!("Failed to write flow file: {}", e))?;
    Ok(path)
}

pub fn load_flow(id: &str) -> Result<Flow, String> {
    if let Some(path) = existing_definition_path(&flows_dir(), id) {
        return load_flow_from_path(&path);
    }
    load_flow_from_path(&definition_path(&flows_dir(), id, "toml"))
}

pub fn load_flow_from_path(path: &Path) -> Result<Flow, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Cannot read flow file '{}': {}", path.display(), e))?;

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext == "yaml" || ext == "yml" {
        let y: YamlFlow = serde_yaml::from_str(&content)
            .map_err(|e| format!("Invalid flow YAML in '{}': {}", path.display(), e))?;
        Ok(convert_yaml_flow(y))
    } else {
        toml::from_str::<Flow>(&content)
            .map_err(|e| format!("Invalid flow TOML in '{}': {}", path.display(), e))
    }
}

pub fn list_flows() -> Vec<Flow> {
    let dir = flows_dir();
    let mut flows = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext == "toml" || ext == "yaml" || ext == "yml" {
                if let Ok(flow) = load_flow_from_path(&path) {
                    flows.push(flow);
                }
            }
        }
    }
    flows.sort_by(|a, b| a.id.cmp(&b.id));
    flows
}

pub fn delete_flow(id: &str) -> Result<(), String> {
    if let Some(path) = existing_definition_path(&flows_dir(), id) {
        return fs::remove_file(&path).map_err(|e| format!("Cannot delete flow '{}': {}", id, e));
    }
    Err(format!("Cannot delete flow '{}': not found", id))
}

pub fn flow_exists(id: &str) -> bool {
    existing_definition_path(&flows_dir(), id).is_some()
}
