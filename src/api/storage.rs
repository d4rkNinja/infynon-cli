use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::api::types::{
    Assertion, Edge, Extraction, Flow, FlowRunResult, Node, OnFail, PromptInput,
};

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
    #[serde(default)]
    prompt_inputs: Vec<PromptInput>,
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
    // Can be a plain string like "status == 200" or a structured mapping
    #[serde(default)]
    check: serde_yaml::Value,
    // Flat string check (internal format): "status == 200" (legacy field)
    #[serde(default)]
    check_str: Option<String>,
    #[serde(default = "bool_true")]
    enabled: bool,
}

fn bool_true() -> bool {
    true
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
    #[serde(default)]
    incoming: Vec<YamlIncomingStep>,
}

#[derive(Debug, Clone, Deserialize)]
struct YamlIncomingStep {
    step: String,
    #[serde(default)]
    carry: Vec<String>,
    #[serde(default)]
    condition: Option<String>,
}

// ── YAML → internal type converters ──────────────────────────────────────────

/// Convert a serde_yaml::Value (either a string or a mapping) to an assertion expression string.
fn yaml_assertion_to_expr(check: &serde_yaml::Value) -> String {
    match check {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Mapping(m) => {
            // Convert mapping keys to strings for easier access
            let get_str = |key: &str| -> Option<String> {
                m.get(serde_yaml::Value::String(key.to_string()))
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
            };
            let get_val = |key: &str| -> Option<serde_json::Value> {
                m.get(serde_yaml::Value::String(key.to_string()))
                    .and_then(|v| serde_json::to_value(v).ok())
            };

            let check_type = get_str("type").unwrap_or_default();
            let operator = get_str("operator");
            let path = get_str("path");
            let value = get_val("value");

            let op = match operator.as_deref().unwrap_or("equals") {
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

            let val_str = match &value {
                Some(v) => match v {
                    serde_json::Value::String(s) => format!("\"{}\"", s),
                    serde_json::Value::Null => String::new(),
                    other => other.to_string(),
                },
                None => String::new(),
            };

            match check_type.as_str() {
                "status" => {
                    if op == "exists" || op == "not exists" || val_str.is_empty() {
                        format!("status {}", op)
                    } else {
                        format!("status {} {}", op, val_str)
                    }
                }
                "json_path" => {
                    let raw = path.as_deref().unwrap_or("$");
                    let field = raw.trim_start_matches("$.");
                    let field = if field == "$" || field.is_empty() {
                        "body".to_string()
                    } else {
                        format!("body.{}", field)
                    };
                    if op == "exists" || op == "not exists" {
                        format!("{} {}", field, op)
                    } else {
                        format!("{} {} {}", field, op, val_str)
                    }
                }
                "header" => {
                    let name = path.as_deref().unwrap_or("");
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
        _ => String::new(),
    }
}

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
            let field = if field == "$" || field.is_empty() {
                "body".to_string()
            } else {
                format!("body.{}", field)
            };
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
            if b.is_null() {
                None
            } else {
                serde_json::to_string(b).ok()
            }
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

    let assertions = y
        .assertions
        .into_iter()
        .map(|a| {
            let check_expr = if !matches!(a.check, serde_yaml::Value::Null) {
                yaml_assertion_to_expr(&a.check)
            } else if let Some(s) = a.check_str {
                s
            } else {
                String::new()
            };
            let on_fail = match a.on_fail.as_deref().unwrap_or("stop") {
                "continue" | "warn" => OnFail::Warn,
                _ => OnFail::Stop,
            };
            Assertion {
                check: check_expr,
                on_fail,
                enabled: a.enabled,
            }
        })
        .filter(|a| !a.check.is_empty())
        .collect();

    let extractions = y
        .extractions
        .into_iter()
        .map(|e| Extraction {
            name: e.name,
            from: e.from,
        })
        .collect();

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
        prompt_inputs: y.prompt_inputs,
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
    let step_map: HashMap<String, String> = y
        .steps
        .iter()
        .filter_map(|s| s.id.as_ref().map(|id| (id.clone(), s.node_id.clone())))
        .collect();

    // Entry: first step with no depends_on (or simply the first step)
    let entry_node = y
        .steps
        .iter()
        .find(|s| s.depends_on.is_empty())
        .or_else(|| y.steps.first())
        .map(|s| s.node_id.clone())
        .unwrap_or_default();

    // Build edges
    let mut edges: Vec<Edge> = Vec::new();
    for step in &y.steps {
        let incoming: Vec<YamlIncomingStep> = if step.incoming.is_empty() {
            step.depends_on
                .iter()
                .cloned()
                .map(|dep_step_id| YamlIncomingStep {
                    step: dep_step_id,
                    carry: step.carry.clone(),
                    condition: step.condition.clone(),
                })
                .collect()
        } else {
            step.incoming.clone()
        };

        for dep in incoming {
            let dep_step_id = dep.step;
            let from_node = step_map.get(&dep_step_id).cloned().unwrap_or(dep_step_id);
            let to_node = step.node_id.clone();
            let already = edges.iter().any(|e| {
                e.from == from_node
                    && e.to == to_node
                    && e.carry == dep.carry
                    && e.condition == dep.condition
            });
            if !already {
                edges.push(Edge {
                    from: from_node,
                    to: to_node,
                    carry: dep.carry,
                    condition: dep.condition,
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

// ── YAML save structs ─────────────────────────────────────────────────────────

#[derive(Serialize)]
struct YamlSaveNode {
    id: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    request: YamlSaveRequest,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    extractions: Vec<YamlSaveExtraction>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    assertions: Vec<YamlSaveAssertion>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    prompt_inputs: Vec<PromptInput>,
}

#[derive(Serialize)]
struct YamlSaveRequest {
    method: String,
    path: String,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    headers: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct YamlSaveExtraction {
    name: String,
    from: String,
}

#[derive(Serialize)]
struct YamlSaveAssertion {
    check: String,
    on_fail: String,
    #[serde(skip_serializing_if = "is_true")]
    enabled: bool,
}

fn is_true(b: &bool) -> bool {
    *b
}

#[derive(Serialize)]
struct YamlSaveFlow {
    id: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    base_url: Option<String>,
    steps: Vec<YamlSaveStep>,
}

#[derive(Serialize)]
struct YamlSaveStep {
    node_id: String,
    id: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    depends_on: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    carry: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    condition: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    incoming: Vec<YamlSaveIncomingStep>,
    on_fail: String,
}

#[derive(Serialize)]
struct YamlSaveIncomingStep {
    step: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    carry: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    condition: Option<String>,
}

// ── YAML save helpers ─────────────────────────────────────────────────────────

/// Returns true if the project uses YAML for nodes or flows.
fn detect_project_yaml() -> bool {
    for dir in [nodes_dir(), flows_dir()] {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if ext == "yaml" || ext == "yml" {
                    return true;
                }
            }
        }
    }
    false
}

fn existing_definition_path(dir: &Path, id: &str) -> Option<PathBuf> {
    ["yaml", "yml", "toml"]
        .into_iter()
        .map(|ext| dir.join(format!("{}.{}", id, ext)))
        .find(|path| path.exists())
}

fn yaml_definition_path(dir: &Path, id: &str) -> Option<PathBuf> {
    ["yaml", "yml"]
        .into_iter()
        .map(|ext| dir.join(format!("{}.{}", id, ext)))
        .find(|path| path.exists())
}

fn node_to_yaml_save(node: &Node) -> YamlSaveNode {
    let body = node
        .body_json
        .as_deref()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok());

    YamlSaveNode {
        id: node.id.clone(),
        name: node.name.clone(),
        description: node.description.clone(),
        request: YamlSaveRequest {
            method: node.method.clone(),
            path: node.path.clone(),
            headers: node.headers.clone(),
            body,
        },
        extractions: node
            .extractions
            .iter()
            .map(|e| YamlSaveExtraction {
                name: e.name.clone(),
                from: e.from.clone(),
            })
            .collect(),
        assertions: node
            .assertions
            .iter()
            .map(|a| YamlSaveAssertion {
                check: a.check.clone(),
                on_fail: match a.on_fail {
                    OnFail::Stop => "stop".to_string(),
                    OnFail::Warn => "warn".to_string(),
                },
                enabled: a.enabled,
            })
            .collect(),
        tags: node.tags.clone(),
        prompt_inputs: node.prompt_inputs.clone(),
    }
}

fn flow_to_yaml_save(flow: &Flow) -> YamlSaveFlow {
    let node_ids = flow.all_node_ids();

    let steps = node_ids
        .iter()
        .map(|node_id| {
            let preds: Vec<&Edge> = flow.predecessors(node_id);
            let incoming: Vec<YamlSaveIncomingStep> = preds
                .iter()
                .map(|e| YamlSaveIncomingStep {
                    step: format!("step-{}", e.from),
                    carry: e.carry.clone(),
                    condition: e.condition.clone(),
                })
                .collect();
            let depends_on = incoming.iter().map(|dep| dep.step.clone()).collect();
            let uniform_metadata = incoming
                .first()
                .map(|first| {
                    incoming
                        .iter()
                        .all(|dep| dep.carry == first.carry && dep.condition == first.condition)
                })
                .unwrap_or(true);
            let carry = if uniform_metadata {
                incoming
                    .first()
                    .map(|dep| dep.carry.clone())
                    .unwrap_or_default()
            } else {
                Vec::new()
            };
            let condition = if uniform_metadata {
                incoming.first().and_then(|dep| dep.condition.clone())
            } else {
                None
            };

            YamlSaveStep {
                node_id: node_id.clone(),
                id: format!("step-{}", node_id),
                depends_on,
                carry,
                condition,
                incoming,
                on_fail: "stop".to_string(),
            }
        })
        .collect();

    YamlSaveFlow {
        id: flow.id.clone(),
        name: flow.name.clone(),
        description: flow.description.clone(),
        base_url: flow.base_url.clone(),
        steps,
    }
}

pub fn save_node_yaml(node: &Node) -> Result<PathBuf, String> {
    let dir = nodes_dir();
    let path = yaml_definition_path(&dir, &node.id)
        .unwrap_or_else(|| dir.join(format!("{}.yaml", node.id)));
    let save = node_to_yaml_save(node);
    let content = serde_yaml::to_string(&save)
        .map_err(|e| format!("Failed to serialize node as YAML: {}", e))?;
    fs::write(&path, content).map_err(|e| format!("Failed to write node YAML file: {}", e))?;
    Ok(path)
}

pub fn save_flow_yaml(flow: &Flow) -> Result<PathBuf, String> {
    let dir = flows_dir();
    let path = yaml_definition_path(&dir, &flow.id)
        .unwrap_or_else(|| dir.join(format!("{}.yaml", flow.id)));
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
    let path = dir.join(format!("{}.toml", node.id));
    let content =
        toml::to_string_pretty(node).map_err(|e| format!("Failed to serialize node: {}", e))?;
    fs::write(&path, content).map_err(|e| format!("Failed to write node file: {}", e))?;
    Ok(path)
}

pub fn load_node(id: &str) -> Result<Node, String> {
    if let Some(path) = existing_definition_path(&nodes_dir(), id) {
        return load_node_from_path(&path);
    }
    load_node_from_path(&nodes_dir().join(format!("{}.toml", id)))
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

pub fn save_flow(flow: &Flow) -> Result<PathBuf, String> {
    // If the project uses YAML files, save as YAML
    if detect_project_yaml() {
        return save_flow_yaml(flow);
    }
    let dir = flows_dir();
    let path = dir.join(format!("{}.toml", flow.id));
    let content =
        toml::to_string_pretty(flow).map_err(|e| format!("Failed to serialize flow: {}", e))?;
    fs::write(&path, content).map_err(|e| format!("Failed to write flow file: {}", e))?;
    Ok(path)
}

pub fn load_flow(id: &str) -> Result<Flow, String> {
    if let Some(path) = existing_definition_path(&flows_dir(), id) {
        return load_flow_from_path(&path);
    }
    load_flow_from_path(&flows_dir().join(format!("{}.toml", id)))
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

// ── Run history I/O ───────────────────────────────────────────────────────────

pub fn save_run_result(result: &FlowRunResult) -> Result<PathBuf, String> {
    let dir = runs_dir();
    let filename = format!("{}__{}.json", result.flow_id, result.run_id);
    let path = dir.join(&filename);
    let content = serde_json::to_string_pretty(result)
        .map_err(|e| format!("Failed to serialize run result: {}", e))?;
    fs::write(&path, content).map_err(|e| format!("Failed to write run result: {}", e))?;
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

#[cfg(test)]
mod tests {
    use super::{convert_yaml_flow, flow_to_yaml_save, YamlFlow};
    use crate::api::types::{Edge, Flow};

    #[test]
    fn yaml_round_trip_preserves_per_incoming_edge_metadata() {
        let flow = Flow {
            id: "checkout".to_string(),
            name: "Checkout".to_string(),
            entry: "login".to_string(),
            edges: vec![
                Edge {
                    from: "login".to_string(),
                    to: "coupon".to_string(),
                    carry: vec!["token".to_string()],
                    condition: Some("status == 200".to_string()),
                },
                Edge {
                    from: "coupon".to_string(),
                    to: "cart".to_string(),
                    carry: vec!["coupon_code".to_string()],
                    condition: Some("body.valid == true".to_string()),
                },
                Edge {
                    from: "login".to_string(),
                    to: "cart".to_string(),
                    carry: vec!["token".to_string()],
                    condition: Some("status == 200".to_string()),
                },
            ],
            description: None,
            base_url: None,
        };

        let yaml = serde_yaml::to_string(&flow_to_yaml_save(&flow)).unwrap();
        let parsed: YamlFlow = serde_yaml::from_str(&yaml).unwrap();
        let round_tripped = convert_yaml_flow(parsed);

        assert_eq!(round_tripped.edges.len(), 3);
        assert!(round_tripped.edges.iter().any(|edge| {
            edge.from == "login"
                && edge.to == "coupon"
                && edge.carry == vec!["token".to_string()]
                && edge.condition.as_deref() == Some("status == 200")
        }));
        assert!(round_tripped.edges.iter().any(|edge| {
            edge.from == "login"
                && edge.to == "cart"
                && edge.carry == vec!["token".to_string()]
                && edge.condition.as_deref() == Some("status == 200")
        }));
        assert!(round_tripped.edges.iter().any(|edge| {
            edge.from == "coupon"
                && edge.to == "cart"
                && edge.carry == vec!["coupon_code".to_string()]
                && edge.condition.as_deref() == Some("body.valid == true")
        }));
    }
}
