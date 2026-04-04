use crate::trace::{
    storage,
    types::{TraceLayer, TraceNote, TraceScope, NoteStatus, PackageRisk, KgEntity, KgEdge, KgGraph, EntityKind, RelationType},
};
use std::str::FromStr;
use chrono::Utc;
use crossterm::{
    event::{self, Event, KeyCode},
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
use std::{fs, io};

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
    fn next(self) -> EditField {
        let all = EditField::all();
        let idx = all.iter().position(|f| *f == self).unwrap_or(0);
        all[(idx + 1) % all.len()]
    }
    fn prev(self) -> EditField {
        let all = EditField::all();
        let idx = all.iter().position(|f| *f == self).unwrap_or(0);
        all[(idx + all.len() - 1) % all.len()]
    }
}

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

    fn get_field(&self, field: EditField) -> &str {
        match field {
            EditField::Id => &self.id,
            EditField::Title => &self.title,
            EditField::Body => &self.body,
            EditField::Layer => &self.layer,
            EditField::Scope => &self.scope,
            EditField::Target => &self.target,
            EditField::Author => &self.author,
            EditField::Tags => &self.tags,
        }
    }

    fn get_field_mut(&mut self, field: EditField) -> &mut String {
        match field {
            EditField::Id => &mut self.id,
            EditField::Title => &mut self.title,
            EditField::Body => &mut self.body,
            EditField::Layer => &mut self.layer,
            EditField::Scope => &mut self.scope,
            EditField::Target => &mut self.target,
            EditField::Author => &mut self.author,
            EditField::Tags => &mut self.tags,
        }
    }
}

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
            Self::Kind => "Kind (file|package|person|decision|endpoint|module|pr|branch|note|vulnerability)",
            Self::Branch => "Branch",
            Self::Meta => "Metadata (key=value, comma-separated)",
        }
    }
    fn next(self) -> Self {
        let a = Self::all();
        let i = a.iter().position(|f| *f == self).unwrap_or(0);
        a[(i + 1) % a.len()]
    }
    fn prev(self) -> Self {
        let a = Self::all();
        let i = a.iter().position(|f| *f == self).unwrap_or(0);
        a[(i + a.len() - 1) % a.len()]
    }
}

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
    fn get_field(&self, f: KgEntityField) -> &str {
        match f {
            KgEntityField::Name => &self.name,
            KgEntityField::Kind => &self.kind,
            KgEntityField::Branch => &self.branch,
            KgEntityField::Meta => &self.meta,
        }
    }
    fn get_field_mut(&mut self, f: KgEntityField) -> &mut String {
        match f {
            KgEntityField::Name => &mut self.name,
            KgEntityField::Kind => &mut self.kind,
            KgEntityField::Branch => &mut self.branch,
            KgEntityField::Meta => &mut self.meta,
        }
    }
}

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
    fn next(self) -> Self {
        let a = Self::all();
        let i = a.iter().position(|f| *f == self).unwrap_or(0);
        a[(i + 1) % a.len()]
    }
    fn prev(self) -> Self {
        let a = Self::all();
        let i = a.iter().position(|f| *f == self).unwrap_or(0);
        a[(i + a.len() - 1) % a.len()]
    }
}

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
    fn get_field(&self, f: KgEdgeField) -> &str {
        match f {
            KgEdgeField::From => &self.from,
            KgEdgeField::To => &self.to,
            KgEdgeField::Relation => &self.relation,
            KgEdgeField::Weight => &self.weight,
            KgEdgeField::Branch => &self.branch,
            KgEdgeField::Evidence => &self.evidence,
        }
    }
    fn get_field_mut(&mut self, f: KgEdgeField) -> &mut String {
        match f {
            KgEdgeField::From => &mut self.from,
            KgEdgeField::To => &mut self.to,
            KgEdgeField::Relation => &mut self.relation,
            KgEdgeField::Weight => &mut self.weight,
            KgEdgeField::Branch => &mut self.branch,
            KgEdgeField::Evidence => &mut self.evidence,
        }
    }
}

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
        if let Ok(mut f) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
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

    fn reload_sources(&mut self) {
        let cfg = storage::load_config().unwrap_or_default();
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
        self.packages_state.selected().and_then(|i| self.packages.get(i))
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

pub fn run() {
    run_inner(TraceTab::Notes, storage::detect_current_branch());
}

pub fn run_kg(branch: Option<String>) {
    let b = branch.unwrap_or_else(storage::detect_current_branch);
    run_inner(TraceTab::Graph, b);
}

fn run_inner(initial_tab: TraceTab, kg_branch: String) {
    let _ = enable_raw_mode();
    let mut stdout = io::stdout();
    let _ = execute!(stdout, EnterAlternateScreen);
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = match Terminal::new(backend) {
        Ok(t) => t,
        Err(_) => {
            let _ = disable_raw_mode();
            return;
        }
    };

    let notes = storage::list_notes().unwrap_or_default();
    let packages = storage::package_risks().unwrap_or_default();
    let audit = load_audit_log();

    let mut list_state = ListState::default();
    if !notes.is_empty() {
        list_state.select(Some(0));
    }
    let mut packages_state = ListState::default();
    if !packages.is_empty() {
        packages_state.select(Some(0));
    }
    let cfg_init = storage::load_config().unwrap_or_default();
    let mut sources_state = ListState::default();
    if !cfg_init.sources.is_empty() {
        sources_state.select(Some(0));
    }

    let mut app = App {
        tab: initial_tab,
        notes,
        packages,
        list_state,
        sources_state,
        packages_state,
        audit_scroll: 0,
        mode: AppMode::Browse,
        status: None,
        audit,
        kg_entities: Vec::new(),
        kg_edges: Vec::new(),
        kg_entity_state: ListState::default(),
        kg_selected_entity: None,
        kg_view: KgView::Entities,
        kg_branch,
        kg_branches: Vec::new(),
        kg_branch_idx: 0,
    };

    if app.tab == TraceTab::Graph {
        app.reload_kg();
        app.reload_kg_branches();
    }

    loop {
        let cfg = storage::load_config().unwrap_or_default();
        if terminal.draw(|f| draw_ui(f, &mut app, &cfg)).is_err() {
            break;
        }

        match event::poll(std::time::Duration::from_millis(80)) {
            Ok(true) => {}
            _ => continue,
        }
        let Ok(Event::Key(key)) = event::read() else {
            continue;
        };

        let quit = match &app.mode {
            AppMode::Browse | AppMode::ViewDetail | AppMode::PackageDetail => {
                handle_browse(&mut app, key.code)
            }
            AppMode::EditForm(_) => {
                handle_form(&mut app, key.code);
                false
            }
            AppMode::DeleteConfirm(_) => {
                handle_delete_confirm(&mut app, key.code);
                false
            }
            AppMode::SourceDeleteConfirm(_) => {
                handle_source_delete_confirm(&mut app, key.code);
                false
            }
            AppMode::KgEntityForm(_) => {
                handle_kg_entity_form(&mut app, key.code);
                false
            }
            AppMode::KgEdgeForm(_) => {
                handle_kg_edge_form(&mut app, key.code);
                false
            }
            AppMode::KgEntityDelete(_) => {
                handle_kg_entity_delete(&mut app, key.code);
                false
            }
            AppMode::KgEdgeDelete(_) => {
                handle_kg_edge_delete(&mut app, key.code);
                false
            }
            AppMode::KgBranchPicker => {
                handle_kg_branch_picker(&mut app, key.code);
                false
            }
        };
        if quit {
            break;
        }
    }

    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
    let _ = terminal.show_cursor();
}

// ─── Key handlers ─────────────────────────────────────────────────────────────

fn handle_browse(app: &mut App, code: KeyCode) -> bool {
    app.status = None; // clear previous status on any key

    match code {
        // ── Back from detail views ──────────────────────────────────────────
        KeyCode::Char('q') | KeyCode::Esc
            if matches!(app.mode, AppMode::ViewDetail | AppMode::PackageDetail) =>
        {
            app.mode = AppMode::Browse;
        }
        KeyCode::Char('q') => return true,

        // ── Tab switching ────────────────────────────────────────────────────
        KeyCode::Char('1') => { app.tab = TraceTab::Overview; app.mode = AppMode::Browse; }
        KeyCode::Char('2') => { app.tab = TraceTab::Sources; app.mode = AppMode::Browse; }
        KeyCode::Char('3') => { app.tab = TraceTab::Notes; app.mode = AppMode::Browse; }
        KeyCode::Char('4') => { app.tab = TraceTab::Packages; app.mode = AppMode::Browse; }
        KeyCode::Char('5') => {
            app.tab = TraceTab::EditLog;
            app.mode = AppMode::Browse;
            app.reload_audit();
        }
        KeyCode::Char('6') => {
            app.tab = TraceTab::Graph;
            app.mode = AppMode::Browse;
            app.reload_kg();
        }
        KeyCode::Right | KeyCode::Char('l')
            if !matches!(app.mode, AppMode::ViewDetail | AppMode::PackageDetail) =>
        {
            let next = (app.tab.index() + 1) % 6;
            app.tab = TraceTab::all()[next];
            app.mode = AppMode::Browse;
            if app.tab == TraceTab::EditLog {
                app.reload_audit();
            }
            if app.tab == TraceTab::Graph {
                app.reload_kg();
            }
        }
        KeyCode::Left | KeyCode::Char('h')
            if !matches!(app.mode, AppMode::ViewDetail | AppMode::PackageDetail) =>
        {
            let prev = (app.tab.index() + 6 - 1) % 6;
            app.tab = TraceTab::all()[prev];
            app.mode = AppMode::Browse;
            if app.tab == TraceTab::Graph {
                app.reload_kg();
            }
        }

        // ── Notes tab ────────────────────────────────────────────────────────
        KeyCode::Down | KeyCode::Char('j') if app.tab == TraceTab::Notes => {
            let len = app.notes.len();
            if len > 0 {
                let next = app.selected_idx().map(|i| (i + 1) % len).unwrap_or(0);
                app.list_state.select(Some(next));
            }
        }
        KeyCode::Up | KeyCode::Char('k') if app.tab == TraceTab::Notes => {
            let len = app.notes.len();
            if len > 0 {
                let prev = app
                    .selected_idx()
                    .map(|i| if i == 0 { len - 1 } else { i - 1 })
                    .unwrap_or(0);
                app.list_state.select(Some(prev));
            }
        }
        KeyCode::Enter if app.tab == TraceTab::Notes => {
            if app.selected_note().is_some() {
                app.mode = AppMode::ViewDetail;
            }
        }
        KeyCode::Char('n') if app.tab == TraceTab::Notes => {
            let author = storage::configured_user()
                .or_else(storage::detect_user_name)
                .unwrap_or_else(|| "unknown".to_string());
            app.mode = AppMode::EditForm(NoteForm::new_create(author));
        }
        KeyCode::Char('e') if app.tab == TraceTab::Notes => {
            if let Some(note) = app.selected_note() {
                let form = NoteForm::from_note(note);
                app.mode = AppMode::EditForm(form);
            }
        }
        KeyCode::Char('d') if app.tab == TraceTab::Notes => {
            if let Some(note) = app.selected_note() {
                let id = note.id.clone();
                app.mode = AppMode::DeleteConfirm(id);
            }
        }
        KeyCode::Char('r') if app.tab == TraceTab::Notes => {
            app.reload_notes();
            app.clamp_selection();
            app.ok("Notes reloaded");
        }

        // ── Sources tab ──────────────────────────────────────────────────────
        KeyCode::Down | KeyCode::Char('j') if app.tab == TraceTab::Sources => {
            let cfg = storage::load_config().unwrap_or_default();
            let len = cfg.sources.len();
            if len > 0 {
                let next = app.sources_state.selected().map(|i| (i + 1) % len).unwrap_or(0);
                app.sources_state.select(Some(next));
            }
        }
        KeyCode::Up | KeyCode::Char('k') if app.tab == TraceTab::Sources => {
            let cfg = storage::load_config().unwrap_or_default();
            let len = cfg.sources.len();
            if len > 0 {
                let prev = app
                    .sources_state
                    .selected()
                    .map(|i| if i == 0 { len - 1 } else { i - 1 })
                    .unwrap_or(0);
                app.sources_state.select(Some(prev));
            }
        }
        KeyCode::Char('d') if app.tab == TraceTab::Sources => {
            let cfg = storage::load_config().unwrap_or_default();
            if let Some(idx) = app.sources_state.selected() {
                if let Some(src) = cfg.sources.get(idx) {
                    app.mode = AppMode::SourceDeleteConfirm(src.id.clone());
                }
            }
        }
        KeyCode::Char('s') if app.tab == TraceTab::Sources => {
            let cfg = storage::load_config().unwrap_or_default();
            if let Some(idx) = app.sources_state.selected() {
                if let Some(src) = cfg.sources.get(idx) {
                    let id = src.id.clone();
                    match storage::set_default_source(&id) {
                        Ok(()) => app.ok(format!("Default source set to '{}'", id)),
                        Err(e) => app.err(e),
                    }
                }
            }
        }
        KeyCode::Char('r') if app.tab == TraceTab::Sources => {
            app.reload_sources();
            app.ok("Sources reloaded");
        }

        // ── Packages tab ─────────────────────────────────────────────────────
        KeyCode::Down | KeyCode::Char('j') if app.tab == TraceTab::Packages => {
            let len = app.packages.len();
            if len > 0 {
                let next = app.packages_state.selected().map(|i| (i + 1) % len).unwrap_or(0);
                app.packages_state.select(Some(next));
            }
        }
        KeyCode::Up | KeyCode::Char('k') if app.tab == TraceTab::Packages => {
            let len = app.packages.len();
            if len > 0 {
                let prev = app
                    .packages_state
                    .selected()
                    .map(|i| if i == 0 { len - 1 } else { i - 1 })
                    .unwrap_or(0);
                app.packages_state.select(Some(prev));
            }
        }
        KeyCode::Enter if app.tab == TraceTab::Packages => {
            if app.selected_package().is_some() {
                app.mode = AppMode::PackageDetail;
            }
        }
        KeyCode::Char('r') if app.tab == TraceTab::Packages => {
            app.reload_packages();
            app.ok("Package risks reloaded");
        }

        // ── EditLog tab ──────────────────────────────────────────────────────
        KeyCode::Down | KeyCode::Char('j') if app.tab == TraceTab::EditLog => {
            if app.audit_scroll + 1 < app.audit.len() {
                app.audit_scroll += 1;
            }
        }
        KeyCode::Up | KeyCode::Char('k') if app.tab == TraceTab::EditLog => {
            app.audit_scroll = app.audit_scroll.saturating_sub(1);
        }
        KeyCode::Char('g') if app.tab == TraceTab::EditLog => {
            app.audit_scroll = 0;
        }
        KeyCode::Char('G') if app.tab == TraceTab::EditLog => {
            app.audit_scroll = app.audit.len().saturating_sub(1);
        }
        KeyCode::Char('r') if app.tab == TraceTab::EditLog => {
            app.reload_audit();
            app.ok("Edit log reloaded");
        }

        // ── Graph tab ───────────────────────────────────────────────────────
        KeyCode::Char('n') if app.tab == TraceTab::Graph && app.kg_view == KgView::Entities => {
            app.mode = AppMode::KgEntityForm(KgEntityForm::new_create(&app.kg_branch));
        }
        KeyCode::Char('n') if app.tab == TraceTab::Graph && app.kg_view == KgView::Edges => {
            app.mode = AppMode::KgEdgeForm(KgEdgeForm::new_create(&app.kg_branch));
        }
        KeyCode::Enter if app.tab == TraceTab::Graph && app.kg_view == KgView::Entities => {
            if let Some(idx) = app.kg_entity_state.selected() {
                if let Some(ent) = app.kg_entities.get(idx) {
                    app.mode = AppMode::KgEntityForm(KgEntityForm::from_entity(ent));
                }
            }
        }
        KeyCode::Enter if app.tab == TraceTab::Graph && app.kg_view == KgView::Edges => {
            if let Some(idx) = app.kg_entity_state.selected() {
                if let Some(edge) = app.kg_edges.get(idx) {
                    app.mode = AppMode::KgEdgeForm(KgEdgeForm::from_edge(edge));
                }
            }
        }
        KeyCode::Char('d') if app.tab == TraceTab::Graph && app.kg_view == KgView::Entities => {
            if let Some(idx) = app.kg_entity_state.selected() {
                if let Some(ent) = app.kg_entities.get(idx) {
                    app.mode = AppMode::KgEntityDelete(ent.id.clone());
                }
            }
        }
        KeyCode::Char('d') if app.tab == TraceTab::Graph && app.kg_view == KgView::Edges => {
            if let Some(idx) = app.kg_entity_state.selected() {
                if let Some(edge) = app.kg_edges.get(idx) {
                    app.mode = AppMode::KgEdgeDelete(edge.id.clone());
                }
            }
        }
        KeyCode::Char('b') if app.tab == TraceTab::Graph => {
            app.reload_kg_branches();
            app.mode = AppMode::KgBranchPicker;
        }
        KeyCode::Char('a') if app.tab == TraceTab::Graph => {
            if app.kg_branch == "*" {
                app.kg_branch = storage::detect_current_branch();
                app.reload_kg();
                app.ok("Showing current branch");
            } else {
                app.kg_branch = "*".to_string();
                app.reload_kg_all();
                app.ok("Showing all branches");
            }
        }
        KeyCode::Char('B') if app.tab == TraceTab::Graph => {
            let _ = storage::ensure_kg_layout();
            let branch = if app.kg_branch == "*" {
                storage::detect_current_branch()
            } else {
                app.kg_branch.clone()
            };
            match storage::auto_build_graph(&branch) {
                Ok((ents, edges)) => {
                    app.ok(format!("Built: {} entities, {} edges", ents, edges));
                    app.reload_kg();
                    app.reload_kg_branches();
                }
                Err(e) => app.err(e),
            }
        }
        KeyCode::Down | KeyCode::Char('j') if app.tab == TraceTab::Graph => {
            match app.kg_view {
                KgView::Entities => {
                    let len = app.kg_entities.len();
                    if len > 0 {
                        let next = app.kg_entity_state.selected().map(|i| (i + 1) % len).unwrap_or(0);
                        app.kg_entity_state.select(Some(next));
                        app.kg_selected_entity = Some(next);
                    }
                }
                KgView::Edges | KgView::Visual => {
                    let len = app.kg_edges.len();
                    if len > 0 {
                        let next = app.kg_entity_state.selected().map(|i| (i + 1) % len).unwrap_or(0);
                        app.kg_entity_state.select(Some(next));
                    }
                }
            }
        }
        KeyCode::Up | KeyCode::Char('k') if app.tab == TraceTab::Graph => {
            match app.kg_view {
                KgView::Entities => {
                    let len = app.kg_entities.len();
                    if len > 0 {
                        let prev = app.kg_entity_state.selected()
                            .map(|i| if i == 0 { len - 1 } else { i - 1 })
                            .unwrap_or(0);
                        app.kg_entity_state.select(Some(prev));
                        app.kg_selected_entity = Some(prev);
                    }
                }
                KgView::Edges | KgView::Visual => {
                    let len = app.kg_edges.len();
                    if len > 0 {
                        let prev = app.kg_entity_state.selected()
                            .map(|i| if i == 0 { len - 1 } else { i - 1 })
                            .unwrap_or(0);
                        app.kg_entity_state.select(Some(prev));
                    }
                }
            }
        }
        KeyCode::Tab if app.tab == TraceTab::Graph => {
            app.kg_view = match app.kg_view {
                KgView::Entities => KgView::Edges,
                KgView::Edges => KgView::Visual,
                KgView::Visual => KgView::Entities,
            };
            app.kg_entity_state = ListState::default();
            let len = match app.kg_view {
                KgView::Entities => app.kg_entities.len(),
                KgView::Edges | KgView::Visual => app.kg_edges.len(),
            };
            if len > 0 {
                app.kg_entity_state.select(Some(0));
            }
        }
        KeyCode::Char('r') if app.tab == TraceTab::Graph => {
            app.reload_kg();
            app.ok("Knowledge graph reloaded");
        }
        KeyCode::Char('v') if app.tab == TraceTab::Graph => {
            app.kg_view = KgView::Visual;
        }
        KeyCode::Char('e') if app.tab == TraceTab::Graph => {
            app.kg_view = KgView::Entities;
        }
        KeyCode::Char('w') if app.tab == TraceTab::Graph => {
            app.kg_view = KgView::Edges;
        }

        _ => {}
    }
    false
}

fn handle_form(app: &mut App, code: KeyCode) {
    let AppMode::EditForm(ref mut form) = app.mode else {
        return;
    };
    match code {
        KeyCode::Esc => {
            app.mode = AppMode::Browse;
            return;
        }
        KeyCode::Tab => {
            let next = form.active_field.next();
            if form.is_edit && next == EditField::Id {
                form.active_field = next.next(); // skip ID when editing
            } else {
                form.active_field = next;
            }
        }
        KeyCode::BackTab => {
            let prev = form.active_field.prev();
            if form.is_edit && prev == EditField::Id {
                form.active_field = prev.prev();
            } else {
                form.active_field = prev;
            }
        }
        KeyCode::Backspace => {
            let field = form.active_field;
            form.get_field_mut(field).pop();
        }
        KeyCode::Char(c) => {
            let field = form.active_field;
            form.get_field_mut(field).push(c);
        }
        KeyCode::Enter => {
            // Extract values before dropping borrow
            let is_edit = form.is_edit;
            let id = form.id.trim().to_string();
            let title = form.title.trim().to_string();
            let body = form.body.trim().to_string();
            let layer_s = form.layer.trim().to_string();
            let scope_s = form.scope.trim().to_string();
            let target = form.target.trim().to_string();
            let author_s = form.author.trim().to_string();
            let tags_s = form.tags.trim().to_string();

            if id.is_empty() && !is_edit {
                app.err("ID is required");
                return;
            }
            if title.is_empty() {
                app.err("Title is required");
                return;
            }
            let layer = match parse_layer(&layer_s) {
                Ok(v) => v,
                Err(e) => {
                    app.err(e);
                    return;
                }
            };
            let scope = match parse_scope(&scope_s) {
                Ok(v) => v,
                Err(e) => {
                    app.err(e);
                    return;
                }
            };
            let tags: Vec<String> = if tags_s.is_empty() {
                Vec::new()
            } else {
                tags_s
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            };
            let author = if author_s.is_empty() {
                storage::configured_user()
                    .or_else(storage::detect_user_name)
                    .unwrap_or_else(|| "unknown".to_string())
            } else {
                author_s
            };

            let (action, summary) = if is_edit {
                match storage::update_note(&id, Some(&title), Some(&body), None) {
                    Ok(()) => (
                        "edit",
                        format!("title={} body_len={}", title, body.len()),
                    ),
                    Err(e) => {
                        app.err(e);
                        return;
                    }
                }
            } else {
                let note = TraceNote {
                    id: id.clone(),
                    title: title.clone(),
                    body: body.clone(),
                    layer,
                    scope,
                    target: target.clone(),
                    files: Vec::new(),
                    tags,
                    related_pr: None,
                    author: author.clone(),
                    actor: None,
                    status: NoteStatus::Active,
                    created_at: String::new(),
                    updated_at: String::new(),
                };
                match storage::create_note(note) {
                    Ok(()) => ("create", format!("title={} layer={} scope={}", title, layer_s, scope_s)),
                    Err(e) => {
                        app.err(e);
                        return;
                    }
                }
            };

            append_audit(AuditEntry {
                timestamp: Utc::now().to_rfc3339(),
                action: action.to_string(),
                note_id: id.clone(),
                author,
                summary,
            });

            app.reload_notes();
            if let Some(pos) = app.notes.iter().position(|n| n.id == id) {
                app.list_state.select(Some(pos));
            }
            app.ok(format!(
                "Note '{}' {}d",
                id,
                if is_edit { "update" } else { "create" }
            ));
            app.mode = AppMode::Browse;
        }
        _ => {}
    }
}

fn handle_delete_confirm(app: &mut App, code: KeyCode) {
    let id = match &app.mode {
        AppMode::DeleteConfirm(id) => id.clone(),
        _ => return,
    };
    match code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            let author = storage::configured_user()
                .or_else(storage::detect_user_name)
                .unwrap_or_else(|| "unknown".to_string());
            match storage::delete_note(&id) {
                Ok(()) => {
                    append_audit(AuditEntry {
                        timestamp: Utc::now().to_rfc3339(),
                        action: "delete".to_string(),
                        note_id: id.clone(),
                        author,
                        summary: format!("deleted note '{}'", id),
                    });
                    app.ok(format!("Note '{}' deleted", id));
                    app.reload_notes();
                    app.clamp_selection();
                }
                Err(e) => app.err(e),
            }
            app.mode = AppMode::Browse;
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.mode = AppMode::Browse;
        }
        _ => {}
    }
}

fn handle_source_delete_confirm(app: &mut App, code: KeyCode) {
    let id = match &app.mode {
        AppMode::SourceDeleteConfirm(id) => id.clone(),
        _ => return,
    };
    match code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            match storage::remove_source(&id) {
                Ok(()) => {
                    app.ok(format!("Source '{}' removed", id));
                    app.reload_sources();
                }
                Err(e) => app.err(e),
            }
            app.mode = AppMode::Browse;
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.mode = AppMode::Browse;
        }
        _ => {}
    }
}

fn handle_kg_entity_form(app: &mut App, code: KeyCode) {
    let AppMode::KgEntityForm(ref mut form) = app.mode else {
        return;
    };
    match code {
        KeyCode::Esc => {
            app.mode = AppMode::Browse;
            return;
        }
        KeyCode::Tab => {
            form.active_field = form.active_field.next();
        }
        KeyCode::BackTab => {
            form.active_field = form.active_field.prev();
        }
        KeyCode::Backspace => {
            let field = form.active_field;
            form.get_field_mut(field).pop();
        }
        KeyCode::Char(c) => {
            let field = form.active_field;
            form.get_field_mut(field).push(c);
        }
        KeyCode::Enter => {
            let is_edit = form.is_edit;
            let original_id = form.original_id.clone();
            let name = form.name.trim().to_string();
            let kind_s = form.kind.trim().to_string();
            let branch = form.branch.trim().to_string();
            let meta_s = form.meta.trim().to_string();

            if name.is_empty() {
                app.err("Name is required");
                return;
            }
            let kind = match EntityKind::from_str(&kind_s) {
                Ok(v) => v,
                Err(e) => {
                    app.err(e);
                    return;
                }
            };
            let mut metadata = std::collections::HashMap::new();
            if !meta_s.is_empty() {
                for pair in meta_s.split(',') {
                    let pair = pair.trim();
                    if let Some((k, v)) = pair.split_once('=') {
                        metadata.insert(k.trim().to_string(), v.trim().to_string());
                    }
                }
            }
            let id = name
                .chars()
                .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
                .collect::<String>();
            let now = Utc::now().to_rfc3339();
            let entity = KgEntity {
                id: id.clone(),
                kind,
                name: name.clone(),
                metadata,
                branch: branch.clone(),
                created_at: now.clone(),
                updated_at: now,
            };

            if is_edit {
                let _ = storage::delete_entity(&original_id);
            }
            match storage::create_entity(entity) {
                Ok(()) => {
                    app.ok(format!(
                        "Entity '{}' {}",
                        name,
                        if is_edit { "updated" } else { "created" }
                    ));
                    if app.kg_branch == "*" {
                        app.reload_kg_all();
                    } else {
                        app.reload_kg();
                    }
                    app.reload_kg_branches();
                }
                Err(e) => app.err(e),
            }
            app.mode = AppMode::Browse;
        }
        _ => {}
    }
}

fn handle_kg_edge_form(app: &mut App, code: KeyCode) {
    let AppMode::KgEdgeForm(ref mut form) = app.mode else {
        return;
    };
    match code {
        KeyCode::Esc => {
            app.mode = AppMode::Browse;
            return;
        }
        KeyCode::Tab => {
            form.active_field = form.active_field.next();
        }
        KeyCode::BackTab => {
            form.active_field = form.active_field.prev();
        }
        KeyCode::Backspace => {
            let field = form.active_field;
            form.get_field_mut(field).pop();
        }
        KeyCode::Char(c) => {
            let field = form.active_field;
            form.get_field_mut(field).push(c);
        }
        KeyCode::Enter => {
            let is_edit = form.is_edit;
            let original_id = form.original_id.clone();
            let from_s = form.from.trim().to_string();
            let to_s = form.to.trim().to_string();
            let rel_s = form.relation.trim().to_string();
            let weight_s = form.weight.trim().to_string();
            let branch = form.branch.trim().to_string();
            let evidence = form.evidence.trim().to_string();

            if from_s.is_empty() || to_s.is_empty() {
                app.err("From and To are required");
                return;
            }
            let relation = match RelationType::from_str(&rel_s) {
                Ok(v) => v,
                Err(e) => {
                    app.err(e);
                    return;
                }
            };
            let weight: f64 = match weight_s.parse() {
                Ok(v) => v,
                Err(_) => {
                    app.err("Invalid weight (use a number like 0.5)");
                    return;
                }
            };

            // Resolve from/to — use the string as-is (it may be an entity ID or name)
            let source = match storage::find_entity_by_name(&from_s, &branch) {
                Ok(Some(e)) => e.id,
                _ => from_s.clone(),
            };
            let target = match storage::find_entity_by_name(&to_s, &branch) {
                Ok(Some(e)) => e.id,
                _ => to_s.clone(),
            };

            let edge_id = format!("{}-{}-{}", source, relation.as_str(), target);
            let now = Utc::now().to_rfc3339();
            let edge = KgEdge {
                id: edge_id,
                source,
                target,
                relation,
                weight,
                branch: branch.clone(),
                evidence,
                created_at: now,
            };

            if is_edit {
                let _ = storage::delete_edge(&original_id);
            }
            match storage::create_edge(edge) {
                Ok(()) => {
                    app.ok(format!(
                        "Edge {}",
                        if is_edit { "updated" } else { "created" }
                    ));
                    if app.kg_branch == "*" {
                        app.reload_kg_all();
                    } else {
                        app.reload_kg();
                    }
                }
                Err(e) => app.err(e),
            }
            app.mode = AppMode::Browse;
        }
        _ => {}
    }
}

fn handle_kg_entity_delete(app: &mut App, code: KeyCode) {
    let id = match &app.mode {
        AppMode::KgEntityDelete(id) => id.clone(),
        _ => return,
    };
    match code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            match storage::delete_entity(&id) {
                Ok(()) => {
                    app.ok(format!("Entity '{}' deleted", id));
                    if app.kg_branch == "*" {
                        app.reload_kg_all();
                    } else {
                        app.reload_kg();
                    }
                    app.reload_kg_branches();
                    // clamp selection
                    let len = app.kg_entities.len();
                    match app.kg_entity_state.selected() {
                        Some(i) if i >= len && len > 0 => app.kg_entity_state.select(Some(len - 1)),
                        Some(_) if len == 0 => app.kg_entity_state.select(None),
                        None if len > 0 => app.kg_entity_state.select(Some(0)),
                        _ => {}
                    }
                }
                Err(e) => app.err(e),
            }
            app.mode = AppMode::Browse;
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.mode = AppMode::Browse;
        }
        _ => {}
    }
}

fn handle_kg_edge_delete(app: &mut App, code: KeyCode) {
    let id = match &app.mode {
        AppMode::KgEdgeDelete(id) => id.clone(),
        _ => return,
    };
    match code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            match storage::delete_edge(&id) {
                Ok(()) => {
                    app.ok(format!("Edge '{}' deleted", id));
                    if app.kg_branch == "*" {
                        app.reload_kg_all();
                    } else {
                        app.reload_kg();
                    }
                    let len = app.kg_edges.len();
                    match app.kg_entity_state.selected() {
                        Some(i) if i >= len && len > 0 => app.kg_entity_state.select(Some(len - 1)),
                        Some(_) if len == 0 => app.kg_entity_state.select(None),
                        None if len > 0 => app.kg_entity_state.select(Some(0)),
                        _ => {}
                    }
                }
                Err(e) => app.err(e),
            }
            app.mode = AppMode::Browse;
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.mode = AppMode::Browse;
        }
        _ => {}
    }
}

fn handle_kg_branch_picker(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Down | KeyCode::Char('j') => {
            if !app.kg_branches.is_empty() {
                app.kg_branch_idx = (app.kg_branch_idx + 1) % app.kg_branches.len();
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if !app.kg_branches.is_empty() {
                app.kg_branch_idx =
                    (app.kg_branch_idx + app.kg_branches.len() - 1) % app.kg_branches.len();
            }
        }
        KeyCode::Enter => {
            if let Some(branch) = app.kg_branches.get(app.kg_branch_idx).cloned() {
                app.kg_branch = branch.clone();
                app.reload_kg();
                app.ok(format!("Switched to branch '{}'", branch));
            }
            app.mode = AppMode::Browse;
        }
        KeyCode::Char('a') => {
            app.kg_branch = "*".to_string();
            app.reload_kg_all();
            app.ok("Showing all branches");
            app.mode = AppMode::Browse;
        }
        KeyCode::Esc => {
            app.mode = AppMode::Browse;
        }
        _ => {}
    }
}

// ─── Drawing ──────────────────────────────────────────────────────────────────

fn draw_ui(
    f: &mut ratatui::Frame,
    app: &mut App,
    cfg: &crate::trace::types::TraceConfig,
) {
    let area = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(area);

    // Tab bar
    let titles: Vec<Line> = TraceTab::all().iter().map(|t| Line::from(t.title())).collect();
    let tabs = Tabs::new(titles)
        .select(app.tab.index())
        .block(Block::default().borders(Borders::ALL).title(" Trace Memory "))
        .style(Style::default().fg(Color::Gray))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(tabs, chunks[0]);

    // Main content area
    match &app.mode {
        AppMode::Browse => match app.tab {
            TraceTab::Overview => draw_overview(f, chunks[1], cfg, app),
            TraceTab::Sources => draw_sources(f, chunks[1], &mut app.sources_state),
            TraceTab::Notes => draw_notes_panel(f, chunks[1], app),
            TraceTab::Packages => draw_packages(f, chunks[1], app),
            TraceTab::EditLog => draw_edit_log(f, chunks[1], &app.audit, app.audit_scroll),
            TraceTab::Graph => draw_graph_panel(f, chunks[1], app),
        },
        AppMode::ViewDetail => draw_note_detail(f, chunks[1], app),
        AppMode::PackageDetail => draw_package_detail(f, chunks[1], app),
        AppMode::EditForm(_) | AppMode::DeleteConfirm(_) | AppMode::SourceDeleteConfirm(_) => {
            // Draw the current tab as background, then overlay the modal
            match app.tab {
                TraceTab::Notes => draw_notes_panel(f, chunks[1], app),
                TraceTab::Sources => draw_sources(f, chunks[1], &mut app.sources_state),
                _ => {}
            }
            match &app.mode {
                AppMode::EditForm(form) => {
                    let is_edit = form.is_edit;
                    let active = form.active_field;
                    let fields: Vec<(EditField, String)> = EditField::all()
                        .iter()
                        .map(|&fld| (fld, form.get_field(fld).to_string()))
                        .collect();
                    let status_clone = app.status.clone();
                    draw_form_modal(f, area, is_edit, active, &fields, status_clone.as_ref());
                }
                AppMode::DeleteConfirm(id) => {
                    let id = id.clone();
                    draw_delete_modal(f, area, &id);
                }
                AppMode::SourceDeleteConfirm(id) => {
                    let id = id.clone();
                    draw_source_delete_modal(f, area, &id);
                }
                _ => {}
            }
        }
        AppMode::KgEntityForm(_)
        | AppMode::KgEdgeForm(_)
        | AppMode::KgEntityDelete(_)
        | AppMode::KgEdgeDelete(_)
        | AppMode::KgBranchPicker => {
            draw_graph_panel(f, chunks[1], app);
            match &app.mode {
                AppMode::KgEntityForm(form) => {
                    let status_clone = app.status.clone();
                    draw_kg_entity_form_modal(f, area, form, status_clone.as_ref());
                }
                AppMode::KgEdgeForm(form) => {
                    let status_clone = app.status.clone();
                    draw_kg_edge_form_modal(f, area, form, status_clone.as_ref());
                }
                AppMode::KgEntityDelete(id) => {
                    let id = id.clone();
                    draw_kg_delete_modal(f, area, "entity", &id);
                }
                AppMode::KgEdgeDelete(id) => {
                    let id = id.clone();
                    draw_kg_delete_modal(f, area, "edge", &id);
                }
                AppMode::KgBranchPicker => {
                    let branches = app.kg_branches.clone();
                    let idx = app.kg_branch_idx;
                    let current = app.kg_branch.clone();
                    draw_kg_branch_picker(f, area, &branches, idx, &current);
                }
                _ => {}
            }
        }
    }

    // Status bar
    let help = match &app.mode {
        AppMode::ViewDetail => " ↑↓/jk: nav   e: edit   d: delete   Esc/q: back",
        AppMode::PackageDetail => " ↑↓/jk: nav   Esc/q: back to list",
        AppMode::EditForm(_) => " Tab: next field   Shift+Tab: prev   Enter: save   Esc: cancel",
        AppMode::DeleteConfirm(_) | AppMode::SourceDeleteConfirm(_)
        | AppMode::KgEntityDelete(_) | AppMode::KgEdgeDelete(_) => {
            " y: confirm delete   n/Esc: cancel"
        }
        AppMode::KgEntityForm(_) | AppMode::KgEdgeForm(_) => {
            " Tab: next  Shift+Tab: prev  Enter: save  Esc: cancel"
        }
        AppMode::KgBranchPicker => {
            " up/down: select  Enter: switch  a: all branches  Esc: cancel"
        }
        AppMode::Browse => match app.tab {
            TraceTab::Notes => {
                " ↑↓/jk: nav   Enter: view   n: new   e: edit   d: delete   r: reload   h/l: tabs   q: quit"
            }
            TraceTab::Sources => {
                " ↑↓/jk: nav   s: set default   d: remove   r: reload   h/l: tabs   q: quit"
            }
            TraceTab::Packages => {
                " ↑↓/jk: nav   Enter: detail   r: reload   h/l: tabs   q: quit"
            }
            TraceTab::EditLog => {
                " ↑↓/jk: scroll   g: top   G: bottom   r: reload   h/l: tabs   q: quit"
            }
            TraceTab::Graph => match app.kg_view {
                KgView::Entities => {
                    " up/down: nav  n: new  Enter: edit  d: delete  b: branch  a: all  B: build  Tab: view  r: reload  q: quit"
                }
                KgView::Edges => {
                    " up/down: nav  n: new  Enter: edit  d: delete  b: branch  a: all  Tab: view  r: reload  q: quit"
                }
                KgView::Visual => {
                    " up/down: nav  b: branch  a: all  B: build  Tab: view  e/w/v: switch  r: reload  q: quit"
                }
            }
            _ => " 1-6: tabs   h/l: switch tab   q: quit",
        },
    };
    let status_text = match &app.status {
        Some((msg, is_err)) => {
            let prefix = if *is_err { "✗ " } else { "✓ " };
            format!(" {} {}  │{}", prefix, msg, help)
        }
        None => help.to_string(),
    };
    let status_style = match &app.status {
        Some((_, true)) => Style::default().fg(Color::Red),
        Some((_, false)) => Style::default().fg(Color::Green),
        None => Style::default().fg(Color::DarkGray),
    };
    let status_bar = Paragraph::new(status_text)
        .style(status_style)
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(status_bar, chunks[2]);
}

fn draw_overview(
    f: &mut ratatui::Frame,
    area: Rect,
    cfg: &crate::trace::types::TraceConfig,
    app: &App,
) {
    let src_count = cfg.sources.len().to_string();
    let note_count = app.notes.len().to_string();
    let pkg_count = app.packages.len().to_string();
    let lines = vec![
        Line::from(""),
        kv("  Repo          ", &cfg.repo_name),
        kv("  Owner         ", &cfg.owner),
        kv(
            "  Default user  ",
            cfg.default_user.as_deref().unwrap_or("-"),
        ),
        kv("  Sources       ", &src_count),
        kv("  Notes         ", &note_count),
        kv("  Pkg findings  ", &pkg_count),
        Line::from(""),
        Line::from(Span::styled(
            "  Tab 5 → Edit Log shows a full audit trail of every create / edit / delete via TUI.",
            Style::default().fg(Color::DarkGray),
        )),
    ];
    let p = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Overview "));
    f.render_widget(p, area);
}

fn draw_sources(f: &mut ratatui::Frame, area: Rect, state: &mut ListState) {
    let cfg = storage::load_config().unwrap_or_default();
    let items: Vec<ListItem> = if cfg.sources.is_empty() {
        vec![ListItem::new(
            "  No sources configured. Run: infynon trace source add-redis / add-sql",
        )]
    } else {
        cfg.sources
            .iter()
            .map(|src| {
                let is_default = cfg.default_source.as_deref() == Some(src.id.as_str());
                let def_span = if is_default {
                    Span::styled(" ★", Style::default().fg(Color::Green))
                } else {
                    Span::raw("  ")
                };
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("  {:<18} ", src.id),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(
                        format!("[{:<8}]", src.kind.as_str()),
                        Style::default().fg(Color::Yellow),
                    ),
                    def_span,
                    Span::raw(format!(
                        "  user: {:<14}  {}",
                        src.owner_user.as_deref().unwrap_or("-"),
                        src.url
                    )),
                ]))
            })
            .collect()
    };
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Sources  [ s=set default  d=remove  r=reload ] "),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    if cfg.sources.is_empty() {
        f.render_widget(list, area);
    } else {
        f.render_stateful_widget(list, area, state);
    }
}

fn draw_notes_panel(f: &mut ratatui::Frame, area: Rect, app: &mut App) {
    let items: Vec<ListItem> = if app.notes.is_empty() {
        vec![ListItem::new(
            "  No notes yet.  Press  n  to create one.",
        )]
    } else {
        app.notes
            .iter()
            .map(|n| {
                let sc = match n.status {
                    NoteStatus::Active => Color::Green,
                    NoteStatus::Stale => Color::Yellow,
                    NoteStatus::Archived => Color::DarkGray,
                };
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("  {:<16} ", n.id),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(
                        format!("[{:<9}]", n.layer.as_str()),
                        Style::default().fg(Color::Blue),
                    ),
                    Span::styled(
                        format!(" {:<9}", n.scope.as_str()),
                        Style::default().fg(Color::Magenta),
                    ),
                    Span::styled(
                        format!(" {:<10}", n.status.as_str()),
                        Style::default().fg(sc),
                    ),
                    Span::styled(
                        format!(" {:<12} ", n.author),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw(&n.title),
                ]))
            })
            .collect()
    };
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Notes  [ n=new  e=edit  d=delete  Enter=view ] "),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    f.render_stateful_widget(list, area, &mut app.list_state);
}

fn draw_note_detail(f: &mut ratatui::Frame, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(38), Constraint::Percentage(62)])
        .split(area);

    draw_notes_panel(f, chunks[0], app);

    if let Some(note) = app.selected_note().cloned() {
        let tags_joined = note.tags.join(", ");
        let updated_short = note.updated_at[..note.updated_at.len().min(19)].to_string();
        let lines = vec![
            Line::from(""),
            kv("  ID:       ", &note.id),
            kv("  Title:    ", &note.title),
            kv("  Layer:    ", note.layer.as_str()),
            kv("  Scope:    ", note.scope.as_str()),
            kv("  Target:   ", &note.target),
            kv("  Author:   ", &note.author),
            kv("  Status:   ", note.status.as_str()),
            kv("  Tags:     ", &tags_joined),
            kv("  Updated:  ", &updated_short),
            Line::from(""),
            Line::from(Span::styled(
                "  Body:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from("  ─────────────────────────────────────────────"),
            Line::from(format!("  {}", note.body)),
        ];
        let p = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Note Detail  [ Esc: back ] "),
            )
            .wrap(Wrap { trim: false });
        f.render_widget(p, chunks[1]);
    }
}

fn draw_packages(f: &mut ratatui::Frame, area: Rect, app: &mut App) {
    let items: Vec<ListItem> = if app.packages.is_empty() {
        vec![ListItem::new(
            "  No vulnerable packages found, or no lock files detected in this directory.",
        )]
    } else {
        app.packages
            .iter()
            .map(|r| {
                let sev_color = sev_color(r.severity.as_str());
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("  {:<9}", r.severity),
                        Style::default().fg(sev_color).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("{}@{}  ", r.package, r.version),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(
                        format!("{:<22}", r.vulnerability_id),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw(format!(
                        "owner: {}",
                        r.installed_by.as_deref().unwrap_or("unknown")
                    )),
                ]))
            })
            .collect()
    };
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Package Risks  [ Enter=detail  r=reload ] "),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    if app.packages.is_empty() {
        f.render_widget(list, area);
    } else {
        f.render_stateful_widget(list, area, &mut app.packages_state);
    }
}

fn draw_package_detail(f: &mut ratatui::Frame, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    draw_packages(f, chunks[0], app);

    if let Some(r) = app.selected_package().cloned() {
        let sc = sev_color(r.severity.as_str());
        let pkg_ver = format!("{}@{}", r.package, r.version);
        let lines = vec![
            Line::from(""),
            kv("  Vulnerability:  ", &r.vulnerability_id),
            kv("  Package:        ", &pkg_ver),
            kv("  Ecosystem:      ", &r.ecosystem),
            Line::from(vec![
                Span::styled(
                    "  Severity:       ",
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    r.severity.clone(),
                    Style::default().fg(sc).add_modifier(Modifier::BOLD),
                ),
            ]),
            kv("  Source file:    ", &r.source_file),
            kv(
                "  Installed by:   ",
                r.installed_by.as_deref().unwrap_or("unknown"),
            ),
            Line::from(""),
            Line::from(Span::styled(
                "  Press Esc to go back.",
                Style::default().fg(Color::DarkGray),
            )),
        ];
        let p = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Package Detail  [ Esc: back ] "),
            )
            .wrap(Wrap { trim: false });
        f.render_widget(p, chunks[1]);
    }
}

fn sev_color(sev: &str) -> Color {
    match sev {
        "CRITICAL" => Color::Red,
        "HIGH" => Color::LightRed,
        "MEDIUM" => Color::Yellow,
        _ => Color::Gray,
    }
}

fn draw_edit_log(f: &mut ratatui::Frame, area: Rect, entries: &[AuditEntry], scroll: usize) {
    let title = if entries.is_empty() {
        " Edit Log ".to_string()
    } else {
        format!(
            " Edit Log  — {} entries  ({}/{}) ",
            entries.len(),
            scroll + 1,
            entries.len()
        )
    };

    let items: Vec<ListItem> = if entries.is_empty() {
        vec![ListItem::new(
            "  No TUI edits recorded yet. Create, edit, or delete a note to begin the audit trail.",
        )]
    } else {
        entries
            .iter()
            .skip(scroll)
            .map(|e| {
                let ac = match e.action.as_str() {
                    "create" => Color::Green,
                    "edit" => Color::Yellow,
                    "delete" => Color::Red,
                    _ => Color::Gray,
                };
                let ts = &e.timestamp[..e.timestamp.len().min(19)];
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("  {} ", ts),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        format!("{:<7} ", e.action.to_uppercase()),
                        Style::default().fg(ac).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("{:<18} ", e.note_id),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(
                        format!("by {:<14} ", e.author),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw(&e.summary),
                ]))
            })
            .collect()
    };
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(list, area);
}

fn draw_form_modal(
    f: &mut ratatui::Frame,
    area: Rect,
    is_edit: bool,
    active: EditField,
    fields: &[(EditField, String)],
    status: Option<&(String, bool)>,
) {
    let popup = centered_rect(78, 90, area);
    f.render_widget(Clear, popup);

    let title = if is_edit {
        " Edit Note  [ Tab: next   Enter: save   Esc: cancel ] "
    } else {
        " New Note   [ Tab: next   Enter: save   Esc: cancel ] "
    };

    let mut lines: Vec<Line> = vec![Line::from("")];

    for (fld, val) in fields {
        let fld = *fld;
        // Skip ID field when editing (it's immutable)
        if is_edit && fld == EditField::Id {
            lines.push(Line::from(vec![Span::styled(
                format!("  ID (locked): {}", val),
                Style::default().fg(Color::DarkGray),
            )]));
            lines.push(Line::from(""));
            continue;
        }

        let is_active = fld == active;
        let lbl_style = if is_active {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Yellow)
        };
        let val_style = if is_active {
            Style::default().fg(Color::White).bg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Gray)
        };

        lines.push(Line::from(vec![Span::styled(
            format!("  {} ", fld.label()),
            lbl_style,
        )]));
        lines.push(Line::from(vec![Span::styled(
            format!("  {}█ ", val),
            val_style,
        )]));
        lines.push(Line::from(""));
    }

    if let Some((msg, is_err)) = status {
        let style = if *is_err {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Green)
        };
        lines.push(Line::from(vec![Span::styled(
            format!("  ⚠  {} ", msg),
            style,
        )]));
    }

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(p, popup);
}

fn draw_delete_modal(f: &mut ratatui::Frame, area: Rect, id: &str) {
    let popup = centered_rect(48, 22, area);
    f.render_widget(Clear, popup);
    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Delete note: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                id,
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  This action cannot be undone.",
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  [y]  confirm delete",
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("      "),
            Span::styled("[n / Esc]  cancel", Style::default().fg(Color::Green)),
        ]),
    ];
    let p = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Confirm Delete ")
            .border_style(Style::default().fg(Color::Red)),
    );
    f.render_widget(p, popup);
}

fn draw_source_delete_modal(f: &mut ratatui::Frame, area: Rect, id: &str) {
    let popup = centered_rect(48, 22, area);
    f.render_widget(Clear, popup);
    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Remove source: ", Style::default().fg(Color::Yellow)),
            Span::styled(id, Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  This will remove the backend from the local config.",
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  [y]  confirm remove",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw("      "),
            Span::styled("[n / Esc]  cancel", Style::default().fg(Color::Green)),
        ]),
    ];
    let p = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Confirm Remove Source ")
            .border_style(Style::default().fg(Color::Red)),
    );
    f.render_widget(p, popup);
}

// ─── Knowledge graph panels ──────────────────────────────────────────────────

fn draw_graph_panel(f: &mut ratatui::Frame, area: Rect, app: &mut App) {
    match app.kg_view {
        KgView::Entities => draw_kg_entities(f, area, app),
        KgView::Edges => draw_kg_edges(f, area, app),
        KgView::Visual => draw_kg_visual(f, area, app),
    }
}

fn entity_kind_color(kind: &EntityKind) -> Color {
    match kind {
        EntityKind::File => Color::Cyan,
        EntityKind::Package => Color::Yellow,
        EntityKind::Person => Color::Green,
        EntityKind::Decision => Color::Magenta,
        EntityKind::Vulnerability => Color::Red,
        EntityKind::Endpoint => Color::LightBlue,
        EntityKind::Module => Color::LightCyan,
        EntityKind::Pr => Color::LightYellow,
        EntityKind::Branch => Color::LightGreen,
        EntityKind::Note => Color::Gray,
    }
}

fn draw_kg_entities(f: &mut ratatui::Frame, area: Rect, app: &mut App) {
    let branch_label = if app.kg_branch == "*" {
        "all".to_string()
    } else {
        app.kg_branch.clone()
    };
    let title = format!(
        " Graph Entities [{}]  [ n=new  Enter=edit  d=delete  b=branch  a=all  B=build ] ",
        branch_label
    );
    let items: Vec<ListItem> = if app.kg_entities.is_empty() {
        vec![ListItem::new(
            "  No entities in knowledge graph. Add entities with: infynon trace kg entity add",
        )]
    } else {
        app.kg_entities
            .iter()
            .map(|e| {
                let kc = entity_kind_color(&e.kind);
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("  {:<14} ", e.kind.as_str()),
                        Style::default().fg(kc).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("{:<30} ", e.name),
                        Style::default().fg(Color::White),
                    ),
                    Span::styled(
                        format!("{}", e.branch),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]))
            })
            .collect()
    };
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    if app.kg_entities.is_empty() {
        f.render_widget(list, area);
    } else {
        f.render_stateful_widget(list, area, &mut app.kg_entity_state);
    }
}

fn draw_kg_edges(f: &mut ratatui::Frame, area: Rect, app: &mut App) {
    let branch_label = if app.kg_branch == "*" {
        "all".to_string()
    } else {
        app.kg_branch.clone()
    };
    let title = format!(
        " Graph Edges [{}]  [ n=new  Enter=edit  d=delete  b=branch  a=all ] ",
        branch_label
    );
    let items: Vec<ListItem> = if app.kg_edges.is_empty() {
        vec![ListItem::new(
            "  No edges in knowledge graph. Add edges with: infynon trace kg edge add",
        )]
    } else {
        app.kg_edges
            .iter()
            .map(|e| {
                let rel_color = match e.relation.as_str() {
                    "depends_on" => Color::Yellow,
                    "modified_by" => Color::Green,
                    "introduced_by" => Color::Red,
                    "exposes" => Color::LightRed,
                    "tested_by" => Color::Cyan,
                    _ => Color::Gray,
                };
                let evidence_short = if e.evidence.len() > 30 {
                    format!("{}...", &e.evidence[..27])
                } else {
                    e.evidence.clone()
                };
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("  {:<20}", e.source),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(
                        " \u{2192} ",
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        format!("{:<20} ", e.target),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(
                        format!("({}) ", e.relation.as_str()),
                        Style::default().fg(rel_color).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("[{:.1}] ", e.weight),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::styled(
                        evidence_short,
                        Style::default().fg(Color::DarkGray),
                    ),
                ]))
            })
            .collect()
    };
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    if app.kg_edges.is_empty() {
        f.render_widget(list, area);
    } else {
        f.render_stateful_widget(list, area, &mut app.kg_entity_state);
    }
}

fn draw_kg_visual(f: &mut ratatui::Frame, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // Left: entity list
    draw_kg_entities(f, chunks[0], app);

    // Right: visual graph
    let selected_id = app
        .kg_selected_entity
        .and_then(|i| app.kg_entities.get(i))
        .map(|e| e.id.clone());

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));

    // Group entities by kind
    let mut grouped: std::collections::BTreeMap<String, Vec<&KgEntity>> =
        std::collections::BTreeMap::new();
    for ent in &app.kg_entities {
        grouped
            .entry(ent.kind.as_str().to_string())
            .or_default()
            .push(ent);
    }

    let max_lines = chunks[1].height.saturating_sub(4) as usize;
    let mut line_count = 0;

    for (kind, entities) in &grouped {
        if line_count >= max_lines {
            break;
        }
        // Section header
        let header = format!("  \u{250c}\u{2500} {} ", kind);
        let pad = (chunks[1].width as usize).saturating_sub(header.len() + 3);
        lines.push(Line::from(Span::styled(
            format!("{}{}\u{2510}", header, "\u{2500}".repeat(pad)),
            Style::default().fg(Color::DarkGray),
        )));
        line_count += 1;

        for ent in entities {
            if line_count >= max_lines {
                break;
            }
            let kc = entity_kind_color(&ent.kind);
            let is_selected = selected_id.as_deref() == Some(&ent.id);
            let name_style = if is_selected {
                Style::default().fg(kc).add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else {
                Style::default().fg(kc)
            };

            // Find outgoing edges for this entity
            let outgoing: Vec<&KgEdge> = app
                .kg_edges
                .iter()
                .filter(|edge| edge.source == ent.id)
                .collect();

            if outgoing.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("  \u{2502}  ", Style::default().fg(Color::DarkGray)),
                    Span::styled(format!("[{}]", ent.name), name_style),
                ]));
                line_count += 1;
            } else {
                for edge in &outgoing {
                    if line_count >= max_lines {
                        break;
                    }
                    let rel_color = match edge.relation.as_str() {
                        "depends_on" => Color::Yellow,
                        "modified_by" => Color::Green,
                        "introduced_by" => Color::Red,
                        "exposes" => Color::LightRed,
                        "tested_by" => Color::Cyan,
                        _ => Color::Gray,
                    };
                    // Find target entity name
                    let target_name = app
                        .kg_entities
                        .iter()
                        .find(|e| e.id == edge.target)
                        .map(|e| e.name.as_str())
                        .unwrap_or(&edge.target);
                    let target_kind = app
                        .kg_entities
                        .iter()
                        .find(|e| e.id == edge.target)
                        .map(|e| &e.kind);
                    let tc = target_kind.map(entity_kind_color).unwrap_or(Color::White);

                    lines.push(Line::from(vec![
                        Span::styled("  \u{2502}  ", Style::default().fg(Color::DarkGray)),
                        Span::styled(format!("[{}]", ent.name), name_style),
                        Span::styled(
                            format!(" \u{2500}\u{2500}{}\u{2500}\u{2500}\u{25b6} ", edge.relation.as_str()),
                            Style::default().fg(rel_color),
                        ),
                        Span::styled(format!("[{}]", target_name), Style::default().fg(tc)),
                    ]));
                    line_count += 1;
                }
            }
        }

        if line_count < max_lines {
            let footer_pad = (chunks[1].width as usize).saturating_sub(5);
            lines.push(Line::from(Span::styled(
                format!("  \u{2514}{}\u{2518}", "\u{2500}".repeat(footer_pad)),
                Style::default().fg(Color::DarkGray),
            )));
            line_count += 1;
        }
    }

    if app.kg_entities.is_empty() && app.kg_edges.is_empty() {
        lines.push(Line::from(Span::styled(
            "  Empty knowledge graph.",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let vg_branch_label = if app.kg_branch == "*" {
        "all".to_string()
    } else {
        app.kg_branch.clone()
    };
    let title = format!(
        " Visual Graph [{}] ",
        vg_branch_label
    );
    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(p, chunks[1]);
}

// ─── KG modals ──────────────────────────────────────────────────────────────

fn draw_kg_entity_form_modal(
    f: &mut ratatui::Frame,
    area: Rect,
    form: &KgEntityForm,
    status: Option<&(String, bool)>,
) {
    let popup = centered_rect(78, 70, area);
    f.render_widget(Clear, popup);

    let title = if form.is_edit {
        " Edit Entity  [ Tab: next   Enter: save   Esc: cancel ] "
    } else {
        " New Entity   [ Tab: next   Enter: save   Esc: cancel ] "
    };

    let mut lines: Vec<Line> = vec![Line::from("")];

    for &fld in &KgEntityField::all() {
        let is_active = fld == form.active_field;
        let lbl_style = if is_active {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Yellow)
        };
        let val_style = if is_active {
            Style::default().fg(Color::White).bg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Gray)
        };

        lines.push(Line::from(vec![Span::styled(
            format!("  {} ", fld.label()),
            lbl_style,
        )]));
        lines.push(Line::from(vec![Span::styled(
            format!("  {}\u{2588} ", form.get_field(fld)),
            val_style,
        )]));
        lines.push(Line::from(""));
    }

    if let Some((msg, is_err)) = status {
        let style = if *is_err {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Green)
        };
        lines.push(Line::from(vec![Span::styled(
            format!("  !  {} ", msg),
            style,
        )]));
    }

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(p, popup);
}

fn draw_kg_edge_form_modal(
    f: &mut ratatui::Frame,
    area: Rect,
    form: &KgEdgeForm,
    status: Option<&(String, bool)>,
) {
    let popup = centered_rect(78, 90, area);
    f.render_widget(Clear, popup);

    let title = if form.is_edit {
        " Edit Edge  [ Tab: next   Enter: save   Esc: cancel ] "
    } else {
        " New Edge   [ Tab: next   Enter: save   Esc: cancel ] "
    };

    let mut lines: Vec<Line> = vec![Line::from("")];

    for &fld in &KgEdgeField::all() {
        let is_active = fld == form.active_field;
        let lbl_style = if is_active {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Yellow)
        };
        let val_style = if is_active {
            Style::default().fg(Color::White).bg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Gray)
        };

        lines.push(Line::from(vec![Span::styled(
            format!("  {} ", fld.label()),
            lbl_style,
        )]));
        lines.push(Line::from(vec![Span::styled(
            format!("  {}\u{2588} ", form.get_field(fld)),
            val_style,
        )]));
        lines.push(Line::from(""));
    }

    if let Some((msg, is_err)) = status {
        let style = if *is_err {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Green)
        };
        lines.push(Line::from(vec![Span::styled(
            format!("  !  {} ", msg),
            style,
        )]));
    }

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(p, popup);
}

fn draw_kg_delete_modal(f: &mut ratatui::Frame, area: Rect, entity_type: &str, id: &str) {
    let popup = centered_rect(48, 22, area);
    f.render_widget(Clear, popup);
    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("  Delete {}: ", entity_type),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                id,
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  This action cannot be undone.",
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  [y]  confirm delete",
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("      "),
            Span::styled("[n / Esc]  cancel", Style::default().fg(Color::Green)),
        ]),
    ];
    let p = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Confirm Delete ")
            .border_style(Style::default().fg(Color::Red)),
    );
    f.render_widget(p, popup);
}

fn draw_kg_branch_picker(
    f: &mut ratatui::Frame,
    area: Rect,
    branches: &[String],
    selected: usize,
    current: &str,
) {
    let popup = centered_rect(50, 60, area);
    f.render_widget(Clear, popup);

    let items: Vec<ListItem> = branches
        .iter()
        .enumerate()
        .map(|(i, b)| {
            let is_current = b == current;
            let marker = if is_current { " *" } else { "" };
            let style = if i == selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else if is_current {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Line::from(Span::styled(
                format!("  {}{}", b, marker),
                style,
            )))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Branch Picker  [ Enter: switch   a: all   Esc: cancel ] ")
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(list, popup);
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn kv<'a>(label: &'a str, value: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled(label, Style::default().fg(Color::Yellow)),
        Span::raw(value),
    ])
}

fn centered_rect(pct_x: u16, pct_y: u16, r: Rect) -> Rect {
    let margin_v = (100 - pct_y) / 2;
    let margin_h = (100 - pct_x) / 2;
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(margin_v),
            Constraint::Percentage(pct_y),
            Constraint::Percentage(margin_v),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(margin_h),
            Constraint::Percentage(pct_x),
            Constraint::Percentage(margin_h),
        ])
        .split(vert[1])[1]
}

fn parse_layer(v: &str) -> Result<TraceLayer, String> {
    v.parse().map_err(|_| format!("Invalid layer '{}'. Use canonical | team | user", v))
}

fn parse_scope(v: &str) -> Result<TraceScope, String> {
    v.parse().map_err(|_| format!("Invalid scope '{}'. Use repo | branch | pr | file | user | session | package", v))
}
