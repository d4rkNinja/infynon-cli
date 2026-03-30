use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::api::types::{Assertion, Edge, Flow, FlowRunResult, Node, Extraction, OnFail};

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

// ── YAML intermediate types ───────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct YamlNode {
    id: String,
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    request: Option<YamlRequest>,
    // Also accept flat format (same as internal TOML format)
    #[serde(default)]
    method: Option<String>,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    headers: HashMap<String, String>,
    #[serde(default)]
    body_json: Option<String>,
    #[serde(default)]
    extractions: Vec<YamlExtraction>,
    #[serde(default)]
    assertions: Vec<YamlAssertion>,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct YamlRequest {
    method: String,
    path: String,
    #[serde(default)]
    headers: HashMap<String, String>,
    #[serde(default)]
    body: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct YamlExtraction {
    name: String,
    from: String,
}

#[derive(Debug, Deserialize)]
struct YamlAssertion {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    on_fail: Option<String>,
    // Structured check (YAML format: check.type / check.operator / check.value)
    #[serde(default)]
    check: Option<YamlCheck>,
    // Flat string check (internal format): "status == 200"
    #[serde(default)]
    check_str: Option<String>,
}

#[derive(Debug, Deserialize)]
struct YamlCheck {
    #[serde(rename = "type")]
    check_type: String,
    #[serde(default)]
    operator: Option<String>,
    #[serde(default)]
    value: Option<serde_json::Value>,
    #[serde(default)]
    path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct YamlFlow {
    id: String,
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    base_url: Option<String>,
    // Step-list format (user-authored YAML)
    #[serde(default)]
    steps: Vec<YamlStep>,
    // Graph format (internal / native)
    #[serde(default)]
    entry: Option<String>,
    #[serde(default)]
    edges: Vec<Edge>,
}

#[derive(Debug, Deserialize)]
struct YamlStep {
    node_id: String,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    depends_on: Vec<String>,
    #[serde(default)]
    carry: Vec<String>,
    #[serde(default)]
    condition: Option<String>,
}

// ── YAML → internal type converters ──────────────────────────────────────────

fn yaml_check_to_expr(check: &YamlCheck) -> String {
    let op = match check.operator.as_deref().unwrap_or("equals") {
        "equals" | "eq" => "==",
        "not_equals" | "ne" => "!=",
        "greater_than" | "gt" => ">",
        "less_than" | "lt" => "<",
        "gte" | "greater_than_or_equal" => ">=",
        "lte" | "less_than_or_equal" => "<=",
        "contains" => "contains",
        "exists" => "exists",
        "not_exists" => "not exists",
        other => other,
    };

    let val_str = match &check.value {
        Some(v) => match v {
            serde_json::Value::String(s) => format!("\"{}\"", s),
            serde_json::Value::Null => String::new(),
            other => other.to_string(),
        },
        None => String::new(),
    };

    match check.check_type.as_str() {
        "status" => {
            if op == "exists" || op == "not exists" || val_str.is_empty() {
                format!("status {}", op)
            } else {
                format!("status {} {}", op, val_str)
            }
        }
        "json_path" => {
            let raw = check.path.as_deref().unwrap_or("$");
            // Convert $.field.sub → body.field.sub, $ alone → body
            let field = raw.trim_start_matches("$.");
            let field = if field == "$" || field.is_empty() { "body".to_string() } else { format!("body.{}", field) };
            if op == "exists" || op == "not exists" {
                format!("{} {}", field, op)
            } else {
                format!("{} {} {}", field, op, val_str)
            }
        }
        "header" => {
            let name = check.path.as_deref().unwrap_or("");
            if op == "exists" || op == "not exists" || val_str.is_empty() {
                format!("header.{} {}", name, op)
            } else {
                format!("header.{} {} {}", name, op, val_str)
            }
        }
        "response_time" | "latency" => {
            format!("latency {} {}", op, val_str)
        }
        other => {
            if val_str.is_empty() {
                format!("{} {}", other, op)
            } else {
                format!("{} {} {}", other, op, val_str)
            }
        }
    }
}

fn convert_yaml_node(y: YamlNode) -> Node {
    // Determine method/path/headers/body — support both nested `request` and flat fields
    let (method, path, headers, body_json) = if let Some(req) = y.request {
        let body_json = req.body.as_ref().and_then(|b| {
            if b.is_null() { None } else { serde_json::to_string(b).ok() }
        });
        (req.method, req.path, req.headers, body_json)
    } else {
        (
            y.method.unwrap_or_else(|| "GET".to_string()),
            y.path.unwrap_or_default(),
            y.headers,
            y.body_json,
        )
    };

    let assertions = y.assertions.into_iter().map(|a| {
        let check_expr = if let Some(c) = &a.check {
            yaml_check_to_expr(c)
        } else if let Some(s) = a.check_str {
            s
        } else {
            String::new()
        };
        let on_fail = match a.on_fail.as_deref().unwrap_or("stop") {
            "continue" | "warn" => OnFail::Warn,
            _ => OnFail::Stop,
        };
        Assertion { check: check_expr, on_fail }
    }).filter(|a| !a.check.is_empty()).collect();

    let extractions = y.extractions.into_iter().map(|e| {
        Extraction { name: e.name, from: e.from }
    }).collect();

    Node {
        id: y.id,
        name: y.name,
        method: method.to_uppercase(),
        path,
        headers,
        body_json,
        extractions,
        assertions,
        tags: y.tags,
        description: y.description,
    }
}

fn convert_yaml_flow(y: YamlFlow) -> Flow {
    // If native graph format (has entry), use it directly
    if let Some(entry) = y.entry {
        if !entry.is_empty() {
            return Flow {
                id: y.id,
                name: y.name,
                entry,
                edges: y.edges,
                description: y.description,
                base_url: y.base_url,
            };
        }
    }

    // Convert step-list format → graph edges
    // Build step_id → node_id map
    let step_map: HashMap<String, String> = y.steps.iter()
        .filter_map(|s| s.id.as_ref().map(|id| (id.clone(), s.node_id.clone())))
        .collect();

    // Entry: first step with no depends_on (or simply the first step)
    let entry_node = y.steps.iter()
        .find(|s| s.depends_on.is_empty())
        .or_else(|| y.steps.first())
        .map(|s| s.node_id.clone())
        .unwrap_or_default();

    // Build edges
    let mut edges: Vec<Edge> = Vec::new();
    for step in &y.steps {
        for dep_step_id in &step.depends_on {
            let from_node = step_map.get(dep_step_id)
                .cloned()
                .unwrap_or_else(|| dep_step_id.clone());
            let to_node = step.node_id.clone();
            let already = edges.iter().any(|e| e.from == from_node && e.to == to_node);
            if !already {
                edges.push(Edge {
                    from: from_node,
                    to: to_node,
                    carry: step.carry.clone(),
                    condition: step.condition.clone(),
                });
            }
        }
    }

    Flow {
        id: y.id,
        name: y.name,
        entry: entry_node,
        edges,
        description: y.description,
        base_url: y.base_url,
    }
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
    // Try YAML first, then TOML
    let yaml_path = nodes_dir().join(format!("{}.yaml", id));
    if yaml_path.exists() {
        return load_node_from_path(&yaml_path);
    }
    let toml_path = nodes_dir().join(format!("{}.toml", id));
    load_node_from_path(&toml_path)
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
    let path = nodes_dir().join(format!("{}.toml", id));
    if path.exists() {
        return fs::remove_file(&path)
            .map_err(|e| format!("Cannot delete node '{}': {}", id, e));
    }
    let yaml_path = nodes_dir().join(format!("{}.yaml", id));
    fs::remove_file(&yaml_path)
        .map_err(|e| format!("Cannot delete node '{}': {}", id, e))
}

pub fn node_exists(id: &str) -> bool {
    nodes_dir().join(format!("{}.toml", id)).exists()
        || nodes_dir().join(format!("{}.yaml", id)).exists()
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
    // Try YAML first, then TOML
    let yaml_path = flows_dir().join(format!("{}.yaml", id));
    if yaml_path.exists() {
        return load_flow_from_path(&yaml_path);
    }
    let toml_path = flows_dir().join(format!("{}.toml", id));
    load_flow_from_path(&toml_path)
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
    let path = flows_dir().join(format!("{}.toml", id));
    if path.exists() {
        return fs::remove_file(&path)
            .map_err(|e| format!("Cannot delete flow '{}': {}", id, e));
    }
    let yaml_path = flows_dir().join(format!("{}.yaml", id));
    fs::remove_file(&yaml_path)
        .map_err(|e| format!("Cannot delete flow '{}': {}", id, e))
}

pub fn flow_exists(id: &str) -> bool {
    flows_dir().join(format!("{}.toml", id)).exists()
        || flows_dir().join(format!("{}.yaml", id)).exists()
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
