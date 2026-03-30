use std::collections::{HashMap, VecDeque};
use std::time::Instant;

use reqwest::blocking::{Client, RequestBuilder};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::Value;
use chrono::Utc;

use crate::api::assertions;
use crate::api::types::{
    Assertion, Edge, Extraction, FlowRunResult, Node, OnFail, PromptInput, StepResult,
};
use crate::api::variables;

// ── HTTP client ───────────────────────────────────────────────────────────────

fn http_client() -> Client {
    Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("infynon-api-tester/1.0")
        .build()
        .expect("Failed to build HTTP client")
}

// ── Single node execution ─────────────────────────────────────────────────────

/// Execute a single node with the given context variables and base URL.
/// Returns a StepResult with extracted variables, assertion results, and timing.
/// If `on_prompt` is provided and the node has `prompt_inputs`, the callback is called
/// before the request is sent to collect user-supplied values.
pub fn execute_node(
    node: &Node,
    context: &HashMap<String, Value>,
    base_url: &str,
    on_prompt: Option<&(dyn Fn(&str, &[PromptInput]) -> HashMap<String, Value> + Send)>,
) -> StepResult {
    let client = http_client();

    // Collect prompt inputs if the node has any
    let mut context = context.clone();
    if !node.prompt_inputs.is_empty() {
        if let Some(prompt_fn) = on_prompt {
            let prompted = prompt_fn(&node.id, &node.prompt_inputs);
            for (k, v) in prompted {
                context.insert(k, v);
            }
        }
    }

    // Substitute variables into path, headers, body
    let path = variables::substitute_path(&node.path, &context);
    let url = format!("{}{}", base_url.trim_end_matches('/'), path);
    let headers = variables::substitute_headers(&node.headers, &context);

    let body_value: Option<Value> = node
        .body_json
        .as_deref()
        .map(|tmpl| variables::substitute_body(tmpl, &context));

    // Build request
    let method = node.method.to_uppercase();
    let req = match method.as_str() {
        "GET"    => client.get(&url),
        "POST"   => client.post(&url),
        "PUT"    => client.put(&url),
        "PATCH"  => client.patch(&url),
        "DELETE" => client.delete(&url),
        "HEAD"   => client.head(&url),
        other    => {
            return StepResult {
                node_id: node.id.clone(),
                node_name: node.name.clone(),
                method: method.clone(),
                url,
                status_code: None,
                duration_ms: 0,
                passed: false,
                assertion_results: vec![],
                extracted: HashMap::new(),
                error: Some(format!("Unsupported HTTP method: {}", other)),
                request_body: None,
                response_body: None,
                response_headers: HashMap::new(),
            };
        }
    };

    // Attach headers
    let req = attach_headers(req, &headers);

    // Attach body
    let req_body_str = body_value.as_ref().map(|v| v.to_string());
    let req = if let Some(ref body) = body_value {
        req.json(body)
    } else {
        req
    };

    // Execute and time
    let started = Instant::now();
    let response = req.send();
    let duration_ms = started.elapsed().as_millis() as u64;

    match response {
        Err(e) => StepResult {
            node_id: node.id.clone(),
            node_name: node.name.clone(),
            method,
            url,
            status_code: None,
            duration_ms,
            passed: false,
            assertion_results: vec![],
            extracted: HashMap::new(),
            error: Some(format!("Request failed: {}", e)),
            request_body: req_body_str,
            response_body: None,
            response_headers: HashMap::new(),
        },
        Ok(resp) => {
            let status = resp.status().as_u16();
            let resp_headers = collect_headers(resp.headers());
            let body_str = resp.text().unwrap_or_default();
            let body_json: Value = serde_json::from_str(&body_str)
                .unwrap_or(Value::String(body_str.clone()));

            // Evaluate assertions (skip disabled ones)
            let enabled_assertions: Vec<&crate::api::types::Assertion> = node
                .assertions
                .iter()
                .filter(|a| a.enabled)
                .collect();
            let assertion_results: Vec<_> = enabled_assertions
                .iter()
                .map(|a| assertions::evaluate(&a.check, status, &body_json, &resp_headers))
                .collect();

            // Determine pass/fail (stop on first failing Stop assertion)
            let passed = check_passed_enabled(&enabled_assertions, &assertion_results);

            // Extract variables from response
            let extracted = extract_variables(&node.extractions, status, &body_json, &resp_headers);

            StepResult {
                node_id: node.id.clone(),
                node_name: node.name.clone(),
                method,
                url,
                status_code: Some(status),
                duration_ms,
                passed,
                assertion_results,
                extracted,
                error: None,
                request_body: req_body_str,
                response_body: Some(body_str),
                response_headers: resp_headers,
            }
        }
    }
}

// ── Flow execution ────────────────────────────────────────────────────────────

pub struct FlowExecuteOptions {
    pub base_url: String,
    /// Pre-seeded context variables injected before the first node runs (e.g. from --set flags).
    pub initial_context: HashMap<String, Value>,
    /// Called after each step so callers (TUI, CLI) can show live progress.
    pub on_step: Option<Box<dyn Fn(&StepResult)>>,
    /// Called before a node fires if that node has prompt_inputs.
    /// Receives the node ID and the list of inputs; must return a map of var → value.
    pub on_prompt: Option<Box<dyn Fn(&str, &[PromptInput]) -> HashMap<String, Value> + Send>>,
}

/// Execute an entire flow, threading context through edges.
/// Returns a FlowRunResult with all step results.
pub fn execute_flow(
    flow: &crate::api::types::Flow,
    nodes: &HashMap<String, Node>,
    opts: FlowExecuteOptions,
) -> FlowRunResult {
    let started_at = Utc::now();
    let run_id = format!("{}", started_at.timestamp_millis());

    let mut context: HashMap<String, Value> = opts.initial_context.clone();
    let mut steps: Vec<StepResult> = Vec::new();
    let mut overall_passed = true;

    // BFS execution following edges
    let mut current_nodes: VecDeque<String> = VecDeque::new();
    current_nodes.push_back(flow.entry.clone());
    let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();

    while let Some(node_id) = current_nodes.pop_front() {

        if visited.contains(&node_id) {
            continue;
        }
        visited.insert(node_id.clone());

        // Look up node
        let node = match nodes.get(&node_id) {
            Some(n) => n,
            None => {
                let err_step = StepResult {
                    node_id: node_id.clone(),
                    node_name: node_id.clone(),
                    method: String::new(),
                    url: String::new(),
                    status_code: None,
                    duration_ms: 0,
                    passed: false,
                    assertion_results: vec![],
                    extracted: HashMap::new(),
                    error: Some(format!("Node '{}' not found in library", node_id)),
                    request_body: None,
                    response_body: None,
                    response_headers: HashMap::new(),
                };
                if let Some(cb) = &opts.on_step {
                    cb(&err_step);
                }
                steps.push(err_step);
                overall_passed = false;
                continue;
            }
        };

        // Execute the node
        let step = execute_node(node, &context, &opts.base_url, opts.on_prompt.as_deref());

        if let Some(cb) = &opts.on_step {
            cb(&step);
        }

        // Merge extracted variables into context
        for (k, v) in &step.extracted {
            context.insert(k.clone(), v.clone());
        }

        // Check if we should stop (only consider enabled assertions)
        let should_stop = !step.passed
            && node.assertions.iter().any(|a| a.enabled && a.on_fail == OnFail::Stop);

        if !step.passed {
            overall_passed = false;
        }

        steps.push(step);

        if should_stop {
            break;
        }

        // Find and queue successors
        for edge in flow.successors(&node_id) {
            // Evaluate edge condition if present
            if let Some(cond) = &edge.condition {
                // Use last step result for condition evaluation
                if let Some(last) = steps.last() {
                    let status = last.status_code.unwrap_or(0);
                    let body: Value = last
                        .response_body
                        .as_deref()
                        .and_then(|s| serde_json::from_str(s).ok())
                        .unwrap_or(Value::Null);
                    let result = assertions::evaluate(cond, status, &body, &last.response_headers);
                    if !result.passed {
                        continue; // condition not met, skip this edge
                    }
                }
            }

            if !visited.contains(&edge.to) {
                current_nodes.push_back(edge.to.clone());
            }
        }
    }

    let finished_at = Utc::now();

    FlowRunResult {
        run_id,
        flow_id: flow.id.clone(),
        flow_name: flow.name.clone(),
        started_at,
        finished_at,
        steps,
        passed: overall_passed,
        base_url: opts.base_url,
        final_context: context,
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn attach_headers(req: RequestBuilder, headers: &HashMap<String, String>) -> RequestBuilder {
    let mut header_map = HeaderMap::new();
    for (k, v) in headers {
        if let (Ok(name), Ok(value)) = (
            HeaderName::from_bytes(k.as_bytes()),
            HeaderValue::from_str(v),
        ) {
            header_map.insert(name, value);
        }
    }
    req.headers(header_map)
}

fn collect_headers(headers: &reqwest::header::HeaderMap) -> HashMap<String, String> {
    headers
        .iter()
        .filter_map(|(k, v)| {
            v.to_str().ok().map(|s| (k.as_str().to_lowercase(), s.to_string()))
        })
        .collect()
}

fn check_passed(assertions: &[Assertion], results: &[crate::api::types::AssertionResult]) -> bool {
    for (assertion, result) in assertions.iter().zip(results.iter()) {
        if !result.passed && assertion.on_fail == OnFail::Stop {
            return false;
        }
    }
    true
}

fn check_passed_enabled(assertions: &[&Assertion], results: &[crate::api::types::AssertionResult]) -> bool {
    for (assertion, result) in assertions.iter().zip(results.iter()) {
        if !result.passed && assertion.on_fail == OnFail::Stop {
            return false;
        }
    }
    true
}

fn extract_variables(
    extractions: &[Extraction],
    status: u16,
    body: &Value,
    headers: &HashMap<String, String>,
) -> HashMap<String, Value> {
    let mut out = HashMap::new();
    for ext in extractions {
        let val = extract_one(&ext.from, status, body, headers);
        if let Some(v) = val {
            out.insert(ext.name.clone(), v);
        }
    }
    out
}

fn extract_one(
    from: &str,
    status: u16,
    body: &Value,
    headers: &HashMap<String, String>,
) -> Option<Value> {
    if from == "status" {
        return Some(Value::Number(status.into()));
    }
    if let Some(path) = from.strip_prefix("body.") {
        return assertions::json_path(body, path).cloned();
    }
    if let Some(name) = from.strip_prefix("header.") {
        let key = name.to_lowercase();
        return headers
            .get(&key)
            .or_else(|| headers.iter().find(|(k, _)| k.to_lowercase() == key).map(|(_, v)| v))
            .map(|v| Value::String(v.clone()));
    }
    // bare "body" — return entire body
    if from == "body" {
        return Some(body.clone());
    }
    None
}
