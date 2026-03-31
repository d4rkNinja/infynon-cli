use std::collections::HashMap;
use std::sync::mpsc;

use crate::api::storage;
use crate::api::types::{FlowRunResult, SecurityProbeResult};

use super::types::*;

// ── App ───────────────────────────────────────────────────────────────────────

pub struct ApiApp {
    pub current_view: ApiView,
    pub should_quit: bool,

    // ── Data ──────────────────────────────────────────────────────────────
    pub flows: Vec<crate::api::types::Flow>,
    pub nodes: HashMap<String, crate::api::types::Node>,
    pub active_flow_idx: usize,   // index into self.flows
    pub last_run: Option<FlowRunResult>,
    pub compare_run: Option<FlowRunResult>,
    pub security_probes: Vec<SecurityProbeResult>,
    /// Last-run pass/fail status per flow id — cached to avoid per-frame disk reads.
    pub flow_run_statuses: HashMap<String, Option<bool>>,

    // ── Live execution feed ────────────────────────────────────────────────
    pub live_steps: Vec<crate::api::types::StepResult>,
    pub live_running: bool,
    pub flow_running: bool,
    pub run_rx: Option<mpsc::Receiver<LiveEvent>>,
    pub run_tx: Option<mpsc::SyncSender<LiveEvent>>,
    pub run_error: Option<String>,

    // ── Navigation ────────────────────────────────────────────────────────
    pub scroll_offset: usize,
    pub selected_index: usize,   // selected node in graph / row in list views

    // ── Graph state ───────────────────────────────────────────────────────
    pub graph_layout: Vec<GraphNode>,
    pub graph_selected_id: Option<String>,
    pub attach_mode: AttachMode,
    pub attach_input: String,    // typed node ID during attach
    pub detail_panel: Option<NodeDetailPanel>,

    // ── Search ────────────────────────────────────────────────────────────
    pub search_active: bool,
    pub search_input: String,

    // ── Notifications ─────────────────────────────────────────────────────
    pub notification: Option<(String, std::time::Instant)>,

    // ── Help overlay ──────────────────────────────────────────────────────
    pub show_help: bool,

    // ── Config/Settings ────────────────────────────────────────────────────
    pub config_output_markdown: bool,
    pub config_output_pdf: bool,

    // ── Nodes view state ───────────────────────────────────────────────────
    pub nodes_filter: NodeFilter,

    // ── Runner sub-view state ──────────────────────────────────────────────
    pub runner_subview: RunnerSubview,

    // ── Flows vertical pipeline state ───────────────────────────────────────
    pub pipeline_selected: usize,

    // ── Prompt input modal ────────────────────────────────────────────────
    pub prompt_modal: Option<PromptModal>,
    pub prompt_reply_tx: Option<mpsc::SyncSender<HashMap<String, serde_json::Value>>>,

    // ── Body editor modal ─────────────────────────────────────────────────
    pub body_editor: Option<BodyEditor>,

    // ── Step detail modal (Live Execution tab) ────────────────────────────
    pub live_selected_step: usize,
    pub step_detail: Option<StepDetailModal>,

    // ── Node field editor modal (Node Library tab) ────────────────────────
    pub node_field_editor: Option<NodeFieldEditor>,

    // ── Env/Context tab (tab 6) ───────────────────────────────────────────
    pub env_selected: usize,
    pub env_reveal: bool,
    pub env_edit: Option<EnvEditState>,
}

pub struct EnvEditState {
    pub original_key: String, // empty = new entry
    pub key_input: String,
    pub value_input: String,
    pub editing_key: bool,    // true when adding new entry and cursor is in key field
}

impl EnvEditState {
    pub fn new_entry() -> Self {
        Self { original_key: String::new(), key_input: String::new(), value_input: String::new(), editing_key: true }
    }
    pub fn edit_existing(key: &str, value: &str) -> Self {
        Self { original_key: key.to_string(), key_input: key.to_string(), value_input: value.to_string(), editing_key: false }
    }
    pub fn is_new(&self) -> bool { self.original_key.is_empty() }
}

impl ApiApp {
    pub fn new(initial_flow_id: Option<&str>) -> Self {
        let flows = storage::list_flows();
        let nodes = storage::load_nodes_map();

        // Select initial flow
        let active_flow_idx = if let Some(fid) = initial_flow_id {
            flows.iter().position(|f| f.id == fid).unwrap_or(0)
        } else {
            0
        };

        // Load last run result for the active flow
        let last_run = flows.get(active_flow_idx)
            .and_then(|f| storage::load_recent_runs(&f.id, 1).into_iter().next());

        // Cache pass/fail status for every flow (avoids N disk reads per render frame)
        let flow_run_statuses: HashMap<String, Option<bool>> = flows.iter()
            .map(|f| {
                let status = storage::load_recent_runs(&f.id, 1).into_iter().next().map(|r| r.passed);
                (f.id.clone(), status)
            })
            .collect();

        // Build initial graph layout
        let graph_layout = if let Some(flow) = flows.get(active_flow_idx) {
            super::runner::compute_graph_layout(flow)
        } else {
            vec![]
        };

        let initial_selected = graph_layout.first().map(|g| g.node_id.clone());

        let initial_view = if flows.is_empty() {
            ApiView::Nodes
        } else {
            ApiView::Dashboard
        };

        Self {
            current_view: initial_view,
            should_quit: false,
            flows,
            nodes,
            active_flow_idx,
            last_run,
            compare_run: None,
            security_probes: vec![],
            flow_run_statuses,
            live_steps: vec![],
            live_running: false,
            flow_running: false,
            run_rx: None,
            run_tx: None,
            run_error: None,
            scroll_offset: 0,
            selected_index: 0,
            graph_layout,
            graph_selected_id: initial_selected,
            attach_mode: AttachMode::Idle,
            attach_input: String::new(),
            detail_panel: None,
            search_active: false,
            search_input: String::new(),
            notification: None,
            show_help: false,
            config_output_markdown: false,
            config_output_pdf: false,
            nodes_filter: NodeFilter::All,
            runner_subview: RunnerSubview::Steps,
            pipeline_selected: 0,
            prompt_modal: None,
            prompt_reply_tx: None,
            body_editor: None,
            live_selected_step: 0,
            step_detail: None,
            node_field_editor: None,
            env_selected: 0,
            env_reveal: false,
            env_edit: None,
        }
    }

    // ── Data helpers ──────────────────────────────────────────────────────

    pub fn active_flow(&self) -> Option<&crate::api::types::Flow> {
        self.flows.get(self.active_flow_idx)
    }

    pub fn refresh_data(&mut self) {
        self.flows = storage::list_flows();
        self.nodes = storage::load_nodes_map();
        // Refresh per-flow run status cache
        self.flow_run_statuses = self.flows.iter()
            .map(|f| {
                let status = storage::load_recent_runs(&f.id, 1).into_iter().next().map(|r| r.passed);
                (f.id.clone(), status)
            })
            .collect();
        // Grab layout + last run before borrowing self mutably
        let (layout, last_run) = if let Some(flow) = self.flows.get(self.active_flow_idx) {
            let layout = super::runner::compute_graph_layout(flow);
            let run = storage::load_recent_runs(&flow.id, 1).into_iter().next();
            (layout, run)
        } else {
            (vec![], None)
        };
        self.graph_layout = layout;
        self.last_run = last_run;
        self.notify("Refreshed");
    }

    pub fn switch_flow(&mut self, idx: usize) {
        if idx < self.flows.len() {
            self.active_flow_idx = idx;
            if let Some(flow) = self.flows.get(idx) {
                self.graph_layout = super::runner::compute_graph_layout(flow);
                self.last_run = storage::load_recent_runs(&flow.id, 1).into_iter().next();
                self.graph_selected_id = self.graph_layout.first().map(|g| g.node_id.clone());
            }
            self.scroll_offset = 0;
            self.selected_index = 0;
        }
    }

    // ── Notifications ─────────────────────────────────────────────────────

    pub fn notify(&mut self, msg: &str) {
        self.notification = Some((msg.to_string(), std::time::Instant::now()));
    }

    pub fn active_notification(&self) -> Option<&str> {
        if let Some((ref msg, when)) = self.notification {
            if when.elapsed().as_secs() < 4 {
                return Some(msg);
            }
        }
        None
    }

    // ── Graph navigation ──────────────────────────────────────────────────

    pub fn graph_node_count(&self) -> usize {
        self.graph_layout.len()
    }

    pub fn selected_graph_node(&self) -> Option<&GraphNode> {
        self.graph_selected_id.as_ref().and_then(|id| {
            self.graph_layout.iter().find(|g| &g.node_id == id)
        })
    }

    pub fn move_graph_selection(&mut self, direction: GraphDirection) {
        let current = match self.selected_graph_node() {
            Some(g) => g.clone(),
            None => {
                if let Some(first) = self.graph_layout.first() {
                    self.graph_selected_id = Some(first.node_id.clone());
                }
                return;
            }
        };

        let new_id = match direction {
            GraphDirection::Down => {
                // Find a node in the next layer
                self.graph_layout.iter()
                    .filter(|g| g.layer == current.layer + 1)
                    .min_by_key(|g| (g.col as i64 - current.col as i64).abs())
                    .map(|g| g.node_id.clone())
            }
            GraphDirection::Up => {
                if current.layer == 0 { return; }
                self.graph_layout.iter()
                    .filter(|g| g.layer + 1 == current.layer)
                    .min_by_key(|g| (g.col as i64 - current.col as i64).abs())
                    .map(|g| g.node_id.clone())
            }
            GraphDirection::Left => {
                self.graph_layout.iter()
                    .filter(|g| g.layer == current.layer && g.col < current.col)
                    .max_by_key(|g| g.col)
                    .map(|g| g.node_id.clone())
            }
            GraphDirection::Right => {
                self.graph_layout.iter()
                    .filter(|g| g.layer == current.layer && g.col > current.col)
                    .min_by_key(|g| g.col)
                    .map(|g| g.node_id.clone())
            }
        };

        if let Some(id) = new_id {
            self.graph_selected_id = Some(id);
        }
    }

    // ── Attach mode ───────────────────────────────────────────────────────

    pub fn start_attach(&mut self) {
        if let Some(from_id) = self.graph_selected_id.clone() {
            self.attach_mode = AttachMode::SelectingTarget { from_node: from_id };
            self.attach_input.clear();
        }
    }

    pub fn confirm_attach(&mut self) {
        let to_node = self.attach_input.trim().to_string();
        if to_node.is_empty() {
            self.attach_mode = AttachMode::Idle;
            return;
        }

        if let AttachMode::SelectingTarget { from_node } = &self.attach_mode.clone() {
            if !self.nodes.contains_key(&to_node) && !self.attach_input.starts_with('!') {
                self.notify(&format!("Node '{}' not found in library", to_node));
                return;
            }

            let from = from_node.clone();
            // Perform the attach
            // We can't call the full cmd here (it uses stdout), so do it inline
            self.do_attach(&from, &to_node);
            self.attach_mode = AttachMode::Idle;
            self.attach_input.clear();
            self.refresh_data();
        }
    }

    fn do_attach(&mut self, from: &str, to: &str) {
        let from_node = self.nodes.get(from).cloned();
        let to_node = self.nodes.get(to).cloned();

        let carry = if let (Some(fn_), Some(tn)) = (from_node, to_node) {
            crate::api::ai::infer_carry(&fn_, &tn)
        } else {
            vec![]
        };

        let edge = crate::api::types::Edge {
            from: from.to_string(),
            to: to.to_string(),
            carry,
            condition: None,
        };

        // Update all flows containing `from`
        let from_str = from.to_string();
        for flow in &mut self.flows {
            if flow.all_node_ids().contains(&from_str) {
                let already = flow.edges.iter().any(|e| e.from == from && e.to == to);
                if !already {
                    flow.edges.push(edge.clone());
                    storage::save_flow(flow).ok();
                }
            }
        }

        self.notify(&format!("Attached {} → {}", from, to));
    }

    // ── Detail panel ──────────────────────────────────────────────────────

    pub fn toggle_detail_panel(&mut self) {
        if self.detail_panel.is_some() {
            self.detail_panel = None;
        } else if let Some(id) = self.graph_selected_id.clone() {
            self.detail_panel = Some(NodeDetailPanel { node_id: id, scroll: 0 });
        }
    }
}
