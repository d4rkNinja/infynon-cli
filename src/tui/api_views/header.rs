use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::tui::api_app::ApiApp;
use crate::tui::theme::*;

use super::truncate;

// ── Info bar (top) ────────────────────────────────────────────────────────────

pub(super) fn render_info_bar(f: &mut Frame, app: &ApiApp, area: Rect) {
    let right_w = (area.width / 4).min(40);
    let halves = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(right_w)])
        .split(area);

    let left_w = halves[0].width as usize;

    // ── Left: flow name + counts ──
    let mut left_spans: Vec<Span> = vec![];

    match app.active_flow() {
        Some(fl) => {
            let name = truncate(&fl.name, left_w.saturating_sub(18).max(8));
            left_spans.push(Span::styled(" \u{25C8} ", Style::default().fg(CYAN)));
            left_spans.push(Span::styled(
                name,
                Style::default().fg(PURPLE).add_modifier(Modifier::BOLD),
            ));
            left_spans.push(Span::styled("  \u{2502}  ", Style::default().fg(DIMMER)));
        }
        None => {
            left_spans.push(Span::styled(" no flow  \u{2502}  ", Style::default().fg(DIMMER)));
        }
    }

    left_spans.push(Span::styled(
        format!("{} flows \u{00B7} {} nodes", app.flows.len(), app.nodes.len()),
        Style::default().fg(DIM),
    ));

    // ── Right: search ──
    let mut right_spans: Vec<Span> = vec![];
    if !app.search_input.is_empty() {
        right_spans.push(Span::styled("/ ", Style::default().fg(YELLOW)));
        right_spans.push(Span::styled(&app.search_input, Style::default().fg(WHITE).add_modifier(Modifier::BOLD)));
    } else {
        right_spans.push(Span::styled("/ ", Style::default().fg(YELLOW)));
        right_spans.push(Span::styled("search...", Style::default().fg(DIMMER)));
    }

    let bg = Style::default().bg(BG_SURFACE);
    f.render_widget(Paragraph::new(Line::from(left_spans)).style(bg), halves[0]);
    f.render_widget(
        Paragraph::new(Line::from(right_spans)).style(bg).alignment(Alignment::Right),
        halves[1],
    );
}

// ── Status bar (bottom) ──────────────────────────────────────────────────────

pub(super) fn render_status_bar(f: &mut Frame, app: &ApiApp, area: Rect) {
    let right_w = (area.width / 3).min(50);
    let halves = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(right_w)])
        .split(area);

    let mut left_spans: Vec<Span> = vec![];

    // Status indicator
    if app.flow_running || app.live_running {
        left_spans.push(Span::styled(" \u{25CF} RUNNING", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)));
    } else if let Some(run) = &app.last_run {
        let (icon, col) = if run.passed { ("\u{2713}", GREEN) } else { ("\u{2717}", RED) };
        left_spans.push(Span::styled(
            format!(" {} {}ms", icon, run.duration_ms()),
            Style::default().fg(col),
        ));
    } else {
        left_spans.push(Span::styled(" ready", Style::default().fg(DIM)));
    }

    left_spans.push(Span::styled("  \u{2502}  ", Style::default().fg(DIMMER)));

    // Current view
    let view_label = app.current_view.label();
    left_spans.push(Span::styled(view_label, Style::default().fg(CYAN)));

    // Right: key hints
    let right_spans = vec![
        Span::styled("[?]", Style::default().fg(YELLOW)),
        Span::styled(" help  ", Style::default().fg(DIMMER)),
        Span::styled("[R]", Style::default().fg(YELLOW)),
        Span::styled(" refresh  ", Style::default().fg(DIMMER)),
        Span::styled("[q]", Style::default().fg(YELLOW)),
        Span::styled(" quit", Style::default().fg(DIMMER)),
    ];

    let bg = Style::default().bg(BG_SURFACE);
    f.render_widget(Paragraph::new(Line::from(left_spans)).style(bg), halves[0]);
    f.render_widget(
        Paragraph::new(Line::from(right_spans)).style(bg).alignment(Alignment::Right),
        halves[1],
    );
}
