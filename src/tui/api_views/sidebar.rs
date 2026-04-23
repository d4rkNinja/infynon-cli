use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::api_app::{ApiApp, ApiView};
use crate::tui::theme::*;

use super::truncate;

// ── Sidebar renderer ─────────────────────────────────────────────────────────

pub(super) fn render_sidebar(f: &mut Frame, app: &ApiApp, area: Rect) {
    let outer = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(BORDER))
        .style(Style::default().bg(BG_SURFACE));
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let nav_h = (ApiView::all().len() * 2 + 1) as u16; // header + 2 lines per item

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),     // Brand
            Constraint::Length(1),     // Separator
            Constraint::Length(nav_h), // Nav
            Constraint::Length(1),     // Separator
            Constraint::Min(6),        // Flow status
            Constraint::Length(1),     // Separator
            Constraint::Length(4),     // Key hints
        ])
        .split(inner);

    render_brand(f, chunks[0]);
    render_sep(f, chunks[1]);
    render_nav(f, app, chunks[2]);
    render_sep(f, chunks[3]);
    render_flow_status(f, app, chunks[4]);
    render_sep(f, chunks[5]);
    render_hints(f, chunks[6]);
}

// ── Brand ──────────────────────────────────────────────────────────────────────

fn render_brand(f: &mut Frame, area: Rect) {
    let lines = vec![
        Line::raw(""),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                "\u{25C6} ",
                Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "WEAVE",
                Style::default().fg(WHITE).add_modifier(Modifier::BOLD),
            ),
        ]),
    ];
    f.render_widget(Paragraph::new(lines), area);
}

// ── Navigation ──────────────────────────────────────────────────────────────

fn render_nav(f: &mut Frame, app: &ApiApp, area: Rect) {
    let mut lines: Vec<Line> = vec![Line::from(vec![Span::styled(
        "  VIEWS",
        Style::default().fg(DIMMER).add_modifier(Modifier::BOLD),
    )])];

    for view in ApiView::all() {
        let is_active = app.current_view == *view;
        let icon = view.icon();
        let label = view.label();

        if is_active {
            // Active: accent bar + highlighted
            lines.push(Line::from(vec![
                Span::styled(
                    "\u{258F} ",
                    Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
                ),
                Span::styled(icon, Style::default().fg(CYAN).add_modifier(Modifier::BOLD)),
                Span::styled(" ", Style::default()),
                Span::styled(
                    label,
                    Style::default().fg(WHITE).add_modifier(Modifier::BOLD),
                ),
            ]));
            lines.push(Line::from(vec![
                Span::styled("    ", Style::default()),
                Span::styled(view.sublabel(), Style::default().fg(CYAN)),
            ]));
        } else {
            // Inactive: dimmed with key hint
            lines.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(icon, Style::default().fg(DIMMER)),
                Span::styled(" ", Style::default()),
                Span::styled(label, Style::default().fg(TEXT_DIM)),
                Span::styled(format!(" {}", view.key()), Style::default().fg(DIMMER)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("    ", Style::default()),
                Span::styled(view.sublabel(), Style::default().fg(DIMMER)),
            ]));
        }
    }

    f.render_widget(Paragraph::new(lines), area);
}

// ── Flow status area ──────────────────────────────────────────────────────────

fn render_flow_status(f: &mut Frame, app: &ApiApp, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    let inner_w = area.width.saturating_sub(2) as usize;

    // Section header
    lines.push(Line::from(vec![Span::styled(
        "  FLOW",
        Style::default().fg(DIMMER).add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::raw(""));

    if app.flow_running || app.live_running {
        lines.push(Line::from(vec![
            Span::styled(
                "  \u{25CF} ",
                Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "RUNNING",
                Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::raw(""));
        if let Some(last) = app.live_steps.last() {
            let node_name = truncate(&last.node_name, inner_w.saturating_sub(6));
            lines.push(Line::from(vec![
                Span::styled("   \u{21B3} ", Style::default().fg(DIM)),
                Span::styled(node_name, Style::default().fg(TEXT_DIM)),
            ]));
        }
    } else if let Some(flow) = app.active_flow() {
        let (status_icon, status_col) = match app.flow_run_statuses.get(&flow.id) {
            Some(Some(true)) => ("\u{2713}", GREEN),
            Some(Some(false)) => ("\u{2717}", RED),
            _ => ("\u{25CB}", DIM),
        };

        let flow_name = truncate(&flow.name, inner_w.saturating_sub(6));
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {} ", status_icon),
                Style::default().fg(status_col),
            ),
            Span::styled(
                flow_name,
                Style::default().fg(WHITE).add_modifier(Modifier::BOLD),
            ),
        ]));

        let node_count = flow.all_node_ids().len();
        lines.push(Line::from(vec![Span::styled(
            format!(
                "   {} node{}",
                node_count,
                if node_count == 1 { "" } else { "s" }
            ),
            Style::default().fg(TEXT_DIM),
        )]));

        // Last run with progress bar
        if let Some(run) = &app.last_run {
            lines.push(Line::raw(""));

            let total = run.steps.len();
            let passed = run.passed_count();
            let pct = if total == 0 {
                0
            } else {
                (passed * 100) / total
            };
            let bar_w = inner_w.saturating_sub(8).max(4);
            let filled = if pct == 0 { 0 } else { (pct * bar_w) / 100 };
            let empty = bar_w.saturating_sub(filled);
            let bar_col = if pct == 100 {
                GREEN
            } else if pct >= 50 {
                YELLOW
            } else {
                RED
            };

            lines.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled("\u{2588}".repeat(filled), Style::default().fg(bar_col)),
                Span::styled("\u{2591}".repeat(empty), Style::default().fg(DIMMER)),
                Span::styled(
                    format!(" {:>3}%", pct),
                    Style::default().fg(bar_col).add_modifier(Modifier::BOLD),
                ),
            ]));

            // Timing
            lines.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(
                    format!("{}ms avg", run.avg_latency_ms()),
                    Style::default().fg(TEXT_DIM),
                ),
            ]));
        }
    } else {
        lines.push(Line::from(vec![
            Span::styled("  \u{25CB} ", Style::default().fg(DIMMER)),
            Span::styled("no flow", Style::default().fg(DIMMER)),
        ]));
        lines.push(Line::raw(""));
        lines.push(Line::from(vec![
            Span::styled("  Press ", Style::default().fg(DIMMER)),
            Span::styled("[3]", Style::default().fg(YELLOW)),
            Span::styled(" Flows", Style::default().fg(DIMMER)),
        ]));
    }

    // Pad to fill area
    while lines.len() < area.height as usize {
        lines.push(Line::raw(""));
    }

    f.render_widget(Paragraph::new(lines), area);
}

// ── Key hints ──────────────────────────────────────────────────────────────────

fn render_hints(f: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled("R", Style::default().fg(YELLOW)),
            Span::styled(" refresh", Style::default().fg(DIMMER)),
        ]),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled("?", Style::default().fg(YELLOW)),
            Span::styled(" help", Style::default().fg(DIMMER)),
        ]),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled("q", Style::default().fg(YELLOW)),
            Span::styled(" quit", Style::default().fg(DIMMER)),
        ]),
        Line::raw(""),
    ];
    f.render_widget(Paragraph::new(lines), area);
}

// ── Separator line ─────────────────────────────────────────────────────────────

fn render_sep(f: &mut Frame, area: Rect) {
    let line = "\u{2500}".repeat(area.width as usize);
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(line, Style::default().fg(DIMMER)))),
        area,
    );
}
