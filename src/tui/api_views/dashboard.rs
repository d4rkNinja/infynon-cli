use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::tui::api_app::ApiApp;
use crate::tui::theme::*;

use super::{rounded_block, truncate};

// ── Dashboard view ────────────────────────────────────────────────────────────

pub(super) fn render_dashboard(f: &mut Frame, app: &ApiApp, area: Rect) {
    if app.flows.is_empty() {
        if app.nodes.is_empty() {
            render_welcome_screen(f, area);
        } else {
            render_overview_nodes_only(f, app, area);
        }
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_flow_list(f, app, chunks[0]);
    render_quick_stats(f, app, chunks[1]);
}

// ── Flow list (left panel) ────────────────────────────────────────────────────

fn render_flow_list(f: &mut Frame, app: &ApiApp, area: Rect) {
    let items: Vec<ListItem> = app.flows.iter().enumerate().map(|(i, flow)| {
        let (status_icon, status_color) = match app.flow_run_statuses.get(&flow.id) {
            Some(Some(true))  => ("\u{2714}", GREEN),   // heavy check
            Some(Some(false)) => ("\u{2718}", RED),     // heavy cross
            _                 => ("\u{00B7}", DIM),      // middle dot
        };
        let is_selected = i == app.active_flow_idx;
        let name_style = if is_selected {
            Style::default().fg(PURPLE).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(TEXT)
        };

        let node_count = flow.all_node_ids().len();
        let base = flow.base_url.as_deref().unwrap_or("\u{2014}"); // em dash
        let node_suffix = if node_count == 1 { " node " } else { " nodes" };

        // Line 1: status icon + flow name + node count
        let w = (area.width as usize / 3).max(12).min(28);
        let line1 = Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(status_icon.to_string(), Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
            Span::styled("  ", Style::default()),
            Span::styled(format!("{:<w$}", truncate(&flow.name, w)), name_style),
            Span::styled(format!("{}{}", node_count, node_suffix), Style::default().fg(DIM)),
        ]);

        // Line 2: base URL indented
        let line2 = Line::from(vec![
            Span::styled("      ", Style::default()),
            Span::styled(truncate(base, area.width.saturating_sub(14) as usize), Style::default().fg(DIMMER)),
        ]);

        ListItem::new(vec![line1, line2])
    }).collect();

    let mut state = ListState::default();
    state.select(Some(app.active_flow_idx));

    // Title with running indicator
    let title = if app.flow_running {
        format!(" Flows ({}) \u{25C9} RUNNING ", app.flows.len())
    } else {
        format!(" Flows ({}) ", app.flows.len())
    };

    let list = List::new(items)
        .block(
            Block::default()
                .title(Span::styled(title, title_style()))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(border_style()),
        )
        .highlight_style(
            Style::default()
                .bg(BG_SELECTED)
                .fg(PURPLE)
                .add_modifier(Modifier::BOLD),
        );

    f.render_stateful_widget(list, area, &mut state);
}

// ── Quick stats (right panel) ─────────────────────────────────────────────────

fn render_quick_stats(f: &mut Frame, app: &ApiApp, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();

    if let Some(run) = &app.last_run {
        let total = run.steps.len();
        let passed = run.passed_count();
        let pass_pct = if total == 0 { 0 } else { (passed * 100) / total };

        // ── Section: Result badge ──
        lines.push(Line::raw(""));
        lines.push(Line::raw(""));

        let (badge_text, badge_bg, badge_fg) = if run.passed {
            ("  \u{2714}  PASSED  ", GREEN, BG)
        } else {
            ("  \u{2718}  FAILED  ", RED, BG)
        };
        lines.push(Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::styled(badge_text, Style::default().fg(badge_fg).bg(badge_bg).add_modifier(Modifier::BOLD)),
        ]));

        lines.push(Line::raw(""));

        // ── Section: Stats ──
        lines.push(Line::from(vec![
            Span::styled(
                "  \u{2500}\u{2500} Run Statistics \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
                Style::default().fg(DIMMER),
            ),
        ]));
        lines.push(Line::raw(""));

        let stats: &[(&str, String, Color)] = &[
            ("  Steps      ", format!("{} / {}", passed, total), WHITE),
            ("  Duration   ", format!("{}ms", run.duration_ms()), CYAN),
            ("  Avg Latency", format!("{}ms", run.avg_latency_ms()), CYAN),
            (
                "  Pass Rate  ",
                format!("{}%", pass_pct),
                if pass_pct == 100 { GREEN } else if pass_pct >= 50 { YELLOW } else { RED },
            ),
        ];
        for (label, value, color) in stats {
            lines.push(Line::from(vec![
                Span::styled(*label, stat_label()),
                Span::styled("  ", Style::default()),
                Span::styled(value.clone(), Style::default().fg(*color).add_modifier(Modifier::BOLD)),
            ]));
        }

        lines.push(Line::raw(""));

        // ── Visual progress bar ──
        let bar_width = (area.width as usize).saturating_sub(12).min(36);
        let filled = if total == 0 { 0 } else { (pass_pct as usize * bar_width) / 100 };
        let empty = bar_width.saturating_sub(filled);
        let bar_color = if pass_pct == 100 { GREEN } else if pass_pct >= 50 { YELLOW } else { RED };

        lines.push(Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::styled("\u{2593}".repeat(filled), Style::default().fg(bar_color)),
            Span::styled("\u{2591}".repeat(empty), Style::default().fg(DIMMER)),
            Span::styled(format!(" {}%", pass_pct), Style::default().fg(bar_color).add_modifier(Modifier::BOLD)),
        ]));

        lines.push(Line::raw(""));
        lines.push(Line::raw(""));

        // ── Section: Last steps mini list ──
        lines.push(Line::from(vec![
            Span::styled(
                "  \u{2500}\u{2500} Steps \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
                Style::default().fg(DIMMER),
            ),
        ]));
        lines.push(Line::raw(""));

        let step_slice: Vec<_> = run.steps.iter().rev().take(5).collect();
        for step in step_slice.into_iter().rev() {
            let (icon, color) = if step.passed { ("\u{2714}", GREEN) } else { ("\u{2718}", RED) };
            let mc = method_color(&step.method);
            lines.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(icon, Style::default().fg(color)),
                Span::styled(" ", Style::default()),
                Span::styled(format!("[{}]", step.method), Style::default().fg(mc)),
                Span::styled(" ", Style::default()),
                Span::styled(truncate(&step.node_name, area.width.saturating_sub(14) as usize), Style::default().fg(TEXT)),
                Span::styled(
                    format!(" {}ms", step.duration_ms),
                    Style::default().fg(DIMMER),
                ),
            ]));
        }

        lines.push(Line::raw(""));
        lines.push(Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled("[Enter]", Style::default().fg(YELLOW)),
            Span::styled(" run again  ", Style::default().fg(DIM)),
        ]));
    } else {
        // No runs yet
        lines.push(Line::raw(""));
        lines.push(Line::raw(""));
        lines.push(Line::raw(""));
        lines.push(Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::styled("\u{25CB}  No runs yet", Style::default().fg(DIM)),
        ]));
        lines.push(Line::raw(""));
        lines.push(Line::raw(""));
        lines.push(Line::from(vec![
            Span::styled("    Press ", Style::default().fg(TEXT_DIM)),
            Span::styled("[Enter]", Style::default().fg(YELLOW)),
            Span::styled(" to run the", Style::default().fg(TEXT_DIM)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("    selected flow.", Style::default().fg(TEXT_DIM)),
        ]));
    }

    let title = if app.flow_running {
        " \u{27F3} Stats "
    } else if app.last_run.is_some() {
        " Stats "
    } else {
        " Stats "
    };

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(title, title_style()))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(border_style()),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(p, area);
}

// ── Nodes-only state (no flows) ───────────────────────────────────────────────

fn render_overview_nodes_only(f: &mut Frame, app: &ApiApp, area: Rect) {
    let outer = Block::default()
        .title(Span::styled(" Overview ", title_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style());
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let mut lines: Vec<Line> = vec![
        Line::raw(""),
        Line::from(vec![
            Span::styled("  No flows yet \u{2014} create one with:", Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::styled("infynon weave flow create <name>", Style::default().fg(CYAN)),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled(
                "  \u{2500}\u{2500} Nodes in library \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
                Style::default().fg(DIMMER),
            ),
        ]),
        Line::raw(""),
    ];

    let mut node_list: Vec<(&String, &crate::api::types::Node)> = app.nodes.iter().collect();
    node_list.sort_by_key(|(id, _)| id.as_str());

    for (id, node) in &node_list {
        let mc = method_color(&node.method);
        lines.push(Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(format!("[{}]", node.method), Style::default().fg(mc)),
            Span::styled(" ", Style::default()),
            Span::styled(format!("{:<20}", truncate(id, 20)), Style::default().fg(TEXT)),
            Span::styled(truncate(&node.path, 36), Style::default().fg(TEXT_DIM)),
        ]));
    }

    let p = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(p, inner);
}

// ── No-flows hint ─────────────────────────────────────────────────────────────

pub(super) fn render_no_flows_hint(f: &mut Frame, area: Rect, tab_name: &str) {
    let p = Paragraph::new(vec![
        Line::raw(""),
        Line::from(vec![
            Span::styled(
                format!("  {} \u{2014} no flows yet.", tab_name),
                Style::default().fg(WHITE).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled("  Press ", Style::default().fg(TEXT_DIM)),
            Span::styled("[1]", Style::default().fg(YELLOW)),
            Span::styled(" for Overview to get started.", Style::default().fg(TEXT_DIM)),
        ]),
    ])
    .block(
        Block::default()
            .title(Span::styled(format!(" {} ", tab_name), title_style()))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style()),
    )
    .wrap(Wrap { trim: false });
    f.render_widget(p, area);
}

// ── Welcome screen (no flows, no nodes) ──────────────────────────────────────

fn render_welcome_screen(f: &mut Frame, area: Rect) {
    let outer = Block::default()
        .title(Span::styled(" Weave \u{2014} API Flow Testing ", title_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(CYAN));
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let section_line = "  \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}";

    let lines = vec![
        Line::raw(""),
        Line::from(vec![
            Span::styled("  No flows detected. ", Style::default().fg(WHITE).add_modifier(Modifier::BOLD)),
            Span::styled("Build an API test in 4 steps:", Style::default().fg(TEXT_DIM)),
        ]),
        Line::raw(""),
        Line::from(vec![Span::styled(section_line, Style::default().fg(DIMMER))]),
        Line::raw(""),
        // Step 1
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(" 1 ", Style::default().fg(BG).bg(CYAN).add_modifier(Modifier::BOLD)),
            Span::styled("  Create nodes", Style::default().fg(TEXT)),
        ]),
        Line::from(vec![
            Span::styled("      ", Style::default()),
            Span::styled(
                "infynon weave node create --ai \"POST /auth/login extracts token\"",
                Style::default().fg(CYAN),
            ),
        ]),
        Line::raw(""),
        // Step 2
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(" 2 ", Style::default().fg(BG).bg(CYAN).add_modifier(Modifier::BOLD)),
            Span::styled("  Create a flow", Style::default().fg(TEXT)),
        ]),
        Line::from(vec![
            Span::styled("      ", Style::default()),
            Span::styled("infynon weave flow create my-flow", Style::default().fg(CYAN)),
        ]),
        Line::raw(""),
        // Step 3
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(" 3 ", Style::default().fg(BG).bg(CYAN).add_modifier(Modifier::BOLD)),
            Span::styled("  Connect nodes", Style::default().fg(TEXT)),
        ]),
        Line::from(vec![
            Span::styled("      ", Style::default()),
            Span::styled(
                "infynon weave attach login-node --to dashboard-node",
                Style::default().fg(CYAN),
            ),
        ]),
        Line::raw(""),
        // Step 4
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(" 4 ", Style::default().fg(BG).bg(GREEN).add_modifier(Modifier::BOLD)),
            Span::styled("  Run the flow", Style::default().fg(TEXT)),
        ]),
        Line::from(vec![
            Span::styled("      ", Style::default()),
            Span::styled(
                "infynon weave flow run my-flow --base-url http://localhost:3000",
                Style::default().fg(CYAN),
            ),
        ]),
        Line::raw(""),
        Line::from(vec![Span::styled(section_line, Style::default().fg(DIMMER))]),
        Line::raw(""),
        // Keyboard shortcuts
        Line::from(vec![
            Span::styled("  Shortcuts:  ", Style::default().fg(TEXT_DIM)),
            Span::styled("[R]", Style::default().fg(YELLOW)),
            Span::styled(" refresh  ", Style::default().fg(DIM)),
            Span::styled("[2]", Style::default().fg(YELLOW)),
            Span::styled(" nodes  ", Style::default().fg(DIM)),
            Span::styled("[?]", Style::default().fg(YELLOW)),
            Span::styled(" help  ", Style::default().fg(DIM)),
            Span::styled("[q]", Style::default().fg(YELLOW)),
            Span::styled(" quit", Style::default().fg(DIM)),
        ]),
    ];

    let p = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(p, inner);
}
