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
            let check_expr: String = if !matches!(a.check, serde_yaml::Value::Null) {
                yaml_assertion_to_expr(&a.check)
            } else {
                a.check_str.unwrap_or_default()
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
