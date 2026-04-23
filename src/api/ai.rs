/// AI / heuristic engine for the API testing system.
///
/// No external LLM required — uses smart pattern analysis:
///   - Variable name matching  (token → Authorization header)
///   - URL structure analysis  (/users/{id} → needs user_id)
///   - HTTP method conventions (POST creates → GET/DELETE uses ID it returns)
///   - Common auth patterns    (login returns token, most others need it)
use std::collections::HashMap;

use crate::api::types::{
    Assertion, Edge, Extraction, FlowRunResult, Node, NodeSuggestion, OnFail, ProbeSeverity,
    ProbeType, SecurityProbeResult, StepResult,
};

// ── Variable inference ────────────────────────────────────────────────────────

/// Infer which variables a node produces (from its extractions) and which it
/// consumes (from {placeholders} in path / headers / body).
pub struct NodeInterface {
    pub produces: Vec<String>,
    pub consumes: Vec<String>,
}

pub fn analyze_node(node: &Node) -> NodeInterface {
    let produces: Vec<String> = node.extractions.iter().map(|e| e.name.clone()).collect();
    let mut consumes: Vec<String> = collect_placeholders(&node.path);
    for v in node.headers.values() {
        consumes.extend(collect_placeholders(v));
    }
    if let Some(body) = &node.body_json {
        consumes.extend(collect_placeholders(body));
    }
    consumes.dedup();
    NodeInterface { produces, consumes }
}

fn is_auth_like(s: &str) -> bool {
    let lower = s.to_lowercase();
    lower.contains("token")
        || lower.contains("key")
        || lower.contains("auth")
        || lower.contains("session")
}

fn collect_placeholders(s: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' {
            let mut name = String::new();
            for inner in chars.by_ref() {
                if inner == '}' {
                    break;
                }
                name.push(inner);
            }
            if !name.is_empty() {
                result.push(name);
            }
        }
    }
    result
}

// ── Edge mapping inference ────────────────────────────────────────────────────

/// Given two nodes, infer what variables should be carried on the edge.
/// Returns a list of variable names from `from_node` that `to_node` needs.
pub fn infer_carry(from_node: &Node, to_node: &Node) -> Vec<String> {
    let from_iface = analyze_node(from_node);
    let to_iface = analyze_node(to_node);

    let mut carry: Vec<String> = Vec::new();

    for produced in &from_iface.produces {
        if to_iface.consumes.contains(produced) {
            carry.push(produced.clone());
        }
    }

    // Also infer common implicit carries by name convention
    for produced in &from_iface.produces {
        if is_auth_like(produced) && !carry.contains(produced) {
            carry.push(produced.clone());
        }
    }

    carry.dedup();
    carry
}

// ── Next-node suggestions ─────────────────────────────────────────────────────

/// Given the current node, suggest what should come next.
///
/// Scoring heuristics:
///   +0.5  — to_node consumes variables that from_node produces
///   +0.3  — URL structure suggests continuation (POST /x → GET /x/{id})
///   +0.2  — method progression makes sense (POST → GET/DELETE, not POST → POST same path)
///   -0.3  — circular (same path + method)
pub fn suggest_next_nodes(current: &Node, candidates: &[Node]) -> Vec<NodeSuggestion> {
    let current_iface = analyze_node(current);
    let mut suggestions: Vec<NodeSuggestion> = Vec::new();

    let current_base_path = base_path(&current.path);

    for candidate in candidates {
        if candidate.id == current.id {
            continue;
        }

        let cand_iface = analyze_node(candidate);
        let mut score: f32 = 0.0;
        let mut reasons: Vec<String> = Vec::new();

        // Variable dependency score
        let matching_vars: Vec<&String> = current_iface
            .produces
            .iter()
            .filter(|p| cand_iface.consumes.contains(p))
            .collect();
        if !matching_vars.is_empty() {
            score += 0.5;
            reasons.push(format!(
                "provides variables needed by this node: {}",
                matching_vars
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        // URL structure score
        let cand_base = base_path(&candidate.path);
        if cand_base.starts_with(&current_base_path) || current_base_path.starts_with(&cand_base) {
            score += 0.2;
            reasons.push("related URL path".to_string());
        }

        // Method progression score
        score += method_progression_score(
            &current.method,
            &candidate.method,
            &current.path,
            &candidate.path,
        );

        // Penalise exact duplicate
        if candidate.method == current.method && candidate.path == current.path {
            score -= 0.3;
        }

        // Auth token flow — if current produces a token, anything that consumes it is a good next step
        if current_iface.produces.iter().any(|p| is_auth_like(p))
            && cand_iface.consumes.iter().any(|c| is_auth_like(c))
        {
            score += 0.3;
            reasons.push("consumes auth token from this node".to_string());
        }

        if score > 0.1 {
            let carry = infer_carry(current, candidate);
            let edge = Edge {
                from: current.id.clone(),
                to: candidate.id.clone(),
                carry,
                condition: None,
            };
            suggestions.push(NodeSuggestion {
                node: candidate.clone(),
                edge,
                reason: if reasons.is_empty() {
                    "heuristic match".to_string()
                } else {
                    reasons.join("; ")
                },
                confidence: score.min(1.0),
            });
        }
    }

    // Sort by confidence descending
    suggestions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
    suggestions
}

fn base_path(path: &str) -> String {
    let mut out = String::new();
    for seg in path.split('/') {
        if !seg.starts_with('{') {
            if !out.is_empty() {
                out.push('/');
            }
            out.push_str(seg);
        }
    }
    out
}

fn method_progression_score(
    from_method: &str,
    to_method: &str,
    from_path: &str,
    to_path: &str,
) -> f32 {
    match (from_method, to_method) {
        ("POST", "GET") => 0.25,                   // create → read
        ("POST", "DELETE") => 0.15,                // create → delete (valid test)
        ("GET", "PUT") | ("GET", "PATCH") => 0.15, // read → update
        ("GET", "DELETE") => 0.1,
        ("PUT", "GET") | ("PATCH", "GET") => 0.2, // update → verify
        ("DELETE", "GET") => 0.2,                 // delete → verify gone
        _ => {
            if from_path == to_path {
                -0.1
            } else {
                0.0
            }
        }
    }
}

// ── Flow builder from node collection ─────────────────────────────────────────

/// Automatically wire a set of nodes into a flow using heuristics.
/// Returns a sorted edge list representing the most likely execution order.
pub fn build_flow_edges(nodes: &[Node]) -> (String, Vec<Edge>) {
    if nodes.is_empty() {
        return (String::new(), vec![]);
    }

    // Find the likely entry node: login/auth, or first POST, or alphabetically first
    let entry = find_entry_node(nodes);
    let mut edges: Vec<Edge> = Vec::new();
    let mut remaining: Vec<&Node> = nodes.iter().filter(|n| n.id != entry.id).collect();
    let mut current = entry;

    while !remaining.is_empty() {
        let suggestions = suggest_next_nodes(
            current,
            &remaining.iter().map(|n| (*n).clone()).collect::<Vec<_>>(),
        );
        if let Some(best) = suggestions.into_iter().next() {
            edges.push(best.edge);
            let next_id = best.node.id;
            current = remaining
                .iter()
                .find(|n| n.id == next_id)
                .copied()
                .unwrap_or(current);
            // Find the node in the original slice for the next iteration
            remaining.retain(|n| n.id != current.id);
        } else {
            break;
        }
    }

    (entry.id.clone(), edges)
}

fn find_entry_node(nodes: &[Node]) -> &Node {
    // Priority 1: node whose path/id contains "login", "auth", "register"
    for n in nodes {
        let path = n.path.to_lowercase();
        let id = n.id.to_lowercase();
        if path.contains("login")
            || path.contains("auth")
            || path.contains("register")
            || id.contains("login")
            || id.contains("auth")
        {
            return n;
        }
    }
    // Priority 2: first POST node
    for n in nodes {
        if n.method == "POST" {
            return n;
        }
    }
    // Fallback: first node alphabetically
    nodes.iter().min_by_key(|n| &n.id).unwrap_or(&nodes[0])
}

// ── Auto-generate assertions for a node ───────────────────────────────────────

/// Generate sensible default assertions for a node based on its method and path.
pub fn generate_assertions(node: &Node) -> Vec<Assertion> {
    let mut assertions = Vec::new();

    // Status code assertion based on method
    let expected_status = match node.method.as_str() {
        "POST" => 201,
        "DELETE" => 204,
        _ => 200,
    };
    assertions.push(Assertion {
        check: format!("status == {}", expected_status),
        on_fail: OnFail::Stop,
        enabled: true,
    });

    // If path suggests a resource with ID, assert body has an id field
    if node.method == "POST" {
        assertions.push(Assertion {
            check: "body.id exists".to_string(),
            on_fail: OnFail::Warn,
            enabled: true,
        });
    }

    // JSON content type for non-DELETE methods
    if node.method != "DELETE" && node.method != "HEAD" {
        assertions.push(Assertion {
            check: "header.content-type contains application/json".to_string(),
            on_fail: OnFail::Warn,
            enabled: true,
        });
    }

    assertions
}

/// Generate default extractions for a node based on method and response shape heuristics.
pub fn generate_extractions(node: &Node) -> Vec<Extraction> {
    let mut extractions = Vec::new();

    // Always extract status
    extractions.push(Extraction {
        name: "status".to_string(),
        from: "status".to_string(),
    });

    let path_lower = node.path.to_lowercase();
    let id_lower = node.id.to_lowercase();

    // Auth/login node: extract token
    if path_lower.contains("login") || path_lower.contains("auth") || id_lower.contains("login") {
        extractions.push(Extraction {
            name: "token".to_string(),
            from: "body.token".to_string(),
        });
        extractions.push(Extraction {
            name: "user_id".to_string(),
            from: "body.user_id".to_string(),
        });
    }

    // POST endpoint that creates a resource: extract returned id
    if node.method == "POST" {
        // Derive resource name from last path segment
        let resource = node
            .path
            .trim_end_matches('/')
            .split('/')
            .last()
            .unwrap_or("resource")
            .trim_matches(|c| c == '{' || c == '}');
        let resource = resource.trim_end_matches('s'); // rough singularize: "users" → "user"
        if !resource.is_empty() && resource != "login" && resource != "auth" {
            extractions.push(Extraction {
                name: format!("{}_id", resource),
                from: "body.id".to_string(),
            });
        }
    }

    extractions
}

// ── Security probe generation ─────────────────────────────────────────────────

/// Run security probes on a completed flow run and return findings.
pub fn run_security_probes(
    flow: &crate::api::types::Flow,
    nodes: &HashMap<String, Node>,
    run_result: &FlowRunResult,
    base_url: &str,
) -> Vec<SecurityProbeResult> {
    let mut probes: Vec<SecurityProbeResult> = Vec::new();

    // 1. Auth bypass: try to skip the entry node and call subsequent nodes directly
    probes.push(probe_auth_bypass(flow, nodes, run_result, base_url));

    // 2. Token reuse: check if re-using a response token on a different endpoint works
    probes.push(probe_missing_rate_limit(nodes, run_result, base_url));

    // 3. Check for SQL injection in all nodes
    for step in &run_result.steps {
        if let Some(node) = nodes.get(&step.node_id) {
            probes.push(probe_sql_injection(node, run_result, base_url));
            break; // one representative probe is enough
        }
    }

    probes
}

fn probe_auth_bypass(
    flow: &crate::api::types::Flow,
    nodes: &HashMap<String, Node>,
    run_result: &FlowRunResult,
    base_url: &str,
) -> SecurityProbeResult {
    use crate::api::executor;

    // Find first non-entry step that succeeded
    let target_step = run_result.steps.iter().skip(1).find(|s| s.passed);

    if let Some(step) = target_step {
        if let Some(node) = nodes.get(&step.node_id) {
            // Execute with empty context (no auth)
            let empty_ctx = HashMap::new();
            let result = executor::execute_node(node, &empty_ctx, base_url, None);
            let status = result.status_code.unwrap_or(0);

            let bypassed = status == 200 || status == 201;
            return SecurityProbeResult {
                probe_type: ProbeType::AuthBypass,
                passed: !bypassed,
                severity: ProbeSeverity::Critical,
                description: format!(
                    "Unauthenticated request to {} {} returned {}",
                    node.method, node.path, status
                ),
                reproduction: Some(format!(
                    "curl -X {} {}{} (no auth header)",
                    node.method, base_url, node.path
                )),
                details: if bypassed {
                    Some(
                        "Endpoint accessible without authentication — CRITICAL vulnerability"
                            .to_string(),
                    )
                } else {
                    Some(format!(
                        "Correctly returned {} for unauthenticated request",
                        status
                    ))
                },
            };
        }
    }

    SecurityProbeResult {
        probe_type: ProbeType::AuthBypass,
        passed: true,
        severity: ProbeSeverity::Low,
        description: "Could not find a testable endpoint for auth bypass probe".to_string(),
        reproduction: None,
        details: None,
    }
}

fn probe_missing_rate_limit(
    nodes: &HashMap<String, Node>,
    run_result: &FlowRunResult,
    base_url: &str,
) -> SecurityProbeResult {
    use crate::api::executor;

    // Find a POST endpoint (login/auth is a good candidate)
    let target = nodes.values().find(|n| {
        n.method == "POST"
            && (n.path.to_lowercase().contains("login") || n.path.to_lowercase().contains("auth"))
    });

    if let Some(node) = target {
        // Send 20 requests rapidly and check if any 429 is returned
        let ctx: HashMap<String, serde_json::Value> = HashMap::new();
        let mut got_429 = false;
        for _ in 0..20 {
            let result = executor::execute_node(node, &ctx, base_url, None);
            if result.status_code == Some(429) {
                got_429 = true;
                break;
            }
        }

        return SecurityProbeResult {
            probe_type: ProbeType::MissingRateLimit,
            passed: got_429,
            severity: ProbeSeverity::High,
            description: format!(
                "POST {} — {} after 20 rapid requests",
                node.path,
                if got_429 {
                    "rate limit (429) triggered ✔"
                } else {
                    "no 429 returned"
                }
            ),
            reproduction: Some(format!(
                "for i in $(seq 1 20); do curl -X POST {}{} ; done",
                base_url, node.path
            )),
            details: if !got_429 {
                Some("No rate limiting detected on authentication endpoint".to_string())
            } else {
                None
            },
        };
    }

    SecurityProbeResult {
        probe_type: ProbeType::MissingRateLimit,
        passed: true,
        severity: ProbeSeverity::Low,
        description: "No login/auth endpoint found to test rate limiting".to_string(),
        reproduction: None,
        details: None,
    }
}

fn probe_sql_injection(
    node: &Node,
    run_result: &FlowRunResult,
    base_url: &str,
) -> SecurityProbeResult {
    use crate::api::executor;

    let payloads = ["' OR '1'='1", "' OR 1=1--", "1; DROP TABLE users--"];
    let mut ctx: HashMap<String, serde_json::Value> = HashMap::new();

    // Inject SQLi payload into any string body field
    for payload in &payloads {
        ctx.insert(
            "id".to_string(),
            serde_json::Value::String(payload.to_string()),
        );
        ctx.insert(
            "user_id".to_string(),
            serde_json::Value::String(payload.to_string()),
        );
        let result = executor::execute_node(node, &ctx, base_url, None);
        let status = result.status_code.unwrap_or(0);

        // A 500 suggests the payload broke the server
        if status == 500 {
            return SecurityProbeResult {
                probe_type: ProbeType::SqlInjection,
                passed: false,
                severity: ProbeSeverity::Critical,
                description: format!(
                    "{} {} returned 500 with SQLi payload — potential SQL injection",
                    node.method, node.path
                ),
                reproduction: Some(format!(
                    "curl -X {} {}{} -d '{{\"id\": \"{}\"}}' ",
                    node.method, base_url, node.path, payload
                )),
                details: Some(
                    "Server returned 500 on SQLi payload — investigate DB error handling"
                        .to_string(),
                ),
            };
        }
    }

    SecurityProbeResult {
        probe_type: ProbeType::SqlInjection,
        passed: true,
        severity: ProbeSeverity::Low,
        description: format!("{} {} — no 500s on SQLi payloads", node.method, node.path),
        reproduction: None,
        details: None,
    }
}

// ── Failure explanation ───────────────────────────────────────────────────────

/// Produce a human-readable explanation of why a flow run failed.
pub fn explain_failure(run: &FlowRunResult) -> String {
    let failed_steps: Vec<&StepResult> = run.steps.iter().filter(|s| !s.passed).collect();

    if failed_steps.is_empty() {
        return "All steps passed — no failure to explain.".to_string();
    }

    let mut lines: Vec<String> = Vec::new();
    lines.push(format!(
        "Flow '{}' failed at {} step(s):\n",
        run.flow_name,
        failed_steps.len()
    ));

    for step in &failed_steps {
        lines.push(format!(
            "  Step: {} {} {}",
            step.node_id, step.method, step.url
        ));

        if let Some(err) = &step.error {
            lines.push(format!("  Error: {}", err));
        }

        if let Some(status) = step.status_code {
            lines.push(format!("  Status: {}", status));
        }

        let failed_assertions: Vec<&crate::api::types::AssertionResult> = step
            .assertion_results
            .iter()
            .filter(|a| !a.passed)
            .collect();
        for fa in &failed_assertions {
            lines.push(format!(
                "  Failed check: {} (actual: {})",
                fa.check, fa.actual
            ));
        }

        // Latency insight
        if step.duration_ms > 2000 {
            lines.push(format!(
                "  ⚠ Slow response: {}ms (potential timeout or server-side issue)",
                step.duration_ms
            ));
        }

        lines.push(String::new());
    }

    // Suggest context: look at what was in context at time of failure
    if !run.final_context.is_empty() {
        lines.push("  Context at failure:".to_string());
        for (k, v) in &run.final_context {
            let display = match v {
                serde_json::Value::String(s) => {
                    if s.len() > 30 {
                        format!("{}...", &s[..30])
                    } else {
                        s.clone()
                    }
                }
                other => other.to_string(),
            };
            lines.push(format!("    {} = {}", k, display));
        }
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::build_flow_edges;
    use crate::api::types::Node;

    #[test]
    fn build_flow_edges_follows_the_selected_next_node() {
        let login = Node::new("login", "Login", "POST", "/auth/login");
        let delete = Node::new("delete", "Delete Session", "DELETE", "/auth/session");
        let profile = Node::new("profile", "Profile", "GET", "/auth/session");

        let (entry, edges) = build_flow_edges(&[login, delete, profile]);

        assert_eq!(entry, "login");
        assert_eq!(edges.len(), 2);
        assert_eq!(edges[0].from, "login");
        assert_eq!(edges[0].to, "profile");
        assert_eq!(edges[1].from, "profile");
        assert_eq!(edges[1].to, "delete");
    }
}
