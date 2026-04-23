use std::collections::HashMap;
use std::fs;

use owo_colors::OwoColorize;

use crate::api::types::{Assertion, Extraction, Flow, Node, OnFail};
use crate::api::{ai, storage};
use crate::tui::logger::Logger;

// ── Public entry point ────────────────────────────────────────────────────────

pub fn cmd_import(
    spec_path: &str,
    flow_name: Option<&str>,
    base_url_override: Option<&str>,
    prefix_filter: Option<&str>,
    dry_run: bool,
) {
    println!();

    // Load and parse spec file
    let content = match fs::read_to_string(spec_path) {
        Ok(c) => c,
        Err(e) => {
            Logger::error(&format!("Cannot read spec file '{}': {}", spec_path, e));
            return;
        }
    };

    let spec: serde_yaml::Value = match serde_yaml::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            Logger::error(&format!("Cannot parse spec file: {}", e));
            return;
        }
    };

    // Detect version
    let is_openapi3 = spec.get("openapi").is_some();
    let is_swagger2 = spec.get("swagger").is_some();

    if !is_openapi3 && !is_swagger2 {
        Logger::error("Not a valid OpenAPI or Swagger spec");
        return;
    }

    // Get title/version for display
    let title = spec["info"]["title"].as_str().unwrap_or("Unknown API");
    let version = spec["info"]["version"].as_str().unwrap_or("?");

    // Get base URL
    let base_url = if let Some(url) = base_url_override {
        url.to_string()
    } else if is_openapi3 {
        spec["servers"][0]["url"]
            .as_str()
            .unwrap_or("http://localhost:3000")
            .to_string()
    } else {
        // Swagger 2.x
        let host = spec["host"].as_str().unwrap_or("localhost");
        let base_path = spec["basePath"].as_str().unwrap_or("");
        format!("{}{}", host, base_path)
    };

    // Get paths
    let paths = match spec.get("paths").and_then(|p| p.as_mapping()) {
        Some(m) => m,
        None => {
            Logger::error("No 'paths' found in spec");
            return;
        }
    };

    let mut nodes: Vec<Node> = Vec::new();

    let methods = ["get", "post", "put", "patch", "delete", "head"];

    for (path_key, path_item) in paths {
        let path_str = match path_key.as_str() {
            Some(s) => s,
            None => continue,
        };

        // Apply prefix filter
        if let Some(prefix) = prefix_filter {
            if !path_str.starts_with(prefix) {
                continue;
            }
        }

        for method in &methods {
            let operation = match path_item.get(*method) {
                Some(op) => op,
                None => continue,
            };

            let node = build_node_from_operation(operation, path_str, method, &spec, is_openapi3);
            nodes.push(node);
        }
    }

    // Print summary header
    println!(
        "  {} OpenAPI Import — {} v{}",
        "◆".bright_cyan(),
        title.bold(),
        version
    );
    println!(
        "  {}",
        "─────────────────────────────────────────────────────".truecolor(50, 50, 80)
    );
    println!(
        "  {}  {}",
        "Base URL:".truecolor(100, 100, 140),
        base_url.bright_cyan()
    );
    println!(
        "  {}  {}",
        "Operations:".truecolor(100, 100, 140),
        nodes.len().to_string().bright_cyan()
    );
    println!();

    // Print each node
    for node in &nodes {
        let assertions_count = node.assertions.len();
        let extractions_count = node.extractions.len();

        let mut extras = Vec::new();
        if assertions_count > 0 {
            extras.push(format!("{} assertions", assertions_count));
        }
        if extractions_count > 0 {
            extras.push(format!("{} extractions", extractions_count));
        }
        let extras_str = if extras.is_empty() {
            String::new()
        } else {
            format!("[{}]", extras.join(", "))
        };

        println!(
            "  {:<6} {:<30} → {:<30} {}",
            node.method.bright_yellow(),
            node.path.truecolor(160, 160, 200),
            node.id.bright_cyan(),
            extras_str.truecolor(100, 100, 140),
        );
    }

    println!();

    if dry_run {
        println!(
            "  {}",
            "[dry-run: no files written]".truecolor(180, 140, 60)
        );
        println!();
        return;
    }

    // Save nodes
    let mut saved = 0;
    for node in &nodes {
        match storage::save_node(node) {
            Ok(_) => saved += 1,
            Err(e) => Logger::error(&format!("Could not save node '{}': {}", node.id, e)),
        }
    }

    println!(
        "  {}  Saved {} nodes to .infynon/api/nodes/",
        "✔".bright_green(),
        saved.to_string().bright_cyan()
    );

    // Create flow if requested
    if let Some(fname) = flow_name {
        let flow_id = fname
            .to_lowercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join("-");

        let nodes_map: HashMap<String, Node> =
            nodes.iter().map(|n| (n.id.clone(), n.clone())).collect();
        let all_nodes: Vec<Node> = nodes.clone();

        let (entry, edges) = ai::build_flow_edges(&all_nodes);
        let mut flow = Flow::new(&flow_id, fname, &entry);
        flow.edges = edges;
        flow.base_url = Some(base_url.clone());

        match storage::save_flow(&flow) {
            Ok(path) => {
                println!(
                    "  {}  Created flow '{}' → {}",
                    "✔".bright_green(),
                    fname.bold(),
                    path.display().to_string().truecolor(100, 100, 140)
                );
            }
            Err(e) => Logger::error(&format!("Could not save flow: {}", e)),
        }
    }

    println!();
}

// ── Node builder ──────────────────────────────────────────────────────────────

fn build_node_from_operation(
    operation: &serde_yaml::Value,
    path_str: &str,
    method: &str,
    spec: &serde_yaml::Value,
    is_openapi3: bool,
) -> Node {
    // Derive node ID
    let operation_id = operation["operationId"].as_str();
    let node_id = if let Some(oid) = operation_id {
        camel_to_kebab(oid)
    } else {
        let clean_path = path_str
            .trim_matches('/')
            .replace('/', "-")
            .replace('{', "")
            .replace('}', "");
        format!("{}-{}", method, clean_path)
    };

    // Node name
    let node_name = operation["summary"]
        .as_str()
        .map(|s| s.to_string())
        .or_else(|| operation_id.map(|s| s.to_string()))
        .unwrap_or_else(|| format!("{} {}", method.to_uppercase(), path_str));

    let mut node = Node::new(&node_id, &node_name, method, path_str);

    // Add description
    if let Some(desc) = operation["description"].as_str() {
        node.description = Some(desc.to_string());
    }

    // Headers
    let needs_content_type = matches!(method, "post" | "put" | "patch");
    if needs_content_type {
        node.headers
            .insert("Content-Type".to_string(), "application/json".to_string());
    }

    // Add Authorization unless it looks like an auth endpoint
    let path_lower = path_str.to_lowercase();
    let is_auth =
        path_lower.contains("auth") || path_lower.contains("login") || path_lower.contains("token");
    if !is_auth {
        node.headers.insert(
            "Authorization".to_string(),
            "Bearer {$AUTH_TOKEN}".to_string(),
        );
    }

    // Body for POST/PUT/PATCH
    if needs_content_type {
        let schema = if is_openapi3 {
            operation
                .get("requestBody")
                .and_then(|rb| rb.get("content"))
                .and_then(|c| c.get("application/json"))
                .and_then(|aj| aj.get("schema"))
        } else {
            // Swagger 2.x: find body parameter
            operation
                .get("parameters")
                .and_then(|params| params.as_sequence())
                .and_then(|params| {
                    params
                        .iter()
                        .find(|p| p.get("in").and_then(|v| v.as_str()) == Some("body"))
                })
                .and_then(|p| p.get("schema"))
        };

        if let Some(schema) = schema {
            if let Some(body) = schema_to_body_template(schema, spec) {
                node.body_json = serde_json::to_string(&body).ok();
            }
        }
    }

    // Extractions from response schema
    let response_schema = get_first_2xx_response_schema(operation, spec, is_openapi3);
    if let Some(schema) = response_schema {
        node.extractions = schema_to_extractions(schema, spec);
    }

    // Assertions
    let first_2xx_code = get_first_2xx_code(operation);
    if let Some(code) = first_2xx_code {
        node.assertions.push(Assertion {
            check: format!("status == {}", code),
            on_fail: OnFail::Stop,
            enabled: true,
        });
    }

    // body exists assertion if there's a response body schema
    if response_schema.is_some() {
        node.assertions.push(Assertion {
            check: "body exists".to_string(),
            on_fail: OnFail::Warn,
            enabled: true,
        });
    }

    node
}

// ── Schema helpers ────────────────────────────────────────────────────────────

fn get_first_2xx_code(operation: &serde_yaml::Value) -> Option<u16> {
    if let Some(responses) = operation.get("responses").and_then(|r| r.as_mapping()) {
        for (code_key, _) in responses {
            if let Some(code_str) = code_key.as_str() {
                if let Ok(code) = code_str.parse::<u16>() {
                    if (200..300).contains(&code) {
                        return Some(code);
                    }
                }
            }
        }
    }
    None
}

fn get_first_2xx_response_schema<'a>(
    operation: &'a serde_yaml::Value,
    spec: &'a serde_yaml::Value,
    is_openapi3: bool,
) -> Option<&'a serde_yaml::Value> {
    let responses = operation.get("responses")?.as_mapping()?;
    for (code_key, response) in responses {
        let code_str = code_key.as_str()?;
        let code: u16 = code_str.parse().ok()?;
        if (200..300).contains(&code) {
            if is_openapi3 {
                return response
                    .get("content")
                    .and_then(|c| c.get("application/json"))
                    .and_then(|aj| aj.get("schema"));
            } else {
                return response.get("schema");
            }
        }
    }
    None
}

fn resolve_ref<'a>(
    schema: &'a serde_yaml::Value,
    spec: &'a serde_yaml::Value,
) -> &'a serde_yaml::Value {
    if let Some(ref_str) = schema.get("$ref").and_then(|r| r.as_str()) {
        // Handles "#/components/schemas/MyModel" and "#/definitions/MyModel"
        let parts: Vec<&str> = ref_str
            .trim_start_matches('#')
            .trim_start_matches('/')
            .split('/')
            .collect();
        let mut current = spec;
        for part in &parts {
            current = match current.get(*part) {
                Some(v) => v,
                None => return schema,
            };
        }
        return current;
    }
    schema
}

fn schema_to_body_template(
    schema: &serde_yaml::Value,
    spec: &serde_yaml::Value,
) -> Option<serde_json::Value> {
    let schema = resolve_ref(schema, spec);

    // Check schema type
    let schema_type = schema
        .get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("object");

    if schema_type != "object" && schema.get("properties").is_none() {
        return None;
    }

    let properties = schema.get("properties")?.as_mapping()?;
    if properties.is_empty() {
        return None;
    }

    let mut obj = serde_json::Map::new();
    for (field_key, field_schema) in properties {
        let field_name = field_key.as_str()?;
        let field_schema = resolve_ref(field_schema, spec);
        let field_type = field_schema
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("string");

        let val = match field_type {
            "integer" | "number" => serde_json::Value::Number(0.into()),
            "boolean" => serde_json::Value::Bool(false),
            "array" => serde_json::Value::Array(vec![]),
            _ => serde_json::Value::String(format!("{{{}}}", field_name)),
        };
        obj.insert(field_name.to_string(), val);
    }

    Some(serde_json::Value::Object(obj))
}

fn schema_to_extractions(schema: &serde_yaml::Value, spec: &serde_yaml::Value) -> Vec<Extraction> {
    let schema = resolve_ref(schema, spec);
    let properties = match schema.get("properties").and_then(|p| p.as_mapping()) {
        Some(p) => p,
        None => return vec![],
    };

    let good_extractions = [
        "id", "token", "url", "key", "session", "hash", "data", "result",
    ];

    let mut extractions = Vec::new();
    for (field_key, _) in properties {
        if extractions.len() >= 5 {
            break;
        }
        let field_name = match field_key.as_str() {
            Some(s) => s,
            None => continue,
        };
        let field_lower = field_name.to_lowercase();
        let is_candidate = good_extractions.contains(&field_lower.as_str())
            || field_lower.ends_with("_id")
            || field_lower.ends_with("_token")
            || field_lower.ends_with("_url")
            || field_lower.contains("token")
            || field_lower.contains("session")
            || field_lower.contains("hash");

        if is_candidate {
            extractions.push(Extraction {
                name: field_name.to_string(),
                from: format!("body.{}", field_name),
            });
        }
    }

    extractions
}

// ── Utility ───────────────────────────────────────────────────────────────────

/// Convert camelCase or PascalCase to kebab-case.
fn camel_to_kebab(s: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c == ' ' {
            result.push('-');
        } else if c.is_uppercase() && i > 0 && !chars[i - 1].is_uppercase() {
            result.push('-');
            result.push(c.to_lowercase().next().unwrap_or(c));
        } else {
            result.push(c.to_lowercase().next().unwrap_or(c));
        }
    }
    result
}
