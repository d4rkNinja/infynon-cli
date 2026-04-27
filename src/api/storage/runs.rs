pub fn save_run_result(result: &FlowRunResult) -> Result<PathBuf, String> {
    let dir = runs_dir();
    let filename = format!(
        "{}__{}.json",
        crate::utils::safe_file_stem(&result.flow_id),
        crate::utils::safe_file_stem(&result.run_id)
    );
    let path = dir.join(&filename);
    let content = serde_json::to_string_pretty(result)
        .map_err(|e| format!("Failed to serialize run result: {}", e))?;
    fs::write(&path, content).map_err(|e| format!("Failed to write run result: {}", e))?;
    Ok(path)
}

/// Load the N most recent run results for a given flow.
pub fn load_recent_runs(flow_id: &str, limit: usize) -> Vec<FlowRunResult> {
    let dir = runs_dir();
    let prefix = format!("{}_", crate::utils::safe_file_stem(flow_id));
    let mut results = Vec::new();

    if let Ok(entries) = fs::read_dir(&dir) {
        let paths: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                p.extension().and_then(|e| e.to_str()) == Some("json")
                    && p.file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n.starts_with(&prefix))
                        .unwrap_or(false)
            })
            .collect();

        // Sort by modification time, newest first
        let mut paths: Vec<_> = paths
            .into_iter()
            .map(|p| {
                let mtime = fs::metadata(&p).and_then(|m| m.modified()).ok();
                (p, mtime)
            })
            .collect();
        paths.sort_by(|a, b| b.1.cmp(&a.1));

        for (path, _) in paths.iter().take(limit) {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(result) = serde_json::from_str::<FlowRunResult>(&content) {
                    results.push(result);
                }
            }
        }
    }
    results
}

/// Load all run results across all flows, newest first, up to limit.
pub fn load_all_recent_runs(limit: usize) -> Vec<FlowRunResult> {
    let dir = runs_dir();
    let mut results = Vec::new();

    if let Ok(entries) = fs::read_dir(&dir) {
        let paths: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
            .collect();

        let mut paths: Vec<_> = paths
            .into_iter()
            .map(|p| {
                let mtime = fs::metadata(&p).and_then(|m| m.modified()).ok();
                (p, mtime)
            })
            .collect();
        paths.sort_by(|a, b| b.1.cmp(&a.1));

        for (path, _) in paths.iter().take(limit) {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(result) = serde_json::from_str::<FlowRunResult>(&content) {
                    results.push(result);
                }
            }
        }
    }
    results
}
