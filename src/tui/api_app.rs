use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent};

use crate::api::storage;
use crate::api::types::{Flow, FlowRunResult, Node, SecurityProbeResult};

// ── Views ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiView {
    Overview,
    FlowGraph,
    LiveExecution,
    LatencyProfiler,
    SecurityProbes,
    CoverageMap,
    StateInspector,
    RunDiff,
    NodeLibrary,
}

impl ApiView {
    pub fn label(&self) -> &str {
        match self {
            ApiView::Overview       => "Overview",
            ApiView::FlowGraph      => "Flow Graph",
            ApiView::LiveExecution  => "Live",
            ApiView::LatencyProfiler => "Latency",
            ApiView::SecurityProbes  => "Security",
            ApiView::CoverageMap     => "Coverage",
            ApiView::StateInspector  => "State",
            ApiView::RunDiff         => "Diff",
            ApiView::NodeLibrary     => "Nodes",
        }
    }

    pub fn key(&self) -> char {
        match self {
            ApiView::Overview        => '1',
            ApiView::FlowGraph       => '2',
            ApiView::LiveExecution   => '3',
            ApiView::LatencyProfiler => '4',
            ApiView::SecurityProbes  => '5',
            ApiView::CoverageMap     => '6',
            ApiView::StateInspector  => '7',
            ApiView::RunDiff         => '8',
            ApiView::NodeLibrary     => '9',
        }
    }

    pub fn all() -> &'static [ApiView] {
        &[
            ApiView::Overview,
            ApiView::FlowGraph,
            ApiView::LiveExecution,
            ApiView::LatencyProfiler,
            ApiView::SecurityProbes,
            ApiView::CoverageMap,
            ApiView::StateInspector,
            ApiView::RunDiff,
            ApiView::NodeLibrary,
        ]
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

    // ── Run comparison (diff view) ────────────────────────────────────────

    pub fn load_comparison_run(&mut self) {
        if let Some(flow) = self.active_flow() {
            let runs = storage::load_recent_runs(&flow.id, 2);
            self.last_run = runs.get(0).cloned();
            self.compare_run = runs.get(1).cloned();
        }
    }

    // ── Key handler ───────────────────────────────────────────────────────

    pub fn handle_key(&mut self, key: KeyEvent) {
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

        // Global keys
        match key.code {
            KeyCode::Char('q') => { self.should_quit = true; return; }
            KeyCode::Char('1') => { self.current_view = ApiView::Overview; self.reset_scroll(); return; }
            KeyCode::Char('2') => { self.current_view = ApiView::FlowGraph; self.reset_scroll(); return; }
            KeyCode::Char('3') => { self.current_view = ApiView::LiveExecution; self.reset_scroll(); return; }
            KeyCode::Char('4') => { self.current_view = ApiView::LatencyProfiler; self.reset_scroll(); return; }
            KeyCode::Char('5') => { self.current_view = ApiView::SecurityProbes; self.reset_scroll(); return; }
            KeyCode::Char('6') => { self.current_view = ApiView::CoverageMap; self.reset_scroll(); return; }
            KeyCode::Char('7') => { self.current_view = ApiView::StateInspector; self.reset_scroll(); return; }
            KeyCode::Char('8') => { self.current_view = ApiView::RunDiff; self.reset_scroll(); return; }
            KeyCode::Char('9') => { self.current_view = ApiView::NodeLibrary; self.reset_scroll(); return; }
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
            ApiView::NodeLibrary => self.handle_list_key(key, self.nodes.len()),
            ApiView::Overview => self.handle_list_key(key, self.flows.len()),
            ApiView::RunDiff => {
                match key.code {
                    KeyCode::Char('d') => self.load_comparison_run(),
                    _ => self.handle_scroll_key(key),
                }
            }
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
    let mut queue = vec![(flow.entry.clone(), 0usize)];
    let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();

    while let Some((node_id, layer)) = queue.first().cloned() {
        queue.remove(0);
        if visited.contains(&node_id) { continue; }
        visited.insert(node_id.clone());
        layers.entry(node_id.clone()).or_insert(layer);

        for edge in flow.successors(&node_id) {
            if !visited.contains(&edge.to) {
                queue.push((edge.to.clone(), layer + 1));
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
