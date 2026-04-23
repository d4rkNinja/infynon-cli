use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::tui::api_app::{ApiApp, AttachMode, NodeField};
use crate::tui::theme::*;

use super::{blank_line, section_header, truncate};

// ── Node detail overlay (Flows tab) ───────────────────────────────────────────

pub(super) fn render_node_detail_overlay(f: &mut Frame, app: &ApiApp, area: Rect) {
    let panel = match &app.detail_panel {
        Some(p) => p,
        None => return,
    };

    let node = match app.nodes.get(&panel.node_id) {
        Some(n) => n,
        None => return,
    };

    let w = (area.width * 3 / 4).max(42).min(64);
    let h = (area.height * 3 / 4).max(16).min(28);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let overlay_area = Rect {
        x,
        y,
        width: w,
        height: h,
    };

    f.render_widget(Clear, overlay_area);

    let mc = method_color(&node.method);
    let mut lines: Vec<Line> = vec![blank_line()];

    // ── Title line: method badge + path ────────────────────────────────────
    lines.push(Line::from(vec![
        Span::styled("  [", Style::default().fg(DIMMER)),
        Span::styled(
            &node.method,
            Style::default().fg(mc).add_modifier(Modifier::BOLD),
        ),
        Span::styled("] ", Style::default().fg(DIMMER)),
        Span::styled(
            truncate(&node.path, (w as usize).saturating_sub(10)),
            Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
        ),
    ]));

    // ── Description (if present) ───────────────────────────────────────────
    if let Some(desc) = &node.description {
        if !desc.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(
                    truncate(desc, (w as usize).saturating_sub(6)),
                    Style::default().fg(TEXT_DIM),
                ),
            ]));
        }
    }

    lines.push(blank_line());

    // ── Headers section ────────────────────────────────────────────────────
    if !node.headers.is_empty() {
        lines.push(section_header("Headers", w as usize));
        for (k, v) in &node.headers {
            lines.push(Line::from(vec![
                Span::styled("    ", Style::default()),
                Span::styled(format!("{}: ", k), Style::default().fg(TEXT_DIM)),
                Span::styled(
                    truncate(v, (w as usize).saturating_sub(20)),
                    Style::default().fg(DIM),
                ),
            ]));
        }
        lines.push(blank_line());
    }

    // ── Body section (pretty-printed JSON, max 8 lines) ────────────────────
    if let Some(body) = &node.body_json {
        if !body.is_empty() {
            lines.push(section_header("Body", w as usize));
            let pretty = serde_json::from_str::<serde_json::Value>(body)
                .map(|v| serde_json::to_string_pretty(&v).unwrap_or_else(|_| body.clone()))
                .unwrap_or_else(|_| body.clone());
            for line in pretty.lines().take(8) {
                lines.push(Line::from(vec![
                    Span::styled("    ", Style::default()),
                    Span::styled(
                        truncate(line, (w as usize).saturating_sub(12)),
                        Style::default().fg(TEXT_DIM),
                    ),
                ]));
            }
            let total = pretty.lines().count();
            if total > 8 {
                lines.push(Line::from(vec![
                    Span::styled("    ", Style::default()),
                    Span::styled(
                        format!("... +{} more lines", total - 8),
                        Style::default().fg(DIMMER),
                    ),
                ]));
            }
            lines.push(blank_line());
        }
    }

    // ── Extractions section ────────────────────────────────────────────────
    if !node.extractions.is_empty() {
        lines.push(section_header("Extractions", w as usize));
        for e in &node.extractions {
            lines.push(Line::from(vec![
                Span::styled("    ", Style::default()),
                Span::styled(&e.name, Style::default().fg(TEXT)),
                Span::styled(" <- ", Style::default().fg(GREEN)),
                Span::styled(&e.from, Style::default().fg(GREEN)),
            ]));
        }
        lines.push(blank_line());
    }

    // ── Assertions section ─────────────────────────────────────────────────
    if !node.assertions.is_empty() {
        lines.push(section_header("Assertions", w as usize));
        for a in &node.assertions {
            let state_icon = if a.enabled {
                Span::styled(" on", Style::default().fg(GREEN))
            } else {
                Span::styled(" off", Style::default().fg(DIMMER))
            };
            lines.push(Line::from(vec![
                Span::styled("    ", Style::default()),
                Span::styled(&a.check, Style::default().fg(CYAN)),
                state_icon,
            ]));
        }
    }

    // ── Last run result ────────────────────────────────────────────────────
    if let Some(run) = &app.last_run {
        if let Some(step) = run.steps.iter().find(|s| s.node_id == node.id) {
            lines.push(blank_line());
            lines.push(section_header("Last Run", w as usize));
            let status_str = step
                .status_code
                .map(|s| s.to_string())
                .unwrap_or_else(|| "ERR".to_string());
            let sc = status_code_color(step.status_code);
            let pass_icon = if step.passed { "pass" } else { "FAIL" };
            let pass_col = if step.passed { GREEN } else { RED };
            lines.push(Line::from(vec![
                Span::styled("    ", Style::default()),
                Span::styled(
                    pass_icon,
                    Style::default().fg(pass_col).add_modifier(Modifier::BOLD),
                ),
                Span::styled("  ", Style::default()),
                Span::styled(status_str, Style::default().fg(sc)),
                Span::styled("  ", Style::default()),
                Span::styled(
                    format!("{}ms", step.duration_ms),
                    Style::default().fg(TEXT_DIM),
                ),
            ]));
        }
    }

    // ── Close hint ─────────────────────────────────────────────────────────
    lines.push(blank_line());
    lines.push(Line::from(vec![
        Span::styled("  [", Style::default().fg(YELLOW)),
        Span::styled(
            "Enter",
            Style::default().fg(YELLOW).add_modifier(Modifier::BOLD),
        ),
        Span::styled("] close", Style::default().fg(DIM)),
    ]));

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(format!(" {} ", node.id), title_style()))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(CYAN))
                .style(Style::default().bg(BG)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(p, overlay_area);
}

// ── Attach overlay ────────────────────────────────────────────────────────────

pub(super) fn render_attach_overlay(f: &mut Frame, app: &ApiApp, area: Rect) {
    let from_id = match &app.attach_mode {
        AttachMode::SelectingTarget { from_node } => from_node.clone(),
        _ => return,
    };

    let w = 48u16;
    let h = 7u16;
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let overlay_area = Rect {
        x,
        y,
        width: w,
        height: h,
    };

    f.render_widget(Clear, overlay_area);

    let lines = vec![
        blank_line(),
        Line::from(vec![
            Span::styled("  From: ", Style::default().fg(DIM)),
            Span::styled(
                &from_id,
                Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  To:   ", Style::default().fg(DIM)),
            Span::styled(
                format!("{}▌", app.attach_input),
                Style::default().fg(YELLOW),
            ),
        ]),
        blank_line(),
        Line::from(vec![
            Span::styled("  [", Style::default().fg(YELLOW)),
            Span::styled(
                "Enter",
                Style::default().fg(YELLOW).add_modifier(Modifier::BOLD),
            ),
            Span::styled("] attach  ", Style::default().fg(DIM)),
            Span::styled("[", Style::default().fg(YELLOW)),
            Span::styled(
                "Esc",
                Style::default().fg(YELLOW).add_modifier(Modifier::BOLD),
            ),
            Span::styled("] cancel", Style::default().fg(DIM)),
        ]),
    ];

    let p = Paragraph::new(lines).block(
        Block::default()
            .title(Span::styled(" Attach Node ", title_style()))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(CYAN))
            .style(Style::default().bg(BG)),
    );

    f.render_widget(p, overlay_area);
}

// ── Help overlay ──────────────────────────────────────────────────────────────

pub(super) fn render_help_overlay(f: &mut Frame, area: Rect) {
    let w = (area.width * 4 / 5).max(50).min(68);
    let h = (area.height * 4 / 5).max(22).min(30);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let overlay_area = Rect {
        x,
        y,
        width: w,
        height: h,
    };

    f.render_widget(Clear, overlay_area);

    let block = Block::default()
        .title(Span::styled(" Keyboard Shortcuts ", title_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(CYAN))
        .style(Style::default().bg(BG));
    let inner = block.inner(overlay_area);
    f.render_widget(block, overlay_area);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    // ── Left column: Global shortcuts ──────────────────────────────────────
    let left_lines = vec![
        blank_line(),
        Line::from(vec![Span::styled(
            " Global",
            Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            " ────────────────────────────",
            Style::default().fg(DIMMER),
        )]),
        shortcut_line("1-6", "Switch views"),
        shortcut_line("[ ]", "Prev/next flow"),
        shortcut_line("R", "Refresh"),
        shortcut_line("/", "Search"),
        shortcut_line("?", "Help"),
        shortcut_line("q", "Quit"),
        blank_line(),
        Line::from(vec![Span::styled(
            " Flows",
            Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            " ────────────────────────────",
            Style::default().fg(DIMMER),
        )]),
        shortcut_line("Arrows", "Navigate nodes"),
        shortcut_line("Enter", "Inspect node"),
        shortcut_line("a", "Attach node"),
        shortcut_line("d", "Detach edge"),
        shortcut_line("r", "Run flow"),
    ];

    // ── Right column: View-specific shortcuts ──────────────────────────────
    let right_lines = vec![
        blank_line(),
        Line::from(vec![Span::styled(
            " Dashboard",
            Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            " ────────────────────────────",
            Style::default().fg(DIMMER),
        )]),
        shortcut_line("Enter", "Run selected flow"),
        shortcut_line("a", "Run all flows"),
        blank_line(),
        Line::from(vec![Span::styled(
            " Nodes",
            Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            " ────────────────────────────",
            Style::default().fg(DIMMER),
        )]),
        shortcut_line("Enter/r", "Run node"),
        shortcut_line("n/p/m", "Edit name/path/method"),
        shortcut_line("b/d", "Edit body / description"),
        shortcut_line("f", "Filter by method"),
        blank_line(),
        Line::from(vec![Span::styled(
            " Runner",
            Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            " ────────────────────────────",
            Style::default().fg(DIMMER),
        )]),
        shortcut_line("Tab", "Switch sub-views"),
        shortcut_line("Up/Dn", "Navigate steps"),
        shortcut_line("Enter", "Inspect step"),
        shortcut_line("r", "Retry step"),
        blank_line(),
        Line::from(vec![Span::styled(
            " Env",
            Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            " ────────────────────────────",
            Style::default().fg(DIMMER),
        )]),
        shortcut_line("n", "New variable"),
        shortcut_line("Enter", "Edit variable"),
        shortcut_line("d", "Delete variable"),
        shortcut_line("v", "Reveal value"),
    ];

    f.render_widget(Paragraph::new(left_lines), cols[0]);
    f.render_widget(Paragraph::new(right_lines), cols[1]);
}

/// Helper to format a single keyboard shortcut line.
fn shortcut_line(key: &str, desc: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!(" {:<7}", key), Style::default().fg(YELLOW)),
        Span::styled(desc.to_string(), Style::default().fg(DIM)),
    ])
}

// ── Settings / Config view ────────────────────────────────────────────────────

pub(super) fn render_settings(f: &mut Frame, app: &ApiApp, area: Rect) {
    let md_check = if app.config_output_markdown {
        Span::styled(
            "[x]",
            Style::default().fg(GREEN).add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled("[ ]", Style::default().fg(TEXT_DIM))
    };
    let pdf_check = if app.config_output_pdf {
        Span::styled(
            "[x]",
            Style::default().fg(GREEN).add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled("[ ]", Style::default().fg(TEXT_DIM))
    };

    let lines = vec![
        blank_line(),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                "Weave Configuration",
                Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
            ),
        ]),
        blank_line(),
        section_header("Run Output", area.width as usize),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            md_check,
            Span::styled("  Save to Markdown    ", Style::default().fg(TEXT)),
            Span::styled("(", Style::default().fg(DIMMER)),
            Span::styled("m", Style::default().fg(YELLOW)),
            Span::styled(")", Style::default().fg(DIMMER)),
        ]),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            pdf_check,
            Span::styled("  Save to PDF         ", Style::default().fg(TEXT)),
            Span::styled("(", Style::default().fg(DIMMER)),
            Span::styled("p", Style::default().fg(YELLOW)),
            Span::styled(")", Style::default().fg(DIMMER)),
        ]),
        blank_line(),
        section_header("Keyboard Hints", area.width as usize),
        shortcut_line("m", "Toggle markdown output"),
        shortcut_line("p", "Toggle PDF output"),
        shortcut_line("R", "Refresh / reload"),
        blank_line(),
    ];

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(" Configuration ", title_style()))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(border_style())
                .style(Style::default().bg(BG)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(p, area);
}

// ── Node field editor modal ───────────────────────────────────────────────────

pub(super) fn render_node_field_editor_modal(f: &mut Frame, app: &ApiApp, area: Rect) {
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

    let w = (area.width * 55 / 100)
        .max(50)
        .min(area.width.saturating_sub(4));
    let h = 9u16;
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let overlay = Rect {
        x,
        y,
        width: w,
        height: h,
    };

    f.render_widget(Clear, overlay);

    let lines = vec![
        blank_line(),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(format!("{}: ", label), Style::default().fg(TEXT_DIM)),
        ]),
        blank_line(),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(">", Style::default().fg(DIMMER)),
            Span::styled(" ", Style::default()),
            Span::styled(&editor.input, Style::default().fg(YELLOW)),
            Span::styled("|", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)),
        ]),
        blank_line(),
        Line::from(vec![
            Span::styled("  [", Style::default().fg(YELLOW)),
            Span::styled(
                "Enter",
                Style::default().fg(YELLOW).add_modifier(Modifier::BOLD),
            ),
            Span::styled("] save  ", Style::default().fg(DIM)),
            Span::styled("[", Style::default().fg(YELLOW)),
            Span::styled(
                "Esc",
                Style::default().fg(YELLOW).add_modifier(Modifier::BOLD),
            ),
            Span::styled("] cancel", Style::default().fg(DIM)),
        ]),
        blank_line(),
    ];

    let title = format!(" Edit {} ", label);
    let p = Paragraph::new(lines).block(
        Block::default()
            .title(Span::styled(title, title_style()))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(CYAN))
            .style(Style::default().bg(BG)),
    );

    f.render_widget(p, overlay);
}
