use crate::ninja::storage;
use crate::ninja::types::NinjaManifest;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Wrap,
    },
    Terminal,
};
use serde_json::Value;
use std::io;
use std::process::Command;
use std::time::Duration;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Panel {
    Workspaces,
    Tasks,
}

#[derive(Clone, Copy)]
enum ActionKind {
    WorkspaceCreate,
    WorkspaceUpdate,
    WorkspaceAddFolder,
    WorkspaceRemoveFolder,
    WorkspaceRemove,
    AgentRootSet,
    TaskCreate,
    TaskUpdate,
    TaskStart,
    TaskResume,
    TaskNote,
    TaskResult,
    TaskComplete,
    TaskFail,
    TaskKill,
    TaskRemove,
}

struct Field {
    name: &'static str,
    prompt: &'static str,
    value: String,
    required: bool,
}

struct Form {
    title: String,
    action: ActionKind,
    fields: Vec<Field>,
    index: usize,
}

struct NinjaTui {
    manifest: NinjaManifest,
    panel: Panel,
    workspace_index: usize,
    task_index: usize,
    scroll: u16,
    message: String,
    form: Option<Form>,
}

impl NinjaTui {
    fn load() -> Result<Self, String> {
        Ok(Self {
            manifest: storage::load_manifest()?,
            panel: Panel::Workspaces,
            workspace_index: 0,
            task_index: 0,
            scroll: 0,
            message: help_text().to_string(),
            form: None,
        })
    }

    fn reload(&mut self) -> Result<(), String> {
        self.manifest = storage::load_manifest()?;
        self.clamp_selection();
        self.scroll = 0;
        Ok(())
    }

    fn clamp_selection(&mut self) {
        if self.workspace_index >= self.manifest.workspaces.len() {
            self.workspace_index = self.manifest.workspaces.len().saturating_sub(1);
        }
        if self.task_index >= self.manifest.tasks.len() {
            self.task_index = self.manifest.tasks.len().saturating_sub(1);
        }
    }

    fn selected_workspace_name(&self) -> Option<String> {
        self.manifest
            .workspaces
            .get(self.workspace_index)
            .map(|workspace| workspace.name.clone())
    }

    fn selected_task_id(&self) -> Option<String> {
        self.manifest
            .tasks
            .get(self.task_index)
            .map(|task| task.id.clone())
    }

    fn detail(&self) -> String {
        match self.panel {
            Panel::Workspaces => self.workspace_detail(),
            Panel::Tasks => self.task_detail(),
        }
    }

    fn workspace_detail(&self) -> String {
        let Some(summary) = self.manifest.workspaces.get(self.workspace_index) else {
            return "No workspace selected.".to_string();
        };
        let Ok(workspace) = storage::load_workspace(&summary.name) else {
            return format!("Workspace '{}' could not be loaded.", summary.name);
        };
        let folders = if workspace.folders.is_empty() {
            "none".to_string()
        } else {
            workspace
                .folders
                .iter()
                .map(|folder| format!("- {} -> {}", folder.folder_name, display_path(&folder.path)))
                .collect::<Vec<_>>()
                .join("\n")
        };
        format!(
            "Overview\n  Name        {}\n  Default     {}\n  Folder      {}\n  Path        {}\n  Description {}\n\nFolders\n{}\n\nModel Slots\n  Super lite       {} ({})\n  Lite             {} ({})\n  Frontier         {} ({})\n  Highest frontier {} ({})\n\nStorage\n  {}",
            workspace.name,
            yes_no(self.manifest.default_workspace.as_deref() == Some(workspace.name.as_str())),
            workspace.folder_name.as_deref().unwrap_or("none"),
            workspace.path.as_deref().map(display_path).unwrap_or_else(|| "none".to_string()),
            workspace.description.as_deref().unwrap_or("none"),
            folders,
            workspace.models.super_lite_model.model.as_deref().unwrap_or("unset"),
            workspace.models.super_lite_model.thinking,
            workspace.models.lite_model.model.as_deref().unwrap_or("unset"),
            workspace.models.lite_model.thinking,
            workspace.models.frontier_model.model.as_deref().unwrap_or("unset"),
            workspace.models.frontier_model.thinking,
            workspace.models.highest_frontier_model.model.as_deref().unwrap_or("unset"),
            workspace.models.highest_frontier_model.thinking,
            display_path(&storage::workspace_file(&workspace.name).display().to_string()),
        )
    }

    fn task_detail(&self) -> String {
        let Some(summary) = self.manifest.tasks.get(self.task_index) else {
            return "No task selected.".to_string();
        };
        let Ok(task) = storage::load_task(&summary.id) else {
            return format!("Task '{}' could not be loaded.", summary.id);
        };
        format!(
            "Overview\n  Task      {}\n  Status    {}\n  Workspace {}\n  Folder    {}\n  Agent     {}\n  Model     {}\n  Thinking  {}\n\nRuntime\n  PID       {}\n  Session   {}\n  Started   {}\n  Ended     {}\n\nLinks\n  Parent    {}\n  Blocked   {}\n  Reason    {}\n\nPrompt\n{}\n\nNotes\n{}\n\nResult\n{}\n\nStorage\n  {}",
            task.id,
            task.status,
            task.workspace.as_deref().unwrap_or("none"),
            task.folder_name.as_deref().unwrap_or("none"),
            task.agent.as_deref().unwrap_or("none"),
            task.model.as_deref().unwrap_or("none"),
            task.thinking.as_deref().unwrap_or("auto"),
            task.pid.map(|pid| pid.to_string()).unwrap_or_else(|| "none".to_string()),
            task.session_id.as_deref().unwrap_or("none"),
            task.started_at.as_deref().unwrap_or("none"),
            task.ended_at.as_deref().unwrap_or("none"),
            task.parent_task_id.as_deref().unwrap_or("none"),
            task.blocked_by.as_deref().unwrap_or("none"),
            task.blocked_reason.as_deref().unwrap_or("none"),
            task.prompt.as_deref().unwrap_or("none"),
            task.notes.as_deref().unwrap_or("none"),
            task.result.as_deref().unwrap_or("none"),
            display_path(&storage::task_file(&task.id).display().to_string()),
        )
    }

    fn move_selection(&mut self, delta: isize) {
        let len = match self.panel {
            Panel::Workspaces => self.manifest.workspaces.len(),
            Panel::Tasks => self.manifest.tasks.len(),
        };
        if len == 0 {
            return;
        }
        let current = match self.panel {
            Panel::Workspaces => self.workspace_index,
            Panel::Tasks => self.task_index,
        };
        let next = current.saturating_add_signed(delta).min(len - 1);
        match self.panel {
            Panel::Workspaces => self.workspace_index = next,
            Panel::Tasks => self.task_index = next,
        }
        self.scroll = 0;
    }
}

pub fn run() -> Result<(), String> {
    storage::ensure_layout()?;
    enable_raw_mode().map_err(|e| e.to_string())?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(|e| e.to_string())?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| e.to_string())?;
    let mut app = NinjaTui::load()?;

    let result = run_loop(&mut terminal, &mut app);

    disable_raw_mode().map_err(|e| e.to_string())?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen).map_err(|e| e.to_string())?;
    terminal.show_cursor().map_err(|e| e.to_string())?;
    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut NinjaTui,
) -> Result<(), String> {
    loop {
        terminal
            .draw(|frame| render(frame, app))
            .map_err(|e| e.to_string())?;
        if !event::poll(Duration::from_millis(200)).map_err(|e| e.to_string())? {
            continue;
        }
        let Event::Key(key) = event::read().map_err(|e| e.to_string())? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        if app.form.is_some() {
            handle_form_key(app, key.code);
            continue;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => break,
            KeyCode::Tab => {
                app.panel = if app.panel == Panel::Workspaces {
                    Panel::Tasks
                } else {
                    Panel::Workspaces
                };
                app.scroll = 0;
            }
            KeyCode::Up => app.move_selection(-1),
            KeyCode::Down => app.move_selection(1),
            KeyCode::PageUp => app.scroll = app.scroll.saturating_sub(8),
            KeyCode::PageDown => app.scroll = app.scroll.saturating_add(8),
            KeyCode::Char('r') => {
                let result = app.reload().map(|_| "Reloaded.".to_string());
                set_message(app, result);
            }
            KeyCode::Char('?') | KeyCode::Char('h') => app.message = help_text().to_string(),
            KeyCode::Char('g') => app.form = Some(agent_root_form()),
            KeyCode::Char('n') => app.form = Some(create_form(app)),
            KeyCode::Char('u') => app.form = Some(update_form(app)),
            KeyCode::Char('a') if app.panel == Panel::Workspaces => {
                app.form = Some(workspace_add_folder_form(app))
            }
            KeyCode::Char('x') if app.panel == Panel::Workspaces => {
                app.form = Some(workspace_remove_folder_form(app))
            }
            KeyCode::Char('d') if app.panel == Panel::Workspaces => {
                app.form = Some(workspace_remove_form(app))
            }
            KeyCode::Char('s') if app.panel == Panel::Tasks => {
                app.form = Some(task_start_form(app))
            }
            KeyCode::Char('m') if app.panel == Panel::Tasks => {
                app.form = Some(task_resume_form(app))
            }
            KeyCode::Char('o') if app.panel == Panel::Tasks => app.form = Some(task_note_form(app)),
            KeyCode::Char('p') if app.panel == Panel::Tasks => {
                app.form = Some(task_result_form(app))
            }
            KeyCode::Char('c') if app.panel == Panel::Tasks => {
                app.form = Some(task_complete_form(app))
            }
            KeyCode::Char('f') if app.panel == Panel::Tasks => app.form = Some(task_fail_form(app)),
            KeyCode::Char('k') if app.panel == Panel::Tasks => app.form = Some(task_kill_form(app)),
            KeyCode::Char('d') if app.panel == Panel::Tasks => {
                app.form = Some(task_remove_form(app))
            }
            _ => {}
        }
    }
    Ok(())
}

fn handle_form_key(app: &mut NinjaTui, code: KeyCode) {
    let Some(form) = app.form.as_mut() else {
        return;
    };
    match code {
        KeyCode::Esc => {
            app.form = None;
            app.message = "Cancelled.".to_string();
        }
        KeyCode::Backspace => {
            if let Some(field) = form.fields.get_mut(form.index) {
                field.value.pop();
            }
        }
        KeyCode::Char(ch) => {
            if let Some(field) = form.fields.get_mut(form.index) {
                field.value.push(ch);
            }
        }
        KeyCode::Enter => {
            let missing = form
                .fields
                .get(form.index)
                .map(|field| field.required && field.value.trim().is_empty())
                .unwrap_or(false);
            if missing {
                app.message = "This field is required.".to_string();
                return;
            }
            if form.index + 1 < form.fields.len() {
                form.index += 1;
            } else {
                let form = app.form.take().expect("form exists");
                let result = run_form_action(&form);
                set_message(app, result);
            }
        }
        _ => {}
    }
}

fn run_form_action(form: &Form) -> Result<String, String> {
    let mut args: Vec<String> = match form.action {
        ActionKind::WorkspaceCreate => vec![
            "workspace".to_string(),
            "create".to_string(),
            value(form, "name"),
            "--mutate".to_string(),
        ],
        ActionKind::WorkspaceUpdate => vec![
            "workspace".to_string(),
            "update".to_string(),
            value(form, "name"),
            "--mutate".to_string(),
        ],
        ActionKind::WorkspaceAddFolder => {
            vec![
                "workspace".to_string(),
                "add-folder".to_string(),
                value(form, "name"),
                "--mutate".to_string(),
            ]
        }
        ActionKind::WorkspaceRemoveFolder => {
            vec![
                "workspace".to_string(),
                "remove-folder".to_string(),
                value(form, "name"),
                "--mutate".to_string(),
            ]
        }
        ActionKind::WorkspaceRemove => {
            vec![
                "workspace".to_string(),
                "remove".to_string(),
                value(form, "name"),
                "--mutate".to_string(),
            ]
        }
        ActionKind::AgentRootSet => vec![
            "workspace".to_string(),
            "agent-root-set".to_string(),
            "--mutate".to_string(),
        ],
        ActionKind::TaskCreate => vec![
            "task".to_string(),
            "create".to_string(),
            value(form, "id"),
            "--mutate".to_string(),
        ],
        ActionKind::TaskUpdate => vec![
            "task".to_string(),
            "update".to_string(),
            value(form, "id"),
            "--mutate".to_string(),
        ],
        ActionKind::TaskStart => vec![
            "task".to_string(),
            "start".to_string(),
            value(form, "id"),
            "--mutate".to_string(),
        ],
        ActionKind::TaskResume => vec![
            "task".to_string(),
            "resume".to_string(),
            value(form, "id"),
            "--mutate".to_string(),
        ],
        ActionKind::TaskNote => vec![
            "task".to_string(),
            "note".to_string(),
            value(form, "id"),
            "--mutate".to_string(),
        ],
        ActionKind::TaskResult => vec![
            "task".to_string(),
            "result".to_string(),
            value(form, "id"),
            "--mutate".to_string(),
        ],
        ActionKind::TaskComplete => vec![
            "task".to_string(),
            "complete".to_string(),
            value(form, "id"),
            "--mutate".to_string(),
        ],
        ActionKind::TaskFail => vec![
            "task".to_string(),
            "fail".to_string(),
            value(form, "id"),
            "--mutate".to_string(),
        ],
        ActionKind::TaskKill => vec![
            "task".to_string(),
            "kill".to_string(),
            value(form, "id"),
            "--mutate".to_string(),
        ],
        ActionKind::TaskRemove => vec![
            "task".to_string(),
            "remove".to_string(),
            value(form, "id"),
            "--mutate".to_string(),
        ],
    };
    for field in &form.fields {
        if ["name", "id"].contains(&field.name) || field.value.trim().is_empty() {
            continue;
        }
        match field.name {
            "default" | "force" | "close_terminal" | "keep_terminal" => {
                if is_yes(&field.value) {
                    args.push(flag_name(field.name));
                }
            }
            _ => {
                args.push(flag_name(field.name));
                args.push(field.value.trim().to_string());
            }
        }
    }
    run_infynon(args)
}

fn run_infynon(args: Vec<impl AsRef<str>>) -> Result<String, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let output = Command::new(exe)
        .args(args.iter().map(|value| value.as_ref()))
        .output()
        .map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !output.status.success() {
        return Err(human_error(&stdout, &stderr));
    }
    Ok(human_success(&stdout))
}

fn human_success(stdout: &str) -> String {
    let Ok(value) = serde_json::from_str::<Value>(stdout) else {
        return stdout.to_string();
    };
    let command = value
        .get("command")
        .and_then(Value::as_str)
        .unwrap_or("command");
    match command {
        "workspace.create" => format!(
            "Workspace '{}' created.",
            value["workspace"]["name"].as_str().unwrap_or("unknown")
        ),
        "workspace.update" => format!(
            "Workspace '{}' updated.",
            value["workspace"]["name"].as_str().unwrap_or("unknown")
        ),
        "workspace.add-folder" => format!(
            "Folder '{}' added.",
            value["added_folder"]["folder_name"]
                .as_str()
                .unwrap_or("unknown")
        ),
        "workspace.remove-folder" => format!(
            "Folder '{}' removed.",
            value["removed_folder_name"].as_str().unwrap_or("unknown")
        ),
        "workspace.remove" => format!(
            "Workspace '{}' removed.",
            value["removed_workspace"].as_str().unwrap_or("unknown")
        ),
        "workspace.agent-root-set" => format!(
            "Agent root set to {}.",
            value["agent_root_path"].as_str().unwrap_or("unknown")
        ),
        "task.create" => format!("Task {} created.", task_id(&value)),
        "task.update" => format!("Task {} updated.", task_id(&value)),
        "task.start" => format!("Task {} started.", task_id(&value)),
        "task.resume" => format!("Task {} resumed.", task_id(&value)),
        "task.note" => format!("Note added to task {}.", task_id(&value)),
        "task.result" => format!("Result added to task {}.", task_id(&value)),
        "task.complete" => format!("Task {} completed.", task_id(&value)),
        "task.fail" => format!("Task {} failed.", task_id(&value)),
        "task.kill" => format!("Task {} killed.", task_id(&value)),
        "task.remove" => format!(
            "Task {} removed.",
            value["removed_task"].as_str().unwrap_or("unknown")
        ),
        _ => format!("{} succeeded.", command),
    }
}

fn human_error(stdout: &str, stderr: &str) -> String {
    if let Ok(value) = serde_json::from_str::<Value>(stdout) {
        if let Some(error) = value.get("error").and_then(Value::as_str) {
            return error.to_string();
        }
    }
    if !stderr.is_empty() {
        stderr.to_string()
    } else if !stdout.is_empty() {
        stdout.to_string()
    } else {
        "Command failed.".to_string()
    }
}

fn task_id(value: &Value) -> &str {
    value["record"]["id"]
        .as_str()
        .or_else(|| value["task"]["id"].as_str())
        .unwrap_or("unknown")
}

fn set_message(app: &mut NinjaTui, result: Result<String, String>) {
    app.message = match result {
        Ok(message) => {
            let _ = app.reload();
            message
        }
        Err(err) => err,
    };
}

fn render(frame: &mut ratatui::Frame, app: &mut NinjaTui) {
    let size = frame.size();
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Min(8),
            Constraint::Length(4),
        ])
        .split(size);
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(44), Constraint::Min(72)])
        .split(outer[1]);
    let lists = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(12), Constraint::Min(10)])
        .split(body[0]);

    frame.render_widget(Clear, size);
    frame.render_widget(header(app), outer[0]);
    render_workspaces(frame, app, lists[0]);
    render_tasks(frame, app, lists[1]);
    render_detail(frame, app, body[1]);
    frame.render_widget(action_bar(app), outer[2]);
    if let Some(form) = &app.form {
        render_form(frame, form);
    }
}

fn header(app: &NinjaTui) -> Paragraph<'_> {
    let root = app
        .manifest
        .agent_root_path
        .as_deref()
        .unwrap_or("not configured");
    let active = if app.panel == Panel::Workspaces {
        "Workspaces"
    } else {
        "Tasks"
    };
    Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                " INFYNON ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                "Workspace and task control plane",
                Style::default().fg(Color::Gray),
            ),
        ]),
        Line::from(vec![
            Span::styled("Root ", Style::default().fg(Color::Gray)),
            Span::raw(compact_path(root, 78)),
            Span::raw("  "),
            Span::styled("Active ", Style::default().fg(Color::Gray)),
            Span::raw(active),
            Span::raw("  "),
            Span::styled("Workspaces ", Style::default().fg(Color::Gray)),
            Span::raw(app.manifest.workspaces.len().to_string()),
            Span::raw("  "),
            Span::styled("Tasks ", Style::default().fg(Color::Gray)),
            Span::raw(app.manifest.tasks.len().to_string()),
        ]),
    ])
    .block(panel_block("INFYNON Coding TUI", true))
    .wrap(Wrap { trim: true })
}

fn render_workspaces(frame: &mut ratatui::Frame, app: &NinjaTui, area: Rect) {
    let viewport = list_viewport_height(area);
    let start = visible_start(app.workspace_index, app.manifest.workspaces.len(), viewport);
    let items = app
        .manifest
        .workspaces
        .iter()
        .skip(start)
        .take(viewport)
        .enumerate()
        .map(|(visible_index, workspace)| {
            let index = start + visible_index;
            let selected = app.panel == Panel::Workspaces && app.workspace_index == index;
            let marker = if selected { ">" } else { " " };
            let default = if app.manifest.default_workspace.as_deref() == Some(&workspace.name) {
                "default"
            } else {
                ""
            };
            ListItem::new(format!(
                "{} {:<29} {}",
                marker,
                truncate(&workspace.name, 29),
                default
            ))
            .style(if selected {
                Style::default().fg(Color::Black).bg(Color::Cyan)
            } else {
                Style::default()
            })
        })
        .collect::<Vec<_>>();
    let widget = List::new(items).block(list_block(
        "Workspaces",
        app.panel == Panel::Workspaces,
        app.manifest.workspaces.len(),
    ));
    frame.render_widget(widget, area);
    render_vertical_scrollbar(
        frame,
        area,
        app.manifest.workspaces.len(),
        app.workspace_index,
        viewport,
    );
}

fn render_tasks(frame: &mut ratatui::Frame, app: &NinjaTui, area: Rect) {
    let viewport = list_viewport_height(area);
    let start = visible_start(app.task_index, app.manifest.tasks.len(), viewport);
    let items = app
        .manifest
        .tasks
        .iter()
        .skip(start)
        .take(viewport)
        .enumerate()
        .map(|(visible_index, task)| {
            let index = start + visible_index;
            let selected = app.panel == Panel::Tasks && app.task_index == index;
            let marker = if selected { ">" } else { " " };
            ListItem::new(format!(
                "{} {:<8} {:<9} {}",
                marker,
                short_id(&task.id),
                truncate(&task.status, 9),
                truncate(task.workspace.as_deref().unwrap_or("no-workspace"), 19)
            ))
            .style(if selected {
                Style::default().fg(Color::Black).bg(Color::Cyan)
            } else {
                status_style(&task.status)
            })
        })
        .collect::<Vec<_>>();
    let widget = List::new(items).block(list_block(
        "Tasks",
        app.panel == Panel::Tasks,
        app.manifest.tasks.len(),
    ));
    frame.render_widget(widget, area);
    render_vertical_scrollbar(
        frame,
        area,
        app.manifest.tasks.len(),
        app.task_index,
        viewport,
    );
}

fn render_detail(frame: &mut ratatui::Frame, app: &mut NinjaTui, area: Rect) {
    let title = if app.panel == Panel::Workspaces {
        "Workspace Detail"
    } else {
        "Task Detail"
    };
    let content = app.detail();
    let viewport = list_viewport_height(area);
    let line_count = content.lines().count().max(1);
    let max_scroll = line_count.saturating_sub(viewport);
    app.scroll = app.scroll.min(max_scroll as u16);
    let widget = Paragraph::new(styled_detail_lines(&content))
        .block(panel_block(title, true))
        .wrap(Wrap { trim: false })
        .scroll((app.scroll, 0));
    frame.render_widget(widget, area);
    render_vertical_scrollbar(frame, area, line_count, app.scroll as usize, viewport);
}

fn action_bar(app: &NinjaTui) -> Paragraph<'_> {
    let mode = if app.panel == Panel::Workspaces {
        "Workspace"
    } else {
        "Task"
    };
    let content = vec![
        Line::from(vec![
            Span::styled("Mode ", Style::default().fg(Color::Gray)),
            Span::styled(mode, Style::default().fg(Color::Cyan)),
            Span::raw("  "),
            Span::styled("Status ", Style::default().fg(Color::Gray)),
            Span::raw(app.message.as_str()),
        ]),
        Line::from(help_text()),
    ];
    Paragraph::new(content)
        .block(panel_block("Actions", false))
        .wrap(Wrap { trim: true })
}

fn render_form(frame: &mut ratatui::Frame, form: &Form) {
    let area = centered_rect(76, 54, frame.size());
    let field = &form.fields[form.index];
    let progress = format!("Field {}/{}", form.index + 1, form.fields.len());
    let content = vec![
        Line::from(Span::styled(
            form.title.as_str(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(progress),
        Line::from(""),
        Line::from(field.prompt),
        Line::from(Span::styled(
            field.value.as_str(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("Enter next/save | Backspace edit | Esc cancel | leave optional fields blank"),
    ];
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(content)
            .block(panel_block("Form", true))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn centered_rect(
    percent_x: u16,
    percent_y: u16,
    area: ratatui::layout::Rect,
) -> ratatui::layout::Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn panel_block(title: &'static str, active: bool) -> Block<'static> {
    let style = if active {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(style)
}

fn list_block(title: &'static str, active: bool, count: usize) -> Block<'static> {
    let style = if active {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };
    Block::default()
        .title(format!("{} ({})", title, count))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(style)
}

fn list_viewport_height(area: Rect) -> usize {
    area.height.saturating_sub(2).max(1) as usize
}

fn visible_start(selected: usize, len: usize, viewport: usize) -> usize {
    if len <= viewport {
        return 0;
    }
    selected.saturating_sub(viewport / 2).min(len - viewport)
}

fn render_vertical_scrollbar(
    frame: &mut ratatui::Frame,
    area: Rect,
    content_len: usize,
    position: usize,
    viewport: usize,
) {
    if content_len <= viewport || area.height <= 3 {
        return;
    }
    let mut state = ScrollbarState::new(content_len)
        .position(position.min(content_len.saturating_sub(1)))
        .viewport_content_length(viewport);
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(None)
        .end_symbol(None)
        .track_symbol(None)
        .thumb_style(Style::default().fg(Color::Cyan));
    frame.render_stateful_widget(
        scrollbar,
        area.inner(&Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut state,
    );
}

fn create_form(app: &NinjaTui) -> Form {
    match app.panel {
        Panel::Workspaces => Form::new(
            "Create Workspace",
            ActionKind::WorkspaceCreate,
            vec![
                field("name", "Workspace name", "", true),
                field("folder_name", "Primary folder name", "root", false),
                field("path", "Absolute folder path", "", false),
                field("description", "Description", "", false),
                field("default", "Set default? y/N", "n", false),
                field("super_lite_model", "Super lite model", "", false),
                field("super_lite_thinking", "Super lite thinking", "", false),
                field("lite_model", "Lite model", "", false),
                field("lite_thinking", "Lite thinking", "", false),
                field("frontier_model", "Frontier model", "", false),
                field("frontier_thinking", "Frontier thinking", "", false),
                field(
                    "highest_frontier_model",
                    "Highest frontier model",
                    "",
                    false,
                ),
                field(
                    "highest_frontier_thinking",
                    "Highest frontier thinking",
                    "",
                    false,
                ),
            ],
        ),
        Panel::Tasks => Form::new(
            "Create Task",
            ActionKind::TaskCreate,
            vec![
                field("id", "UUIDv4 task id", "", true),
                field("workspace", "Workspace", "", false),
                field("folder_name", "Folder name", "", false),
                field("agent", "Agent", "codex", false),
                field("model", "Model", "", false),
                field(
                    "thinking",
                    "Thinking auto|low|medium|high|xhigh",
                    "auto",
                    false,
                ),
                field("prompt", "Task prompt", "", false),
                field("command", "Command", "", false),
                field("pid", "PID", "", false),
                field("session_id", "Session id", "", false),
                field("notes", "Notes", "", false),
                field("result", "Result", "", false),
                field("blocked_by", "Blocked by task id", "", false),
                field("blocked_reason", "Blocked reason", "", false),
                field("status", "Status", "draft", false),
            ],
        ),
    }
}

fn update_form(app: &NinjaTui) -> Form {
    match app.panel {
        Panel::Workspaces => Form::new(
            "Update Workspace",
            ActionKind::WorkspaceUpdate,
            vec![
                field(
                    "name",
                    "Workspace name",
                    &app.selected_workspace_name().unwrap_or_default(),
                    true,
                ),
                field("folder_name", "Replace primary folder name", "", false),
                field("path", "Replace primary absolute path", "", false),
                field("description", "Description", "", false),
                field("default", "Set default? y/N", "n", false),
                field("super_lite_model", "Super lite model", "", false),
                field("super_lite_thinking", "Super lite thinking", "", false),
                field("lite_model", "Lite model", "", false),
                field("lite_thinking", "Lite thinking", "", false),
                field("frontier_model", "Frontier model", "", false),
                field("frontier_thinking", "Frontier thinking", "", false),
                field(
                    "highest_frontier_model",
                    "Highest frontier model",
                    "",
                    false,
                ),
                field(
                    "highest_frontier_thinking",
                    "Highest frontier thinking",
                    "",
                    false,
                ),
            ],
        ),
        Panel::Tasks => Form::new(
            "Update Task",
            ActionKind::TaskUpdate,
            vec![
                field(
                    "id",
                    "Task id",
                    &app.selected_task_id().unwrap_or_default(),
                    true,
                ),
                field("workspace", "Workspace", "", false),
                field("folder_name", "Folder name", "", false),
                field("agent", "Agent", "", false),
                field("model", "Model", "", false),
                field("thinking", "Thinking", "", false),
                field("prompt", "Prompt", "", false),
                field("command", "Command", "", false),
                field("status", "Status", "", false),
                field("pid", "PID", "", false),
                field("session_id", "Session id", "", false),
                field("notes", "Replace notes", "", false),
                field("result", "Replace result", "", false),
                field("blocked_by", "Blocked by task id", "", false),
                field("blocked_reason", "Blocked reason", "", false),
                field("parent_task_id", "Parent task id", "", false),
            ],
        ),
    }
}

fn workspace_add_folder_form(app: &NinjaTui) -> Form {
    Form::new(
        "Add Workspace Folder",
        ActionKind::WorkspaceAddFolder,
        vec![
            field(
                "name",
                "Workspace name",
                &app.selected_workspace_name().unwrap_or_default(),
                true,
            ),
            field("folder_name", "Folder name", "", true),
            field("path", "Absolute folder path", "", true),
        ],
    )
}

fn workspace_remove_folder_form(app: &NinjaTui) -> Form {
    Form::new(
        "Remove Workspace Folder",
        ActionKind::WorkspaceRemoveFolder,
        vec![
            field(
                "name",
                "Workspace name",
                &app.selected_workspace_name().unwrap_or_default(),
                true,
            ),
            field("folder_name", "Folder name", "", true),
        ],
    )
}

fn workspace_remove_form(app: &NinjaTui) -> Form {
    Form::new(
        "Remove Workspace",
        ActionKind::WorkspaceRemove,
        vec![field(
            "name",
            "Workspace name",
            &app.selected_workspace_name().unwrap_or_default(),
            true,
        )],
    )
}

fn agent_root_form() -> Form {
    Form::new(
        "Set Agent Root",
        ActionKind::AgentRootSet,
        vec![field("path", "Absolute INFYNON agent root path", "", true)],
    )
}

fn task_start_form(app: &NinjaTui) -> Form {
    Form::new(
        "Start Task",
        ActionKind::TaskStart,
        vec![
            field(
                "id",
                "Task id",
                &app.selected_task_id().unwrap_or_default(),
                true,
            ),
            field("pid", "PID override", "", false),
            field("session_id", "Session id", "", false),
        ],
    )
}

fn task_resume_form(app: &NinjaTui) -> Form {
    Form::new(
        "Resume Task",
        ActionKind::TaskResume,
        vec![
            field(
                "id",
                "Task id",
                &app.selected_task_id().unwrap_or_default(),
                true,
            ),
            field("session_id", "Session id", "", false),
            field("prompt", "Follow-up prompt", "", false),
        ],
    )
}

fn task_note_form(app: &NinjaTui) -> Form {
    Form::new(
        "Append Task Note",
        ActionKind::TaskNote,
        vec![
            field(
                "id",
                "Task id",
                &app.selected_task_id().unwrap_or_default(),
                true,
            ),
            field("text", "Note text", "", true),
        ],
    )
}

fn task_result_form(app: &NinjaTui) -> Form {
    Form::new(
        "Append Task Result",
        ActionKind::TaskResult,
        vec![
            field(
                "id",
                "Task id",
                &app.selected_task_id().unwrap_or_default(),
                true,
            ),
            field("text", "Result text", "", true),
        ],
    )
}

fn task_complete_form(app: &NinjaTui) -> Form {
    Form::new(
        "Complete Task",
        ActionKind::TaskComplete,
        vec![
            field(
                "id",
                "Task id",
                &app.selected_task_id().unwrap_or_default(),
                true,
            ),
            field("result", "Final result", "", true),
            field("notes", "Final notes", "", false),
            field("keep_terminal", "Keep terminal open? y/N", "n", false),
        ],
    )
}

fn task_fail_form(app: &NinjaTui) -> Form {
    Form::new(
        "Fail Task",
        ActionKind::TaskFail,
        vec![
            field(
                "id",
                "Task id",
                &app.selected_task_id().unwrap_or_default(),
                true,
            ),
            field("reason", "Failure reason", "", true),
            field("result", "Failure result", "", false),
            field("keep_terminal", "Keep terminal open? y/N", "n", false),
        ],
    )
}

fn task_kill_form(app: &NinjaTui) -> Form {
    Form::new(
        "Kill Task",
        ActionKind::TaskKill,
        vec![
            field(
                "id",
                "Task id",
                &app.selected_task_id().unwrap_or_default(),
                true,
            ),
            field("pid", "PID override", "", false),
            field("reason", "Kill reason", "", false),
            field("force", "Force kill? y/N", "n", false),
        ],
    )
}

fn task_remove_form(app: &NinjaTui) -> Form {
    Form::new(
        "Remove Task",
        ActionKind::TaskRemove,
        vec![field(
            "id",
            "Task id",
            &app.selected_task_id().unwrap_or_default(),
            true,
        )],
    )
}

impl Form {
    fn new(title: &str, action: ActionKind, fields: Vec<Field>) -> Self {
        Self {
            title: title.to_string(),
            action,
            fields,
            index: 0,
        }
    }
}

fn field(name: &'static str, prompt: &'static str, value: &str, required: bool) -> Field {
    Field {
        name,
        prompt,
        value: value.to_string(),
        required,
    }
}

fn value(form: &Form, name: &str) -> String {
    form.fields
        .iter()
        .find(|field| field.name == name)
        .map(|field| field.value.trim().to_string())
        .unwrap_or_default()
}

fn flag_name(name: &str) -> String {
    format!("--{}", name.replace('_', "-"))
}

fn is_yes(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "y" | "yes" | "true" | "1"
    )
}

fn short_id(value: &str) -> &str {
    value.get(..8).unwrap_or(value)
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}

fn status_style(status: &str) -> Style {
    match status.to_ascii_lowercase().as_str() {
        "running" => Style::default().fg(Color::Green),
        "completed" | "complete" => Style::default().fg(Color::Blue),
        "failed" | "error" => Style::default().fg(Color::Red),
        "blocked" => Style::default().fg(Color::Yellow),
        "killed" => Style::default().fg(Color::DarkGray),
        _ => Style::default().fg(Color::White),
    }
}

fn compact_path(value: &str, max_chars: usize) -> String {
    let cleaned = display_path(value);
    truncate_middle(&cleaned, max_chars)
}

fn display_path(value: &str) -> String {
    let without_extended_prefix = value
        .strip_prefix("\\\\?\\UNC\\")
        .map(|path| format!("\\\\{}", path))
        .or_else(|| value.strip_prefix("\\\\?\\").map(str::to_string))
        .unwrap_or_else(|| value.to_string());
    if cfg!(windows) {
        without_extended_prefix.replace('/', "\\")
    } else {
        without_extended_prefix
    }
}

fn styled_detail_lines(content: &str) -> Vec<Line<'static>> {
    content
        .lines()
        .map(|line| {
            if line.trim().is_empty() {
                Line::from("")
            } else if !line.starts_with(' ') && !line.starts_with('-') {
                Line::from(Span::styled(
                    line.to_string(),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
            } else if line.starts_with("  ") {
                Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(Color::White),
                ))
            } else {
                Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(Color::Gray),
                ))
            }
        })
        .collect()
}

fn truncate(value: &str, max_chars: usize) -> String {
    let count = value.chars().count();
    if count <= max_chars {
        return value.to_string();
    }
    if max_chars <= 3 {
        return ".".repeat(max_chars);
    }
    let keep = max_chars.saturating_sub(3);
    let mut result = value.chars().take(keep).collect::<String>();
    result.push_str("...");
    result
}

fn truncate_middle(value: &str, max_chars: usize) -> String {
    let count = value.chars().count();
    if count <= max_chars {
        return value.to_string();
    }
    if max_chars <= 3 {
        return truncate(value, max_chars);
    }
    let left = (max_chars - 3) / 2;
    let right = max_chars - left - 3;
    let start = value.chars().take(left).collect::<String>();
    let end = value
        .chars()
        .rev()
        .take(right)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<String>();
    format!("{}...{}", start, end)
}

fn help_text() -> &'static str {
    "Tab switch | Up/Down move | PgUp/PgDn scroll | n new | u update | g root | r reload | q quit | workspace a/x folder, d remove | task s/m start/resume, o/p note/result, c/f complete/fail, k/d kill/remove"
}
