mod dashboard;
mod diff;
mod environment;
mod flows;
mod header;
mod modals;
mod nodes;
mod overlays;
mod runner;
mod sidebar;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

use crate::tui::api_app::{ApiApp, ApiView, AttachMode};
use crate::tui::theme::*;

// ── Shared helpers ───────────────────────────────────────────────────────────

pub(super) fn section_header(title: &str, width: usize) -> Line<'static> {
    let inner_w = width.saturating_sub(2);
    let dashes = inner_w.saturating_sub(title.len() + 4);
    let left_d = dashes / 2;
    let right_d = dashes - left_d;
    Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(
            format!(
                "{} {} {}",
                "\u{2500}".repeat(left_d),
                title,
                "\u{2500}".repeat(right_d)
            ),
            Style::default().fg(DIMMER),
        ),
    ])
}

pub(super) fn blank_line() -> Line<'static> {
    Line::raw("")
}

pub(crate) fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!(
            "{}\u{2026}",
            s.chars().take(max.saturating_sub(1)).collect::<String>()
        )
    }
}

/// Dynamic truncation based on available width with overhead.
pub(crate) fn dyn_truncate(s: &str, area_w: usize, overhead: usize) -> String {
    truncate(s, area_w.saturating_sub(overhead).max(4))
}

/// Build a left-aligned section header line padded to width.
pub(super) fn responsive_section(name: &str, width: usize) -> Line<'static> {
    Line::from(vec![Span::styled(
        crate::tui::theme::section_line_left(name, width),
        Style::default().fg(DIMMER),
    )])
}

pub(super) fn rounded_block<'a>(title: &'a str, style: Style) -> Block<'a> {
    Block::default()
        .title(Span::styled(title, style))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style())
}

pub(super) fn rounded_block_active<'a>(title: &'a str) -> Block<'a> {
    Block::default()
        .title(Span::styled(title, title_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER_ACTIVE))
}

// ── Top-level render ─────────────────────────────────────────────────────────

pub fn render(f: &mut Frame, app: &mut ApiApp) {
    let area = f.size();

    // Top-level: 3 rows (info bar, main area, status bar)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    header::render_info_bar(f, app, chunks[0]);

    // Main area: sidebar (20 col) + content (flex)
    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(20), Constraint::Min(0)])
        .split(chunks[1]);

    sidebar::render_sidebar(f, app, main[0]);

    match app.current_view {
        ApiView::Dashboard => dashboard::render_dashboard(f, app, main[1]),
        ApiView::Nodes => nodes::render_nodes_view(f, app, main[1]),
        ApiView::Flows => flows::render_flows_view(f, app, main[1]),
        ApiView::Runner => runner::render_runner_view(f, app, main[1]),
        ApiView::Environment => environment::render_env_context(f, app, main[1]),
        ApiView::Settings => overlays::render_settings(f, app, main[1]),
    }

    header::render_status_bar(f, app, chunks[2]);

    // Overlays (always on top)
    if app.show_help {
        overlays::render_help_overlay(f, area);
    }
    if app.current_view == ApiView::Flows && app.detail_panel.is_some() {
        overlays::render_node_detail_overlay(f, app, area);
    }
    if app.attach_mode != AttachMode::Idle {
        overlays::render_attach_overlay(f, app, area);
    }
    if app.prompt_modal.is_some() {
        modals::render_prompt_modal(f, app, area);
    }
    modals::render_body_editor(f, app, area);
    modals::render_step_detail_modal(f, app, area);
    overlays::render_node_field_editor_modal(f, app, area);
}
