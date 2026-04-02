use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::api::types::StepResult;
use crate::tui::api_app::{ApiApp, RunnerSubview};
use crate::tui::theme::*;

use super::{truncate, dashboard::render_no_flows_hint, environment, diff};

// ── Runner view dispatch ──────────────────────────────────────────────────────

pub(super) fn render_runner_view(f: &mut Frame, app: &ApiApp, area: Rect) {
    // Split: sub-tab bar (1 row) + content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(area);

    render_subtab_bar(f, app, chunks[0]);

    match app.runner_subview {
        RunnerSubview::Steps   => render_live_execution(f, app, chunks[1]),
        RunnerSubview::Latency => render_latency_profiler(f, app, chunks[1]),
        RunnerSubview::Diff    => diff::render_run_diff(f, app, chunks[1]),
        RunnerSubview::Context => environment::render_state_inspector(f, app, chunks[1]),
    }
}

// ── Sub-tab bar for Runner sub-views ──────────────────────────────────────────

fn render_subtab_bar(f: &mut Frame, app: &ApiApp, area: Rect) {
    let mut spans: Vec<Span> = vec![Span::raw(" ")];
    let all = RunnerSubview::all();
    let last_idx = all.len().saturating_sub(1);

    for (i, sub) in all.iter().enumerate() {
        let is_active = app.runner_subview == *sub;
        let label = sub.label();

        if is_active {
            spans.push(Span::styled(
                format!("\u{25B8} {}", label),
                Style::default().fg(ORANGE).add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(label, Style::default().fg(DIM)));
        }
        if i < last_idx {
            spans.push(Span::styled("  \u{2502}  ", Style::default().fg(DIMMER)));
        }
    }

    // Right-side hint
    spans.push(Span::styled("  ", Style::default()));
    spans.push(Span::styled("[Tab]", Style::default().fg(ORANGE)));
    spans.push(Span::styled(" switch", Style::default().fg(DIMMER)));

    let p = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(BG_SURFACE));
    f.render_widget(p, area);
}

// ── Live execution (Steps subview) ────────────────────────────────────────────

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

    // Running indicator in title
    let title = if app.flow_running || app.live_running {
        " Steps \u{25CF} RUNNING "
    } else if steps.is_empty() {
        " Steps "
    } else {
        " Last Run "
    };

    let step_count = steps.len();

    let items: Vec<ListItem> = steps.iter().enumerate().map(|(idx, step)| {
        // Status icon and color
        let (icon, icon_color) = if step.passed {
            ("\u{2714}", GREEN)
        } else if step.error.is_some() {
            ("\u{2718}", RED)
        } else {
            ("\u{26A0}", YELLOW)
        };

        // Status code
        let status = step.status_code.map(|s| s.to_string()).unwrap_or_else(|| "ERR".to_string());
        let sc_color = status_code_color(step.status_code);

        // Method color
        let m_color = method_color(&step.method);

        // Line 1: step number + icon + node_id + method badge + status code + url
        let w = (area.width as usize / 4).max(8).min(24);
        let line1 = Line::from(vec![
            Span::styled(format!("{:>2}. ", idx + 1), Style::default().fg(DIMMER)),
            Span::styled(format!("{} ", icon), Style::default().fg(icon_color).add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{:<w$}", truncate(&step.node_id, w)),
                Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("{:>7}", step.method), Style::default().fg(m_color)),
            Span::styled(" ", Style::default()),
            Span::styled(status, Style::default().fg(sc_color).add_modifier(Modifier::BOLD)),
            Span::styled("  ", Style::default()),
            Span::styled(truncate(&step.url, area.width.saturating_sub(44) as usize), Style::default().fg(DIM)),
        ]);

        // Line 2: latency + extracted variables
        let mut line2_spans = vec![
            Span::styled("      ", Style::default()),
            Span::styled(format!("{}ms", step.duration_ms), Style::default().fg(TEXT_DIM)),
        ];

        // Extracted variables summary
        if !step.extracted.is_empty() {
            let vars: Vec<String> = step.extracted.iter().map(|(k, v)| {
                let val_str = match v {
                    serde_json::Value::String(s) => truncate(s, 16).to_string(),
                    other => truncate(&other.to_string(), 16).to_string(),
                };
                format!("{} \u{2190} {}", k, val_str)
            }).collect();
            let vars_summary = truncate(&vars.join(" \u{00B7} "), area.width.saturating_sub(14) as usize);
            line2_spans.push(Span::styled(" \u{00B7} ", Style::default().fg(DIM)));
            line2_spans.push(Span::styled(vars_summary, Style::default().fg(ORANGE)));
        }

        let line2 = Line::from(line2_spans);

        let mut item_lines = vec![line1, line2];

        // Failed assertions (indented below)
        for ar in &step.assertion_results {
            if !ar.passed {
                item_lines.push(Line::from(vec![
                    Span::styled("        ", Style::default()),
                    Span::styled("\u{2718} ", Style::default().fg(RED)),
                    Span::styled(&ar.check, Style::default().fg(RED)),
                    Span::styled(format!("  ({})", ar.actual), Style::default().fg(DIMMER)),
                ]));
            }
        }

        // Error + request body preview for failed requests
        if let Some(err) = &step.error {
            item_lines.push(Line::from(vec![
                Span::styled("        ", Style::default()),
                Span::styled(format!("\u{26A1} {}", truncate(err, 60)), Style::default().fg(RED)),
            ]));
            // Show compact request body so user can see what was sent without opening modal
            if let Some(body) = &step.request_body {
                if !body.is_empty() {
                    item_lines.push(Line::from(vec![
                        Span::styled("        ", Style::default()),
                        Span::styled("body: ", Style::default().fg(DIMMER)),
                        Span::styled(truncate(body, area.width.saturating_sub(18) as usize), Style::default().fg(YELLOW)),
                    ]));
                }
            }
        }

        ListItem::new(item_lines)
    }).collect();

    // Bottom bar with keyboard hints
    let bottom_hint = if step_count > 0 {
        format!(
            " {} steps  \u{00B7} [\u{2191}\u{2193}] navigate [Enter] inspect [r] retry [b] body ",
            step_count
        )
    } else {
        " No steps recorded ".to_string()
    };

    let mut list_state = ListState::default();
    if !steps.is_empty() {
        list_state.select(Some(app.live_selected_step.min(steps.len().saturating_sub(1))));
    }

    let list = List::new(items)
        .block(
            Block::default()
                .title(Span::styled(title, title_style()))
                .title_bottom(Span::styled(bottom_hint, Style::default().fg(DIMMER)))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(border_style()),
        )
        .highlight_style(Style::default().bg(BG_SELECTED).fg(ORANGE).add_modifier(Modifier::BOLD));

    f.render_stateful_widget(list, area, &mut list_state);
}

// ── Latency profiler ──────────────────────────────────────────────────────────

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
                Line::from(vec![Span::styled(
                    "  Run a flow first:",
                    dim_style(),
                )]),
                Line::from(vec![Span::styled(
                    "    infynon weave flow run <flow-id> --base-url http://localhost:3000",
                    Style::default().fg(CYAN),
                )]),
                Line::raw(""),
                Line::from(vec![
                    Span::styled("  Then press ", dim_style()),
                    Span::styled("[R]", Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)),
                    Span::styled(" to refresh.", dim_style()),
                ]),
            ])
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(border_style()))
            .wrap(Wrap { trim: false });
            f.render_widget(p, area);
            return;
        }
    };

    if run.steps.is_empty() {
        let p = Paragraph::new(vec![
            Line::raw(""),
            Line::from(vec![Span::styled("  No step data in run.", dim_style())]),
        ])
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(border_style()));
        f.render_widget(p, area);
        return;
    }

    let block = Block::default()
        .title(Span::styled(" Latency Profiler ", title_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style());
    let inner = block.inner(area);
    f.render_widget(block, area);

    let max_ms = run.steps.iter().map(|s| s.duration_ms).max().unwrap_or(1).max(1);
    let bar_width = if inner.width > 30 { (inner.width as usize).saturating_sub(32) } else { 20 };

    let mut lines: Vec<Line> = vec![Line::raw("")];

    // Sort by latency descending
    let mut sorted_steps: Vec<&StepResult> = run.steps.iter().collect();
    sorted_steps.sort_by(|a, b| b.duration_ms.cmp(&a.duration_ms));

    for step in &sorted_steps {
        let bar_len = ((step.duration_ms as usize * bar_width) / max_ms as usize).max(1);
        let bar_fill = "\u{2588}".repeat(bar_len);
        let bar_trail = "\u{2591}".repeat(bar_width.saturating_sub(bar_len));
        let color = if step.duration_ms > 1000 {
            RED
        } else if step.duration_ms > 300 {
            ORANGE
        } else if step.duration_ms > 100 {
            YELLOW
        } else {
            GREEN
        };

        lines.push(Line::from(vec![
            Span::styled(format!("  {:<20}", truncate(&step.node_id, 20)), Style::default().fg(TEXT_DIM)),
            Span::styled(bar_fill, Style::default().fg(color)),
            Span::styled(bar_trail, Style::default().fg(DIMMER)),
            Span::styled(format!(" {}ms", step.duration_ms), Style::default().fg(WHITE).add_modifier(Modifier::BOLD)),
        ]));
    }

    // Summary section
    lines.push(Line::raw(""));
    lines.push(Line::from(vec![
        Span::styled(
            format!("  \u{2500}\u{2500} Summary \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}"),
            Style::default().fg(DIMMER),
        ),
    ]));
    lines.push(Line::raw(""));

    // Percentile row
    lines.push(Line::from(vec![
        Span::styled("  P50: ", Style::default().fg(TEXT_DIM)),
        Span::styled(format!("{:<8}", format!("{}ms", percentile(&run.steps, 50))), Style::default().fg(WHITE).add_modifier(Modifier::BOLD)),
        Span::styled(" P95: ", Style::default().fg(TEXT_DIM)),
        Span::styled(format!("{:<8}", format!("{}ms", percentile(&run.steps, 95))), Style::default().fg(WHITE).add_modifier(Modifier::BOLD)),
        Span::styled(" P99: ", Style::default().fg(TEXT_DIM)),
        Span::styled(format!("{:<8}", format!("{}ms", percentile(&run.steps, 99))), Style::default().fg(WHITE).add_modifier(Modifier::BOLD)),
    ]));

    // Max/Avg/Total row
    let avg = run.avg_latency_ms();
    let total = run.duration_ms();
    lines.push(Line::from(vec![
        Span::styled("  Max: ", Style::default().fg(TEXT_DIM)),
        Span::styled(format!("{:<8}", format!("{}ms", max_ms)), Style::default().fg(WHITE).add_modifier(Modifier::BOLD)),
        Span::styled(" Avg: ", Style::default().fg(TEXT_DIM)),
        Span::styled(format!("{:<8}", format!("{}ms", avg)), Style::default().fg(WHITE).add_modifier(Modifier::BOLD)),
        Span::styled(" Total: ", Style::default().fg(TEXT_DIM)),
        Span::styled(format!("{}ms", total), Style::default().fg(WHITE).add_modifier(Modifier::BOLD)),
    ]));

    // Node count
    lines.push(Line::from(vec![
        Span::styled(
            format!("  {} nodes", run.steps.len()),
            Style::default().fg(DIMMER),
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
