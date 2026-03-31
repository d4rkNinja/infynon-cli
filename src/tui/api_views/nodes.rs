use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::tui::api_app::ApiApp;
use crate::tui::theme::*;

use super::{truncate, dashboard::render_no_flows_hint};

// ── Nodes view (redesigned Node Library + Node Inspector) ─────────────────────

pub(super) fn render_nodes_view(f: &mut Frame, app: &ApiApp, area: Rect) {
    let flow_node_ids: std::collections::HashSet<String> = app.active_flow()
        .map(|f| f.all_node_ids().into_iter().collect())
        .unwrap_or_default();

    let mut node_list: Vec<(&String, &crate::api::types::Node)> = app.nodes.iter().collect();
    node_list.sort_by_key(|(id, _)| id.as_str());

    // Apply search filter
    let filtered: Vec<_> = if app.search_active || !app.search_input.is_empty() {
        let q = app.search_input.to_lowercase();
        node_list.iter()
            .filter(|(id, n)| {
                id.to_lowercase().contains(&q)
                    || n.path.to_lowercase().contains(&q)
                    || n.name.to_lowercase().contains(&q)
            })
            .collect()
    } else {
        node_list.iter().collect()
    };

    // Split into left (Node Library) and right (Node Inspector) panels
    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    // ── Left Panel: Node Library ───────────────────────────────────────────

    if filtered.is_empty() {
        render_empty_library(f, app, &app.search_input, panes[0]);
        // Still render the right panel with "no node selected"
        render_node_library_detail(f, app, None, panes[1]);
        return;
    }

    let selected_clamped = app.selected_index.min(filtered.len().saturating_sub(1));
    let inner_width = panes[0].width.saturating_sub(2) as usize; // subtract borders

    let mut lines: Vec<Line> = Vec::new();

    for (i, (id, node)) in filtered.iter().enumerate() {
        let is_selected = i == selected_clamped;
        let in_flow = flow_node_ids.contains(*id);

        // Add a blank separator line between cards (not before the first one)
        if i > 0 {
            lines.push(Line::raw(""));
        }

        // Line 1: marker + node name + method badge
        let marker = if is_selected { "▸" } else { "●" };
        let marker_style = if is_selected {
            Style::default().fg(CYAN).add_modifier(Modifier::BOLD)
        } else if in_flow {
            Style::default().fg(CYAN)
        } else {
            Style::default().fg(DIMMER)
        };

        let name_style = if is_selected {
            Style::default().fg(CYAN).add_modifier(Modifier::BOLD)
        } else if in_flow {
            Style::default().fg(TEXT)
        } else {
            Style::default().fg(DIM)
        };

        let mc = method_color(&node.method);
        let method_label = format!("[{}]", node.method);
        let method_badge = Style::default().fg(mc).add_modifier(Modifier::BOLD);

        // Calculate spacing so method badge is right-aligned
        let name_display = truncate(&node.name, 24);
        let line1_content_len = 4 + name_display.chars().count(); // "  ● " + name
        let badge_len = method_label.len();
        let spacing = if inner_width > line1_content_len + badge_len + 2 {
            inner_width - line1_content_len - badge_len - 1
        } else {
            1
        };

        lines.push(Line::from(vec![
            Span::styled(format!("  {} ", marker), marker_style),
            Span::styled(name_display, name_style),
            Span::styled(" ".repeat(spacing), Style::default()),
            Span::styled(method_label, method_badge),
        ]));

        // Line 2: path (dimmed)
        let path_display = truncate(&node.path, inner_width.saturating_sub(4).max(10));
        lines.push(Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::styled(path_display, Style::default().fg(TEXT_DIM)),
        ]));

        // Line 3: extraction/assertion counts
        let ext_count = node.extractions.len();
        let assert_count = node.assertions.len();
        let summary = if ext_count == 0 && assert_count == 0 {
            "    no extractions or assertions".to_string()
        } else {
            let mut parts = Vec::new();
            if ext_count > 0 {
                parts.push(format!("{} extraction{}", ext_count, if ext_count == 1 { "" } else { "s" }));
            }
            if assert_count > 0 {
                parts.push(format!("{} assertion{}", assert_count, if assert_count == 1 { "" } else { "s" }));
            }
            format!("    {}", parts.join(" \u{00b7} "))
        };
        lines.push(Line::from(vec![
            Span::styled(summary, Style::default().fg(DIMMER)),
        ]));
    }

    // Build the bottom hints bar
    let filter_label = app.nodes_filter.label();
    let search_indicator = if app.search_active { "  \u{25b8} SEARCH" } else { "" };
    let bottom_title = format!(
        " Enter:run  r:run-node  n:name  p:path  m:method  b:body  d:desc  f:{}  /:search{} ",
        filter_label, search_indicator,
    );

    let search_suffix = if !app.search_input.is_empty() {
        format!(" \u{2014} search: {}", app.search_input)
    } else {
        String::new()
    };

    let node_count = app.nodes.len();
    let title = format!(" Node Library ({} node{}){} ", node_count, if node_count == 1 { "" } else { "s" }, search_suffix);

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(title, title_style()))
                .title_bottom(Span::styled(bottom_title, Style::default().fg(DIMMER)))
                .borders(Borders::ALL)
                .border_style(border_style())
        )
        .scroll((calc_library_scroll(selected_clamped, filtered.len()), 0))
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, panes[0]);

    // ── Right Panel: Node Inspector ────────────────────────────────────────

    let selected_node = filtered.get(selected_clamped).map(|(id, node)| (*id, *node));
    render_node_library_detail(f, app, selected_node, panes[1]);
}

/// Calculate scroll offset for the library panel so the selected card stays visible.
/// Each card is 3 or 4 lines (blank separator + 3 card lines), except the first card (3 lines).
fn calc_library_scroll(selected: usize, _total: usize) -> u16 {
    // Each card after the first takes 4 vertical lines (1 blank + 3 content).
    // The first card takes 3 lines (no preceding blank).
    // So the starting line of card N is: N * 4 (for N > 0), or 0 (for N == 0).
    // But we leave a ~2 line margin above the selected card.
    if selected == 0 {
        0
    } else {
        // Approximate: each card is 4 lines (separator + 3 lines)
        let card_start = selected * 4;
        if card_start > 4 {
            (card_start - 4) as u16
        } else {
            0
        }
    }
}

/// Render the empty state for the Node Library panel.
fn render_empty_library(f: &mut Frame, app: &ApiApp, search: &str, area: Rect) {
    let lines = if !search.is_empty() {
        vec![
            Line::raw(""),
            Line::from(vec![
                Span::styled("  No nodes match: ", dim_style()),
                Span::styled(search, Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)),
            ]),
            Line::raw(""),
            Line::from(vec![
                Span::styled("  Press ", dim_style()),
                Span::styled("Esc", Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)),
                Span::styled(" to clear search.", dim_style()),
            ]),
        ]
    } else {
        vec![
            Line::raw(""),
            Line::from(vec![
                Span::styled("  No nodes yet.", Style::default().fg(WHITE).add_modifier(Modifier::BOLD)),
            ]),
            Line::raw(""),
            Line::from(vec![Span::styled("  Create nodes with:", dim_style())]),
            Line::raw(""),
            Line::from(vec![
                Span::styled("    infynon weave node create", Style::default().fg(CYAN)),
            ]),
            Line::from(vec![
                Span::styled("    infynon weave node create --ai \"POST /auth/login extracts token\"", Style::default().fg(CYAN)),
            ]),
            Line::raw(""),
            Line::from(vec![
                Span::styled("  Nodes are stored in .infynon/api/nodes/", dim_style()),
            ]),
            Line::raw(""),
            Line::from(vec![
                Span::styled("  Press ", dim_style()),
                Span::styled("R", Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)),
                Span::styled(" to refresh after creating nodes.", dim_style()),
            ]),
        ]
    };

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(" Node Library \u{2014} empty ", title_style()))
                .borders(Borders::ALL)
                .border_style(border_style())
        )
        .wrap(Wrap { trim: false });
    f.render_widget(p, area);
}

// ── Right Panel: Node Inspector ────────────────────────────────────────────────

fn render_node_library_detail(
    f: &mut Frame,
    _app: &ApiApp,
    selected: Option<(&String, &crate::api::types::Node)>,
    area: Rect,
) {
    let (node_id, node) = match selected {
        Some(pair) => pair,
        None => {
            let lines = vec![
                Line::raw(""),
                Line::from(vec![
                    Span::styled("  Select a node from the library", Style::default().fg(DIM)),
                ]),
                Line::from(vec![
                    Span::styled("  to inspect its details here.", Style::default().fg(DIM)),
                ]),
                Line::raw(""),
                Line::from(vec![
                    Span::styled("  Use ", Style::default().fg(DIMMER)),
                    Span::styled("\u{2191}\u{2193}", Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)),
                    Span::styled(" to navigate the list.", Style::default().fg(DIMMER)),
                ]),
            ];
            let p = Paragraph::new(lines)
                .block(
                    Block::default()
                        .title(Span::styled(" Node Inspector ", title_style()))
                        .borders(Borders::ALL)
                        .border_style(border_style())
                )
                .wrap(Wrap { trim: false });
            f.render_widget(p, area);
            return;
        }
    };

    let mut lines: Vec<Line> = Vec::new();
    let inner_width = area.width.saturating_sub(2) as usize; // subtract borders

    // ── Header: method badge + path ────────────────────────────────────────
    lines.push(Line::raw(""));

    let mc = method_color(&node.method);
    let method_badge = format!("[{}]", node.method);
    lines.push(Line::from(vec![
        Span::styled(format!("  {} ", method_badge), Style::default().fg(mc).add_modifier(Modifier::BOLD)),
        Span::styled(&node.path, Style::default().fg(WHITE).add_modifier(Modifier::BOLD)),
    ]));

    // Node name (secondary header line)
    lines.push(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(&node.name, Style::default().fg(TEXT)),
        Span::styled(format!("  id:{}", truncate(node_id, 20)), Style::default().fg(DIMMER)),
    ]));

    // ── Description ────────────────────────────────────────────────────────
    if let Some(desc) = &node.description {
        if !desc.is_empty() {
            lines.push(Line::raw(""));
            lines.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(desc, Style::default().fg(TEXT_DIM)),
            ]));
        }
    }

    // ── Headers section ────────────────────────────────────────────────────
    if !node.headers.is_empty() {
        lines.push(Line::raw(""));
        lines.push(section_header("Headers", node.headers.len(), inner_width));
        for (k, v) in &node.headers {
            let max_val = inner_width.saturating_sub(24).max(10);
            let v_display = truncate(v, max_val);
            lines.push(Line::from(vec![
                Span::styled(format!("  {:<18}", truncate(k, 18)), Style::default().fg(TEXT_DIM)),
                Span::styled("  ", Style::default()),
                Span::styled(v_display, Style::default().fg(DIM)),
            ]));
        }
    }

    // ── Body section ───────────────────────────────────────────────────────
    if let Some(body) = &node.body_json {
        if !body.is_empty() {
            lines.push(Line::raw(""));
            lines.push(section_header("Body", 0, inner_width));
            let pretty = serde_json::from_str::<serde_json::Value>(body)
                .map(|v| serde_json::to_string_pretty(&v).unwrap_or_else(|_| body.clone()))
                .unwrap_or_else(|_| body.clone());
            let body_color = ratatui::style::Color::Rgb(200, 200, 180);
            let max_body_lines = 8;
            for line_text in pretty.lines().take(max_body_lines) {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(line_text.to_string(), Style::default().fg(body_color)),
                ]));
            }
            let total = pretty.lines().count();
            if total > max_body_lines {
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("  ... {} more lines", total - max_body_lines),
                        Style::default().fg(DIMMER),
                    ),
                ]));
            }
        }
    }

    // ── Assertions section ─────────────────────────────────────────────────
    if !node.assertions.is_empty() {
        lines.push(Line::raw(""));
        lines.push(section_header("Assertions", node.assertions.len(), inner_width));
        for a in &node.assertions {
            let (icon, icon_col) = if a.enabled { ("\u{2714}", GREEN) } else { ("\u{2718}", DIM) };
            let on_fail_label = match &a.on_fail {
                crate::api::types::OnFail::Stop => "stop",
                crate::api::types::OnFail::Warn => "warn",
            };
            let check_display = truncate(&a.check, inner_width.saturating_sub(22).max(10));
            lines.push(Line::from(vec![
                Span::styled(format!("  {} ", icon), Style::default().fg(icon_col)),
                Span::styled(check_display, if a.enabled {
                    Style::default().fg(TEXT)
                } else {
                    Style::default().fg(DIMMER)
                }),
                Span::styled(format!("  \u{2192} {}", on_fail_label), Style::default().fg(DIMMER)),
            ]));
        }
    }

    // ── Extractions section ────────────────────────────────────────────────
    if !node.extractions.is_empty() {
        lines.push(Line::raw(""));
        lines.push(section_header("Extractions", node.extractions.len(), inner_width));
        for e in &node.extractions {
            let max_from = inner_width.saturating_sub(22).max(10);
            lines.push(Line::from(vec![
                Span::styled(format!("  {:<16}", truncate(&e.name, 16)), Style::default().fg(CYAN)),
                Span::styled(" \u{2190} ", Style::default().fg(DIMMER)),
                Span::styled(truncate(&e.from, max_from), Style::default().fg(TEXT_DIM)),
            ]));
        }
    }

    // ── Prompt Inputs section ──────────────────────────────────────────────
    if !node.prompt_inputs.is_empty() {
        lines.push(Line::raw(""));
        lines.push(section_header("Prompt Inputs", node.prompt_inputs.len(), inner_width));
        for pi in &node.prompt_inputs {
            let secret_badge = if pi.secret {
                Span::styled(" secret", Style::default().fg(ORANGE))
            } else {
                Span::raw("")
            };
            let type_label = match pi.prompt_type {
                crate::api::types::PromptType::Text => "",
                crate::api::types::PromptType::Boolean => " bool",
                crate::api::types::PromptType::Select => " select",
                crate::api::types::PromptType::Multiselect => " multi",
            };
            let type_badge = if type_label.is_empty() {
                Span::raw("")
            } else {
                Span::styled(type_label, Style::default().fg(PURPLE))
            };
            lines.push(Line::from(vec![
                Span::styled(format!("  {:<14}", truncate(&pi.var, 14)), Style::default().fg(CYAN)),
                Span::styled(format!("\"{}\"", truncate(&pi.label, 16)), Style::default().fg(TEXT_DIM)),
                type_badge,
                secret_badge,
            ]));
        }
    }

    // ── Edit hints bar (always at bottom) ──────────────────────────────────
    lines.push(Line::raw(""));
    lines.push(section_header("Edit", 0, inner_width));
    lines.push(Line::from(vec![
        Span::styled("  n", Style::default().fg(YELLOW)),
        Span::styled(" name  ", Style::default().fg(DIM)),
        Span::styled("p", Style::default().fg(YELLOW)),
        Span::styled(" path  ", Style::default().fg(DIM)),
        Span::styled("m", Style::default().fg(YELLOW)),
        Span::styled(" method  ", Style::default().fg(DIM)),
        Span::styled("b", Style::default().fg(YELLOW)),
        Span::styled(" body", Style::default().fg(DIM)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  d", Style::default().fg(YELLOW)),
        Span::styled(" desc  ", Style::default().fg(DIM)),
        Span::styled("Enter", Style::default().fg(YELLOW)),
        Span::styled(" run node  ", Style::default().fg(DIM)),
        Span::styled("a", Style::default().fg(YELLOW)),
        Span::styled(" attach to flow", Style::default().fg(DIM)),
    ]));

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(" Node Inspector ", title_style()))
                .borders(Borders::ALL)
                .border_style(border_style())
        )
        .wrap(Wrap { trim: false });

    f.render_widget(p, area);
}

// ── Helpers ────────────────────────────────────────────────────────────────────

/// Build a section header line like `── Headers (3) ──────────` padded to panel width.
fn section_header(name: &str, count: usize, width: usize) -> Line<'static> {
    let label = if count > 0 {
        format!("\u{2500}\u{2500} {} ({}) ", name, count)
    } else {
        format!("\u{2500}\u{2500} {} ", name)
    };
    let label_len = label.chars().count();
    let dash_count = if width > label_len + 2 { width - label_len - 2 } else { 0 };
    let full = format!("  {}{}", label, "\u{2500}".repeat(dash_count));
    Line::from(vec![
        Span::styled(full, Style::default().fg(DIMMER)),
    ])
}
