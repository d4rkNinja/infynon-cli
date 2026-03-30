use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Sparkline, Wrap,
    },
};

use crate::api::types::{FlowRunResult, ProbeSeverity, StepResult};
use crate::tui::api_app::{ApiApp, ApiView, AttachMode, GraphNode};
use crate::tui::theme::*;

// ── Top-level render ─────────────────────────────────────────────────────────

pub fn render(f: &mut Frame, app: &mut ApiApp) {
    let area = f.size();

    // Layout: info bar (1) | tab strip (1) | separator (1) | content (fill) | status (1)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // info bar
            Constraint::Length(1), // tab strip
            Constraint::Length(1), // separator line
            Constraint::Min(0),    // content
            Constraint::Length(1), // status bar
        ])
        .split(area);

    render_info_bar(f, app, chunks[0]);
    render_tab_strip(f, app, chunks[1]);
    render_separator(f, chunks[2]);

    match app.current_view {
        ApiView::Overview        => render_overview(f, app, chunks[3]),
        ApiView::FlowGraph       => render_flow_graph(f, app, chunks[3]),
        ApiView::LiveExecution   => render_live_execution(f, app, chunks[3]),
        ApiView::LatencyProfiler => render_latency_profiler(f, app, chunks[3]),
        ApiView::SecurityProbes  => render_security_probes(f, app, chunks[3]),
        ApiView::CoverageMap     => render_coverage_map(f, app, chunks[3]),
        ApiView::StateInspector  => render_state_inspector(f, app, chunks[3]),
        ApiView::RunDiff         => render_run_diff(f, app, chunks[3]),
        ApiView::NodeLibrary     => render_node_library(f, app, chunks[3]),
    }

    render_status_bar(f, app, chunks[4]);

    // Overlays (always on top)
    if app.show_help {
        render_help_overlay(f, area);
    }
    if app.current_view == ApiView::FlowGraph && app.detail_panel.is_some() {
        render_node_detail_overlay(f, app, area);
    }
    if app.attach_mode != AttachMode::Idle {
        render_attach_overlay(f, app, area);
    }
}

// ── Header: info bar ─────────────────────────────────────────────────────────

fn render_info_bar(f: &mut Frame, app: &ApiApp, area: Rect) {
    // Left side: brand + flow context + stats
    let mut spans: Vec<Span> = vec![
        Span::styled(" ◆ WEAVE", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)),
        Span::styled("  │  ", Style::default().fg(DIMMER)),
    ];

    match app.active_flow() {
        Some(fl) => {
            spans.push(Span::styled("◈ ", Style::default().fg(CYAN)));
            spans.push(Span::styled(fl.name.clone(), Style::default().fg(PURPLE).add_modifier(Modifier::BOLD)));
        }
        None => {
            spans.push(Span::styled("no flow selected", Style::default().fg(DIMMER)));
        }
    }

    spans.push(Span::styled("  │  ", Style::default().fg(DIMMER)));
    spans.push(Span::styled(
        format!("{} flows  {}  nodes", app.flows.len(), app.nodes.len()),
        Style::default().fg(DIM),
    ));

    // Shortcuts — right side (separated with enough space)
    spans.push(Span::styled("     ", Style::default()));
    for (key, label) in &[("R", "refresh"), ("/", "search"), ("?", "help"), ("q", "quit")] {
        spans.push(Span::styled(key.to_string(), Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)));
        spans.push(Span::styled(format!(" {}  ", label), Style::default().fg(DIMMER)));
    }

    let p = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(BG_HIGHLIGHT));
    f.render_widget(p, area);
}

// ── Header: tab strip ────────────────────────────────────────────────────────

fn render_tab_strip(f: &mut Frame, app: &ApiApp, area: Rect) {
    let mut spans: Vec<Span> = vec![Span::raw(" ")];

    for view in ApiView::all() {
        let is_active = app.current_view == *view;
        let num = view.key().to_string();
        let name = view.label();

        if is_active {
            // Active: solid cyan block  ▌ N · Label ▐
            spans.push(Span::styled("▌", Style::default().fg(CYAN).bg(BG_HIGHLIGHT)));
            spans.push(Span::styled(
                format!(" {} · {} ", num, name),
                Style::default().fg(BG).bg(CYAN).add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled("▐ ", Style::default().fg(CYAN).bg(BG_HIGHLIGHT)));
        } else {
            spans.push(Span::styled(
                format!(" {} ", num),
                Style::default().fg(DIMMER),
            ));
            spans.push(Span::styled(
                format!("{}  ", name),
                Style::default().fg(DIM),
            ));
        }
    }

    // Append flow-switching hint at the end
    spans.push(Span::styled("   [ ] flows", Style::default().fg(DIMMER)));

    let p = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(BG_HIGHLIGHT));
    f.render_widget(p, area);
}

// ── Header: separator line ────────────────────────────────────────────────────

fn render_separator(f: &mut Frame, area: Rect) {
    let line = "─".repeat(area.width as usize);
    let p = Paragraph::new(Line::from(vec![
        Span::styled(line, Style::default().fg(BORDER)),
    ]));
    f.render_widget(p, area);
}

// ── Status bar ────────────────────────────────────────────────────────────────

fn render_status_bar(f: &mut Frame, app: &ApiApp, area: Rect) {
    let mut spans: Vec<Span> = vec![];

    if let Some(msg) = app.active_notification() {
        // Notification toast
        spans.push(Span::styled(" ✦ ", Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)));
        spans.push(Span::styled(msg.to_string(), Style::default().fg(YELLOW)));
    } else {
        // Last run summary
        if let Some(run) = &app.last_run {
            let (icon, color) = if run.passed { ("✔", GREEN) } else { ("✘", RED) };
            spans.push(Span::styled(format!(" {} ", icon), Style::default().fg(color)));
            spans.push(Span::styled(
                format!("{}/{} steps  {}ms avg", run.passed_count(), run.steps.len(), run.avg_latency_ms()),
                Style::default().fg(TEXT_DIM),
            ));
            spans.push(Span::styled("  │  ", Style::default().fg(DIMMER)));
        } else {
            spans.push(Span::styled(" ○ no run yet  │  ", Style::default().fg(DIMMER)));
        }

        // Search indicator
        if !app.search_input.is_empty() {
            spans.push(Span::styled("/ ", Style::default().fg(YELLOW)));
            spans.push(Span::styled(&app.search_input, Style::default().fg(WHITE).add_modifier(Modifier::BOLD)));
            spans.push(Span::styled("  │  ", Style::default().fg(DIMMER)));
        }

        // Active flow position hint
        if app.flows.len() > 1 {
            spans.push(Span::styled(
                format!("flow {}/{}", app.active_flow_idx + 1, app.flows.len()),
                Style::default().fg(DIMMER),
            ));
            spans.push(Span::styled("  │  ", Style::default().fg(DIMMER)));
        }

        spans.push(Span::styled("R refresh  [ ] flows  ? help", Style::default().fg(DIMMER)));
    }

    let p = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(BG_HIGHLIGHT));
    f.render_widget(p, area);
}

// ── View 1: Overview ─────────────────────────────────────────────────────────

fn render_overview(f: &mut Frame, app: &ApiApp, area: Rect) {
    if app.flows.is_empty() {
        render_welcome_screen(f, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left: flow list
    render_flow_list(f, app, chunks[0]);
    // Right: quick stats
    render_quick_stats(f, app, chunks[1]);
}

fn render_flow_list(f: &mut Frame, app: &ApiApp, area: Rect) {
    let items: Vec<ListItem> = app.flows.iter().enumerate().map(|(i, flow)| {
        let status_icon = match app.flow_run_statuses.get(&flow.id) {
            Some(Some(true))  => Span::styled("✔ ", Style::default().fg(GREEN)),
            Some(Some(false)) => Span::styled("✘ ", Style::default().fg(RED)),
            _ => Span::styled("· ", Style::default().fg(DIM)),
        };
        let name = Span::styled(
            format!("{} ({} nodes)", flow.name, flow.all_node_ids().len()),
            if i == app.active_flow_idx {
                Style::default().fg(CYAN).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(TEXT)
            },
        );
        ListItem::new(Line::from(vec![status_icon, name]))
    }).collect();

    let mut state = ListState::default();
    state.select(Some(app.active_flow_idx));

    let list = List::new(items)
        .block(
            Block::default()
                .title(Span::styled(" Flows ", title_style()))
                .borders(Borders::ALL)
                .border_style(border_style()),
        )
        .highlight_style(selected_style());

    f.render_stateful_widget(list, area, &mut state);
}

fn render_quick_stats(f: &mut Frame, app: &ApiApp, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(vec![Span::raw("")]));

    if let Some(run) = &app.last_run {
        let pass_pct = if run.steps.is_empty() { 0 } else {
            (run.passed_count() * 100) / run.steps.len()
        };

        lines.push(Line::from(vec![
            Span::styled("  Last Run ", stat_label()),
            Span::styled(
                if run.passed { "PASSED" } else { "FAILED" },
                Style::default().fg(if run.passed { GREEN } else { RED }).add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Steps    ", stat_label()),
            Span::styled(
                format!("{}/{} passed", run.passed_count(), run.steps.len()),
                stat_value(),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Duration ", stat_label()),
            Span::styled(format!("{}ms", run.duration_ms()), stat_value()),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Avg lat  ", stat_label()),
            Span::styled(format!("{}ms", run.avg_latency_ms()), stat_value()),
        ]));

        lines.push(Line::raw(""));

        // Pass rate gauge
        let pct_label = format!("Pass rate  {}%", pass_pct);
        lines.push(Line::from(vec![Span::styled(format!("  {}", pct_label), stat_label())]));
    } else {
        lines.push(Line::from(vec![
            Span::styled("  No runs yet. Run a flow with: infynon weave flow run <id>", dim_style()),
        ]));
    }

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(" Stats ", title_style()))
                .borders(Borders::ALL)
                .border_style(border_style()),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(p, area);
}

// ── View 2: Flow Graph ────────────────────────────────────────────────────────

fn render_flow_graph(f: &mut Frame, app: &ApiApp, area: Rect) {
    let flow = match app.active_flow() {
        Some(f) => f,
        None => {
            render_welcome_screen(f, area);
            return;
        }
    };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(32)])
        .split(area);

    // Left: graph canvas
    render_graph_canvas(f, app, chunks[0]);
    // Right: legend + shortcuts
    render_graph_sidebar(f, app, flow, chunks[1]);
}

fn render_graph_canvas(f: &mut Frame, app: &ApiApp, area: Rect) {
    let block = Block::default()
        .title(Span::styled(" Flow Graph ", title_style()))
        .borders(Borders::ALL)
        .border_style(border_style());
    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.graph_layout.is_empty() {
        let p = Paragraph::new("No nodes in flow. Add nodes and attach them.")
            .style(dim_style());
        f.render_widget(p, inner);
        return;
    }

    // Compute grid: each node box is ~22 wide, ~4 tall
    // layers go top-to-bottom, cols go left-to-right
    let node_w: u16 = 22;
    let node_h: u16 = 4;
    let h_gap: u16 = 2;
    let v_gap: u16 = 2;

    let max_layer = app.graph_layout.iter().map(|g| g.layer).max().unwrap_or(0);
    let max_col = app.graph_layout.iter().map(|g| g.col).max().unwrap_or(0);

    // Render nodes
    for gnode in &app.graph_layout {
        let x = inner.x + (gnode.col as u16) * (node_w + h_gap);
        let y = inner.y + (gnode.layer as u16) * (node_h + v_gap);

        if x + node_w > inner.right() || y + node_h > inner.bottom() {
            continue; // out of bounds
        }

        let node_rect = Rect { x, y, width: node_w, height: node_h };

        let node = app.nodes.get(&gnode.node_id);
        let is_selected = app.graph_selected_id.as_deref() == Some(&gnode.node_id);

        // Get last step result for this node
        let step_result = app.last_run.as_ref().and_then(|run| {
            run.steps.iter().find(|s| s.node_id == gnode.node_id)
        });

        let border_style = if is_selected {
            Style::default().fg(CYAN).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(BORDER)
        };

        let status_icon = match step_result {
            Some(s) if s.passed => "✔",
            Some(_) => "✘",
            None => "·",
        };
        let status_color = match step_result {
            Some(s) if s.passed => GREEN,
            Some(_) => RED,
            None => DIM,
        };

        let method_str = node.map(|n| n.method.as_str()).unwrap_or("?");
        let path_str = node.map(|n| {
            let p = &n.path;
            if p.len() > 16 { format!("{}…", &p[..15]) } else { p.clone() }
        }).unwrap_or_else(|| gnode.node_id.clone());

        let id_display = if gnode.node_id.len() > 18 {
            format!("{}…", &gnode.node_id[..17])
        } else {
            gnode.node_id.clone()
        };

        let latency = step_result.map(|s| format!("{}ms", s.duration_ms)).unwrap_or_default();

        let lines = vec![
            Line::from(vec![
                Span::styled(format!("{} ", status_icon), Style::default().fg(status_color)),
                Span::styled(id_display, if is_selected {
                    Style::default().fg(CYAN).add_modifier(Modifier::BOLD)
                } else {
                    normal_style()
                }),
            ]),
            Line::from(vec![
                Span::styled(format!("{} ", method_str), Style::default().fg(YELLOW)),
                Span::styled(path_str, Style::default().fg(TEXT_DIM)),
            ]),
            Line::from(vec![Span::styled(latency, Style::default().fg(DIM))]),
        ];

        let node_block = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).border_style(border_style));
        f.render_widget(node_block, node_rect);

        // Draw arrows to successors
        if let Some(flow) = app.active_flow() {
            for edge in flow.successors(&gnode.node_id) {
                if let Some(target) = app.graph_layout.iter().find(|g| g.node_id == edge.to) {
                    let ty = inner.y + (target.layer as u16) * (node_h + v_gap);
                    let arrow_x = x + node_w / 2;
                    let arrow_start_y = y + node_h;
                    let arrow_end_y = ty.saturating_sub(1);

                    if arrow_x < inner.right() && arrow_start_y <= arrow_end_y {
                        for y_pos in arrow_start_y..=arrow_end_y {
                            if y_pos >= inner.bottom() { break; }
                            let sym = if y_pos == arrow_end_y { "↓" } else { "│" };
                            let r = Rect { x: arrow_x, y: y_pos, width: 1, height: 1 };
                            f.render_widget(Paragraph::new(sym).style(Style::default().fg(DIMMER)), r);
                        }
                    }
                }
            }
        }
    }

}

fn render_graph_sidebar(f: &mut Frame, app: &ApiApp, flow: &crate::api::types::Flow, area: Rect) {
    let selected_node = app.graph_selected_id.as_ref()
        .and_then(|id| app.nodes.get(id));

    let mut lines: Vec<Line> = vec![Line::raw("")];

    if let Some(node) = selected_node {
        lines.push(Line::from(vec![
            Span::styled("  Selected: ", stat_label()),
        ]));
        lines.push(Line::from(vec![
            Span::styled(format!("  {}", node.id), Style::default().fg(CYAN).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(vec![
            Span::styled(format!("  {} {}", node.method, node.path), Style::default().fg(YELLOW)),
        ]));
        lines.push(Line::raw(""));

        // Last run result
        if let Some(run) = &app.last_run {
            if let Some(step) = run.steps.iter().find(|s| s.node_id == node.id) {
                let icon = if step.passed { "✔" } else { "✘" };
                let color = if step.passed { GREEN } else { RED };
                lines.push(Line::from(vec![
                    Span::styled(format!("  {} {} {}ms", icon, step.status_code.unwrap_or(0), step.duration_ms), Style::default().fg(color)),
                ]));
            }
        }

        lines.push(Line::raw(""));
        lines.push(Line::from(vec![Span::styled("  Extractions:", stat_label())]));
        for e in &node.extractions {
            lines.push(Line::from(vec![
                Span::styled(format!("   {} ← {}", e.name, e.from), Style::default().fg(TEXT_DIM)),
            ]));
        }
    } else {
        lines.push(Line::from(vec![Span::styled("  No node selected", dim_style())]));
    }

    lines.push(Line::raw(""));
    lines.push(Line::from(vec![Span::styled("  ── Controls ──", dim_style())]));
    lines.push(Line::from(vec![Span::styled("  ↑↓←→  navigate", dim_style())]));
    lines.push(Line::from(vec![Span::styled("  Enter  inspect", dim_style())]));
    lines.push(Line::from(vec![Span::styled("  a  attach node", dim_style())]));
    lines.push(Line::from(vec![Span::styled("  d  detach edge", dim_style())]));
    lines.push(Line::from(vec![Span::styled("  x  chaos inject", dim_style())]));
    lines.push(Line::from(vec![Span::styled("  R  refresh", dim_style())]));

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(" Info ", title_style()))
                .borders(Borders::ALL)
                .border_style(border_style()),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(p, area);
}

// ── View 3: Live Execution ────────────────────────────────────────────────────

fn render_live_execution(f: &mut Frame, app: &ApiApp, area: Rect) {
    let steps: Vec<&StepResult> = if app.live_steps.is_empty() {
        app.last_run.as_ref()
            .map(|r| r.steps.iter().collect())
            .unwrap_or_default()
    } else {
        app.live_steps.iter().collect()
    };

    let items: Vec<ListItem> = steps.iter().map(|step| {
        let icon = if step.passed {
            Span::styled("✔ ", Style::default().fg(GREEN))
        } else if step.error.is_some() {
            Span::styled("✘ ", Style::default().fg(RED))
        } else {
            Span::styled("⚠ ", Style::default().fg(YELLOW))
        };

        let status = step.status_code.map(|s| s.to_string()).unwrap_or_else(|| "ERR".to_string());
        let line = Line::from(vec![
            icon,
            Span::styled(format!("{:<18}", step.node_id), Style::default().fg(CYAN)),
            Span::styled(format!(" {} ", step.method), Style::default().fg(YELLOW)),
            Span::styled(format!("{:<32}", truncate(&step.url, 32)), Style::default().fg(TEXT_DIM)),
            Span::styled(format!(" {} ", status), Style::default().fg(WHITE).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{}ms", step.duration_ms), Style::default().fg(DIM)),
        ]);

        let mut item_lines = vec![line];

        // Show failed assertions
        for ar in &step.assertion_results {
            if !ar.passed {
                item_lines.push(Line::from(vec![
                    Span::raw("   "),
                    Span::styled("✘ ", Style::default().fg(RED)),
                    Span::styled(&ar.check, Style::default().fg(RED)),
                    Span::styled(format!("  (actual: {})", ar.actual), Style::default().fg(DIMMER)),
                ]));
            }
        }

        if let Some(err) = &step.error {
            item_lines.push(Line::from(vec![
                Span::raw("   "),
                Span::styled(format!("⚡ {}", truncate(err, 60)), Style::default().fg(RED)),
            ]));
        }

        ListItem::new(item_lines)
    }).collect();

    let title = if app.live_running { " Live Execution ⟳ " } else { " Last Run " };

    let list = List::new(items)
        .block(
            Block::default()
                .title(Span::styled(title, title_style()))
                .borders(Borders::ALL)
                .border_style(border_style()),
        );

    f.render_widget(list, area);
}

// ── View 4: Latency Profiler ──────────────────────────────────────────────────

fn render_latency_profiler(f: &mut Frame, app: &ApiApp, area: Rect) {
    let run = match &app.last_run {
        Some(r) => r,
        None => {
            let p = Paragraph::new(vec![
                Line::raw(""),
                Line::from(vec![Span::styled("  No run data yet.", Style::default().fg(WHITE).add_modifier(Modifier::BOLD))]),
                Line::raw(""),
                Line::from(vec![Span::styled("  Run a flow first:", dim_style())]),
                Line::from(vec![Span::styled("    infynon weave flow run <flow-id> --base-url http://localhost:3000", Style::default().fg(CYAN))]),
                Line::raw(""),
                Line::from(vec![
                    Span::styled("  Then press ", dim_style()),
                    Span::styled("R", Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)),
                    Span::styled(" to refresh.", dim_style()),
                ]),
            ])
            .block(Block::default().borders(Borders::ALL).border_style(border_style()))
            .wrap(Wrap { trim: false });
            f.render_widget(p, area);
            return;
        }
    };

    let block = Block::default()
        .title(Span::styled(" Latency Profiler ", title_style()))
        .borders(Borders::ALL)
        .border_style(border_style());
    let inner = block.inner(area);
    f.render_widget(block, area);

    let max_ms = run.steps.iter().map(|s| s.duration_ms).max().unwrap_or(1).max(1);

    let mut lines: Vec<Line> = vec![Line::raw("")];

    let mut sorted_steps: Vec<&StepResult> = run.steps.iter().collect();
    sorted_steps.sort_by(|a, b| b.duration_ms.cmp(&a.duration_ms));

    for step in &sorted_steps {
        let bar_len = ((step.duration_ms * 40) / max_ms) as usize;
        let bar = "█".repeat(bar_len);
        let color = if step.duration_ms > 1000 { RED }
            else if step.duration_ms > 300 { YELLOW }
            else { GREEN };

        lines.push(Line::from(vec![
            Span::styled(format!("  {:<20}", truncate(&step.node_id, 20)), dim_style()),
            Span::styled(format!("{:<42}", bar), Style::default().fg(color)),
            Span::styled(format!(" {}ms", step.duration_ms), Style::default().fg(WHITE).add_modifier(Modifier::BOLD)),
        ]));
    }

    lines.push(Line::raw(""));
    lines.push(Line::from(vec![
        Span::styled(
            format!("  Average: {}ms   P95: {}ms   Max: {}ms",
                run.avg_latency_ms(),
                percentile(&run.steps, 95),
                max_ms,
            ),
            stat_label(),
        ),
    ]));

    let p = Paragraph::new(lines);
    f.render_widget(p, inner);
}

fn percentile(steps: &[StepResult], pct: u64) -> u64 {
    if steps.is_empty() { return 0; }
    let mut lats: Vec<u64> = steps.iter().map(|s| s.duration_ms).collect();
    lats.sort_unstable();
    let idx = ((pct as usize * lats.len()).saturating_sub(1)) / 100;
    lats[idx.min(lats.len() - 1)]
}

// ── View 5: Security Probes ───────────────────────────────────────────────────

fn render_security_probes(f: &mut Frame, app: &ApiApp, area: Rect) {
    let probes = &app.security_probes;

    if probes.is_empty() {
        let p = Paragraph::new(vec![
            Line::raw(""),
            Line::from(vec![Span::styled("  No security probes run yet.", dim_style())]),
            Line::raw(""),
            Line::from(vec![Span::styled("  Run probes with:", dim_style())]),
            Line::from(vec![Span::styled("  infynon weave ai probe <flow-id>", Style::default().fg(CYAN))]),
        ])
        .block(
            Block::default()
                .title(Span::styled(" Security Probes ", title_style()))
                .borders(Borders::ALL)
                .border_style(border_style()),
        );
        f.render_widget(p, area);
        return;
    }

    let items: Vec<ListItem> = probes.iter().map(|probe| {
        let (icon, icon_style) = if probe.passed {
            ("✔", Style::default().fg(GREEN))
        } else {
            match probe.severity {
                ProbeSeverity::Critical | ProbeSeverity::High => ("✘", Style::default().fg(RED)),
                ProbeSeverity::Medium => ("⚠", Style::default().fg(YELLOW)),
                ProbeSeverity::Low    => ("ℹ", Style::default().fg(CYAN)),
            }
        };

        let severity_span = if !probe.passed {
            Span::styled(
                format!(" [{}]", probe.severity.label()),
                Style::default().fg(RED).add_modifier(Modifier::BOLD),
            )
        } else {
            Span::raw("")
        };

        let mut lines = vec![Line::from(vec![
            Span::styled(format!("  {} ", icon), icon_style),
            Span::styled(probe.probe_type.label(), Style::default().fg(WHITE).add_modifier(Modifier::BOLD)),
            severity_span,
        ])];

        lines.push(Line::from(vec![
            Span::raw("     "),
            Span::styled(&probe.description, Style::default().fg(TEXT_DIM)),
        ]));

        if !probe.passed {
            if let Some(details) = &probe.details {
                lines.push(Line::from(vec![
                    Span::raw("     "),
                    Span::styled(truncate(details, 70), Style::default().fg(ORANGE)),
                ]));
            }
        }

        lines.push(Line::raw(""));
        ListItem::new(lines)
    }).collect();

    let critical = probes.iter().filter(|p| !p.passed && p.severity == ProbeSeverity::Critical).count();
    let high = probes.iter().filter(|p| !p.passed && p.severity == ProbeSeverity::High).count();
    let title = format!(" Security Probes — {} critical  {} high ", critical, high);

    let list = List::new(items)
        .block(
            Block::default()
                .title(Span::styled(title, title_style()))
                .borders(Borders::ALL)
                .border_style(border_style()),
        );

    f.render_widget(list, area);
}

// ── View 6: Coverage Map ──────────────────────────────────────────────────────

fn render_coverage_map(f: &mut Frame, app: &ApiApp, area: Rect) {
    let flow = match app.active_flow() {
        Some(f) => f,
        None => {
            f.render_widget(
                Paragraph::new("No active flow.").style(dim_style())
                    .block(Block::default().borders(Borders::ALL).border_style(border_style())),
                area,
            );
            return;
        }
    };

    let flow_node_ids = flow.all_node_ids();
    let total_nodes = app.nodes.len().max(1);
    let covered_nodes = flow_node_ids.len();
    let pct = (covered_nodes * 100) / total_nodes;

    let block = Block::default()
        .title(Span::styled(format!(" Coverage Map — {}% ", pct), title_style()))
        .borders(Borders::ALL)
        .border_style(border_style());
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(inner);

    // Coverage gauge
    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(CYAN).bg(DIMMER))
        .percent(pct as u16)
        .label(format!("{}% endpoints in flow", pct));
    f.render_widget(gauge, chunks[0]);

    // Node coverage list
    let mut items: Vec<ListItem> = Vec::new();

    for (id, node) in &app.nodes {
        let in_flow = flow_node_ids.contains(id);
        let tested = app.last_run.as_ref()
            .map(|r| r.steps.iter().any(|s| &s.node_id == id))
            .unwrap_or(false);

        let (icon, style) = match (in_flow, tested) {
            (true, true)  => ("▓ ", Style::default().fg(GREEN)),
            (true, false) => ("░ ", Style::default().fg(YELLOW)),
            (false, _)    => ("· ", Style::default().fg(DIM)),
        };

        items.push(ListItem::new(Line::from(vec![
            Span::styled(icon, style),
            Span::styled(format!("{:<20}", truncate(id, 20)), style),
            Span::styled(format!(" {} {}", node.method, truncate(&node.path, 30)), Style::default().fg(TEXT_DIM)),
            if !in_flow {
                Span::styled("  not in flow", Style::default().fg(DIMMER))
            } else if !tested {
                Span::styled("  not run", Style::default().fg(YELLOW))
            } else {
                Span::styled("  ✔", Style::default().fg(GREEN))
            },
        ])));
    }

    let list = List::new(items);
    f.render_widget(list, chunks[1]);
}

// ── View 7: State Inspector ───────────────────────────────────────────────────

fn render_state_inspector(f: &mut Frame, app: &ApiApp, area: Rect) {
    let run = match &app.last_run {
        Some(r) => r,
        None => {
            f.render_widget(
                Paragraph::new("No run yet.").style(dim_style())
                    .block(Block::default().borders(Borders::ALL).border_style(border_style())),
                area,
            );
            return;
        }
    };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left: final context
    render_context_panel(f, run, chunks[0]);
    // Right: schema drift (compare with previous run)
    render_schema_drift_panel(f, app, run, chunks[1]);
}

fn render_context_panel(f: &mut Frame, run: &FlowRunResult, area: Rect) {
    let mut lines: Vec<Line> = vec![Line::raw("")];

    for (key, val) in &run.final_context {
        let display = match val {
            serde_json::Value::String(s) => {
                if s.len() > 40 { format!("{}…", &s[..40]) } else { s.clone() }
            }
            other => truncate(&other.to_string(), 40),
        };
        lines.push(Line::from(vec![
            Span::styled(format!("  {:<20}", truncate(key, 20)), Style::default().fg(CYAN)),
            Span::styled(format!(" = {}", display), Style::default().fg(TEXT_DIM)),
        ]));
    }

    if lines.len() == 1 {
        lines.push(Line::from(vec![Span::styled("  No context captured", dim_style())]));
    }

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(" Final Context ", title_style()))
                .borders(Borders::ALL)
                .border_style(border_style()),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(p, area);
}

fn render_schema_drift_panel(f: &mut Frame, app: &ApiApp, run: &FlowRunResult, area: Rect) {
    let mut lines: Vec<Line> = vec![Line::raw("")];

    if let Some(prev_run) = &app.compare_run {
        // Compare step responses between runs
        for step in &run.steps {
            let prev_step = prev_run.steps.iter().find(|s| s.node_id == step.node_id);
            if let Some(prev) = prev_step {
                let curr_keys = extract_json_keys(step.response_body.as_deref());
                let prev_keys = extract_json_keys(prev.response_body.as_deref());

                let added: Vec<&String> = curr_keys.iter().filter(|k| !prev_keys.contains(k)).collect();
                let removed: Vec<&String> = prev_keys.iter().filter(|k| !curr_keys.contains(k)).collect();

                if !added.is_empty() || !removed.is_empty() {
                    lines.push(Line::from(vec![
                        Span::styled(format!("  {} ", step.node_id), Style::default().fg(CYAN).add_modifier(Modifier::BOLD)),
                    ]));
                    for k in &added {
                        lines.push(Line::from(vec![
                            Span::styled(format!("    + {}", k), Style::default().fg(GREEN)),
                        ]));
                    }
                    for k in &removed {
                        lines.push(Line::from(vec![
                            Span::styled(format!("    - {}", k), Style::default().fg(RED)),
                        ]));
                    }
                    lines.push(Line::raw(""));
                }
            }
        }

        if lines.len() == 1 {
            lines.push(Line::from(vec![Span::styled("  No schema changes detected", Style::default().fg(GREEN))]));
        }
    } else {
        lines.push(Line::from(vec![Span::styled("  Load a comparison run with [d]", dim_style())]));
        lines.push(Line::raw(""));
        lines.push(Line::from(vec![Span::styled("  This will diff response schemas", dim_style())]));
        lines.push(Line::from(vec![Span::styled("  between the two most recent runs.", dim_style())]));
    }

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(" Schema Drift ", title_style()))
                .borders(Borders::ALL)
                .border_style(border_style()),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(p, area);
}

fn extract_json_keys(body: Option<&str>) -> Vec<String> {
    body.and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
        .and_then(|v| v.as_object().map(|o| o.keys().cloned().collect()))
        .unwrap_or_default()
}

// ── View 8: Run Diff ──────────────────────────────────────────────────────────

fn render_run_diff(f: &mut Frame, app: &ApiApp, area: Rect) {
    let (run_a, run_b) = match (&app.last_run, &app.compare_run) {
        (Some(a), Some(b)) => (a, b),
        _ => {
            let p = Paragraph::new(vec![
                Line::raw(""),
                Line::from(vec![Span::styled("  No comparison runs loaded.", dim_style())]),
                Line::raw(""),
                Line::from(vec![Span::styled("  Press [d] to load last 2 runs for comparison.", dim_style())]),
            ])
            .block(
                Block::default()
                    .title(Span::styled(" Run Diff ", title_style()))
                    .borders(Borders::ALL)
                    .border_style(border_style()),
            );
            f.render_widget(p, area);
            return;
        }
    };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_run_column(f, run_a, "Run A (newer)", chunks[0]);
    render_run_column(f, run_b, "Run B (older)", chunks[1]);
}

fn render_run_column(f: &mut Frame, run: &FlowRunResult, title: &str, area: Rect) {
    let mut lines: Vec<Line> = vec![Line::raw("")];

    let overall = if run.passed {
        Line::from(vec![Span::styled("  ✔ PASSED", Style::default().fg(GREEN).add_modifier(Modifier::BOLD))])
    } else {
        Line::from(vec![Span::styled("  ✘ FAILED", Style::default().fg(RED).add_modifier(Modifier::BOLD))])
    };
    lines.push(overall);
    lines.push(Line::from(vec![
        Span::styled(
            format!("  {}/{} steps  {}ms avg", run.passed_count(), run.steps.len(), run.avg_latency_ms()),
            stat_label(),
        ),
    ]));
    lines.push(Line::raw(""));

    for step in &run.steps {
        let icon = if step.passed { "✔" } else { "✘" };
        let color = if step.passed { GREEN } else { RED };
        lines.push(Line::from(vec![
            Span::styled(format!("  {} {:<18}", icon, truncate(&step.node_id, 18)), Style::default().fg(color)),
            Span::styled(
                step.status_code.map(|s| s.to_string()).unwrap_or_else(|| "ERR".to_string()),
                Style::default().fg(WHITE),
            ),
            Span::styled(format!("  {}ms", step.duration_ms), Style::default().fg(DIM)),
        ]));
    }

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(format!(" {} ", title), title_style()))
                .borders(Borders::ALL)
                .border_style(border_style()),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(p, area);
}

// ── View 9: Node Library ──────────────────────────────────────────────────────

fn render_node_library(f: &mut Frame, app: &ApiApp, area: Rect) {
    let flow_node_ids: std::collections::HashSet<String> = app.active_flow()
        .map(|f| f.all_node_ids().into_iter().collect())
        .unwrap_or_default();

    let mut node_list: Vec<(&String, &crate::api::types::Node)> = app.nodes.iter().collect();
    node_list.sort_by_key(|(id, _)| id.as_str());

    // Apply search filter
    let filtered: Vec<_> = if app.search_active || !app.search_input.is_empty() {
        let q = app.search_input.to_lowercase();
        node_list.iter()
            .filter(|(id, n)| id.to_lowercase().contains(&q) || n.path.to_lowercase().contains(&q))
            .collect()
    } else {
        node_list.iter().collect()
    };

    if filtered.is_empty() {
        let msg = if !app.search_input.is_empty() {
            vec![
                Line::raw(""),
                Line::from(vec![
                    Span::styled("  No nodes match: ", dim_style()),
                    Span::styled(&app.search_input, Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)),
                ]),
                Line::raw(""),
                Line::from(vec![Span::styled("  Press Esc to clear search.", dim_style())]),
            ]
        } else {
            vec![
                Line::raw(""),
                Line::from(vec![Span::styled("  No nodes yet.", Style::default().fg(WHITE).add_modifier(Modifier::BOLD))]),
                Line::raw(""),
                Line::from(vec![Span::styled("  Create nodes with:", dim_style())]),
                Line::raw(""),
                Line::from(vec![Span::styled("    infynon weave node create", Style::default().fg(CYAN))]),
                Line::from(vec![Span::styled("    infynon weave node create --ai \"POST /auth/login extracts token\"", Style::default().fg(CYAN))]),
                Line::raw(""),
                Line::from(vec![Span::styled("  Nodes are stored in .infynon/api/nodes/", dim_style())]),
                Line::raw(""),
                Line::from(vec![
                    Span::styled("  Press ", dim_style()),
                    Span::styled("R", Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)),
                    Span::styled(" to refresh after creating nodes.", dim_style()),
                ]),
            ]
        };
        let p = Paragraph::new(msg)
            .block(Block::default()
                .title(Span::styled(" Node Library — empty ", title_style()))
                .borders(Borders::ALL)
                .border_style(border_style()))
            .wrap(Wrap { trim: false });
        f.render_widget(p, area);
        return;
    }

    let items: Vec<ListItem> = filtered.iter().enumerate().map(|(i, (id, node))| {
        let in_flow = flow_node_ids.contains(*id);
        let flow_marker = if in_flow {
            Span::styled("●", Style::default().fg(CYAN))
        } else {
            Span::styled("·", Style::default().fg(DIMMER))
        };

        let method_style = match node.method.as_str() {
            "GET"    => Style::default().fg(GREEN),
            "POST"   => Style::default().fg(CYAN),
            "PUT"    => Style::default().fg(YELLOW),
            "PATCH"  => Style::default().fg(ORANGE),
            "DELETE" => Style::default().fg(RED),
            _        => normal_style(),
        };

        let is_selected = i == app.selected_index;
        let id_style = if is_selected {
            Style::default().fg(CYAN).add_modifier(Modifier::BOLD)
        } else {
            normal_style()
        };

        Line::from(vec![
            Span::raw("  "),
            flow_marker,
            Span::raw(" "),
            Span::styled(format!("{:<22}", truncate(id, 22)), id_style),
            Span::styled(format!("{:<8}", node.method), method_style),
            Span::styled(format!("{:<32}", truncate(&node.path, 32)), Style::default().fg(TEXT_DIM)),
            Span::styled(
                format!("{} ext  {} assert", node.extractions.len(), node.assertions.len()),
                dim_style(),
            ),
        ]).into()
    }).collect();

    let search_suffix = if !app.search_input.is_empty() {
        format!(" — search: {}", app.search_input)
    } else {
        String::new()
    };

    let mut list_state = ListState::default();
    list_state.select(Some(app.selected_index.min(filtered.len().saturating_sub(1))));

    let list = List::new(items)
        .block(
            Block::default()
                .title(Span::styled(
                    format!(" Node Library ({} nodes){} ", app.nodes.len(), search_suffix),
                    title_style(),
                ))
                .borders(Borders::ALL)
                .border_style(border_style()),
        )
        .highlight_style(selected_style());

    f.render_stateful_widget(list, area, &mut list_state);
}

// ── Overlays ──────────────────────────────────────────────────────────────────

fn render_node_detail_overlay(f: &mut Frame, app: &ApiApp, area: Rect) {
    let panel = match &app.detail_panel {
        Some(p) => p,
        None => return,
    };

    let node = match app.nodes.get(&panel.node_id) {
        Some(n) => n,
        None => return,
    };

    // Center overlay
    let w = area.width.min(60);
    let h = area.height.min(24);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let overlay_area = Rect { x, y, width: w, height: h };

    f.render_widget(Clear, overlay_area);

    let mut lines: Vec<Line> = vec![Line::raw("")];
    lines.push(Line::from(vec![
        Span::styled(format!("  {} ", node.method), Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)),
        Span::styled(&node.path, Style::default().fg(CYAN).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::raw(""));

    if !node.headers.is_empty() {
        lines.push(Line::from(vec![Span::styled("  Headers:", dim_style())]));
        for (k, v) in &node.headers {
            lines.push(Line::from(vec![
                Span::styled(format!("    {}: ", k), Style::default().fg(TEXT_DIM)),
                Span::styled(truncate(v, 30), Style::default().fg(DIM)),
            ]));
        }
        lines.push(Line::raw(""));
    }

    if let Some(body) = &node.body_json {
        lines.push(Line::from(vec![Span::styled("  Body:", dim_style())]));
        lines.push(Line::from(vec![
            Span::styled(format!("    {}", truncate(body, 50)), Style::default().fg(TEXT_DIM)),
        ]));
        lines.push(Line::raw(""));
    }

    if !node.extractions.is_empty() {
        lines.push(Line::from(vec![Span::styled("  Extractions:", dim_style())]));
        for e in &node.extractions {
            lines.push(Line::from(vec![
                Span::styled(format!("    {} ← {}", e.name, e.from), Style::default().fg(GREEN)),
            ]));
        }
        lines.push(Line::raw(""));
    }

    if !node.assertions.is_empty() {
        lines.push(Line::from(vec![Span::styled("  Assertions:", dim_style())]));
        for a in &node.assertions {
            lines.push(Line::from(vec![
                Span::styled(format!("    {}", a.check), Style::default().fg(CYAN)),
            ]));
        }
    }

    // Show last step result
    if let Some(run) = &app.last_run {
        if let Some(step) = run.steps.iter().find(|s| s.node_id == node.id) {
            lines.push(Line::raw(""));
            let status_color = if step.passed { GREEN } else { RED };
            lines.push(Line::from(vec![
                Span::styled("  Last run: ", dim_style()),
                Span::styled(
                    format!("{} {}ms", step.status_code.map(|s| s.to_string()).unwrap_or_else(|| "ERR".to_string()), step.duration_ms),
                    Style::default().fg(status_color),
                ),
            ]));
        }
    }

    lines.push(Line::raw(""));
    lines.push(Line::from(vec![Span::styled("  [Enter] close", dim_style())]));

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(format!(" {} ", node.id), title_style()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(CYAN)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(p, overlay_area);
}

fn render_attach_overlay(f: &mut Frame, app: &ApiApp, area: Rect) {
    let from_id = match &app.attach_mode {
        AttachMode::SelectingTarget { from_node } => from_node.clone(),
        _ => return,
    };

    let w = 48u16;
    let h = 6u16;
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + area.height / 2 - 3;
    let overlay_area = Rect { x, y, width: w, height: h };

    f.render_widget(Clear, overlay_area);

    let lines = vec![
        Line::raw(""),
        Line::from(vec![
            Span::styled("  From: ", dim_style()),
            Span::styled(&from_id, Style::default().fg(CYAN).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  To:   ", dim_style()),
            Span::styled(format!("{}▌", app.attach_input), Style::default().fg(YELLOW)),
        ]),
        Line::raw(""),
        Line::from(vec![Span::styled("  [Enter] attach  [Esc] cancel", dim_style())]),
    ];

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(" Attach Node ", title_style()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(CYAN)),
        );

    f.render_widget(p, overlay_area);
}

fn render_help_overlay(f: &mut Frame, area: Rect) {
    let w = 56u16;
    let h = 22u16;
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let overlay_area = Rect { x, y, width: w, height: h };

    f.render_widget(Clear, overlay_area);

    let lines = vec![
        Line::raw(""),
        Line::from(vec![Span::styled("  ── Global ──────────────────────", dim_style())]),
        Line::from(vec![Span::styled("  1-9    Switch views", normal_style())]),
        Line::from(vec![Span::styled("  [ ]    Previous/next flow", normal_style())]),
        Line::from(vec![Span::styled("  R      Refresh data", normal_style())]),
        Line::from(vec![Span::styled("  /      Search", normal_style())]),
        Line::from(vec![Span::styled("  ?      This help", normal_style())]),
        Line::from(vec![Span::styled("  q      Quit", normal_style())]),
        Line::raw(""),
        Line::from(vec![Span::styled("  ── Flow Graph ──────────────────", dim_style())]),
        Line::from(vec![Span::styled("  ↑↓←→   Navigate nodes", normal_style())]),
        Line::from(vec![Span::styled("  Enter  Inspect node", normal_style())]),
        Line::from(vec![Span::styled("  a      Attach new node to selected", normal_style())]),
        Line::from(vec![Span::styled("  d      Detach edge from selected", normal_style())]),
        Line::from(vec![Span::styled("  x      Chaos inject", normal_style())]),
        Line::raw(""),
        Line::from(vec![Span::styled("  ── Run Diff ────────────────────", dim_style())]),
        Line::from(vec![Span::styled("  d      Load comparison runs", normal_style())]),
        Line::raw(""),
        Line::from(vec![Span::styled("  Any key to close this help", dim_style())]),
        Line::raw(""),
    ];

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(" Help ", title_style()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(CYAN)),
        );

    f.render_widget(p, overlay_area);
}

// ── Welcome / empty-state screen ─────────────────────────────────────────────

fn render_welcome_screen(f: &mut Frame, area: Rect) {
    let outer = Block::default()
        .title(Span::styled(" Weave — API Flow Testing ", title_style()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(CYAN));
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let sep = "  ─────────────────────────────────────────────────────────────────";

    let lines = vec![
        Line::raw(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("No flows detected. ", Style::default().fg(WHITE).add_modifier(Modifier::BOLD)),
            Span::styled("Build an API test in 4 commands:", Style::default().fg(TEXT_DIM)),
        ]),
        Line::raw(""),
        Line::from(vec![Span::styled(sep, Style::default().fg(DIMMER))]),
        Line::raw(""),
        Line::from(vec![
            Span::styled("  1 ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)),
            Span::styled("Create nodes  ", Style::default().fg(TEXT_DIM)),
            Span::styled(
                "infynon weave node create --ai \"POST /auth/login extracts token\"",
                Style::default().fg(CYAN),
            ),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled("  2 ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)),
            Span::styled("Create a flow  ", Style::default().fg(TEXT_DIM)),
            Span::styled("infynon weave flow create my-flow", Style::default().fg(CYAN)),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled("  3 ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)),
            Span::styled("Connect nodes  ", Style::default().fg(TEXT_DIM)),
            Span::styled(
                "infynon weave attach login-node --to dashboard-node",
                Style::default().fg(CYAN),
            ),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled("  4 ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)),
            Span::styled("Run the flow   ", Style::default().fg(TEXT_DIM)),
            Span::styled(
                "infynon weave flow run my-flow --base-url http://localhost:3000",
                Style::default().fg(CYAN),
            ),
        ]),
        Line::raw(""),
        Line::from(vec![Span::styled(sep, Style::default().fg(DIMMER))]),
        Line::raw(""),
        Line::from(vec![
            Span::styled("  Shortcuts:  ", Style::default().fg(TEXT_DIM)),
            Span::styled("R", Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)),
            Span::styled(" refresh  ", Style::default().fg(DIM)),
            Span::styled("9", Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)),
            Span::styled(" node library  ", Style::default().fg(DIM)),
            Span::styled("?", Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)),
            Span::styled(" help  ", Style::default().fg(DIM)),
            Span::styled("q", Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)),
            Span::styled(" quit", Style::default().fg(DIM)),
        ]),
    ];

    let p = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(p, inner);
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}
