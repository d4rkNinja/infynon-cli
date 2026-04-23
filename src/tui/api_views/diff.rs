use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

use crate::api::types::FlowRunResult;
use crate::tui::api_app::ApiApp;
use crate::tui::theme::*;

use super::{dashboard::render_no_flows_hint, truncate};

// ── Run diff view ─────────────────────────────────────────────────────────────

pub(super) fn render_run_diff(f: &mut Frame, app: &ApiApp, area: Rect) {
    if app.flows.is_empty() {
        render_no_flows_hint(f, area, "Run Diff");
        return;
    }
    let (run_a, run_b) = match (&app.last_run, &app.compare_run) {
        (Some(a), Some(b)) => (a, b),
        _ => {
            let p = Paragraph::new(vec![
                Line::raw(""),
                Line::from(vec![Span::styled(
                    "  No comparison runs loaded.",
                    dim_style(),
                )]),
                Line::raw(""),
                Line::from(vec![
                    Span::styled("  Press ", dim_style()),
                    Span::styled(
                        "[d]",
                        Style::default().fg(YELLOW).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" to load last 2 runs for comparison.", dim_style()),
                ]),
            ])
            .block(
                Block::default()
                    .title(Span::styled(" Run Diff ", title_style()))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(border_style()),
            );
            f.render_widget(p, area);
            return;
        }
    };

    // Collect node IDs from both runs for diff highlighting
    let nodes_a: std::collections::HashMap<&str, &crate::api::types::StepResult> = run_a
        .steps
        .iter()
        .map(|s| (s.node_id.as_str(), s))
        .collect();
    let nodes_b: std::collections::HashMap<&str, &crate::api::types::StepResult> = run_b
        .steps
        .iter()
        .map(|s| (s.node_id.as_str(), s))
        .collect();

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_run_column(f, run_a, "Run A (newer)", Some(&nodes_b), chunks[0]);
    render_run_column(f, run_b, "Run B (older)", Some(&nodes_a), chunks[1]);
}

fn render_run_column(
    f: &mut Frame,
    run: &FlowRunResult,
    title: &str,
    other_run_nodes: Option<&std::collections::HashMap<&str, &crate::api::types::StepResult>>,
    area: Rect,
) {
    let mut lines: Vec<Line> = vec![Line::raw("")];
    let panel_w = area.width.saturating_sub(2) as usize;

    // PASSED/FAILED badge
    let badge = if run.passed {
        Line::from(vec![Span::styled(
            "  \u{2714} PASSED",
            Style::default().fg(GREEN).add_modifier(Modifier::BOLD),
        )])
    } else {
        Line::from(vec![Span::styled(
            "  \u{2718} FAILED",
            Style::default().fg(RED).add_modifier(Modifier::BOLD),
        )])
    };
    lines.push(badge);

    // Summary line: steps passed/total, avg latency
    lines.push(Line::from(vec![
        Span::styled(
            format!("  {}/{} steps passed", run.passed_count(), run.steps.len()),
            Style::default().fg(TEXT_DIM),
        ),
        Span::styled(
            format!("  \u{00B7}  {}ms avg", run.avg_latency_ms()),
            Style::default().fg(TEXT_DIM),
        ),
    ]));

    // Separator - responsive to panel width
    let sep_w = panel_w.saturating_sub(2).max(10);
    lines.push(Line::from(vec![Span::styled(
        format!("  {}", "\u{2500}".repeat(sep_w)),
        Style::default().fg(DIMMER),
    )]));

    // Responsive node ID width
    let node_w = (panel_w / 3).max(8).min(24);

    // Per-step list
    for step in &run.steps {
        let (icon, icon_color) = if step.passed {
            ("\u{2714}", GREEN)
        } else {
            ("\u{2718}", RED)
        };

        let status = step
            .status_code
            .map(|s| s.to_string())
            .unwrap_or_else(|| "ERR".to_string());
        let sc_color = status_code_color(step.status_code);

        // Check if this step differs from the other run
        let diff_marker = if let Some(other_nodes) = other_run_nodes {
            match other_nodes.get(step.node_id.as_str()) {
                Some(other_step) => {
                    if other_step.passed != step.passed {
                        // Different pass/fail status — highlight
                        if step.passed {
                            " \u{2191}" // up arrow: this run passed, other didn't
                        } else {
                            " \u{2193}" // down arrow: this run failed, other passed
                        }
                    } else {
                        ""
                    }
                }
                None => " +", // new step not in other run
            }
        } else {
            ""
        };

        let diff_color = if step.passed { GREEN } else { RED };

        let mut spans = vec![
            Span::styled(format!("  {} ", icon), Style::default().fg(icon_color)),
            Span::styled(
                format!("{:<w$}", truncate(&step.node_id, node_w), w = node_w),
                Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(status, Style::default().fg(sc_color)),
            Span::styled(
                format!("  {}ms", step.duration_ms),
                Style::default().fg(DIM),
            ),
        ];

        if !diff_marker.is_empty() {
            spans.push(Span::styled(
                diff_marker.to_string(),
                Style::default().fg(diff_color).add_modifier(Modifier::BOLD),
            ));
        }

        lines.push(Line::from(spans));
    }

    // Footer with total duration
    lines.push(Line::raw(""));
    lines.push(Line::from(vec![Span::styled(
        format!("  Total: {}ms", run.duration_ms()),
        Style::default().fg(DIMMER),
    )]));

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(format!(" {} ", title), title_style()))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(border_style()),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(p, area);
}
