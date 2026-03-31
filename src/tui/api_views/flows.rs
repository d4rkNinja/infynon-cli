use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

use crate::tui::api_app::ApiApp;
use crate::tui::theme::*;

use super::{truncate, dashboard::render_no_flows_hint};

// ── Layout constants (tuned for perfect spacing) ─────────────────────────────

const NODE_H: u16 = 5;   // 3 content lines + 2 border lines
const H_GAP: u16 = 4;    // horizontal gap between columns
const V_GAP: u16 = 3;    // vertical gap between layers

// ── Flows view (flow graph) ──────────────────────────────────────────────────

pub(super) fn render_flows_view(f: &mut Frame, app: &ApiApp, area: Rect) {
    let flow = match app.active_flow() {
        Some(fl) => fl,
        None => {
            render_no_flows_hint(f, area, "Flow Graph");
            return;
        }
    };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70),
            Constraint::Percentage(30),
        ])
        .split(area);

    render_graph_canvas(f, app, chunks[0]);
    render_graph_sidebar(f, app, flow, chunks[1]);
}

// ── Helper: render single character at position ──────────────────────────────

fn draw_char(f: &mut Frame, x: u16, y: u16, ch: &str, style: Style) {
    let r = Rect { x, y, width: 1, height: 1 };
    f.render_widget(Paragraph::new(ch).style(style), r);
}

// ── Graph canvas (left panel) ────────────────────────────────────────────────

fn render_graph_canvas(f: &mut Frame, app: &ApiApp, area: Rect) {
    let block = Block::default()
        .title(Span::styled(" Flow Graph ", title_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style());
    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.graph_layout.is_empty() {
        let lines = vec![
            Line::raw(""),
            Line::from(vec![
                Span::styled("  No nodes in this flow.", Style::default().fg(DIM)),
            ]),
            Line::raw(""),
            Line::from(vec![
                Span::styled("  Add nodes and connect them:", Style::default().fg(DIMMER)),
            ]),
            Line::from(vec![
                Span::styled("    infynon weave attach <node> --to <flow>", Style::default().fg(CYAN)),
            ]),
        ];
        f.render_widget(Paragraph::new(lines), inner);
        return;
    }

    let flow = app.active_flow();

    // ── Calculate dynamic node width ──────────────────────────────────────
    let max_col = app.graph_layout.iter().map(|g| g.col).max().unwrap_or(0);
    let num_cols = (max_col + 1) as u16;
    let total_gap_w = num_cols.saturating_sub(1) * H_GAP;
    let node_w = if num_cols == 0 {
        28
    } else {
        ((inner.width.saturating_sub(total_gap_w)) / num_cols).clamp(22, 38)
    };

    // ── Centering offsets ─────────────────────────────────────────────────
    let max_layer = app.graph_layout.iter().map(|g| g.layer).max().unwrap_or(0);
    let num_layers = (max_layer + 1) as u16;
    let total_graph_w = num_cols * node_w + num_cols.saturating_sub(1) * H_GAP;
    let total_graph_h = num_layers * NODE_H + num_layers.saturating_sub(1) * V_GAP;
    let x_offset = inner.width.saturating_sub(total_graph_w) / 2;
    let y_offset = inner.height.saturating_sub(total_graph_h) / 2;

    // Helper closure to get node rect
    let get_node_rect = |col: usize, layer: usize| -> Rect {
        let x = inner.x + x_offset + col as u16 * (node_w + H_GAP);
        let y = inner.y + y_offset + layer as u16 * (NODE_H + V_GAP);
        Rect { x, y, width: node_w, height: NODE_H }
    };

    // ── Pass 1: draw connections (behind nodes) ──────────────────────────
    if let Some(flow) = flow {
        for gnode in &app.graph_layout {
            let src_rect = get_node_rect(gnode.col, gnode.layer);
            for edge in flow.successors(&gnode.node_id) {
                if let Some(target) = app.graph_layout.iter().find(|g| g.node_id == edge.to) {
                    let tgt_rect = get_node_rect(target.col, target.layer);
                    draw_connection(f, inner, src_rect, tgt_rect, node_w, &edge.carry);
                }
            }
        }
    }

    // ── Pass 2: draw node cards (on top of connections) ──────────────────
    for gnode in &app.graph_layout {
        let rect = get_node_rect(gnode.col, gnode.layer);
        if rect.x + rect.width > inner.right() || rect.y + rect.height > inner.bottom() {
            continue;
        }

        let node = app.nodes.get(&gnode.node_id);
        let is_selected = app.graph_selected_id.as_deref() == Some(&gnode.node_id);

        let step_result = app.last_run.as_ref().and_then(|run| {
            run.steps.iter().find(|s| s.node_id == gnode.node_id)
        });

        // ── Selection styling (MAJOR visual difference) ─────────────────────
        let (border_color, border_mod, bg_color) = if is_selected {
            (ORANGE, Modifier::BOLD, BG_NODE_SELECTED)
        } else {
            (BORDER, Modifier::empty(), BG_SURFACE)
        };

        // Status icon with color
        let (status_icon, status_color) = match step_result {
            Some(s) if s.passed => ("\u{2714}", GREEN),
            Some(_) => ("\u{2718}", RED),
            None => ("\u{25CB}", DIM),
        };

        // Line 1: selection marker + status + name
        let display_name = node.map(|n| n.name.as_str()).unwrap_or(&gnode.node_id);
        let name_max = (node_w as usize).saturating_sub(5).max(4);
        let truncated_name = truncate(display_name, name_max);

        let sel_marker = if is_selected { "\u{25B6} " } else { "  " };
        let name_color = if is_selected { ORANGE } else { TEXT };

        let line1 = Line::from(vec![
            Span::styled(sel_marker, Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{} ", status_icon), Style::default().fg(status_color)),
            Span::styled(truncated_name, Style::default().fg(name_color).add_modifier(Modifier::BOLD)),
        ]);

        // Line 2: method + path
        let method_str = node.map(|n| n.method.as_str()).unwrap_or("?");
        let mc = method_color(method_str);
        let path_max = (node_w as usize).saturating_sub(method_str.len() + 5).max(4);
        let path_str = node.map(|n| truncate(&n.path, path_max)).unwrap_or_default();
        let path_color = if is_selected { TEXT_DIM } else { DIM };

        let line2 = Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(format!("{:>6} ", method_str), Style::default().fg(mc).add_modifier(Modifier::BOLD)),
            Span::styled(path_str, Style::default().fg(path_color)),
        ]);

        // Line 3: latency + status code (or description)
        let line3 = if let Some(step) = step_result {
            let latency = format!("{}ms", step.duration_ms);
            let sc = step.status_code.unwrap_or(0);
            let sc_color = status_code_color(step.status_code);
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(latency, Style::default().fg(TEXT_DIM)),
                Span::styled(" \u{00B7} ", Style::default().fg(DIMMER)),
                Span::styled(format!("{}", sc), Style::default().fg(sc_color).add_modifier(Modifier::BOLD)),
            ])
        } else if let Some(n) = node {
            if let Some(desc) = &n.description {
                let desc_max = (node_w as usize).saturating_sub(3).max(4);
                Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(truncate(desc, desc_max), Style::default().fg(DIMMER)),
                ])
            } else {
                Line::from(Span::styled("", Style::default()))
            }
        } else {
            Line::from(Span::styled("", Style::default()))
        };

        // Render node with background
        let node_block = Paragraph::new(vec![line1, line2, line3])
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color).add_modifier(border_mod))
                    .style(Style::default().bg(bg_color))
            );
        f.render_widget(node_block, rect);
    }
}

// ── Connection drawing with L-shaped paths ───────────────────────────────────

fn draw_connection(
    f: &mut Frame,
    inner: Rect,
    src: Rect,
    tgt: Rect,
    node_w: u16,
    carry: &[String],
) {
    let src_cx = src.x + node_w / 2;
    let src_by = src.y + src.height;
    let tgt_cx = tgt.x + node_w / 2;
    let tgt_ty = tgt.y;

    if src_by >= inner.bottom() || tgt_ty <= src_by || tgt_ty > inner.bottom() {
        return;
    }

    let conn_style = Style::default().fg(CYAN);
    let gap_rows = tgt_ty.saturating_sub(src_by);

    if gap_rows == 0 {
        return;
    }

    if src_cx == tgt_cx {
        // Same column: vertical line + arrowhead
        for y in src_by..tgt_ty {
            if y >= inner.bottom() { break; }
            let sym = if y == tgt_ty - 1 { "\u{25BC}" } else { "\u{2502}" };
            draw_char(f, src_cx, y, sym, conn_style);
        }

        // Carry label beside vertical line
        if !carry.is_empty() && src_by + 1 < inner.bottom() {
            let carry_str = carry.join(",");
            let label_max = (inner.right().saturating_sub(src_cx + 2) as usize).min(12).max(4);
            let carry_label = truncate(&carry_str, label_max);
            let label_x = (src_cx + 2).min(inner.right().saturating_sub(carry_label.len() as u16 + 2));
            let r = Rect { x: label_x, y: src_by + 1, width: carry_label.len() as u16 + 1, height: 1 };
            f.render_widget(
                Paragraph::new(format!("\u{250A}{}", carry_label)).style(Style::default().fg(PURPLE)),
                r,
            );
        }
    } else {
        // Different columns: L-shaped path
        let mid_y = src_by + gap_rows / 2;

        // Vertical from source bottom to mid_y
        for y in src_by..mid_y {
            if y >= inner.bottom() { break; }
            draw_char(f, src_cx, y, "\u{2502}", conn_style);
        }

        // Corner + horizontal + corner
        if mid_y < inner.bottom() {
            let (left_x, right_x) = if src_cx < tgt_cx { (src_cx, tgt_cx) } else { (tgt_cx, src_cx) };

            let src_corner = if src_cx < tgt_cx { "\u{2514}" } else { "\u{2518}" };
            let tgt_corner = if tgt_cx > src_cx { "\u{2510}" } else { "\u{250C}" };

            draw_char(f, src_cx, mid_y, src_corner, conn_style);

            for x in (left_x + 1)..right_x {
                if x >= inner.right() { break; }
                draw_char(f, x, mid_y, "\u{2500}", conn_style);
            }

            // Carry label along horizontal segment
            if !carry.is_empty() && (right_x - left_x) > 4 {
                let carry_str = carry.join(",");
                let avail = (right_x - left_x - 2) as usize;
                let carry_label = truncate(&carry_str, avail.min(14).max(4));
                let label_start = left_x + 1;
                let label_w = carry_label.len() as u16;
                if label_start + label_w < right_x {
                    let r = Rect { x: label_start, y: mid_y, width: label_w, height: 1 };
                    f.render_widget(
                        Paragraph::new(carry_label).style(Style::default().fg(PURPLE)),
                        r,
                    );
                }
            }

            draw_char(f, tgt_cx, mid_y, tgt_corner, conn_style);
        }

        // Vertical from mid_y to target top
        for y in (mid_y + 1)..tgt_ty {
            if y >= inner.bottom() { break; }
            draw_char(f, tgt_cx, y, "\u{2502}", conn_style);
        }

        // Arrowhead at target top
        if tgt_ty > 0 && tgt_ty <= inner.bottom() {
            draw_char(f, tgt_cx, tgt_ty.saturating_sub(1), "\u{25BC}", conn_style);
        }
    }
}

// ── Sidebar (right panel) ────────────────────────────────────────────────────

fn render_graph_sidebar(f: &mut Frame, app: &ApiApp, flow: &crate::api::types::Flow, area: Rect) {
    let selected_id = app.graph_selected_id.as_ref();
    let selected_node = selected_id.and_then(|id| app.nodes.get(id));
    let inner_w = area.width.saturating_sub(2) as usize;

    let mut lines: Vec<Line> = Vec::new();

    if let (Some(id), Some(node)) = (selected_id, selected_node) {
        // ── Section 1: Node identity ─────────────────────────────────────
        lines.push(Line::raw(""));
        lines.push(Line::from(vec![
            Span::styled(truncate(&node.name, inner_w.saturating_sub(2)), Style::default().fg(CYAN).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(vec![
            Span::styled(truncate(&node.id, inner_w.saturating_sub(2)), Style::default().fg(DIM)),
        ]));
        lines.push(Line::raw(""));

        // ── Section 2: Method badge + path ───────────────────────────────
        let mc = method_color(&node.method);
        let path_max = inner_w.saturating_sub(node.method.len() + 4).max(4);
        lines.push(Line::from(vec![
            Span::styled(format!(" {} ", node.method), Style::default().fg(BG).bg(mc).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" {}", truncate(&node.path, path_max)), Style::default().fg(TEXT)),
        ]));
        lines.push(Line::raw(""));

        // ── Section 3: Run result ────────────────────────────────────────
        let step_result = app.last_run.as_ref().and_then(|run| {
            run.steps.iter().find(|s| s.node_id == *id)
        });

        lines.push(Line::from(vec![
            Span::styled(" \u{2500}\u{2500} Last Run", Style::default().fg(DIMMER)),
        ]));

        if let Some(step) = step_result {
            let (icon, color) = if step.passed { ("\u{2714} PASS", GREEN) } else { ("\u{2718} FAIL", RED) };
            let sc = step.status_code.unwrap_or(0);
            let sc_color = status_code_color(step.status_code);
            lines.push(Line::from(vec![
                Span::styled(format!("  {} ", icon), Style::default().fg(color).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{}", sc), Style::default().fg(sc_color)),
            ]));
            lines.push(Line::from(vec![
                Span::styled(format!("  {}ms", step.duration_ms), Style::default().fg(TEXT_DIM)),
            ]));
            if let Some(err) = &step.error {
                let err_max = inner_w.saturating_sub(8).max(10);
                lines.push(Line::from(vec![
                    Span::styled(format!("  err: {}", truncate(err, err_max)), Style::default().fg(RED)),
                ]));
            }
        } else {
            lines.push(Line::from(vec![
                Span::styled("  not run yet", Style::default().fg(DIMMER)),
            ]));
        }
        lines.push(Line::raw(""));

        // ── Section 4: Extractions ───────────────────────────────────────
        if !node.extractions.is_empty() {
            lines.push(Line::from(vec![
                Span::styled(format!(" \u{2500}\u{2500} Extractions ({})", node.extractions.len()), Style::default().fg(DIMMER)),
            ]));
            for ext in &node.extractions {
                let name_max = (inner_w / 3).max(6).min(14);
                let from_max = inner_w.saturating_sub(name_max + 5).max(6);
                lines.push(Line::from(vec![
                    Span::styled(format!("  {} ", truncate(&ext.name, name_max)), Style::default().fg(TEAL)),
                    Span::styled(format!("\u{2190} {}", truncate(&ext.from, from_max)), Style::default().fg(TEXT_DIM)),
                ]));
            }
            lines.push(Line::raw(""));
        }

        // ── Section 5: Successor nodes ───────────────────────────────────
        let successors = flow.successors(id);
        if !successors.is_empty() {
            lines.push(Line::from(vec![
                Span::styled(" \u{2500}\u{2500} Connected To", Style::default().fg(DIMMER)),
            ]));
            for edge in &successors {
                let tgt_name = app.nodes.get(&edge.to)
                    .map(|n| truncate(&n.name, inner_w.saturating_sub(10).max(6)))
                    .unwrap_or_else(|| truncate(&edge.to, inner_w.saturating_sub(10).max(6)));
                let carry_hint = if edge.carry.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", truncate(&edge.carry.join(","), 8))
                };
                let carry_max = inner_w.saturating_sub(tgt_name.len() + 6).max(4);
                lines.push(Line::from(vec![
                    Span::styled("  \u{2192} ", Style::default().fg(CYAN)),
                    Span::styled(tgt_name, Style::default().fg(TEXT)),
                    Span::styled(truncate(&carry_hint, carry_max), Style::default().fg(DIMMER)),
                ]));
            }
            lines.push(Line::raw(""));
        }

        // ── Section 6: Assertions summary ────────────────────────────────
        if !node.assertions.is_empty() {
            lines.push(Line::from(vec![
                Span::styled(format!(" \u{2500}\u{2500} Assertions ({})", node.assertions.len()), Style::default().fg(DIMMER)),
            ]));
            for assertion in &node.assertions {
                let check_max = inner_w.saturating_sub(6).max(8);
                let passed = step_result.as_ref().and_then(|s| {
                    s.assertion_results.iter().find(|ar| ar.check == assertion.check).map(|ar| ar.passed)
                });
                let (marker, mc) = match passed {
                    Some(true) => ("\u{2714}", GREEN),
                    Some(false) => ("\u{2718}", RED),
                    None => ("\u{00B7}", DIM),
                };
                lines.push(Line::from(vec![
                    Span::styled(format!(" {} ", marker), Style::default().fg(mc)),
                    Span::styled(truncate(&assertion.check, check_max), Style::default().fg(TEXT_DIM)),
                ]));
            }
            lines.push(Line::raw(""));
        }
    } else {
        // ── No node selected ─────────────────────────────────────────────
        lines.push(Line::raw(""));
        lines.push(Line::from(vec![
            Span::styled(" Select a Node", Style::default().fg(TEXT).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::raw(""));
        lines.push(Line::from(vec![
            Span::styled(" Navigate the flow graph", Style::default().fg(TEXT_DIM)),
        ]));
        lines.push(Line::from(vec![
            Span::styled(" using arrow keys.", Style::default().fg(TEXT_DIM)),
        ]));
        lines.push(Line::raw(""));
        lines.push(Line::from(vec![
            Span::styled(" Press Enter to inspect", Style::default().fg(TEXT_DIM)),
        ]));
        lines.push(Line::from(vec![
            Span::styled(" a node in detail.", Style::default().fg(TEXT_DIM)),
        ]));
        lines.push(Line::raw(""));
    }

    // ── Controls section (always at bottom) ────────────────────────────────
    lines.push(Line::from(vec![
        Span::styled(" \u{2500}\u{2500} Controls ", Style::default().fg(DIMMER)),
    ]));
    lines.push(Line::from(vec![
        Span::styled(" \u{2191}\u{2193}\u{2190}\u{2192}", Style::default().fg(CYAN)),
        Span::styled("  navigate", Style::default().fg(TEXT_DIM)),
    ]));
    lines.push(Line::from(vec![
        Span::styled(" Enter", Style::default().fg(CYAN)),
        Span::styled("  inspect", Style::default().fg(TEXT_DIM)),
    ]));
    lines.push(Line::from(vec![
        Span::styled(" a", Style::default().fg(CYAN)),
        Span::styled("  attach node", Style::default().fg(TEXT_DIM)),
    ]));
    lines.push(Line::from(vec![
        Span::styled(" d", Style::default().fg(CYAN)),
        Span::styled("  detach edge", Style::default().fg(TEXT_DIM)),
    ]));
    lines.push(Line::from(vec![
        Span::styled(" x", Style::default().fg(CYAN)),
        Span::styled("  chaos inject", Style::default().fg(TEXT_DIM)),
    ]));
    lines.push(Line::from(vec![
        Span::styled(" R", Style::default().fg(CYAN)),
        Span::styled("  refresh", Style::default().fg(TEXT_DIM)),
    ]));

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(" Info ", title_style()))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(border_style()),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(p, area);
}
