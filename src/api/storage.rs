use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::api::types::{Flow, FlowRunResult, Node};

// ── Directory helpers ─────────────────────────────────────────────────────────

/// Returns `.infynon/api/nodes/` relative to the current directory, creating it if needed.
pub fn nodes_dir() -> PathBuf {
    let dir = PathBuf::from(".infynon/api/nodes");
    fs::create_dir_all(&dir).ok();
    dir
}

/// Returns `.infynon/api/flows/` relative to the current directory, creating it if needed.
pub fn flows_dir() -> PathBuf {
    let dir = PathBuf::from(".infynon/api/flows");
    fs::create_dir_all(&dir).ok();
    dir
}

/// Returns `.infynon/api/runs/` for storing run history.
pub fn runs_dir() -> PathBuf {
    let dir = PathBuf::from(".infynon/api/runs");
    fs::create_dir_all(&dir).ok();
    dir
}

// ── Node I/O ──────────────────────────────────────────────────────────────────

pub fn save_node(node: &Node) -> Result<PathBuf, String> {
    let dir = nodes_dir();
    let path = dir.join(format!("{}.toml", node.id));
    let content = toml::to_string_pretty(node)
        .map_err(|e| format!("Failed to serialize node: {}", e))?;
    fs::write(&path, content)
        .map_err(|e| format!("Failed to write node file: {}", e))?;
    Ok(path)
}

pub fn load_node(id: &str) -> Result<Node, String> {
    let path = nodes_dir().join(format!("{}.toml", id));
    load_node_from_path(&path)
}

pub fn load_node_from_path(path: &Path) -> Result<Node, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Cannot read node file '{}': {}", path.display(), e))?;
    toml::from_str::<Node>(&content)
        .map_err(|e| format!("Invalid node TOML in '{}': {}", path.display(), e))
}

pub fn list_nodes() -> Vec<Node> {
    let dir = nodes_dir();
    let mut nodes = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("toml") {
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
    let path = nodes_dir().join(format!("{}.toml", id));
    fs::remove_file(&path)
        .map_err(|e| format!("Cannot delete node '{}': {}", id, e))
}

pub fn node_exists(id: &str) -> bool {
    nodes_dir().join(format!("{}.toml", id)).exists()
}

/// Load all nodes as a map for fast lookup during flow execution.
pub fn load_nodes_map() -> HashMap<String, Node> {
    list_nodes()
        .into_iter()
        .map(|n| (n.id.clone(), n))
        .collect()
}

// ── Flow I/O ──────────────────────────────────────────────────────────────────

pub fn save_flow(flow: &Flow) -> Result<PathBuf, String> {
    let dir = flows_dir();
    let path = dir.join(format!("{}.toml", flow.id));
    let content = toml::to_string_pretty(flow)
        .map_err(|e| format!("Failed to serialize flow: {}", e))?;
    fs::write(&path, content)
        .map_err(|e| format!("Failed to write flow file: {}", e))?;
    Ok(path)
}

pub fn load_flow(id: &str) -> Result<Flow, String> {
    let path = flows_dir().join(format!("{}.toml", id));
    load_flow_from_path(&path)
}

pub fn load_flow_from_path(path: &Path) -> Result<Flow, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Cannot read flow file '{}': {}", path.display(), e))?;
    toml::from_str::<Flow>(&content)
        .map_err(|e| format!("Invalid flow TOML in '{}': {}", path.display(), e))
}

pub fn list_flows() -> Vec<Flow> {
    let dir = flows_dir();
    let mut flows = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("toml") {
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
    let path = flows_dir().join(format!("{}.toml", id));
    fs::remove_file(&path)
        .map_err(|e| format!("Cannot delete flow '{}': {}", id, e))
}

pub fn flow_exists(id: &str) -> bool {
    flows_dir().join(format!("{}.toml", id)).exists()
}

// ── Run history I/O ───────────────────────────────────────────────────────────

pub fn save_run_result(result: &FlowRunResult) -> Result<PathBuf, String> {
    let dir = runs_dir();
    let filename = format!("{}__{}.json", result.flow_id, result.run_id);
    let path = dir.join(&filename);
    let content = serde_json::to_string_pretty(result)
        .map_err(|e| format!("Failed to serialize run result: {}", e))?;
    fs::write(&path, content)
        .map_err(|e| format!("Failed to write run result: {}", e))?;
    Ok(path)
}

/// Load the N most recent run results for a given flow.
pub fn load_recent_runs(flow_id: &str, limit: usize) -> Vec<FlowRunResult> {
    let dir = runs_dir();
    let prefix = format!("{}_", flow_id);
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

        // Sort by modification time, newest first (fetch metadata once per file)
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
