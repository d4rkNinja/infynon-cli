use std::collections::HashMap;
use std::sync::mpsc;

use crossterm::event::{KeyCode, KeyEvent};

use crate::api::storage;
use crate::api::types::{Flow, FlowRunResult, Node, PromptInput, SecurityProbeResult, StepResult};

// ── Views ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiView {
    Overview,
    FlowGraph,
    LiveExecution,
    LatencyProfiler,
    SecurityProbes,
    EnvContext,
    StateInspector,
    RunDiff,
    NodeLibrary,
    Config,
}

impl ApiView {
    pub fn label(&self) -> &str {
        match self {
            ApiView::Overview        => "Overview",
            ApiView::FlowGraph       => "Flow Graph",
            ApiView::LiveExecution   => "Live",
            ApiView::LatencyProfiler => "Latency",
            ApiView::SecurityProbes  => "Security",
            ApiView::EnvContext      => "Env / Ctx",
            ApiView::StateInspector  => "State",
            ApiView::RunDiff         => "Diff",
            ApiView::NodeLibrary     => "Node Lib",
            ApiView::Config          => "Config",
        }
    }

    pub fn key(&self) -> char {
        match self {
            ApiView::Overview        => '1',
            ApiView::FlowGraph       => '2',
            ApiView::LiveExecution   => '3',
            ApiView::LatencyProfiler => '4',
            ApiView::SecurityProbes  => '5',
            ApiView::EnvContext      => '6',
            ApiView::StateInspector  => '7',
            ApiView::RunDiff         => '8',
            ApiView::NodeLibrary     => '9',
            ApiView::Config          => '0',
        }
    }

    pub fn all() -> &'static [ApiView] {
        &[
            ApiView::Overview,
            ApiView::FlowGraph,
            ApiView::LiveExecution,
            ApiView::LatencyProfiler,
            ApiView::SecurityProbes,
            ApiView::EnvContext,
            ApiView::StateInspector,
            ApiView::RunDiff,
            ApiView::NodeLibrary,
            ApiView::Config,
        ]
    }
}

// ── Live execution events ─────────────────────────────────────────────────────

pub enum LiveEvent {
    Step(StepResult),
    Done { passed: bool },
    Error(String),
    NeedInput { node_id: String, inputs: Vec<PromptInput> },
}

// ── Step detail modal ─────────────────────────────────────────────────────────

pub struct StepDetailModal {
    pub step: crate::api::types::StepResult,
    pub scroll: usize,
}

// ── Node field editor ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum NodeField {
    Name,
    Path,
    Method,
    Description,
}

pub struct NodeFieldEditor {
    pub node_id: String,
    pub field: NodeField,
    pub input: String,
}

// ── Body editor ───────────────────────────────────────────────────────────────

pub struct BodyEditor {
    pub node_id: String,
    pub lines: Vec<String>,   // content split by newlines
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub scroll_top: usize,    // vertical scroll for long bodies
}

impl BodyEditor {
    pub fn new(node_id: String, body_json: Option<&str>) -> Self {
        let content = body_json
            .map(|b| {
                // Pretty-print if valid JSON, else keep raw
                serde_json::from_str::<serde_json::Value>(b)
                    .map(|v| serde_json::to_string_pretty(&v).unwrap_or_else(|_| b.to_string()))
                    .unwrap_or_else(|_| b.to_string())
            })
            .unwrap_or_else(|| "{}".to_string());
        let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
        let lines = if lines.is_empty() { vec![String::new()] } else { lines };
        Self { node_id, lines, cursor_row: 0, cursor_col: 0, scroll_top: 0 }
    }

    pub fn to_string(&self) -> String {
        self.lines.join("\n")
    }

    // Current line length
    pub fn current_line_len(&self) -> usize {
        self.lines.get(self.cursor_row).map(|l| l.len()).unwrap_or(0)
    }
}

// ── Prompt modal ──────────────────────────────────────────────────────────────

pub struct PromptModal {
    pub node_id: String,
    pub inputs: Vec<PromptInput>,
    pub values: Vec<String>,       // one entry per input, same order
    pub current_field: usize,      // which field is being edited
}

impl PromptModal {
    pub fn new(node_id: String, inputs: Vec<PromptInput>) -> Self {
        let len = inputs.len();
        Self { node_id, inputs, values: vec![String::new(); len], current_field: 0 }
    }
}

// ── Attach mode state ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum AttachMode {
    /// No attach in progress
    Idle,
    /// User pressed 'a' on a node — waiting for target selection
    SelectingTarget { from_node: String },
    /// User typed a target — show confirmation
    Confirming { from_node: String, to_node: String },
}

// ── Node detail panel ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct NodeDetailPanel {
    pub node_id: String,
    pub scroll: usize,
}

// ── Graph layout ──────────────────────────────────────────────────────────────

/// Computed layout position for a node in the graph view.
#[derive(Debug, Clone)]
pub struct GraphNode {
    pub node_id: String,
    pub layer: usize,
    pub col: usize,
}

// ── App ───────────────────────────────────────────────────────────────────────

pub struct ApiApp {
    pub current_view: ApiView,
    pub should_quit: bool,

    // ── Data ──────────────────────────────────────────────────────────────
    pub flows: Vec<Flow>,
    pub nodes: HashMap<String, Node>,
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

    // ── Config tab (tab 0) ────────────────────────────────────────────────
    pub config_output_markdown: bool,
    pub config_output_pdf: bool,

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
            compute_graph_layout(flow)
        } else {
            vec![]
        };

        let initial_selected = graph_layout.first().map(|g| g.node_id.clone());

        // Default to FlowGraph view if a flow is loaded, else Overview
        let initial_view = if flows.is_empty() {
            ApiView::NodeLibrary
        } else {
            ApiView::FlowGraph
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

    pub fn active_flow(&self) -> Option<&Flow> {
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
            let layout = compute_graph_layout(flow);
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
                self.graph_layout = compute_graph_layout(flow);
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
            use crate::api::commands::attach::cmd_attach;
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

    // ── Flow runner ───────────────────────────────────────────────────────

    pub fn start_flow_run(&mut self) {
        let flow = match self.flows.get(self.active_flow_idx) {
            Some(f) => f.clone(),
            None => return,
        };
        let nodes = self.nodes.clone();
        let base_url = flow.base_url.clone().unwrap_or_else(|| "http://localhost:3000".to_string());

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
            let result = execute_flow(&flow, &nodes, FlowExecuteOptions {
                base_url,
                initial_context: std::collections::HashMap::new(),
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
            tx2.send(LiveEvent::Done { passed: result.passed }).ok();
        });

        self.current_view = ApiView::LiveExecution;
    }

    pub fn start_node_run(&mut self, node_id: &str) {
        let node = match self.nodes.get(node_id).cloned() {
            Some(n) => n,
            None => {
                self.notify(&format!("Node '{}' not found", node_id));
                return;
            }
        };
        let base_url = self.active_flow()
            .and_then(|f| f.base_url.clone())
            .unwrap_or_else(|| "http://localhost:3000".to_string());

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
            use std::collections::HashMap;
            let context: HashMap<String, serde_json::Value> = HashMap::new();
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
            tx.send(LiveEvent::Step(step)).ok();
            tx.send(LiveEvent::Done { passed }).ok();
        });

        self.current_view = ApiView::LiveExecution;
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
                    self.live_steps.push(step);
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

    // ── Key handler ───────────────────────────────────────────────────────

    pub fn handle_prompt_modal_key(&mut self, key: KeyEvent) {
        let modal = match self.prompt_modal.as_mut() {
            Some(m) => m,
            None => return,
        };
        match key.code {
            KeyCode::Char(c) => {
                let idx = modal.current_field;
                modal.values[idx].push(c);
            }
            KeyCode::Backspace => {
                let idx = modal.current_field;
                modal.values[idx].pop();
            }
            KeyCode::Tab | KeyCode::Down => {
                let next = (modal.current_field + 1) % modal.inputs.len();
                modal.current_field = next;
            }
            KeyCode::Up => {
                if modal.current_field > 0 {
                    modal.current_field -= 1;
                } else {
                    modal.current_field = modal.inputs.len().saturating_sub(1);
                }
            }
            KeyCode::Enter => {
                let last = modal.inputs.len().saturating_sub(1);
                if modal.current_field < last {
                    modal.current_field += 1;
                } else {
                    // Submit
                    let mut map: HashMap<String, serde_json::Value> = HashMap::new();
                    for (i, pi) in modal.inputs.iter().enumerate() {
                        let raw = modal.values.get(i).cloned().unwrap_or_default();
                        let val = if raw.is_empty() {
                            pi.default.clone().unwrap_or_default()
                        } else {
                            raw
                        };
                        map.insert(pi.var.clone(), serde_json::Value::String(val));
                    }
                    if let Some(tx) = self.prompt_reply_tx.take() {
                        tx.send(map).ok();
                    }
                    self.prompt_modal = None;
                }
            }
            KeyCode::Esc => {
                // Cancel — send empty map so the background thread unblocks
                if let Some(tx) = self.prompt_reply_tx.take() {
                    tx.send(HashMap::new()).ok();
                }
                self.prompt_modal = None;
            }
            _ => {}
        }
    }

    pub fn open_body_editor(&mut self) {
        let mut node_ids: Vec<String> = self.nodes.keys().cloned().collect();
        node_ids.sort();
        let node_id = match node_ids.get(self.selected_index.min(node_ids.len().saturating_sub(1))) {
            Some(id) => id.clone(),
            None => return,
        };
        let body = self.nodes.get(&node_id).and_then(|n| n.body_json.as_deref());
        self.body_editor = Some(BodyEditor::new(node_id, body));
    }

    pub fn handle_body_editor_key(&mut self, key: KeyEvent) {
        use crossterm::event::KeyModifiers;
        let editor = match self.body_editor.as_mut() {
            Some(e) => e,
            None => return,
        };

        match key.code {
            KeyCode::Esc => {
                self.body_editor = None;
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Save body to node
                let content = editor.to_string();
                let node_id = editor.node_id.clone();
                self.body_editor = None;

                if let Some(node) = self.nodes.get_mut(&node_id) {
                    // Validate JSON (compact it for storage)
                    let body_val = serde_json::from_str::<serde_json::Value>(&content).ok();
                    node.body_json = Some(body_val
                        .map(|v| serde_json::to_string(&v).unwrap_or(content.clone()))
                        .unwrap_or(content));
                    let node_clone = node.clone();
                    match crate::api::storage::save_node(&node_clone) {
                        Ok(_) => self.notify("Body saved"),
                        Err(e) => self.notify(&format!("Save failed: {}", e)),
                    }
                }
            }
            KeyCode::Enter => {
                // Split current line at cursor
                let rest = {
                    let line = editor.lines.get_mut(editor.cursor_row).unwrap();
                    let rest = line[editor.cursor_col..].to_string();
                    line.truncate(editor.cursor_col);
                    rest
                };
                editor.cursor_row += 1;
                editor.cursor_col = 0;
                editor.lines.insert(editor.cursor_row, rest);
            }
            KeyCode::Backspace => {
                if editor.cursor_col > 0 {
                    let line = editor.lines.get_mut(editor.cursor_row).unwrap();
                    editor.cursor_col -= 1;
                    line.remove(editor.cursor_col);
                } else if editor.cursor_row > 0 {
                    // Merge with previous line
                    let current = editor.lines.remove(editor.cursor_row);
                    editor.cursor_row -= 1;
                    editor.cursor_col = editor.lines[editor.cursor_row].len();
                    editor.lines[editor.cursor_row].push_str(&current);
                }
            }
            KeyCode::Delete => {
                let line_len = editor.current_line_len();
                if editor.cursor_col < line_len {
                    editor.lines[editor.cursor_row].remove(editor.cursor_col);
                } else if editor.cursor_row + 1 < editor.lines.len() {
                    // Merge next line into current
                    let next = editor.lines.remove(editor.cursor_row + 1);
                    editor.lines[editor.cursor_row].push_str(&next);
                }
            }
            KeyCode::Left => {
                if editor.cursor_col > 0 {
                    editor.cursor_col -= 1;
                } else if editor.cursor_row > 0 {
                    editor.cursor_row -= 1;
                    editor.cursor_col = editor.current_line_len();
                }
            }
            KeyCode::Right => {
                let len = editor.current_line_len();
                if editor.cursor_col < len {
                    editor.cursor_col += 1;
                } else if editor.cursor_row + 1 < editor.lines.len() {
                    editor.cursor_row += 1;
                    editor.cursor_col = 0;
                }
            }
            KeyCode::Up => {
                if editor.cursor_row > 0 {
                    editor.cursor_row -= 1;
                    editor.cursor_col = editor.cursor_col.min(editor.current_line_len());
                    // Adjust scroll
                    if editor.cursor_row < editor.scroll_top {
                        editor.scroll_top = editor.cursor_row;
                    }
                }
            }
            KeyCode::Down => {
                if editor.cursor_row + 1 < editor.lines.len() {
                    editor.cursor_row += 1;
                    editor.cursor_col = editor.cursor_col.min(editor.current_line_len());
                }
            }
            KeyCode::Home => {
                editor.cursor_col = 0;
            }
            KeyCode::End => {
                editor.cursor_col = editor.current_line_len();
            }
            KeyCode::Char(c) => {
                // Insert character at cursor
                if editor.lines.is_empty() {
                    editor.lines.push(String::new());
                }
                editor.lines[editor.cursor_row].insert(editor.cursor_col, c);
                editor.cursor_col += 1;
            }
            KeyCode::Tab => {
                // Insert 2 spaces
                for _ in 0..2 {
                    editor.lines[editor.cursor_row].insert(editor.cursor_col, ' ');
                    editor.cursor_col += 1;
                }
            }
            _ => {}
        }

        // Adjust scroll_top so cursor stays visible (assume ~20 visible lines)
        let visible_lines = 20usize;
        let editor = self.body_editor.as_mut().unwrap();
        if editor.cursor_row >= editor.scroll_top + visible_lines {
            editor.scroll_top = editor.cursor_row.saturating_sub(visible_lines - 1);
        }
        if editor.cursor_row < editor.scroll_top {
            editor.scroll_top = editor.cursor_row;
        }
    }

    fn handle_live_key(&mut self, key: KeyEvent) {
        // If modal is open, handle scroll/close
        if let Some(ref mut modal) = self.step_detail {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => { self.step_detail = None; }
                KeyCode::Up   => { if modal.scroll > 0 { modal.scroll -= 1; } }
                KeyCode::Down => { modal.scroll += 1; }
                KeyCode::Home => { modal.scroll = 0; }
                _ => {}
            }
            return;
        }

        let step_count = if self.live_steps.is_empty() {
            self.last_run.as_ref().map(|r| r.steps.len()).unwrap_or(0)
        } else {
            self.live_steps.len()
        };

        match key.code {
            KeyCode::Up => {
                if self.live_selected_step > 0 { self.live_selected_step -= 1; }
            }
            KeyCode::Down => {
                if self.live_selected_step + 1 < step_count { self.live_selected_step += 1; }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                let steps: Vec<_> = if self.live_steps.is_empty() {
                    self.last_run.as_ref().map(|r| r.steps.clone()).unwrap_or_default()
                } else {
                    self.live_steps.clone()
                };
                if let Some(step) = steps.get(self.live_selected_step) {
                    self.step_detail = Some(StepDetailModal { step: step.clone(), scroll: 0 });
                }
            }
            _ => {}
        }
    }

    pub fn open_node_field_editor(&mut self, field: NodeField) {
        let mut node_ids: Vec<String> = self.nodes.keys().cloned().collect();
        node_ids.sort();
        let node_id = match node_ids.get(self.selected_index.min(node_ids.len().saturating_sub(1))) {
            Some(id) => id.clone(),
            None => return,
        };
        let initial = match &field {
            NodeField::Name => self.nodes.get(&node_id).map(|n| n.name.clone()).unwrap_or_default(),
            NodeField::Path => self.nodes.get(&node_id).map(|n| n.path.clone()).unwrap_or_default(),
            NodeField::Description => self.nodes.get(&node_id).and_then(|n| n.description.clone()).unwrap_or_default(),
            NodeField::Method => self.nodes.get(&node_id).map(|n| n.method.clone()).unwrap_or_default(),
        };
        self.node_field_editor = Some(NodeFieldEditor { node_id, field, input: initial });
    }

    pub fn cycle_node_method(&mut self) {
        let methods = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD"];
        let mut node_ids: Vec<String> = self.nodes.keys().cloned().collect();
        node_ids.sort();
        let node_id = match node_ids.get(self.selected_index.min(node_ids.len().saturating_sub(1))) {
            Some(id) => id.clone(),
            None => return,
        };
        if let Some(node) = self.nodes.get_mut(&node_id) {
            let curr_idx = methods.iter().position(|&m| m == node.method.as_str()).unwrap_or(0);
            node.method = methods[(curr_idx + 1) % methods.len()].to_string();
            let node_clone = node.clone();
            match crate::api::storage::save_node(&node_clone) {
                Ok(_) => self.notify(&format!("Method → {}", node_clone.method)),
                Err(e) => self.notify(&format!("Save failed: {}", e)),
            }
        }
    }

    pub fn handle_node_field_editor_key(&mut self, key: KeyEvent) {
        let editor = match self.node_field_editor.as_mut() {
            Some(e) => e,
            None => return,
        };
        match key.code {
            KeyCode::Esc => { self.node_field_editor = None; }
            KeyCode::Backspace => { editor.input.pop(); }
            KeyCode::Enter => {
                // Save
                let node_id = editor.node_id.clone();
                let field = editor.field.clone();
                let value = editor.input.clone();
                self.node_field_editor = None;

                if let Some(node) = self.nodes.get_mut(&node_id) {
                    match field {
                        NodeField::Name => node.name = value,
                        NodeField::Path => {
                            let path = if value.starts_with('/') { value } else { format!("/{}", value) };
                            node.path = path;
                        }
                        NodeField::Description => {
                            node.description = if value.is_empty() { None } else { Some(value) };
                        }
                        NodeField::Method => {} // handled by cycle
                    }
                    let node_clone = node.clone();
                    match crate::api::storage::save_node(&node_clone) {
                        Ok(_) => self.notify("Node saved"),
                        Err(e) => self.notify(&format!("Save failed: {}", e)),
                    }
                }
            }
            KeyCode::Char(c) => { editor.input.push(c); }
            _ => {}
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        // Prompt modal intercepts all keys
        if self.prompt_modal.is_some() {
            self.handle_prompt_modal_key(key);
            return;
        }

        // Step detail modal intercepts all keys (before body editor check)
        if self.step_detail.is_some() {
            self.handle_live_key(key);
            return;
        }

        // Body editor intercepts all keys
        if self.body_editor.is_some() {
            self.handle_body_editor_key(key);
            return;
        }

        // Node field editor intercepts all keys
        if self.node_field_editor.is_some() {
            self.handle_node_field_editor_key(key);
            return;
        }

        // Help overlay — any key closes it
        if self.show_help {
            self.show_help = false;
            return;
        }

        // Attach mode input
        if let AttachMode::SelectingTarget { .. } = &self.attach_mode {
            match key.code {
                KeyCode::Esc => {
                    self.attach_mode = AttachMode::Idle;
                    self.attach_input.clear();
                }
                KeyCode::Enter => self.confirm_attach(),
                KeyCode::Backspace => { self.attach_input.pop(); }
                KeyCode::Char(c) => { self.attach_input.push(c); }
                _ => {}
            }
            return;
        }

        // Search mode
        if self.search_active {
            match key.code {
                KeyCode::Esc | KeyCode::Enter => { self.search_active = false; }
                KeyCode::Backspace => { self.search_input.pop(); }
                KeyCode::Char(c) => { self.search_input.push(c); }
                _ => {}
            }
            return;
        }

        // Env editor — intercept before global digit keys
        if self.env_edit.is_some() {
            self.handle_env_key(key);
            return;
        }

        // Global keys
        match key.code {
            KeyCode::Char('q') => { self.should_quit = true; return; }
            KeyCode::Char('1') => { self.current_view = ApiView::Overview; self.reset_scroll(); return; }
            KeyCode::Char('2') => { self.current_view = ApiView::FlowGraph; self.reset_scroll(); return; }
            KeyCode::Char('3') => { self.current_view = ApiView::LiveExecution; self.reset_scroll(); return; }
            KeyCode::Char('4') => { self.current_view = ApiView::LatencyProfiler; self.reset_scroll(); return; }
            KeyCode::Char('5') => { self.current_view = ApiView::SecurityProbes; self.reset_scroll(); return; }
            KeyCode::Char('6') => { self.current_view = ApiView::EnvContext; self.reset_scroll(); return; }
            KeyCode::Char('7') => { self.current_view = ApiView::StateInspector; self.reset_scroll(); return; }
            KeyCode::Char('8') => { self.current_view = ApiView::RunDiff; self.reset_scroll(); return; }
            KeyCode::Char('9') => { self.current_view = ApiView::NodeLibrary; self.reset_scroll(); return; }
            KeyCode::Char('0') => { self.current_view = ApiView::Config; self.reset_scroll(); return; }
            KeyCode::Char('?') => { self.show_help = true; return; }
            KeyCode::Char('/') => { self.search_active = true; self.search_input.clear(); return; }
            KeyCode::Char('R') => { self.refresh_data(); return; }
            // Switch active flow with [ ]
            KeyCode::Char('[') => {
                if self.active_flow_idx > 0 { self.switch_flow(self.active_flow_idx - 1); }
                return;
            }
            KeyCode::Char(']') => {
                if self.active_flow_idx + 1 < self.flows.len() {
                    self.switch_flow(self.active_flow_idx + 1);
                }
                return;
            }
            _ => {}
        }

        // View-specific keys
        match self.current_view {
            ApiView::FlowGraph => self.handle_graph_key(key),
            ApiView::NodeLibrary => self.handle_node_library_key(key),
            ApiView::Overview => self.handle_overview_key(key),
            ApiView::LiveExecution => self.handle_live_key(key),
            ApiView::RunDiff => {
                match key.code {
                    KeyCode::Char('d') => self.load_comparison_run(),
                    _ => self.handle_scroll_key(key),
                }
            }
            ApiView::EnvContext => self.handle_env_key(key),
            ApiView::Config => self.handle_config_key(key),
            _ => self.handle_scroll_key(key),
        }
    }

    fn handle_graph_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up    => self.move_graph_selection(GraphDirection::Up),
            KeyCode::Down  => self.move_graph_selection(GraphDirection::Down),
            KeyCode::Left  => self.move_graph_selection(GraphDirection::Left),
            KeyCode::Right => self.move_graph_selection(GraphDirection::Right),
            KeyCode::Enter => self.toggle_detail_panel(),
            KeyCode::Char('a') => self.start_attach(),
            KeyCode::Char('d') => {
                // Detach selected → its successor (first one)
                if let Some(sel_id) = &self.graph_selected_id.clone() {
                    if let Some(flow) = self.flows.get_mut(self.active_flow_idx) {
                        if let Some(succ) = flow.successors(sel_id).first().map(|e| e.to.clone()) {
                            flow.edges.retain(|e| !(e.from == *sel_id && e.to == succ));
                            storage::save_flow(flow).ok();
                            self.notify(&format!("Detached {} → {}", sel_id, succ));
                            self.refresh_data();
                        }
                    }
                }
            }
            KeyCode::Char('x') => {
                // Chaos: open detail panel for chaos injection
                self.toggle_detail_panel();
                self.notify("Chaos inject: inspect the node, then re-run");
            }
            _ => {}
        }
    }

    fn handle_overview_key(&mut self, key: KeyEvent) {
        let max = self.flows.len();
        match key.code {
            KeyCode::Enter => {
                self.start_flow_run();
            }
            KeyCode::Char('a') => {
                // Run all flows sequentially (simplified: just run active for now)
                self.start_flow_run();
            }
            _ => self.handle_list_key(key, max),
        }
    }

    fn handle_node_library_key(&mut self, key: KeyEvent) {
        let max = self.nodes.len();
        match key.code {
            KeyCode::Char('r') | KeyCode::Enter => {
                // Get sorted node list and pick selected
                let mut node_ids: Vec<String> = self.nodes.keys().cloned().collect();
                node_ids.sort();
                if let Some(node_id) = node_ids.get(self.selected_index.min(node_ids.len().saturating_sub(1))) {
                    let node_id = node_id.clone();
                    self.start_node_run(&node_id);
                }
            }
            KeyCode::Char('b') => { self.open_body_editor(); }
            KeyCode::Char('n') => { self.open_node_field_editor(NodeField::Name); }
            KeyCode::Char('p') => { self.open_node_field_editor(NodeField::Path); }
            KeyCode::Char('d') => { self.open_node_field_editor(NodeField::Description); }
            KeyCode::Char('m') => { self.cycle_node_method(); }
            _ => self.handle_list_key(key, max),
        }
    }

    fn handle_config_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('m') => {
                self.config_output_markdown = !self.config_output_markdown;
                self.notify(if self.config_output_markdown { "Markdown output: ON" } else { "Markdown output: OFF" });
            }
            KeyCode::Char('p') => {
                self.config_output_pdf = !self.config_output_pdf;
                self.notify(if self.config_output_pdf { "PDF output: ON" } else { "PDF output: OFF" });
            }
            KeyCode::Char('R') => self.refresh_data(),
            _ => {}
        }
    }

    pub fn handle_env_key(&mut self, key: KeyEvent) {
        use crate::api::commands::env as env_cmd;

        // ── Edit mode ──────────────────────────────────────────────────────────
        if self.env_edit.is_some() {
            let action: Option<(String, String, String)> = {
                let edit = self.env_edit.as_mut().unwrap();
                match key.code {
                    KeyCode::Esc => {
                        self.env_edit = None;
                        return;
                    }
                    KeyCode::Tab => {
                        if edit.is_new() { edit.editing_key = !edit.editing_key; }
                        return;
                    }
                    KeyCode::Enter => {
                        if edit.is_new() && edit.editing_key {
                            if !edit.key_input.trim().is_empty() { edit.editing_key = false; }
                            return;
                        }
                        Some((edit.key_input.trim().to_string(), edit.value_input.clone(), edit.original_key.clone()))
                    }
                    KeyCode::Backspace => {
                        if edit.editing_key { edit.key_input.pop(); } else { edit.value_input.pop(); }
                        return;
                    }
                    KeyCode::Char(c) => {
                        if edit.editing_key { edit.key_input.push(c); } else { edit.value_input.push(c); }
                        return;
                    }
                    _ => return,
                }
            };
            if let Some((key_str, val_str, orig_key)) = action {
                self.env_edit = None;
                self.env_reveal = false;
                if !key_str.is_empty() {
                    if !orig_key.is_empty() && orig_key != key_str {
                        let _ = env_cmd::env_delete_key(&orig_key);
                    }
                    match env_cmd::env_upsert(&key_str, &val_str) {
                        Ok(_) => self.notify(&format!("Saved: {}", key_str)),
                        Err(e) => self.notify(&format!("Save failed: {}", e)),
                    }
                }
            }
            return;
        }

        // ── Browse mode ────────────────────────────────────────────────────────
        let entries = env_cmd::env_list();
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.env_selected > 0 { self.env_selected -= 1; }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.env_selected + 1 < entries.len() { self.env_selected += 1; }
            }
            KeyCode::Char('n') => {
                self.env_edit = Some(EnvEditState::new_entry());
                self.env_reveal = false;
            }
            KeyCode::Enter => {
                if let Some((k, v)) = entries.get(self.env_selected) {
                    self.env_edit = Some(EnvEditState::edit_existing(k, v));
                    self.env_reveal = false;
                }
            }
            KeyCode::Char('d') | KeyCode::Delete => {
                if let Some((k, _)) = entries.get(self.env_selected) {
                    let key_clone = k.clone();
                    match env_cmd::env_delete_key(&key_clone) {
                        Ok(true) => {
                            self.notify(&format!("Deleted: {}", key_clone));
                            let new_len = entries.len().saturating_sub(1);
                            self.env_selected = self.env_selected.min(new_len.saturating_sub(1));
                        }
                        Ok(false) => self.notify("Key not found"),
                        Err(e) => self.notify(&format!("Error: {}", e)),
                    }
                }
            }
            KeyCode::Char('v') => {
                self.env_reveal = !self.env_reveal;
                self.notify(if self.env_reveal { "Values revealed" } else { "Sensitive values hidden" });
            }
            _ => {}
        }
    }

    fn handle_list_key(&mut self, key: KeyEvent, max: usize) {
        match key.code {
            KeyCode::Up => {
                if self.selected_index > 0 { self.selected_index -= 1; }
            }
            KeyCode::Down => {
                if self.selected_index + 1 < max { self.selected_index += 1; }
            }
            _ => {}
        }
    }

    fn handle_scroll_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up   => { if self.scroll_offset > 0 { self.scroll_offset -= 1; } }
            KeyCode::Down => { self.scroll_offset += 1; }
            KeyCode::Home => { self.scroll_offset = 0; }
            _ => {}
        }
    }

    fn reset_scroll(&mut self) {
        self.scroll_offset = 0;
        self.selected_index = 0;
    }
}

// ── Graph layout algorithm ────────────────────────────────────────────────────

pub enum GraphDirection { Up, Down, Left, Right }

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
