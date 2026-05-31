use crate::api::types::{FlowRunResult, PromptInput, StepResult};

// ── Views ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiView {
    Dashboard,
    Nodes,
    Flows,
    Runner,
    Environment,
    Settings,
}

impl ApiView {
    pub fn label(&self) -> &str {
        match self {
            ApiView::Dashboard => "Dashboard",
            ApiView::Nodes => "Nodes",
            ApiView::Flows => "Flows",
            ApiView::Runner => "Runner",
            ApiView::Environment => "Env",
            ApiView::Settings => "Settings",
        }
    }

    pub fn key(&self) -> char {
        match self {
            ApiView::Dashboard => '1',
            ApiView::Nodes => '2',
            ApiView::Flows => '3',
            ApiView::Runner => '4',
            ApiView::Environment => '5',
            ApiView::Settings => '6',
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            ApiView::Dashboard => "\u{25C9}",   // ◉
            ApiView::Nodes => "\u{25CE}",       // ◎
            ApiView::Flows => "\u{25C8}",       // ◈
            ApiView::Runner => "\u{25B6}",      // ▶
            ApiView::Environment => "\u{2699}", // ⚙
            ApiView::Settings => "\u{2600}",    // ☀
        }
    }

    pub fn sublabel(&self) -> &str {
        match self {
            ApiView::Dashboard => "overview",
            ApiView::Nodes => "node lib",
            ApiView::Flows => "graph",
            ApiView::Runner => "execution",
            ApiView::Environment => "variables",
            ApiView::Settings => "config",
        }
    }

    pub fn all() -> &'static [ApiView] {
        &[
            ApiView::Dashboard,
            ApiView::Nodes,
            ApiView::Flows,
            ApiView::Runner,
            ApiView::Environment,
            ApiView::Settings,
        ]
    }
}

// ── Node filter ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum NodeFilter {
    All,
    ByMethod(String),
}

impl NodeFilter {
    pub fn label(&self) -> &str {
        match self {
            NodeFilter::All => "All",
            NodeFilter::ByMethod(m) => m,
        }
    }

    pub fn cycle(&self) -> Self {
        match self {
            NodeFilter::All => NodeFilter::ByMethod("GET".into()),
            NodeFilter::ByMethod(m) if m == "GET" => NodeFilter::ByMethod("POST".into()),
            NodeFilter::ByMethod(m) if m == "POST" => NodeFilter::ByMethod("PUT".into()),
            NodeFilter::ByMethod(m) if m == "PUT" => NodeFilter::ByMethod("PATCH".into()),
            NodeFilter::ByMethod(m) if m == "PATCH" => NodeFilter::ByMethod("DELETE".into()),
            _ => NodeFilter::All,
        }
    }
}

// ── Runner sub-views ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunnerSubview {
    Steps,
    Latency,
    Diff,
    Context,
}

impl RunnerSubview {
    pub fn label(&self) -> &str {
        match self {
            RunnerSubview::Steps => "Steps",
            RunnerSubview::Latency => "Latency",
            RunnerSubview::Diff => "Diff",
            RunnerSubview::Context => "Context",
        }
    }

    pub fn all() -> &'static [RunnerSubview] {
        &[
            RunnerSubview::Steps,
            RunnerSubview::Latency,
            RunnerSubview::Diff,
            RunnerSubview::Context,
        ]
    }
}

// ── Live execution events ─────────────────────────────────────────────────────

pub enum LiveEvent {
    Step(StepResult),
    /// Sent once the entire flow run completes, carrying the full result so the
    /// TUI can update `last_run` before `refresh_data()` touches storage.
    FlowResult(FlowRunResult),
    Done {
        passed: bool,
    },
    Error(String),
    NeedInput {
        node_id: String,
        inputs: Vec<PromptInput>,
    },
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
    pub lines: Vec<String>, // content split by newlines
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub scroll_top: usize, // vertical scroll for long bodies
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
        let lines = if lines.is_empty() {
            vec![String::new()]
        } else {
            lines
        };
        Self {
            node_id,
            lines,
            cursor_row: 0,
            cursor_col: 0,
            scroll_top: 0,
        }
    }

    // Current line length
    pub fn current_line_len(&self) -> usize {
        self.lines
            .get(self.cursor_row)
            .map(|l| l.len())
            .unwrap_or(0)
    }
}

// ── Prompt modal ──────────────────────────────────────────────────────────────

impl std::fmt::Display for BodyEditor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.lines.join("\n"))
    }
}

pub struct PromptModal {
    pub node_id: String,
    pub inputs: Vec<PromptInput>,
    pub values: Vec<String>,           // one entry per input, same order
    pub current_field: usize,          // which field is being edited
    pub option_cursors: Vec<usize>,    // for select/multiselect: which option is highlighted
    pub multi_checked: Vec<Vec<bool>>, // for multiselect: which options are checked
}

impl PromptModal {
    pub fn new(node_id: String, inputs: Vec<PromptInput>) -> Self {
        use crate::api::types::PromptType;
        let len = inputs.len();
        let mut values = Vec::with_capacity(len);
        let mut option_cursors = Vec::with_capacity(len);
        let mut multi_checked = Vec::with_capacity(len);
        for pi in &inputs {
            let default_val = match pi.prompt_type {
                PromptType::Boolean => pi
                    .default
                    .as_deref()
                    .map(|d| {
                        if d == "true" || d == "yes" || d == "1" {
                            "true"
                        } else {
                            "false"
                        }
                    })
                    .unwrap_or("false")
                    .to_string(),
                PromptType::Select => pi.default.as_deref().unwrap_or("").to_string(),
                _ => String::new(),
            };
            values.push(default_val);
            let cursor = match pi.prompt_type {
                PromptType::Select | PromptType::Multiselect => pi
                    .default
                    .as_deref()
                    .and_then(|d| pi.options.iter().position(|o| o == d))
                    .unwrap_or(0),
                _ => 0,
            };
            option_cursors.push(cursor);
            let checked: Vec<bool> = if pi.prompt_type == PromptType::Multiselect {
                let selected: std::collections::HashSet<&str> = pi
                    .default
                    .as_deref()
                    .map(|d| d.split(',').map(|s| s.trim()).collect())
                    .unwrap_or_default();
                pi.options
                    .iter()
                    .map(|o| selected.contains(o.as_str()))
                    .collect()
            } else {
                vec![false; pi.options.len()]
            };
            multi_checked.push(checked);
        }
        Self {
            node_id,
            inputs,
            values,
            current_field: 0,
            option_cursors,
            multi_checked,
        }
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

pub enum GraphDirection {
    Up,
    Down,
    Left,
    Right,
}
