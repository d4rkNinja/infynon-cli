use crate::trace::{
    storage,
    types::{
        EntityKind, KgEdge, KgEntity, KgGraph, NoteStatus, PackageRisk, RelationType, TraceLayer,
        TraceNote, TraceScope,
    },
};
use chrono::Utc;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs, Wrap},
    Terminal,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::{fs, io};

macro_rules! impl_field_nav {
    ($t:ty) => {
        impl $t {
            fn next(self) -> Self {
                let all = Self::all();
                let idx = all.iter().position(|f| *f == self).unwrap_or(0);
                all[(idx + 1) % all.len()]
            }
            fn prev(self) -> Self {
                let all = Self::all();
                let idx = all.iter().position(|f| *f == self).unwrap_or(0);
                all[(idx + all.len() - 1) % all.len()]
            }
        }
    };
}

macro_rules! impl_field_accessors {
    ($form:ty, $field:ty, { $($variant:ident => $member:ident),+ $(,)? }) => {
        impl $form {
            fn get_field(&self, f: $field) -> &str {
                match f { $(<$field>::$variant => &self.$member,)+ }
            }
            fn get_field_mut(&mut self, f: $field) -> &mut String {
                match f { $(<$field>::$variant => &mut self.$member,)+ }
            }
        }
    };
}

// ─── Tabs ─────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum TraceTab {
    Overview,
    Sources,
    Notes,
    Packages,
    EditLog,
    Graph,
}

impl TraceTab {
    fn all() -> [TraceTab; 6] {
        [
            TraceTab::Overview,
            TraceTab::Sources,
            TraceTab::Notes,
            TraceTab::Packages,
            TraceTab::EditLog,
            TraceTab::Graph,
        ]
    }
    fn title(&self) -> &'static str {
        match self {
            TraceTab::Overview => "Overview",
            TraceTab::Sources => "Sources",
            TraceTab::Notes => "Notes",
            TraceTab::Packages => "Packages",
            TraceTab::EditLog => "Edit Log",
            TraceTab::Graph => "Graph",
        }
    }
    fn index(&self) -> usize {
        TraceTab::all().iter().position(|t| t == self).unwrap_or(0)
    }
}

// ─── Edit fields ──────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum EditField {
    Id,
    Title,
    Body,
    Layer,
    Scope,
    Target,
    Author,
    Tags,
}

impl Default for EditField {
    fn default() -> Self {
        EditField::Id
    }
}

impl EditField {
    fn all() -> [EditField; 8] {
        [
            EditField::Id,
            EditField::Title,
            EditField::Body,
            EditField::Layer,
            EditField::Scope,
            EditField::Target,
            EditField::Author,
            EditField::Tags,
        ]
    }
    fn label(&self) -> &'static str {
        match self {
            EditField::Id => "ID",
            EditField::Title => "Title",
            EditField::Body => "Body",
            EditField::Layer => "Layer  (canonical | team | user)",
            EditField::Scope => "Scope  (repo | branch | pr | file | user | session | package)",
            EditField::Target => "Target",
            EditField::Author => "Author",
            EditField::Tags => "Tags  (comma-separated)",
        }
    }
}

impl_field_nav!(EditField);

// ─── Note form state ──────────────────────────────────────────────────────────

#[derive(Default)]
struct NoteForm {
    id: String,
    title: String,
    body: String,
    layer: String,
    scope: String,
    target: String,
    author: String,
    tags: String,
    active_field: EditField,
    is_edit: bool,
}

impl NoteForm {
    fn new_create(default_author: String) -> Self {
        Self {
            layer: "user".to_string(),
            scope: "repo".to_string(),
            author: default_author,
            active_field: EditField::Id,
            is_edit: false,
            ..Default::default()
        }
    }

    fn from_note(note: &TraceNote) -> Self {
        Self {
            id: note.id.clone(),
            title: note.title.clone(),
            body: note.body.clone(),
            layer: note.layer.as_str().to_string(),
            scope: note.scope.as_str().to_string(),
            target: note.target.clone(),
            author: note.author.clone(),
            tags: note.tags.join(", "),
            active_field: EditField::Title,
            is_edit: true,
        }
    }
}

impl_field_accessors!(NoteForm, EditField, {
    Id => id, Title => title, Body => body, Layer => layer,
    Scope => scope, Target => target, Author => author, Tags => tags
});

// ─── KG Entity form ──────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum KgEntityField {
    Name,
    Kind,
    Branch,
    Meta,
}

impl KgEntityField {
    fn all() -> [Self; 4] {
        [Self::Name, Self::Kind, Self::Branch, Self::Meta]
    }
    fn label(&self) -> &'static str {
        match self {
            Self::Name => "Name",
            Self::Kind => {
                "Kind (file|package|person|decision|endpoint|module|pr|branch|note|vulnerability)"
            }
            Self::Branch => "Branch",
            Self::Meta => "Metadata (key=value, comma-separated)",
        }
    }
}

impl_field_nav!(KgEntityField);

impl Default for KgEntityField {
    fn default() -> Self {
        Self::Name
    }
}

struct KgEntityForm {
    name: String,
    kind: String,
    branch: String,
    meta: String,
    active_field: KgEntityField,
    is_edit: bool,
    original_id: String,
}

impl KgEntityForm {
    fn new_create(branch: &str) -> Self {
        Self {
            name: String::new(),
            kind: "file".to_string(),
            branch: branch.to_string(),
            meta: String::new(),
            active_field: KgEntityField::Name,
            is_edit: false,
            original_id: String::new(),
        }
    }
    fn from_entity(e: &KgEntity) -> Self {
        let meta = e
            .metadata
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(", ");
        Self {
            name: e.name.clone(),
            kind: e.kind.as_str().to_string(),
            branch: e.branch.clone(),
            meta,
            active_field: KgEntityField::Name,
            is_edit: true,
            original_id: e.id.clone(),
        }
    }
}

impl_field_accessors!(KgEntityForm, KgEntityField, {
    Name => name, Kind => kind, Branch => branch, Meta => meta
});

// ─── KG Edge form ────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum KgEdgeField {
    From,
    To,
    Relation,
    Weight,
    Branch,
    Evidence,
}

impl KgEdgeField {
    fn all() -> [Self; 6] {
        [
            Self::From,
            Self::To,
            Self::Relation,
            Self::Weight,
            Self::Branch,
            Self::Evidence,
        ]
    }
    fn label(&self) -> &'static str {
        match self {
            Self::From => "From (entity name)",
            Self::To => "To (entity name)",
            Self::Relation => "Relation (depends_on|introduced_by|modified_by|affects|decided_by|relates_to|supersedes|conflicts_with|documents|exposes|owns)",
            Self::Weight => "Weight (0.0-1.0)",
            Self::Branch => "Branch",
            Self::Evidence => "Evidence",
        }
    }
}

impl_field_nav!(KgEdgeField);

impl Default for KgEdgeField {
    fn default() -> Self {
        Self::From
    }
}

struct KgEdgeForm {
    from: String,
    to: String,
    relation: String,
    weight: String,
    branch: String,
    evidence: String,
    active_field: KgEdgeField,
    is_edit: bool,
    original_id: String,
}

impl KgEdgeForm {
    fn new_create(branch: &str) -> Self {
        Self {
            from: String::new(),
            to: String::new(),
            relation: "relates_to".to_string(),
            weight: "1.0".to_string(),
            branch: branch.to_string(),
            evidence: String::new(),
            active_field: KgEdgeField::From,
            is_edit: false,
            original_id: String::new(),
        }
    }
    fn from_edge(e: &KgEdge) -> Self {
        Self {
            from: e.source.clone(),
            to: e.target.clone(),
            relation: e.relation.as_str().to_string(),
            weight: format!("{:.1}", e.weight),
            branch: e.branch.clone(),
            evidence: e.evidence.clone(),
            active_field: KgEdgeField::From,
            is_edit: true,
            original_id: e.id.clone(),
        }
    }
}

impl_field_accessors!(KgEdgeForm, KgEdgeField, {
    From => from, To => to, Relation => relation, Weight => weight,
    Branch => branch, Evidence => evidence
});

// ─── App mode ─────────────────────────────────────────────────────────────────

enum AppMode {
    Browse,
    ViewDetail,
    EditForm(NoteForm),
    DeleteConfirm(String),
    SourceDeleteConfirm(String),
    PackageDetail,
    KgEntityForm(KgEntityForm),
    KgEdgeForm(KgEdgeForm),
    KgEntityDelete(String),
    KgEdgeDelete(String),
    KgBranchPicker,
}

// ─── Audit log ────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
struct AuditEntry {
    timestamp: String,
    action: String,
    note_id: String,
    author: String,
    summary: String,
}

fn audit_log_path() -> std::path::PathBuf {
    storage::trace_dir().join("state").join("tui_edits.jsonl")
}

fn append_audit(entry: AuditEntry) {
    if let Ok(line) = serde_json::to_string(&entry) {
        let path = audit_log_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        use std::io::Write as W;
        if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(&path) {
            let _ = writeln!(f, "{}", line);
        }
    }
}

fn load_audit_log() -> Vec<AuditEntry> {
    let path = audit_log_path();
    if !path.exists() {
        return Vec::new();
    }
    let Ok(content) = fs::read_to_string(&path) else {
        return Vec::new();
    };
    let mut entries: Vec<AuditEntry> = content
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();
    entries.reverse(); // newest first
    entries
}

// ─── Knowledge graph view ────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum KgView {
    Entities,
    Edges,
    Visual,
}

// ─── App state ────────────────────────────────────────────────────────────────

struct App {
    tab: TraceTab,
    notes: Vec<TraceNote>,
    packages: Vec<PackageRisk>,
    list_state: ListState,
    sources_state: ListState,
    packages_state: ListState,
    audit_scroll: usize,
    mode: AppMode,
    status: Option<(String, bool)>, // (message, is_error)
    audit: Vec<AuditEntry>,
    kg_entities: Vec<KgEntity>,
    kg_edges: Vec<KgEdge>,
    kg_entity_state: ListState,
    kg_selected_entity: Option<usize>,
    kg_view: KgView,
    kg_branch: String,
    kg_branches: Vec<String>,
    kg_branch_idx: usize,
}

impl App {
    fn reload_notes(&mut self) {
        self.notes = storage::list_notes().unwrap_or_default();
    }

    fn reload_packages(&mut self) {
        self.packages = storage::package_risks().unwrap_or_default();
        self.packages_state = ListState::default();
        if !self.packages.is_empty() {
            self.packages_state.select(Some(0));
        }
    }

    fn reload_sources(&mut self, cfg: &crate::trace::types::TraceConfig) {
        let count = cfg.sources.len();
        self.sources_state = ListState::default();
        if count > 0 {
            self.sources_state.select(Some(0));
        }
    }

    fn reload_audit(&mut self) {
        self.audit = load_audit_log();
        self.audit_scroll = 0;
    }

    fn reload_kg(&mut self) {
        let graph = storage::load_graph(Some(&self.kg_branch)).unwrap_or_default();
        self.kg_entities = graph.entities;
        self.kg_edges = graph.edges;
        self.kg_entity_state = ListState::default();
        if !self.kg_entities.is_empty() {
            self.kg_entity_state.select(Some(0));
        }
    }

    fn reload_kg_all(&mut self) {
        let graph = storage::load_graph(None).unwrap_or_default();
        self.kg_entities = graph.entities;
        self.kg_edges = graph.edges;
        self.kg_entity_state = ListState::default();
        if !self.kg_entities.is_empty() {
            self.kg_entity_state.select(Some(0));
        }
    }

    fn reload_kg_branches(&mut self) {
        let all_entities = storage::list_entities(None, None).unwrap_or_default();
        let mut branches: Vec<String> = all_entities.iter().map(|e| e.branch.clone()).collect();
        branches.sort();
        branches.dedup();
        if branches.is_empty() {
            branches.push(storage::detect_current_branch());
        }
        self.kg_branches = branches;
        self.kg_branch_idx = self
            .kg_branches
            .iter()
            .position(|b| b == &self.kg_branch)
            .unwrap_or(0);
    }

    fn selected_idx(&self) -> Option<usize> {
        self.list_state.selected()
    }

    fn selected_note(&self) -> Option<&TraceNote> {
        self.list_state.selected().and_then(|i| self.notes.get(i))
    }

    fn selected_package(&self) -> Option<&PackageRisk> {
        self.packages_state
            .selected()
            .and_then(|i| self.packages.get(i))
    }

    fn ok(&mut self, msg: impl Into<String>) {
        self.status = Some((msg.into(), false));
    }

    fn err(&mut self, msg: impl Into<String>) {
        self.status = Some((msg.into(), true));
    }

    fn clamp_packages(&mut self) {
        let len = self.packages.len();
        match self.packages_state.selected() {
            None if len > 0 => self.packages_state.select(Some(0)),
            Some(i) if i >= len && len > 0 => self.packages_state.select(Some(len - 1)),
            Some(_) if len == 0 => self.packages_state.select(None),
            _ => {}
        }
    }

    fn clamp_selection(&mut self) {
        let len = self.notes.len();
        match self.list_state.selected() {
            None if len > 0 => self.list_state.select(Some(0)),
            Some(i) if i >= len && len > 0 => self.list_state.select(Some(len - 1)),
            Some(_) if len == 0 => self.list_state.select(None),
            _ => {}
        }
    }
}

// ─── Public entry points ─────────────────────────────────────────────────────
include!("tui/runtime.rs");
include!("tui/handlers_browse.rs");
include!("tui/handlers_forms.rs");
include!("tui/views_core.rs");
include!("tui/views_modal.rs");
include!("tui/views_graph.rs");
