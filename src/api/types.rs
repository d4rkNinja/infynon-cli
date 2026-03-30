use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// ── Node ─────────────────────────────────────────────────────────────────────

/// A single API call — the atomic unit of a flow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub name: String,
    pub method: String,
    pub path: String, // may contain {variable} placeholders
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// Body template as a JSON string (may contain {variable} placeholders)
    #[serde(default)]
    pub body_json: Option<String>,
    #[serde(default)]
    pub extractions: Vec<Extraction>,
    #[serde(default)]
    pub assertions: Vec<Assertion>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub description: Option<String>,
}

impl Node {
    pub fn new(id: &str, name: &str, method: &str, path: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            method: method.to_uppercase(),
            path: path.to_string(),
            headers: HashMap::new(),
            body_json: None,
            extractions: vec![],
            assertions: vec![],
            tags: vec![],
            description: None,
        }
    }
}

// ── Extraction ────────────────────────────────────────────────────────────────

/// Pulls a value from the response and stores it in the context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extraction {
    /// Name of the variable to store the extracted value as.
    pub name: String,
    /// Where to extract from:
    ///   "status"               → HTTP status code (as integer)
    ///   "body.<json.path>"     → field from JSON body (dot-notation)
    ///   "header.<name>"        → response header value
    pub from: String,
}

// ── Assertion ─────────────────────────────────────────────────────────────────

/// A condition that must hold after a node executes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assertion {
    /// Expression to evaluate, e.g.:
    ///   "status == 201"
    ///   "body.cart_id exists"
    ///   "body.user.name == \"alice\""
    ///   "body.count > 0"
    ///   "header.content-type contains \"application/json\""
    pub check: String,
    #[serde(default)]
    pub on_fail: OnFail,
    #[serde(default = "bool_true")]
    pub enabled: bool,
}

fn bool_true() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum OnFail {
    #[default]
    Stop,
    Warn,
}

// ── Edge ─────────────────────────────────────────────────────────────────────

/// A directed connection between two nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from: String,
    pub to: String,
    /// Variable names from context to carry into the next node.
    #[serde(default)]
    pub carry: Vec<String>,
    /// Optional condition — if present, this edge is only followed when true.
    /// Uses same expression syntax as assertions.
    #[serde(default)]
    pub condition: Option<String>,
}

// ── Flow ──────────────────────────────────────────────────────────────────────

/// A named directed graph of nodes connected by edges.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Flow {
    pub id: String,
    pub name: String,
    /// ID of the first node to execute.
    pub entry: String,
    #[serde(default)]
    pub edges: Vec<Edge>,
    #[serde(default)]
    pub description: Option<String>,
    /// Default base URL for this flow (can be overridden at run time).
    #[serde(default)]
    pub base_url: Option<String>,
}

impl Flow {
    pub fn new(id: &str, name: &str, entry: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            entry: entry.to_string(),
            edges: vec![],
            description: None,
            base_url: None,
        }
    }

    /// All node IDs reachable in this flow (BFS order).
    pub fn all_node_ids(&self) -> Vec<String> {
        let mut visited = vec![self.entry.clone()];
        let mut queue = vec![self.entry.clone()];
        while let Some(current) = queue.first().cloned() {
            queue.remove(0);
            for edge in &self.edges {
                if edge.from == current && !visited.contains(&edge.to) {
                    visited.push(edge.to.clone());
                    queue.push(edge.to.clone());
                }
            }
        }
        visited
    }

    /// Successors of a given node.
    pub fn successors(&self, node_id: &str) -> Vec<&Edge> {
        self.edges.iter().filter(|e| e.from == node_id).collect()
    }

    /// Predecessors of a given node.
    pub fn predecessors(&self, node_id: &str) -> Vec<&Edge> {
        self.edges.iter().filter(|e| e.to == node_id).collect()
    }
}

// ── Run results ───────────────────────────────────────────────────────────────

/// Result of running a single node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub node_id: String,
    pub node_name: String,
    pub method: String,
    pub url: String,
    pub status_code: Option<u16>,
    pub duration_ms: u64,
    pub passed: bool,
    pub assertion_results: Vec<AssertionResult>,
    pub extracted: HashMap<String, serde_json::Value>,
    pub error: Option<String>,
    pub request_body: Option<String>,
    pub response_body: Option<String>,
    pub response_headers: HashMap<String, String>,
}

/// Result of a single assertion check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionResult {
    pub check: String,
    pub passed: bool,
    pub actual: String,
    pub message: Option<String>,
}

/// Result of running an entire flow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowRunResult {
    pub run_id: String,
    pub flow_id: String,
    pub flow_name: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub steps: Vec<StepResult>,
    pub passed: bool,
    pub base_url: String,
    pub final_context: HashMap<String, serde_json::Value>,
}

impl FlowRunResult {
    pub fn duration_ms(&self) -> i64 {
        (self.finished_at - self.started_at).num_milliseconds()
    }

    pub fn passed_count(&self) -> usize {
        self.steps.iter().filter(|s| s.passed).count()
    }

    pub fn failed_count(&self) -> usize {
        self.steps.iter().filter(|s| !s.passed).count()
    }

    pub fn avg_latency_ms(&self) -> u64 {
        if self.steps.is_empty() { return 0; }
        self.steps.iter().map(|s| s.duration_ms).sum::<u64>() / self.steps.len() as u64
    }
}

// ── Node suggestion from AI ───────────────────────────────────────────────────

/// A candidate node suggested by the AI heuristic engine.
#[derive(Debug, Clone)]
pub struct NodeSuggestion {
    pub node: Node,
    pub edge: Edge,
    pub reason: String,
    pub confidence: f32, // 0.0 – 1.0
}

// ── Security probe result ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityProbeResult {
    pub probe_type: ProbeType,
    pub passed: bool,
    pub severity: ProbeSeverity,
    pub description: String,
    pub reproduction: Option<String>, // curl command or similar
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProbeType {
    AuthBypass,
    Idor,
    PrivilegeEscalation,
    TokenReuseAfterLogout,
    MissingRateLimit,
    BrokenStateHandling,
    SqlInjection,
    PathTraversal,
}

impl ProbeType {
    pub fn label(&self) -> &str {
        match self {
            ProbeType::AuthBypass => "Auth Bypass",
            ProbeType::Idor => "IDOR",
            ProbeType::PrivilegeEscalation => "Privilege Escalation",
            ProbeType::TokenReuseAfterLogout => "Token Reuse After Logout",
            ProbeType::MissingRateLimit => "Missing Rate Limit",
            ProbeType::BrokenStateHandling => "Broken State Handling",
            ProbeType::SqlInjection => "SQL Injection",
            ProbeType::PathTraversal => "Path Traversal",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProbeSeverity {
    Critical,
    High,
    Medium,
    Low,
}

impl ProbeSeverity {
    pub fn label(&self) -> &str {
        match self {
            ProbeSeverity::Critical => "CRITICAL",
            ProbeSeverity::High => "HIGH",
            ProbeSeverity::Medium => "MEDIUM",
            ProbeSeverity::Low => "LOW",
        }
    }
}
