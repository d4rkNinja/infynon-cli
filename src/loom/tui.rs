use crate::loom::{
    storage,
    types::{LoomLayer, LoomNote, LoomScope, NoteStatus, PackageRisk},
};
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
enum LoomTab {
    Overview,
    Sources,
    Notes,
    Packages,
    EditLog,
}

impl LoomTab {
    fn all() -> [LoomTab; 5] {
        [
            LoomTab::Overview,
            LoomTab::Sources,
            LoomTab::Notes,
            LoomTab::Packages,
            LoomTab::EditLog,
        ]
    }
    fn title(&self) -> &'static str {
        match self {
            LoomTab::Overview => "Overview",
            LoomTab::Sources => "Sources",
            LoomTab::Notes => "Notes",
            LoomTab::Packages => "Packages",
            LoomTab::EditLog => "Edit Log",
        }
    }
    fn index(&self) -> usize {
        LoomTab::all().iter().position(|t| t == self).unwrap_or(0)
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

    fn from_note(note: &LoomNote) -> Self {
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

// ─── App mode ─────────────────────────────────────────────────────────────────

enum AppMode {
    Browse,
    ViewDetail,
    EditForm(NoteForm),
    DeleteConfirm(String),
    SourceDeleteConfirm(String),
    PackageDetail,
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
    storage::loom_dir().join("state").join("tui_edits.jsonl")
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

// ─── App state ────────────────────────────────────────────────────────────────

struct App {
    tab: LoomTab,
    notes: Vec<LoomNote>,
    packages: Vec<PackageRisk>,
    list_state: ListState,
    sources_state: ListState,
    packages_state: ListState,
    audit_scroll: usize,
    mode: AppMode,
    status: Option<(String, bool)>, // (message, is_error)
    audit: Vec<AuditEntry>,
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

    fn selected_idx(&self) -> Option<usize> {
        self.list_state.selected()
    }

    fn selected_note(&self) -> Option<&LoomNote> {
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

// ─── Public entry point ───────────────────────────────────────────────────────

pub fn run() {
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
        tab: LoomTab::Notes,
        notes,
        packages,
        list_state,
        sources_state,
        packages_state,
        audit_scroll: 0,
        mode: AppMode::Browse,
        status: None,
        audit,
    };

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
        KeyCode::Char('1') => { app.tab = LoomTab::Overview; app.mode = AppMode::Browse; }
        KeyCode::Char('2') => { app.tab = LoomTab::Sources; app.mode = AppMode::Browse; }
        KeyCode::Char('3') => { app.tab = LoomTab::Notes; app.mode = AppMode::Browse; }
        KeyCode::Char('4') => { app.tab = LoomTab::Packages; app.mode = AppMode::Browse; }
        KeyCode::Char('5') => {
            app.tab = LoomTab::EditLog;
            app.mode = AppMode::Browse;
            app.reload_audit();
        }
        KeyCode::Right | KeyCode::Char('l')
            if !matches!(app.mode, AppMode::ViewDetail | AppMode::PackageDetail) =>
        {
            let next = (app.tab.index() + 1) % 5;
            app.tab = LoomTab::all()[next];
            app.mode = AppMode::Browse;
            if app.tab == LoomTab::EditLog {
                app.reload_audit();
            }
        }
        KeyCode::Left | KeyCode::Char('h')
            if !matches!(app.mode, AppMode::ViewDetail | AppMode::PackageDetail) =>
        {
            let prev = (app.tab.index() + 5 - 1) % 5;
            app.tab = LoomTab::all()[prev];
            app.mode = AppMode::Browse;
        }

        // ── Notes tab ────────────────────────────────────────────────────────
        KeyCode::Down | KeyCode::Char('j') if app.tab == LoomTab::Notes => {
            let len = app.notes.len();
            if len > 0 {
                let next = app.selected_idx().map(|i| (i + 1) % len).unwrap_or(0);
                app.list_state.select(Some(next));
            }
        }
        KeyCode::Up | KeyCode::Char('k') if app.tab == LoomTab::Notes => {
            let len = app.notes.len();
            if len > 0 {
                let prev = app
                    .selected_idx()
                    .map(|i| if i == 0 { len - 1 } else { i - 1 })
                    .unwrap_or(0);
                app.list_state.select(Some(prev));
            }
        }
        KeyCode::Enter if app.tab == LoomTab::Notes => {
            if app.selected_note().is_some() {
                app.mode = AppMode::ViewDetail;
            }
        }
        KeyCode::Char('n') if app.tab == LoomTab::Notes => {
            let author = storage::configured_user()
                .or_else(storage::detect_user_name)
                .unwrap_or_else(|| "unknown".to_string());
            app.mode = AppMode::EditForm(NoteForm::new_create(author));
        }
        KeyCode::Char('e') if app.tab == LoomTab::Notes => {
            if let Some(note) = app.selected_note() {
                let form = NoteForm::from_note(note);
                app.mode = AppMode::EditForm(form);
            }
        }
        KeyCode::Char('d') if app.tab == LoomTab::Notes => {
            if let Some(note) = app.selected_note() {
                let id = note.id.clone();
                app.mode = AppMode::DeleteConfirm(id);
            }
        }
        KeyCode::Char('r') if app.tab == LoomTab::Notes => {
            app.reload_notes();
            app.clamp_selection();
            app.ok("Notes reloaded");
        }

        // ── Sources tab ──────────────────────────────────────────────────────
        KeyCode::Down | KeyCode::Char('j') if app.tab == LoomTab::Sources => {
            let cfg = storage::load_config().unwrap_or_default();
            let len = cfg.sources.len();
            if len > 0 {
                let next = app.sources_state.selected().map(|i| (i + 1) % len).unwrap_or(0);
                app.sources_state.select(Some(next));
            }
        }
        KeyCode::Up | KeyCode::Char('k') if app.tab == LoomTab::Sources => {
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
        KeyCode::Char('d') if app.tab == LoomTab::Sources => {
            let cfg = storage::load_config().unwrap_or_default();
            if let Some(idx) = app.sources_state.selected() {
                if let Some(src) = cfg.sources.get(idx) {
                    app.mode = AppMode::SourceDeleteConfirm(src.id.clone());
                }
            }
        }
        KeyCode::Char('s') if app.tab == LoomTab::Sources => {
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
        KeyCode::Char('r') if app.tab == LoomTab::Sources => {
            app.reload_sources();
            app.ok("Sources reloaded");
        }

        // ── Packages tab ─────────────────────────────────────────────────────
        KeyCode::Down | KeyCode::Char('j') if app.tab == LoomTab::Packages => {
            let len = app.packages.len();
            if len > 0 {
                let next = app.packages_state.selected().map(|i| (i + 1) % len).unwrap_or(0);
                app.packages_state.select(Some(next));
            }
        }
        KeyCode::Up | KeyCode::Char('k') if app.tab == LoomTab::Packages => {
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
        KeyCode::Enter if app.tab == LoomTab::Packages => {
            if app.selected_package().is_some() {
                app.mode = AppMode::PackageDetail;
            }
        }
        KeyCode::Char('r') if app.tab == LoomTab::Packages => {
            app.reload_packages();
            app.ok("Package risks reloaded");
        }

        // ── EditLog tab ──────────────────────────────────────────────────────
        KeyCode::Down | KeyCode::Char('j') if app.tab == LoomTab::EditLog => {
            if app.audit_scroll + 1 < app.audit.len() {
                app.audit_scroll += 1;
            }
        }
        KeyCode::Up | KeyCode::Char('k') if app.tab == LoomTab::EditLog => {
            app.audit_scroll = app.audit_scroll.saturating_sub(1);
        }
        KeyCode::Char('g') if app.tab == LoomTab::EditLog => {
            app.audit_scroll = 0;
        }
        KeyCode::Char('G') if app.tab == LoomTab::EditLog => {
            app.audit_scroll = app.audit.len().saturating_sub(1);
        }
        KeyCode::Char('r') if app.tab == LoomTab::EditLog => {
            app.reload_audit();
            app.ok("Edit log reloaded");
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
                let note = LoomNote {
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

// ─── Drawing ──────────────────────────────────────────────────────────────────

fn draw_ui(
    f: &mut ratatui::Frame,
    app: &mut App,
    cfg: &crate::loom::types::LoomConfig,
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
    let titles: Vec<Line> = LoomTab::all().iter().map(|t| Line::from(t.title())).collect();
    let tabs = Tabs::new(titles)
        .select(app.tab.index())
        .block(Block::default().borders(Borders::ALL).title(" Loom Memory "))
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
            LoomTab::Overview => draw_overview(f, chunks[1], cfg, app),
            LoomTab::Sources => draw_sources(f, chunks[1], &mut app.sources_state),
            LoomTab::Notes => draw_notes_panel(f, chunks[1], app),
            LoomTab::Packages => draw_packages(f, chunks[1], app),
            LoomTab::EditLog => draw_edit_log(f, chunks[1], &app.audit, app.audit_scroll),
        },
        AppMode::ViewDetail => draw_note_detail(f, chunks[1], app),
        AppMode::PackageDetail => draw_package_detail(f, chunks[1], app),
        AppMode::EditForm(_) | AppMode::DeleteConfirm(_) | AppMode::SourceDeleteConfirm(_) => {
            // Draw the current tab as background, then overlay the modal
            match app.tab {
                LoomTab::Notes => draw_notes_panel(f, chunks[1], app),
                LoomTab::Sources => draw_sources(f, chunks[1], &mut app.sources_state),
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
    }

    // Status bar
    let help = match &app.mode {
        AppMode::ViewDetail => " ↑↓/jk: nav   e: edit   d: delete   Esc/q: back",
        AppMode::PackageDetail => " ↑↓/jk: nav   Esc/q: back to list",
        AppMode::EditForm(_) => " Tab: next field   Shift+Tab: prev   Enter: save   Esc: cancel",
        AppMode::DeleteConfirm(_) | AppMode::SourceDeleteConfirm(_) => {
            " y: confirm delete   n/Esc: cancel"
        }
        AppMode::Browse => match app.tab {
            LoomTab::Notes => {
                " ↑↓/jk: nav   Enter: view   n: new   e: edit   d: delete   r: reload   h/l: tabs   q: quit"
            }
            LoomTab::Sources => {
                " ↑↓/jk: nav   s: set default   d: remove   r: reload   h/l: tabs   q: quit"
            }
            LoomTab::Packages => {
                " ↑↓/jk: nav   Enter: detail   r: reload   h/l: tabs   q: quit"
            }
            LoomTab::EditLog => {
                " ↑↓/jk: scroll   g: top   G: bottom   r: reload   h/l: tabs   q: quit"
            }
            _ => " 1-5: tabs   h/l: switch tab   q: quit",
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
    cfg: &crate::loom::types::LoomConfig,
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
            "  No sources configured. Run: infynon loom source add-redis / add-sql",
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

fn parse_layer(v: &str) -> Result<LoomLayer, String> {
    v.parse().map_err(|_| format!("Invalid layer '{}'. Use canonical | team | user", v))
}

fn parse_scope(v: &str) -> Result<LoomScope, String> {
    v.parse().map_err(|_| format!("Invalid scope '{}'. Use repo | branch | pr | file | user | session | package", v))
}
