use std::collections::HashMap;
use std::sync::mpsc;

use crate::api::storage;
use crate::api::types::{Flow, FlowRunResult};

use super::types::*;

// ── Graph layout algorithm ────────────────────────────────────────────────────

/// BFS-based layered layout for the flow graph.
/// Returns GraphNode entries with (layer, col) positions.
pub fn compute_graph_layout(flow: &Flow) -> Vec<GraphNode> {
    if flow.entry.is_empty() {
        return vec![];
    }

    // BFS to assign layers
    let mut layers: HashMap<String, usize> = HashMap::new();
    let mut queue: std::collections::VecDeque<(String, usize)> = std::collections::VecDeque::new();
    queue.push_back((flow.entry.clone(), 0usize));
    let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();

    while let Some((node_id, layer)) = queue.pop_front() {
        if visited.contains(&node_id) { continue; }
        visited.insert(node_id.clone());
        layers.entry(node_id.clone()).or_insert(layer);

        for edge in flow.successors(&node_id) {
            if !visited.contains(&edge.to) {
                queue.push_back((edge.to.clone(), layer + 1));
            }
        }
    }

    if layers.is_empty() {
        return vec![];
    }

    // Group by layer and assign columns
    let max_layer = *layers.values().max().unwrap_or(&0);
    let mut result: Vec<GraphNode> = Vec::new();

    for layer in 0..=max_layer {
        let mut nodes_in_layer: Vec<String> = layers.iter()
            .filter(|(_, &l)| l == layer)
            .map(|(id, _)| id.clone())
            .collect();
        nodes_in_layer.sort();

        for (col, node_id) in nodes_in_layer.into_iter().enumerate() {
            result.push(GraphNode { node_id, layer, col });
        }
    }

    result
}

// ── Flow runner methods on ApiApp ─────────────────────────────────────────────

impl super::app_state::ApiApp {
    // ── Flow runner ───────────────────────────────────────────────────────

    pub fn start_flow_run(&mut self) {
        let flow = match self.flows.get(self.active_flow_idx) {
            Some(f) => f.clone(),
            None => return,
        };
        let nodes = self.nodes.clone();
        let base_url = match flow.base_url.clone()
            .or_else(|| crate::api::commands::env::env_base_url())
        {
            Some(u) => u,
            None => {
                self.notify("BASE_URL not set — add it in Env tab or .infynon/.env");
                return;
            }
        };

        let (tx, rx) = mpsc::sync_channel::<LiveEvent>(100);
        self.run_rx = Some(rx);
        self.run_tx = Some(tx.clone());
        self.live_steps.clear();
        self.live_running = true;
        self.flow_running = true;
        self.run_error = None;
        self.live_selected_step = 0;
        self.step_detail = None;

        // Create reply channel for prompt answers
        let (reply_tx, reply_rx) = mpsc::sync_channel::<HashMap<String, serde_json::Value>>(1);
        self.prompt_reply_tx = Some(reply_tx);

        std::thread::spawn(move || {
            use crate::api::executor::{execute_flow, FlowExecuteOptions};
            let tx2 = tx.clone();
            let tx_prompt = tx.clone();
            // Pre-seed context with env vars so {VAR} placeholders already in env
            // don't trigger prompts unnecessarily.
            let initial_context = crate::api::variables::load_env_context();
            let result = execute_flow(&flow, &nodes, FlowExecuteOptions {
                base_url,
                initial_context,
                on_step: Some(Box::new(move |step| {
                    tx.send(LiveEvent::Step(step.clone())).ok();
                })),
                on_prompt: Some(Box::new(move |node_id: &str, inputs: &[crate::api::types::PromptInput]| {
                    tx_prompt.send(LiveEvent::NeedInput {
                        node_id: node_id.to_string(),
                        inputs: inputs.to_vec(),
                    }).ok();
                    // Block until the TUI sends back the values
                    reply_rx.recv().unwrap_or_default()
                })),
            });
            let passed = result.passed;
            // Persist run result to disk so future sessions can load it
            crate::api::storage::save_run_result(&result).ok();
            // Send full result to TUI so last_run is updated immediately
            tx2.send(LiveEvent::FlowResult(result)).ok();
            tx2.send(LiveEvent::Done { passed }).ok();
        });

        self.current_view = ApiView::Runner;
        self.runner_subview = RunnerSubview::Steps;
    }

    pub fn start_node_run(&mut self, node_id: &str) {
        let node = match self.nodes.get(node_id).cloned() {
            Some(n) => n,
            None => {
                self.notify(&format!("Node '{}' not found", node_id));
                return;
            }
        };
        let base_url = match self.active_flow()
            .and_then(|f| f.base_url.clone())
            .or_else(|| crate::api::commands::env::env_base_url())
        {
            Some(u) => u,
            None => {
                self.notify("BASE_URL not set — add it in Env tab or .infynon/.env");
                return;
            }
        };

        let (tx, rx) = mpsc::sync_channel::<LiveEvent>(100);
        self.run_rx = Some(rx);
        self.run_tx = Some(tx.clone());
        self.live_steps.clear();
        self.live_running = true;
        self.flow_running = true;
        self.run_error = None;
        self.live_selected_step = 0;
        self.step_detail = None;

        // Create reply channel for prompt answers
        let (reply_tx, reply_rx) = mpsc::sync_channel::<HashMap<String, serde_json::Value>>(1);
        self.prompt_reply_tx = Some(reply_tx);

        std::thread::spawn(move || {
            use crate::api::executor::execute_node;
            // Pre-seed context with env vars so {VAR} placeholders in the node
            // that are already in the env file don't trigger prompts.
            let context = crate::api::variables::load_env_context();
            let tx_prompt = tx.clone();
            let on_prompt = move |node_id: &str, inputs: &[crate::api::types::PromptInput]| {
                tx_prompt.send(LiveEvent::NeedInput {
                    node_id: node_id.to_string(),
                    inputs: inputs.to_vec(),
                }).ok();
                reply_rx.recv().unwrap_or_default()
            };
            let step = execute_node(&node, &context, &base_url, Some(&on_prompt));
            let passed = step.passed;
            // Wrap single-node result in a FlowRunResult so it persists and shows in last_run
            let result = crate::api::types::FlowRunResult {
                run_id: format!("{}", chrono::Utc::now().timestamp_millis()),
                flow_id: format!("node_{}", node.id),
                flow_name: node.name.clone(),
                started_at: chrono::Utc::now(),
                finished_at: chrono::Utc::now(),
                steps: vec![step.clone()],
                passed,
                base_url,
                final_context: HashMap::new(),
            };
            crate::api::storage::save_run_result(&result).ok();
            tx.send(LiveEvent::FlowResult(result)).ok();
            tx.send(LiveEvent::Step(step)).ok();
            tx.send(LiveEvent::Done { passed }).ok();
        });

        self.current_view = ApiView::Runner;
        self.runner_subview = RunnerSubview::Steps;
    }

    pub fn poll_run_events(&mut self) {
        loop {
            let event = match &self.run_rx {
                Some(rx) => match rx.try_recv() {
                    Ok(e) => e,
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        self.live_running = false;
                        self.flow_running = false;
                        self.run_rx = None;
                        break;
                    }
                },
                None => break,
            };

            match event {
                LiveEvent::Step(step) => {
                    // Cap live_steps to prevent unbounded memory growth
                    const MAX_LIVE_STEPS: usize = 1000;
                    if self.live_steps.len() >= MAX_LIVE_STEPS {
                        self.live_steps.remove(0);
                    }
                    self.live_steps.push(step);
                }
                LiveEvent::FlowResult(result) => {
                    // Update last_run immediately with the just-completed run
                    self.last_run = Some(result);
                }
                LiveEvent::Done { passed } => {
                    self.live_running = false;
                    self.flow_running = false;
                    self.run_rx = None;
                    self.prompt_reply_tx = None;
                    self.notify(if passed { "Flow passed ✔" } else { "Flow failed ✘" });
                    self.refresh_data();
                    // Auto-save report if config flags are set
                    if let Some(run) = &self.last_run {
                        let fmt = match (self.config_output_markdown, self.config_output_pdf) {
                            (true, true) => Some("both"),
                            (true, false) => Some("markdown"),
                            (false, true) => Some("pdf"),
                            _ => None,
                        };
                        if let Some(f) = fmt {
                            crate::api::commands::flow::save_run_report_pub(run, f);
                        }
                    }
                    break;
                }
                LiveEvent::Error(e) => {
                    self.live_running = false;
                    self.flow_running = false;
                    self.run_error = Some(e);
                    self.run_rx = None;
                    self.prompt_reply_tx = None;
                    break;
                }
                LiveEvent::NeedInput { node_id, inputs } => {
                    self.prompt_modal = Some(PromptModal::new(node_id, inputs));
                    // Don't break — keep polling (the run is paused waiting for reply)
                }
            }
        }
    }

    // ── Run comparison (diff view) ────────────────────────────────────────

    pub fn load_comparison_run(&mut self) {
        if let Some(flow) = self.active_flow() {
            let runs = storage::load_recent_runs(&flow.id, 2);
            self.last_run = runs.get(0).cloned();
            self.compare_run = runs.get(1).cloned();
        }
    }
}
