use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, List, ListItem, ListState, Paragraph, Sparkline, Wrap,
    },
};

use crate::api::types::{FlowRunResult, ProbeSeverity, StepResult};
use crate::tui::api_app::{ApiApp, ApiView, AttachMode, BodyEditor, GraphNode, NodeField, NodeFieldEditor, PromptModal, StepDetailModal};
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
        ApiView::EnvContext      => render_env_context(f, app, chunks[3]), // takes &mut ApiApp for list state
        ApiView::StateInspector  => render_state_inspector(f, app, chunks[3]),
        ApiView::RunDiff         => render_run_diff(f, app, chunks[3]),
        ApiView::NodeLibrary     => render_node_library(f, app, chunks[3]),
        ApiView::Config          => render_config(f, app, chunks[3]),
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
    if app.prompt_modal.is_some() {
        render_prompt_modal(f, app, area);
    }
    render_body_editor(f, app, area);
    render_step_detail_modal(f, app, area);
    render_node_field_editor_modal(f, app, area);
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

    // Left: flow list
    render_flow_list(f, app, chunks[0]);
    // Right: quick stats
    render_quick_stats(f, app, chunks[1]);
}

fn render_flow_list(f: &mut Frame, app: &ApiApp, area: Rect) {
    let items: Vec<ListItem> = app.flows.iter().enumerate().map(|(i, flow)| {
        let (status_icon, status_color) = match app.flow_run_statuses.get(&flow.id) {
            Some(Some(true))  => ("✔", GREEN),
            Some(Some(false)) => ("✘", RED),
            _ => ("·", DIM),
        };
        let is_selected = i == app.active_flow_idx;
        let name_style = if is_selected {
            Style::default().fg(CYAN).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(TEXT)
        };

        let node_count = flow.all_node_ids().len();
        let base = flow.base_url.as_deref().unwrap_or("—");

        let line = Line::from(vec![
            Span::styled(format!("{} ", status_icon), Style::default().fg(status_color)),
            Span::styled(format!("{:<22}", truncate(&flow.name, 22)), name_style),
            Span::styled(format!(" {:>2} nodes", node_count), Style::default().fg(DIM)),
            Span::styled(format!("  {}", truncate(base, 24)), Style::default().fg(DIMMER)),
        ]);
        ListItem::new(line)
    }).collect();

    let mut state = ListState::default();
    state.select(Some(app.active_flow_idx));

    let hints = if app.flow_running {
        " Flows  [RUNNING...]"
    } else {
        " Flows  Enter·run  a·run-all  ↑↓·select"
    };

    let list = List::new(items)
        .block(
            Block::default()
                .title(Span::styled(hints, title_style()))
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
            render_no_flows_hint(f, area, "Flow Graph");
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
    if app.flows.is_empty() && app.live_steps.is_empty() {
        render_no_flows_hint(f, area, "Live Execution");
        return;
    }
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

    let title = if app.flow_running { " Live Execution ⟳ RUNNING... " } else if app.live_running { " Live Execution ⟳ " } else { " Last Run " };

    let mut list_state = ListState::default();
    if !steps.is_empty() {
        list_state.select(Some(app.live_selected_step.min(steps.len().saturating_sub(1))));
    }

    let list = List::new(items)
        .block(
            Block::default()
                .title(Span::styled(title, title_style()))
                .title_bottom(Span::styled(
                    " ↑↓ navigate  Enter: inspect step  ",
                    Style::default().fg(DIMMER),
                ))
                .borders(Borders::ALL)
                .border_style(border_style()),
        )
        .highlight_style(selected_style());

    f.render_stateful_widget(list, area, &mut list_state);
}

// ── View 4: Latency Profiler ──────────────────────────────────────────────────

fn render_latency_profiler(f: &mut Frame, app: &ApiApp, area: Rect) {
    if app.flows.is_empty() {
        render_no_flows_hint(f, area, "Latency Profiler");
        return;
    }
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
    if app.flows.is_empty() {
        render_no_flows_hint(f, area, "Security Probes");
        return;
    }
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

fn render_env_context(f: &mut Frame, app: &mut ApiApp, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    render_env_panel(f, app, chunks[0]);
    render_flow_context_panel(f, app, chunks[1]);
}

fn render_env_panel(f: &mut Frame, app: &mut ApiApp, area: Rect) {
    use crate::api::commands::env as env_cmd;

    let entries = env_cmd::env_list();

    // Clamp selection
    if entries.is_empty() {
        app.env_selected = 0;
    } else {
        app.env_selected = app.env_selected.min(entries.len() - 1);
    }

    let editing = app.env_edit.is_some();
    let bottom_height: u16 = if editing { 5 } else { 2 };

    // Outer block (yellow border while editing)
    let border = if editing { Style::default().fg(YELLOW) } else { border_style() };
    let block_title = if editing { " Environment Variables — editing " } else { " Environment Variables (.infynon/.env) " };
    let block = Block::default()
        .title(Span::styled(block_title, title_style()))
        .borders(Borders::ALL)
        .border_style(border);
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Split inner: header | list | editor-or-hints
    let splits = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(bottom_height),
        ])
        .split(inner);

    // Header row
    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(format!("  {:<30}", "KEY"), Style::default().fg(DIMMER)),
            Span::styled("VALUE", Style::default().fg(DIMMER)),
        ]),
        Line::from(vec![Span::styled(format!("  {}", "─".repeat(62)), Style::default().fg(DIM))]),
    ]);
    f.render_widget(header, splits[0]);

    // List
    if entries.is_empty() {
        let hint = Paragraph::new(vec![
            Line::raw(""),
            Line::from(vec![Span::styled("  No variables set.", Style::default().fg(DIM))]),
            Line::raw(""),
            Line::from(vec![Span::styled("  Press [n] to add one.", Style::default().fg(DIMMER))]),
            Line::from(vec![Span::styled("  Reference in nodes as {$KEY}", Style::default().fg(DIMMER))]),
        ]);
        f.render_widget(hint, splits[1]);
    } else {
        let items: Vec<ListItem> = entries.iter().enumerate().map(|(i, (key, value))| {
            let selected = i == app.env_selected;
            let sensitive = env_cmd::looks_sensitive(key);
            let display = if !app.env_reveal && sensitive {
                env_cmd::mask(value)
            } else {
                truncate(value, 35).to_string()
            };
            let (key_style, val_style) = if selected {
                (
                    Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
                    Style::default().fg(WHITE),
                )
            } else {
                (
                    Style::default().fg(CYAN),
                    Style::default().fg(TEXT_DIM),
                )
            };
            let tag = if sensitive && !app.env_reveal {
                Span::styled(" [hidden]", Style::default().fg(DIMMER))
            } else {
                Span::raw("")
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("  {:<30}", truncate(key, 30)), key_style),
                Span::styled(display, val_style),
                tag,
            ]))
        }).collect();

        let mut list_state = ListState::default();
        list_state.select(Some(app.env_selected));

        let list = List::new(items)
            .highlight_style(Style::default().bg(BG_HIGHLIGHT));

        f.render_stateful_widget(list, splits[1], &mut list_state);
    }

    // Editor section or keyboard hints
    if let Some(edit) = &app.env_edit {
        let key_cursor  = if edit.editing_key  { "▌" } else { "" };
        let val_cursor  = if !edit.editing_key { "▌" } else { "" };
        let key_label_style = if edit.editing_key  { Style::default().fg(YELLOW) } else { Style::default().fg(TEXT_DIM) };
        let val_label_style = if !edit.editing_key { Style::default().fg(YELLOW) } else { Style::default().fg(TEXT_DIM) };
        let field_hint = if edit.is_new() {
            "  [Enter] Next/Save  [Tab] Switch field  [Esc] Cancel"
        } else {
            "  [Enter] Save  [Esc] Cancel"
        };
        let editor = Paragraph::new(vec![
            Line::from(vec![Span::styled(format!("  {}", "─".repeat(62)), Style::default().fg(DIM))]),
            Line::from(vec![
                Span::styled("  Key:   ", Style::default().fg(DIMMER)),
                Span::styled(format!("{}{}", edit.key_input, key_cursor), key_label_style),
            ]),
            Line::from(vec![
                Span::styled("  Value: ", Style::default().fg(DIMMER)),
                Span::styled(format!("{}{}", edit.value_input, val_cursor), val_label_style),
            ]),
            Line::from(vec![Span::styled(field_hint, Style::default().fg(DIMMER))]),
        ]);
        f.render_widget(editor, splits[2]);
    } else {
        let hints = Paragraph::new(vec![
            Line::from(vec![Span::styled(
                "  [n] Add  [Enter] Edit  [d] Delete  [v] Reveal  ↑↓/jk Navigate",
                Style::default().fg(DIMMER),
            )]),
        ]);
        f.render_widget(hints, splits[2]);
    }
}

fn render_flow_context_panel(f: &mut Frame, app: &ApiApp, area: Rect) {
    let mut lines: Vec<Line> = vec![Line::raw("")];

    match &app.last_run {
        None => {
            lines.push(Line::from(vec![Span::styled("  No flow run yet.", Style::default().fg(DIM))]));
            lines.push(Line::raw(""));
            lines.push(Line::from(vec![Span::styled("  Run a flow to see", Style::default().fg(DIMMER))]));
            lines.push(Line::from(vec![Span::styled("  extracted context here.", Style::default().fg(DIMMER))]));
            lines.push(Line::raw(""));
            lines.push(Line::from(vec![Span::styled("  Tip: seed initial vars with", Style::default().fg(DIMMER))]));
            lines.push(Line::from(vec![Span::styled("  --set KEY=VALUE at run time.", Style::default().fg(DIMMER))]));
        }
        Some(run) if run.final_context.is_empty() => {
            lines.push(Line::from(vec![Span::styled("  No context captured.", Style::default().fg(DIM))]));
            lines.push(Line::raw(""));
            lines.push(Line::from(vec![Span::styled("  Nodes must declare capture", Style::default().fg(DIMMER))]));
            lines.push(Line::from(vec![Span::styled("  rules to extract variables.", Style::default().fg(DIMMER))]));
        }
        Some(run) => {
            lines.push(Line::from(vec![
                Span::styled(format!("  {:<22}", "VARIABLE"), Style::default().fg(DIMMER)),
                Span::styled("VALUE", Style::default().fg(DIMMER)),
            ]));
            lines.push(Line::from(vec![
                Span::styled(format!("  {}", "─".repeat(36)), Style::default().fg(DIM)),
            ]));
            let mut sorted: Vec<_> = run.final_context.iter().collect();
            sorted.sort_by_key(|(k, _)| k.as_str());
            for (key, val) in &sorted {
                use crate::api::commands::env as env_cmd;
                let raw = match val {
                    serde_json::Value::String(s) => truncate(s, 18).to_string(),
                    other => truncate(&other.to_string(), 18).to_string(),
                };
                let display = if env_cmd::looks_sensitive(key) { env_cmd::mask(&raw) } else { raw };
                lines.push(Line::from(vec![
                    Span::styled(format!("  {:<22}", truncate(key, 22)), Style::default().fg(CYAN).add_modifier(Modifier::BOLD)),
                    Span::styled(display, Style::default().fg(TEXT_DIM)),
                ]));
            }
            lines.push(Line::raw(""));
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {} var(s) from last run", run.final_context.len()),
                    Style::default().fg(DIMMER),
                ),
            ]));
        }
    }

    let block = Block::default()
        .title(Span::styled(" Flow Context (last run) ", title_style()))
        .borders(Borders::ALL)
        .border_style(border_style());
    let para = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
    f.render_widget(para, area);
}

// ── View 7: State Inspector ───────────────────────────────────────────────────

fn render_state_inspector(f: &mut Frame, app: &ApiApp, area: Rect) {
    if app.flows.is_empty() {
        render_no_flows_hint(f, area, "State Inspector");
        return;
    }
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
    if app.flows.is_empty() {
        render_no_flows_hint(f, area, "Run Diff");
        return;
    }
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
    let selected_clamped = app.selected_index.min(filtered.len().saturating_sub(1));
    list_state.select(Some(selected_clamped));

    // Split into left list + right detail panel
    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    let list = List::new(items)
        .block(
            Block::default()
                .title(Span::styled(
                    format!(" Node Library ({} nodes){} ", app.nodes.len(), search_suffix),
                    title_style(),
                ))
                .title_bottom(Span::styled(
                    " Enter/r: run  n: name  p: path  m: method  b: body  d: desc  ↑↓: nav  /: search ",
                    Style::default().fg(DIMMER),
                ))
                .borders(Borders::ALL)
                .border_style(border_style()),
        )
        .highlight_style(selected_style());

    f.render_stateful_widget(list, panes[0], &mut list_state);

    // Right pane: node detail
    let selected_node = filtered.get(selected_clamped).map(|(id, node)| (*id, *node));
    render_node_library_detail(f, app, selected_node, panes[1]);
}

fn render_node_library_detail(
    f: &mut Frame,
    app: &ApiApp,
    selected: Option<(&String, &crate::api::types::Node)>,
    area: Rect,
) {
    let (node_id, node) = match selected {
        Some(p) => p,
        None => {
            let p = Paragraph::new(vec![
                Line::raw(""),
                Line::from(vec![Span::styled("  No node selected.", dim_style())]),
            ])
            .block(
                Block::default()
                    .title(Span::styled(" Node Details ", title_style()))
                    .borders(Borders::ALL)
                    .border_style(border_style()),
            );
            f.render_widget(p, area);
            return;
        }
    };

    let mut lines: Vec<Line> = vec![Line::raw("")];

    // Name + ID
    lines.push(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(&node.name, Style::default().fg(CYAN).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(vec![
        Span::styled(format!("  {}", node_id), Style::default().fg(DIMMER)),
    ]));

    // Method + Path
    let method_color = match node.method.as_str() {
        "GET"    => GREEN,
        "POST"   => CYAN,
        "PUT"    => YELLOW,
        "PATCH"  => ORANGE,
        "DELETE" => RED,
        _ => WHITE,
    };
    lines.push(Line::raw(""));
    lines.push(Line::from(vec![
        Span::styled(format!("  {}", node.method), Style::default().fg(method_color).add_modifier(Modifier::BOLD)),
        Span::styled("  ", Style::default()),
        Span::styled(&node.path, Style::default().fg(WHITE)),
    ]));

    // Description
    if let Some(desc) = &node.description {
        if !desc.is_empty() {
            lines.push(Line::raw(""));
            lines.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(desc, Style::default().fg(TEXT_DIM)),
            ]));
        }
    }

    // Headers
    if !node.headers.is_empty() {
        lines.push(Line::raw(""));
        lines.push(Line::from(vec![Span::styled("  ── Headers ──────────────────────", Style::default().fg(DIMMER))]));
        for (k, v) in &node.headers {
            let v_display = if v.len() > 28 { format!("{}…", &v[..27]) } else { v.clone() };
            lines.push(Line::from(vec![
                Span::styled(format!("  {:<20}", truncate(k, 20)), Style::default().fg(TEXT_DIM)),
                Span::styled(v_display, Style::default().fg(DIM)),
            ]));
        }
    }

    // Body
    if let Some(body) = &node.body_json {
        if !body.is_empty() {
            lines.push(Line::raw(""));
            lines.push(Line::from(vec![Span::styled("  ── Body ─────────────────────────", Style::default().fg(DIMMER))]));
            let pretty = serde_json::from_str::<serde_json::Value>(body)
                .map(|v| serde_json::to_string_pretty(&v).unwrap_or_else(|_| body.clone()))
                .unwrap_or_else(|_| body.clone());
            for owned_line in pretty.lines().take(8).map(|l| l.to_string()).collect::<Vec<_>>() {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(owned_line, Style::default().fg(ratatui::style::Color::Rgb(200, 200, 180))),
                ]));
            }
        }
    }

    // Assertions
    if !node.assertions.is_empty() {
        lines.push(Line::raw(""));
        lines.push(Line::from(vec![
            Span::styled(format!("  ── Assertions ({}) ─────────────────", node.assertions.len()), Style::default().fg(DIMMER)),
        ]));
        for a in &node.assertions {
            let (icon, col, enabled) = if a.enabled { ("✔", GREEN, true) } else { ("✘", DIM, false) };
            lines.push(Line::from(vec![
                Span::styled(format!("  {} ", icon), Style::default().fg(col)),
                Span::styled(
                    truncate(&a.check, 30),
                    if enabled { Style::default().fg(TEXT) } else { Style::default().fg(DIMMER) },
                ),
            ]));
        }
    }

    // Extractions
    if !node.extractions.is_empty() {
        lines.push(Line::raw(""));
        lines.push(Line::from(vec![
            Span::styled(format!("  ── Extractions ({}) ────────────────", node.extractions.len()), Style::default().fg(DIMMER)),
        ]));
        for e in &node.extractions {
            lines.push(Line::from(vec![
                Span::styled(format!("  {:<16}", truncate(&e.name, 16)), Style::default().fg(CYAN)),
                Span::styled(" ← ", Style::default().fg(DIMMER)),
                Span::styled(truncate(&e.from, 20), Style::default().fg(TEXT_DIM)),
            ]));
        }
    }

    // Prompt inputs
    if !node.prompt_inputs.is_empty() {
        lines.push(Line::raw(""));
        lines.push(Line::from(vec![
            Span::styled(format!("  ── Prompt Inputs ({}) ──────────────", node.prompt_inputs.len()), Style::default().fg(DIMMER)),
        ]));
        for pi in &node.prompt_inputs {
            let secret_tag = if pi.secret { "  secret" } else { "" };
            lines.push(Line::from(vec![
                Span::styled(format!("  {:<14}", truncate(&pi.var, 14)), Style::default().fg(CYAN)),
                Span::styled(format!("\"{}\"", truncate(&pi.label, 14)), Style::default().fg(TEXT_DIM)),
                Span::styled(secret_tag, Style::default().fg(DIMMER)),
            ]));
        }
    }

    // Edit hints
    lines.push(Line::raw(""));
    lines.push(Line::from(vec![Span::styled("  ── Edit ─────────────────────────", Style::default().fg(DIMMER))]));
    lines.push(Line::from(vec![
        Span::styled("  n", Style::default().fg(YELLOW)),
        Span::styled(" name  ", Style::default().fg(DIM)),
        Span::styled("p", Style::default().fg(YELLOW)),
        Span::styled(" path  ", Style::default().fg(DIM)),
        Span::styled("m", Style::default().fg(YELLOW)),
        Span::styled(" method", Style::default().fg(DIM)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  b", Style::default().fg(YELLOW)),
        Span::styled(" body  ", Style::default().fg(DIM)),
        Span::styled("d", Style::default().fg(YELLOW)),
        Span::styled(" desc  ", Style::default().fg(DIM)),
        Span::styled("Enter", Style::default().fg(YELLOW)),
        Span::styled(" run", Style::default().fg(DIM)),
    ]));

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(" Node Details ", title_style()))
                .borders(Borders::ALL)
                .border_style(border_style()),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(p, area);
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

// ── Config view (tab 0) ───────────────────────────────────────────────────────

fn render_config(f: &mut Frame, app: &ApiApp, area: Rect) {
    let md_check = if app.config_output_markdown { "[x]" } else { "[ ]" };
    let pdf_check = if app.config_output_pdf { "[x]" } else { "[ ]" };

    let url_display = if app.config_editing_url {
        format!("{}▌", app.config_url_input)
    } else {
        app.default_base_url.clone()
    };

    let lines = vec![
        Line::raw(""),
        Line::from(vec![Span::styled("  ◆ Weave Configuration", Style::default().fg(CYAN).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::styled("  ──────────────────────────────────────────────────────", Style::default().fg(BORDER))]),
        Line::raw(""),
        Line::from(vec![Span::styled("  Run Output", Style::default().fg(WHITE).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::styled("  ──────────", Style::default().fg(DIMMER))]),
        Line::from(vec![
            Span::styled(format!("  {} Save to Markdown    ", md_check), if app.config_output_markdown { Style::default().fg(GREEN) } else { Style::default().fg(TEXT_DIM) }),
            Span::styled("(toggle with 'm')", Style::default().fg(DIMMER)),
        ]),
        Line::from(vec![
            Span::styled(format!("  {} Save to PDF         ", pdf_check), if app.config_output_pdf { Style::default().fg(GREEN) } else { Style::default().fg(TEXT_DIM) }),
            Span::styled("(toggle with 'p')", Style::default().fg(DIMMER)),
        ]),
        Line::raw(""),
        Line::from(vec![Span::styled("  Run Behavior", Style::default().fg(WHITE).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::styled("  ────────────", Style::default().fg(DIMMER))]),
        Line::from(vec![
            Span::styled("  Default Base URL:  ", Style::default().fg(TEXT_DIM)),
            Span::styled(&url_display, Style::default().fg(CYAN).add_modifier(Modifier::BOLD)),
            Span::styled("   (press 'e' to edit)", Style::default().fg(DIMMER)),
        ]),
        Line::raw(""),
        Line::from(vec![Span::styled("  Keyboard Hints", Style::default().fg(WHITE).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::styled("  ──────────────", Style::default().fg(DIMMER))]),
        Line::from(vec![Span::styled("  m  Toggle markdown output", Style::default().fg(DIM))]),
        Line::from(vec![Span::styled("  p  Toggle PDF output", Style::default().fg(DIM))]),
        Line::from(vec![Span::styled("  e  Edit default base URL", Style::default().fg(DIM))]),
        Line::from(vec![Span::styled("  R  Refresh / reload", Style::default().fg(DIM))]),
        Line::raw(""),
    ];

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(" Configuration ", title_style()))
                .borders(Borders::ALL)
                .border_style(border_style()),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(p, area);
}

// ── Overview helper: nodes-only state ────────────────────────────────────────

fn render_overview_nodes_only(f: &mut Frame, app: &ApiApp, area: Rect) {
    let outer = Block::default()
        .title(Span::styled(" Overview ", title_style()))
        .borders(Borders::ALL)
        .border_style(border_style());
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let mut lines: Vec<Line> = vec![
        Line::raw(""),
        Line::from(vec![
            Span::styled("  No flows yet — create one with: ", Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)),
            Span::styled("infynon weave flow create <name>", Style::default().fg(CYAN)),
        ]),
        Line::raw(""),
        Line::from(vec![Span::styled("  ── Nodes in library ──────────────────────────────────────────", Style::default().fg(DIMMER))]),
        Line::raw(""),
    ];

    let mut node_list: Vec<(&String, &crate::api::types::Node)> = app.nodes.iter().collect();
    node_list.sort_by_key(|(id, _)| id.as_str());

    for (id, node) in &node_list {
        let method_style = match node.method.as_str() {
            "GET"    => Style::default().fg(GREEN),
            "POST"   => Style::default().fg(CYAN),
            "PUT"    => Style::default().fg(YELLOW),
            "PATCH"  => Style::default().fg(ORANGE),
            "DELETE" => Style::default().fg(RED),
            _        => normal_style(),
        };
        lines.push(Line::from(vec![
            Span::styled(format!("  {:<24}", truncate(id, 24)), Style::default().fg(TEXT)),
            Span::styled(format!("{:<8}", node.method), method_style),
            Span::styled(truncate(&node.path, 40), Style::default().fg(TEXT_DIM)),
        ]));
    }

    let p = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(p, inner);
}

// ── No-flows hint (tabs 2-8 empty state) ─────────────────────────────────────

fn render_no_flows_hint(f: &mut Frame, area: Rect, tab_name: &str) {
    let p = Paragraph::new(vec![
        Line::raw(""),
        Line::from(vec![
            Span::styled(
                format!("  {} — no flows yet.", tab_name),
                Style::default().fg(WHITE).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled("  Press ", dim_style()),
            Span::styled("1", Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)),
            Span::styled(" for Overview to get started.", dim_style()),
        ]),
    ])
    .block(
        Block::default()
            .title(Span::styled(format!(" {} ", tab_name), title_style()))
            .borders(Borders::ALL)
            .border_style(border_style()),
    )
    .wrap(Wrap { trim: false });
    f.render_widget(p, area);
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

// ── Prompt modal ──────────────────────────────────────────────────────────────

fn render_prompt_modal(f: &mut Frame, app: &ApiApp, area: Rect) {
    let modal = match &app.prompt_modal {
        Some(m) => m,
        None => return,
    };

    let input_count = modal.inputs.len() as u16;
    // height: title(1) + border(2) + subtitle(1) + blank(1) + inputs*(2 each) + footer(1) + blank(1)
    let h = (input_count * 2 + 6).max(8).min(area.height.saturating_sub(4));
    let w = (area.width * 60 / 100).max(50).min(area.width.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let overlay_area = Rect { x, y, width: w, height: h };

    f.render_widget(Clear, overlay_area);

    let mut lines: Vec<Line> = vec![
        Line::raw(""),
        Line::from(vec![Span::styled(
            "  This node needs values before it can send the request.",
            dim_style(),
        )]),
        Line::raw(""),
    ];

    for (i, pi) in modal.inputs.iter().enumerate() {
        let label = if pi.label.is_empty() { pi.var.as_str() } else { pi.label.as_str() };
        let is_current = i == modal.current_field;

        let label_style = if is_current {
            Style::default().fg(CYAN).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(DIM)
        };

        lines.push(Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(label, label_style),
        ]));

        let raw_val = modal.values.get(i).map(|s| s.as_str()).unwrap_or("");
        let display_val = if pi.secret {
            "*".repeat(raw_val.len())
        } else if raw_val.is_empty() {
            if let Some(ref d) = pi.default {
                format!("{} (default)", d)
            } else {
                String::new()
            }
        } else {
            raw_val.to_string()
        };

        let cursor = if is_current { "▌" } else { "" };
        let val_style = if is_current {
            Style::default().fg(YELLOW)
        } else if raw_val.is_empty() {
            Style::default().fg(DIMMER)
        } else {
            Style::default().fg(WHITE)
        };

        lines.push(Line::from(vec![
            Span::styled("  › ", dim_style()),
            Span::styled(format!("{}{}", display_val, cursor), val_style),
        ]));
    }

    lines.push(Line::raw(""));
    lines.push(Line::from(vec![Span::styled(
        "  Tab/↓ next  ↑ prev  Enter submit  Esc cancel",
        dim_style(),
    )]));

    let title = format!(" ◆ Input Required — {} ", modal.node_id);
    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(title, title_style()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(CYAN)),
        );

    f.render_widget(p, overlay_area);
}

// ── Body editor modal ─────────────────────────────────────────────────────────

fn render_body_editor(f: &mut Frame, app: &ApiApp, area: Rect) {
    use ratatui::style::Color;
    let editor = match &app.body_editor {
        Some(e) => e,
        None => return,
    };

    // Full-screen overlay with margin
    let w = area.width.saturating_sub(4).max(40);
    let h = area.height.saturating_sub(4).max(10);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let overlay = Rect { x, y, width: w, height: h };

    f.render_widget(Clear, overlay);

    // Visible content area inside the block borders
    let inner_h = h.saturating_sub(4) as usize; // minus title + borders + footer
    let visible_lines = inner_h.max(1);

    let scroll_top = editor.scroll_top;
    let end = (scroll_top + visible_lines).min(editor.lines.len());

    let mut content_lines: Vec<Line> = Vec::new();

    for (abs_i, line) in editor.lines[scroll_top..end].iter().enumerate() {
        let line_idx = scroll_top + abs_i;
        let is_cursor_line = line_idx == editor.cursor_row;

        let line_no = Span::styled(
            format!("{:>3} ", line_idx + 1),
            Style::default().fg(Color::Rgb(80, 80, 120)),
        );

        if is_cursor_line {
            // Render cursor inline
            let col = editor.cursor_col.min(line.len());
            let before = &line[..col];
            let cursor_char = if col < line.len() {
                line.chars().nth(col).unwrap_or(' ')
            } else {
                ' '
            };
            let after = if col < line.len() { &line[col + cursor_char.len_utf8()..] } else { "" };

            content_lines.push(Line::from(vec![
                line_no,
                Span::styled(before, Style::default().fg(Color::White)),
                Span::styled(
                    cursor_char.to_string(),
                    Style::default().fg(Color::Black).bg(Color::Cyan),
                ),
                Span::styled(after, Style::default().fg(Color::White)),
            ]));
        } else {
            content_lines.push(Line::from(vec![
                line_no,
                Span::styled(line.as_str(), Style::default().fg(Color::Rgb(200, 200, 220))),
            ]));
        }
    }

    // Padding if fewer lines than visible area
    while content_lines.len() < visible_lines {
        content_lines.push(Line::raw(""));
    }

    // Footer
    content_lines.push(Line::from(vec![Span::styled(
        "  Ctrl+S save  Esc cancel  ↑↓←→ move  Enter newline  Backspace delete",
        Style::default().fg(Color::Rgb(100, 100, 140)),
    )]));

    let line_count = editor.lines.len();
    let title = format!(" ◆ Edit Body — {} ({} lines) ", editor.node_id, line_count);

    let p = Paragraph::new(content_lines)
        .block(
            Block::default()
                .title(Span::styled(title, Style::default().fg(CYAN).add_modifier(Modifier::BOLD)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(CYAN)),
        );

    f.render_widget(p, overlay);
}

// ── Step detail modal ─────────────────────────────────────────────────────────

fn render_step_detail_modal(f: &mut Frame, app: &ApiApp, area: Rect) {
    use ratatui::style::Color;
    let modal = match &app.step_detail {
        Some(m) => m,
        None => return,
    };
    let step = &modal.step;

    // Near-full-screen overlay
    let w = area.width.saturating_sub(4).max(60);
    let h = area.height.saturating_sub(2).max(20);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let overlay = Rect { x, y, width: w, height: h };
    f.render_widget(Clear, overlay);

    let mut lines: Vec<Line> = vec![];

    // Header row
    let status_str = step.status_code.map(|s| s.to_string()).unwrap_or_else(|| "ERR".to_string());
    let status_color = match step.status_code {
        Some(s) if s < 300 => Color::Green,
        Some(s) if s < 400 => Color::Yellow,
        Some(_) => Color::Red,
        None => Color::Red,
    };
    lines.push(Line::raw(""));
    lines.push(Line::from(vec![
        Span::styled("  Status  ", Style::default().fg(Color::Rgb(100, 100, 140))),
        Span::styled(&status_str, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
        Span::styled("   Method  ", Style::default().fg(Color::Rgb(100, 100, 140))),
        Span::styled(&step.method, Style::default().fg(Color::Yellow)),
        Span::styled("   Time  ", Style::default().fg(Color::Rgb(100, 100, 140))),
        Span::styled(format!("{}ms", step.duration_ms), Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  URL     ", Style::default().fg(Color::Rgb(100, 100, 140))),
        Span::styled(&step.url, Style::default().fg(Color::Cyan)),
    ]));
    lines.push(Line::raw(""));

    // Error
    if let Some(err) = &step.error {
        lines.push(Line::from(vec![Span::styled("  ── Error ──────────────────────────────────", Style::default().fg(Color::Red))]));
        for chunk in err.chars().collect::<Vec<_>>().chunks(w.saturating_sub(6) as usize) {
            let s: String = chunk.iter().collect();
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(s, Style::default().fg(Color::Red)),
            ]));
        }
        lines.push(Line::raw(""));
    }

    // Assertions
    if !step.assertion_results.is_empty() {
        lines.push(Line::from(vec![Span::styled("  ── Assertions ─────────────────────────────", Style::default().fg(Color::Rgb(100, 100, 140)))]));
        for ar in &step.assertion_results {
            let (icon, col) = if ar.passed {
                ("  ✔ ", Color::Green)
            } else {
                ("  ✘ ", Color::Red)
            };
            lines.push(Line::from(vec![
                Span::styled(icon, Style::default().fg(col)),
                Span::styled(&ar.check, Style::default().fg(col)),
                Span::styled(format!("  →  {}", ar.actual), Style::default().fg(Color::Rgb(130, 130, 130))),
            ]));
        }
        lines.push(Line::raw(""));
    }

    // Extracted variables
    if !step.extracted.is_empty() {
        lines.push(Line::from(vec![Span::styled("  ── Extracted Variables ─────────────────────", Style::default().fg(Color::Rgb(100, 100, 140)))]));
        for (k, v) in &step.extracted {
            lines.push(Line::from(vec![
                Span::styled(format!("  {:<20}", k), Style::default().fg(Color::Cyan)),
                Span::styled(v.to_string(), Style::default().fg(Color::White)),
            ]));
        }
        lines.push(Line::raw(""));
    }

    // Request body
    if let Some(req_body) = &step.request_body {
        if !req_body.is_empty() {
            lines.push(Line::from(vec![Span::styled("  ── Request Body ───────────────────────────", Style::default().fg(Color::Rgb(100, 100, 140)))]));
            let pretty = serde_json::from_str::<serde_json::Value>(req_body)
                .map(|v| serde_json::to_string_pretty(&v).unwrap_or_else(|_| req_body.clone()))
                .unwrap_or_else(|_| req_body.clone());
            for owned_line in pretty.lines().take(20).map(|l| l.to_string()).collect::<Vec<_>>() {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(owned_line, Style::default().fg(Color::Rgb(200, 200, 180))),
                ]));
            }
            lines.push(Line::raw(""));
        }
    }

    // Response body
    if let Some(resp_body) = &step.response_body {
        if !resp_body.is_empty() {
            lines.push(Line::from(vec![Span::styled("  ── Response Body ──────────────────────────", Style::default().fg(Color::Rgb(100, 100, 140)))]));
            let pretty = serde_json::from_str::<serde_json::Value>(resp_body)
                .map(|v| serde_json::to_string_pretty(&v).unwrap_or_else(|_| resp_body.clone()))
                .unwrap_or_else(|_| resp_body.clone());
            for owned_line in pretty.lines().take(40).map(|l| l.to_string()).collect::<Vec<_>>() {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(owned_line, Style::default().fg(Color::Rgb(180, 220, 180))),
                ]));
            }
            lines.push(Line::raw(""));
        }
    }

    // Scroll
    let scroll = modal.scroll.min(lines.len().saturating_sub(1));

    let title = format!(" ◆ Step Details — {} ", step.node_id);
    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(title, Style::default().fg(CYAN).add_modifier(Modifier::BOLD)))
                .title_bottom(Span::styled(
                    " ↑↓ scroll  Esc close ",
                    Style::default().fg(Color::Rgb(80, 80, 120)),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(CYAN)),
        )
        .scroll((scroll as u16, 0));

    f.render_widget(p, overlay);
}

// ── Node field editor modal ───────────────────────────────────────────────────

fn render_node_field_editor_modal(f: &mut Frame, app: &ApiApp, area: Rect) {
    use ratatui::style::Color;
    let editor = match &app.node_field_editor {
        Some(e) => e,
        None => return,
    };

    let label = match editor.field {
        NodeField::Name => "Node Name",
        NodeField::Path => "Request Path",
        NodeField::Description => "Description",
        NodeField::Method => "Method",
    };

    let w = (area.width * 55 / 100).max(50).min(area.width.saturating_sub(4));
    let h = 7u16;
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let overlay = Rect { x, y, width: w, height: h };

    f.render_widget(Clear, overlay);

    let lines = vec![
        Line::raw(""),
        Line::from(vec![
            Span::styled(format!("  {}:  ", label), Style::default().fg(Color::Rgb(150, 150, 200))),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled("  › ", Style::default().fg(Color::Rgb(100, 100, 140))),
            Span::styled(&editor.input, Style::default().fg(Color::Yellow)),
            Span::styled("▌", Style::default().fg(Color::Cyan)),
        ]),
        Line::raw(""),
        Line::from(vec![Span::styled(
            "  Enter to save  Esc to cancel",
            Style::default().fg(Color::Rgb(80, 80, 120)),
        )]),
    ];

    let title = format!(" ◆ Edit {} ", label);
    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(title, Style::default().fg(CYAN).add_modifier(Modifier::BOLD)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(CYAN)),
        );

    f.render_widget(p, overlay);
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}
