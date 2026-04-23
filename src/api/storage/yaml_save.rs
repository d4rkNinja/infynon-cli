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

