use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::tui::api_app::ApiApp;
use crate::tui::theme::*;

use super::{blank_line, section_header, truncate};

// ── JSON syntax highlighter ───────────────────────────────────────────────────

/// Parse a single line of JSON and return spans with syntax coloring.
/// Keys in CYAN, string values in GREEN, numbers in YELLOW, booleans/null in ORANGE,
/// punctuation in DIM.
fn json_highlight_line(line: &str) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        // Whitespace
        if c == ' ' || c == '\t' {
            let start = i;
            while i < chars.len() && (chars[i] == ' ' || chars[i] == '\t') {
                i += 1;
            }
            spans.push(Span::styled(
                chars[start..i].iter().collect::<String>(),
                Style::default(),
            ));
            continue;
        }

        // Punctuation: { } [ ] : ,
        if c == '{' || c == '}' || c == '[' || c == ']' || c == ':' || c == ',' {
            spans.push(Span::styled(
                c.to_string(),
                Style::default().fg(DIM),
            ));
            i += 1;
            continue;
        }

        // String
        if c == '"' {
            let start = i;
            i += 1; // skip opening quote
            while i < chars.len() && chars[i] != '"' {
                if chars[i] == '\\' {
                    i += 1; // skip escaped char
                }
                i += 1;
            }
            if i < chars.len() {
                i += 1; // skip closing quote
            }
            let s: String = chars[start..i].iter().collect();
            // Determine if this is a key (followed by ':') or a value
            let mut lookahead = i;
            while lookahead < chars.len() && (chars[lookahead] == ' ' || chars[lookahead] == '\t') {
                lookahead += 1;
            }
            if lookahead < chars.len() && chars[lookahead] == ':' {
                // Key
                spans.push(Span::styled(s, Style::default().fg(CYAN)));
            } else {
                // String value
                spans.push(Span::styled(s, Style::default().fg(GREEN)));
            }
            continue;
        }

        // Number
        if c == '-' || c.is_ascii_digit() {
            let start = i;
            if c == '-' {
                i += 1;
            }
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.' || chars[i] == 'e' || chars[i] == 'E' || chars[i] == '+' || chars[i] == '-') {
                i += 1;
            }
            let s: String = chars[start..i].iter().collect();
            spans.push(Span::styled(s, Style::default().fg(YELLOW)));
            continue;
        }

        // Boolean / null keywords
        if c == 't' || c == 'f' || c == 'n' {
            let keyword_len = if c == 't' && i + 3 < chars.len() {
                let slice: String = chars[i..i + 4].iter().collect();
                if slice == "true" { Some(4) } else { None }
            } else if c == 'f' && i + 4 < chars.len() {
                let slice: String = chars[i..i + 5].iter().collect();
                if slice == "false" { Some(5) } else { None }
            } else if c == 'n' && i + 3 < chars.len() {
                let slice: String = chars[i..i + 4].iter().collect();
                if slice == "null" { Some(4) } else { None }
            } else {
                None
            };
            if let Some(len) = keyword_len {
                let s: String = chars[i..i + len].iter().collect();
                spans.push(Span::styled(s, Style::default().fg(ORANGE)));
                i += len;
                continue;
            }
        }

        // Fallback: single character
        spans.push(Span::styled(
            c.to_string(),
            Style::default().fg(TEXT_DIM),
        ));
        i += 1;
    }

    spans
}

// ── Prompt modal ──────────────────────────────────────────────────────────────

pub(super) fn render_prompt_modal(f: &mut Frame, app: &ApiApp, area: Rect) {
    use crate::api::types::PromptType;
    let modal = match &app.prompt_modal {
        Some(m) => m,
        None => return,
    };

    // ── Calculate dynamic height ───────────────────────────────────────────
    let field_heights: Vec<u16> = modal.inputs.iter().map(|pi| match pi.prompt_type {
        PromptType::Select | PromptType::Multiselect => (2 + pi.options.len() as u16).max(3),
        _ => 3,
    }).collect();
    let total_fields_h: u16 = field_heights.iter().sum();
    let h = (total_fields_h + 8).max(10).min(area.height.saturating_sub(4));
    let w = (area.width * 65 / 100).max(52).min(area.width.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let overlay_area = Rect { x, y, width: w, height: h };

    f.render_widget(Clear, overlay_area);

    let node_label = truncate(&modal.node_id, 32);
    let title = format!(" Input Required - {} ", node_label);

    let outer_block = Block::default()
        .title(Span::styled(title, title_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(CYAN))
        .style(Style::default().bg(BG));
    let inner = outer_block.inner(overlay_area);
    f.render_widget(outer_block, overlay_area);

    // ── Build layout: instruction + fields + footer ────────────────────────
    let mut constraints = vec![Constraint::Length(2)];
    for h in &field_heights {
        constraints.push(Constraint::Length(*h));
    }
    constraints.push(Constraint::Min(1));

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    // ── Instruction text ───────────────────────────────────────────────────
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" Provide values before the request fires:", Style::default().fg(TEXT_DIM)),
        ])),
        sections[0],
    );

    // ── Render each prompt field ───────────────────────────────────────────
    for (i, pi) in modal.inputs.iter().enumerate() {
        let field_area = sections[i + 1];
        let is_current = i == modal.current_field;
        let label = if pi.label.is_empty() { pi.var.as_str() } else { pi.label.as_str() };

        let border_sty = if is_current {
            Style::default().fg(CYAN)
        } else {
            Style::default().fg(BORDER)
        };
        let title_sty = if is_current {
            Style::default().fg(CYAN).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(DIM)
        };

        match pi.prompt_type {
            PromptType::Text => {
                let raw_val = modal.values.get(i).map(|s| s.as_str()).unwrap_or("");
                let display_val = if pi.secret {
                    "•".repeat(raw_val.len())
                } else if raw_val.is_empty() {
                    pi.default.as_deref()
                        .map(|d| format!("{} (default)", d))
                        .unwrap_or_default()
                } else {
                    raw_val.to_string()
                };
                let cursor = if is_current { "|" } else { "" };
                let val_style = if is_current {
                    Style::default().fg(YELLOW)
                } else if raw_val.is_empty() {
                    Style::default().fg(DIMMER)
                } else {
                    Style::default().fg(WHITE)
                };

                let fb = Block::default()
                    .title(Span::styled(format!(" {} ", label), title_sty))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(border_sty);
                let fi = fb.inner(field_area);
                f.render_widget(fb, field_area);
                f.render_widget(
                    Paragraph::new(Line::from(vec![
                        Span::raw(" "),
                        Span::styled(format!("{}{}", display_val, cursor), val_style),
                    ])),
                    fi,
                );
            }
            PromptType::Boolean => {
                let is_true = modal.values.get(i).map(|v| v == "true").unwrap_or(false);
                let fb = Block::default()
                    .title(Span::styled(format!(" {} ", label), title_sty))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(border_sty);
                let fi = fb.inner(field_area);
                f.render_widget(fb, field_area);

                let (yes_style, no_style) = if is_true {
                    (
                        Style::default().fg(GREEN).add_modifier(Modifier::BOLD | Modifier::REVERSED),
                        Style::default().fg(DIM),
                    )
                } else {
                    (
                        Style::default().fg(DIM),
                        Style::default().fg(RED).add_modifier(Modifier::BOLD | Modifier::REVERSED),
                    )
                };
                f.render_widget(
                    Paragraph::new(Line::from(vec![
                        Span::raw(" "),
                        Span::styled("  Yes  ", yes_style),
                        Span::raw("  "),
                        Span::styled("  No  ", no_style),
                        Span::styled("   y/n or Space to toggle", Style::default().fg(DIMMER)),
                    ])),
                    fi,
                );
            }
            PromptType::Select => {
                let cursor_idx = modal.option_cursors.get(i).copied().unwrap_or(0);
                let fb = Block::default()
                    .title(Span::styled(format!(" {} ", label), title_sty))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(border_sty);
                let fi = fb.inner(field_area);
                f.render_widget(fb, field_area);

                let items: Vec<ListItem> = pi.options.iter().enumerate().map(|(j, opt)| {
                    let is_sel = j == cursor_idx;
                    let style = if is_sel && is_current {
                        Style::default().fg(CYAN).add_modifier(Modifier::BOLD)
                    } else if is_sel {
                        Style::default().fg(WHITE)
                    } else {
                        Style::default().fg(DIM)
                    };
                    let prefix = if is_sel { "> " } else { "  " };
                    ListItem::new(Line::from(vec![
                        Span::styled(format!(" {}{}", prefix, opt), style),
                    ]))
                }).collect();
                let mut list_state = ListState::default();
                list_state.select(Some(cursor_idx));
                f.render_stateful_widget(
                    List::new(items).highlight_style(Style::default().bg(BG_SELECTED).fg(CYAN)),
                    fi,
                    &mut list_state,
                );
            }
            PromptType::Multiselect => {
                let cursor_idx = modal.option_cursors.get(i).copied().unwrap_or(0);
                let checked = modal.multi_checked.get(i).map(|v| v.as_slice()).unwrap_or(&[]);
                let fb = Block::default()
                    .title(Span::styled(format!(" {} ", label), title_sty))
                    .title_bottom(Span::styled(
                        " Space: toggle  Enter: confirm ",
                        Style::default().fg(DIMMER),
                    ))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(border_sty);
                let fi = fb.inner(field_area);
                f.render_widget(fb, field_area);

                let items: Vec<ListItem> = pi.options.iter().enumerate().map(|(j, opt)| {
                    let is_cur = j == cursor_idx && is_current;
                    let is_checked = checked.get(j).copied().unwrap_or(false);
                    let checkbox = if is_checked { "[+]" } else { "[ ]" };
                    let cb_style = if is_checked {
                        Style::default().fg(GREEN)
                    } else {
                        Style::default().fg(DIM)
                    };
                    let label_style = if is_cur {
                        Style::default().fg(CYAN).add_modifier(Modifier::BOLD)
                    } else if is_checked {
                        Style::default().fg(WHITE)
                    } else {
                        Style::default().fg(DIM)
                    };
                    ListItem::new(Line::from(vec![
                        Span::raw(" "),
                        Span::styled(checkbox, cb_style),
                        Span::raw(" "),
                        Span::styled(opt.as_str(), label_style),
                    ]))
                }).collect();
                let mut list_state = ListState::default();
                list_state.select(Some(cursor_idx));
                f.render_stateful_widget(
                    List::new(items).highlight_style(Style::default().bg(BG_SELECTED)),
                    fi,
                    &mut list_state,
                );
            }
        }
    }

    // ── Footer with navigation hints ───────────────────────────────────────
    let footer_area = sections[modal.inputs.len() + 1];
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" [", Style::default().fg(YELLOW)),
            Span::styled("Tab", Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)),
            Span::styled("] next field  ", Style::default().fg(DIM)),
            Span::styled("[", Style::default().fg(YELLOW)),
            Span::styled("Up/Dn", Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)),
            Span::styled("] navigate  ", Style::default().fg(DIM)),
            Span::styled("[", Style::default().fg(YELLOW)),
            Span::styled("Enter", Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)),
            Span::styled("] confirm  ", Style::default().fg(DIM)),
            Span::styled("[", Style::default().fg(YELLOW)),
            Span::styled("Esc", Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)),
            Span::styled("] cancel", Style::default().fg(DIM)),
        ])),
        footer_area,
    );
}

// ── Body editor modal ─────────────────────────────────────────────────────────

pub(super) fn render_body_editor(f: &mut Frame, app: &ApiApp, area: Rect) {
    let editor = match &app.body_editor {
        Some(e) => e,
        None => return,
    };

    // Full screen minus 2px margin
    let w = area.width.saturating_sub(4).max(40);
    let h = area.height.saturating_sub(4).max(10);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let overlay = Rect { x, y, width: w, height: h };

    f.render_widget(Clear, overlay);

    let inner_h = h.saturating_sub(4) as usize;
    let visible_lines = inner_h.max(1);

    let scroll_top = editor.scroll_top;
    let end = (scroll_top + visible_lines).min(editor.lines.len());

    let mut content_lines: Vec<Line> = Vec::new();

    for (abs_i, line) in editor.lines[scroll_top..end].iter().enumerate() {
        let line_idx = scroll_top + abs_i;
        let is_cursor_line = line_idx == editor.cursor_row;

        // 3-digit line number, DIM
        let line_no = Span::styled(
            format!("{:>3} ", line_idx + 1),
            Style::default().fg(DIMMER),
        );

        if is_cursor_line {
            let col = editor.cursor_col.min(line.len());
            let before = &line[..col];
            let cursor_char = if col < line.len() {
                line.chars().nth(col).unwrap_or(' ')
            } else {
                ' '
            };
            let after = if col < line.len() {
                &line[col + cursor_char.len_utf8()..]
            } else {
                ""
            };

            content_lines.push(Line::from(vec![
                line_no,
                Span::styled(before, Style::default().fg(TEXT)),
                Span::styled(
                    cursor_char.to_string(),
                    Style::default().bg(CYAN).fg(BG),
                ),
                Span::styled(after, Style::default().fg(TEXT)),
            ]));
        } else {
            // Apply JSON syntax highlighting for non-cursor lines
            let mut line_spans = vec![line_no];
            line_spans.extend(json_highlight_line(line));
            content_lines.push(Line::from(line_spans));
        }
    }

    // Pad remaining visible area
    while content_lines.len() < visible_lines {
        content_lines.push(Line::raw(""));
    }

    // ── Footer with keyboard hints ─────────────────────────────────────────
    content_lines.push(Line::from(vec![
        Span::styled(" [", Style::default().fg(YELLOW)),
        Span::styled("Ctrl+S", Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)),
        Span::styled("] save  ", Style::default().fg(DIM)),
        Span::styled("[", Style::default().fg(YELLOW)),
        Span::styled("Esc", Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)),
        Span::styled("] cancel  ", Style::default().fg(DIM)),
        Span::styled("[", Style::default().fg(YELLOW)),
        Span::styled("Arrows", Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)),
        Span::styled("] move  ", Style::default().fg(DIM)),
        Span::styled("[", Style::default().fg(YELLOW)),
        Span::styled("Enter", Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)),
        Span::styled("] newline", Style::default().fg(DIM)),
    ]));

    let line_count = editor.lines.len();
    let title = format!(" Edit Body - {} ({} lines) ", editor.node_id, line_count);

    let p = Paragraph::new(content_lines)
        .block(
            Block::default()
                .title(Span::styled(title, title_style()))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(CYAN))
                .style(Style::default().bg(BG)),
        );

    f.render_widget(p, overlay);
}

// ── Step detail modal ─────────────────────────────────────────────────────────

pub(super) fn render_step_detail_modal(f: &mut Frame, app: &ApiApp, area: Rect) {
    let modal = match &app.step_detail {
        Some(m) => m,
        None => return,
    };
    let step = &modal.step;

    // Almost full screen
    let w = area.width.saturating_sub(4).max(60);
    let h = area.height.saturating_sub(2).max(20);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let overlay = Rect { x, y, width: w, height: h };
    f.render_widget(Clear, overlay);

    let mut lines: Vec<Line> = vec![];

    // ── Header: status code + method + time ────────────────────────────────
    let status_str = step.status_code
        .map(|s| s.to_string())
        .unwrap_or_else(|| "ERR".to_string());
    let sc = status_code_color(step.status_code);
    let mc = method_color(&step.method);

    lines.push(blank_line());
    lines.push(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(&status_str, Style::default().fg(sc).add_modifier(Modifier::BOLD)),
        Span::styled("  ", Style::default()),
        Span::styled("[", Style::default().fg(DIMMER)),
        Span::styled(&step.method, Style::default().fg(mc).add_modifier(Modifier::BOLD)),
        Span::styled("]", Style::default().fg(DIMMER)),
        Span::styled("  ", Style::default()),
        Span::styled(format!("{}ms", step.duration_ms), Style::default().fg(TEXT_DIM)),
    ]));

    // ── URL ────────────────────────────────────────────────────────────────
    lines.push(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(truncate(&step.url, (w as usize).saturating_sub(4)), Style::default().fg(CYAN)),
    ]));
    lines.push(blank_line());

    // ── Error section (if any) ─────────────────────────────────────────────
    if let Some(err) = &step.error {
        if !err.is_empty() {
            lines.push(section_header("Error", w as usize));
            let max_w = (w as usize).saturating_sub(6);
            for chunk in err.chars().collect::<Vec<_>>().chunks(max_w) {
                let s: String = chunk.iter().collect();
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(s, Style::default().fg(RED)),
                ]));
            }
            lines.push(blank_line());
        }
    }

    // ── Assertions section ─────────────────────────────────────────────────
    if !step.assertion_results.is_empty() {
        lines.push(section_header("Assertions", w as usize));
        for ar in &step.assertion_results {
            let (icon, col) = if ar.passed { ("+", GREEN) } else { ("x", RED) };
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("[{}]", icon), Style::default().fg(col)),
                Span::raw(" "),
                Span::styled(&ar.check, Style::default().fg(col)),
                Span::styled(" -> ", Style::default().fg(DIMMER)),
                Span::styled(&ar.actual, Style::default().fg(TEXT_DIM)),
            ]));
        }
        lines.push(blank_line());
    }

    // ── Extracted variables ────────────────────────────────────────────────
    if !step.extracted.is_empty() {
        lines.push(section_header("Extracted Variables", w as usize));
        let key_w = ((w as usize) / 3).max(8).min(24);
        for (k, v) in &step.extracted {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("{:<w$}", truncate(k, key_w), w = key_w), Style::default().fg(CYAN)),
                Span::styled("= ", Style::default().fg(DIMMER)),
                Span::styled(v.to_string(), Style::default().fg(WHITE)),
            ]));
        }
        lines.push(blank_line());
    }

    // ── Request body (pretty JSON, responsive line limit) ────────────────
    let req_max_lines = ((h as usize) / 4).max(6).min(20);
    if let Some(req_body) = &step.request_body {
        if !req_body.is_empty() {
            lines.push(section_header("Request Body", w as usize));
            let pretty = serde_json::from_str::<serde_json::Value>(req_body)
                .map(|v| serde_json::to_string_pretty(&v).unwrap_or_else(|_| req_body.clone()))
                .unwrap_or_else(|_| req_body.clone());
            for owned_line in pretty.lines().take(req_max_lines).map(|l| l.to_string()).collect::<Vec<_>>() {
                let mut spans = vec![Span::raw("  ")];
                spans.extend(json_highlight_line(&owned_line));
                lines.push(Line::from(spans));
            }
            let total = pretty.lines().count();
            if total > req_max_lines {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        format!("... +{} more lines", total - req_max_lines),
                        Style::default().fg(DIMMER),
                    ),
                ]));
            }
            lines.push(blank_line());
        }
    }

    // ── Response body (pretty JSON, responsive line limit) ───────────────
    let resp_max_lines = ((h as usize) / 2).max(8).min(40);
    if let Some(resp_body) = &step.response_body {
        if !resp_body.is_empty() {
            lines.push(section_header("Response Body", w as usize));
            let pretty = serde_json::from_str::<serde_json::Value>(resp_body)
                .map(|v| serde_json::to_string_pretty(&v).unwrap_or_else(|_| resp_body.clone()))
                .unwrap_or_else(|_| resp_body.clone());
            for owned_line in pretty.lines().take(resp_max_lines).map(|l| l.to_string()).collect::<Vec<_>>() {
                let mut spans = vec![Span::raw("  ")];
                spans.extend(json_highlight_line(&owned_line));
                lines.push(Line::from(spans));
            }
            let total = pretty.lines().count();
            if total > resp_max_lines {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        format!("... +{} more lines", total - resp_max_lines),
                        Style::default().fg(DIMMER),
                    ),
                ]));
            }
        }
    }

    let scroll = modal.scroll.min(lines.len().saturating_sub(1));

    let title = format!(" Step Details - {} ", step.node_id);
    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(title, title_style()))
                .title_bottom(Span::styled(
                    " [Up/Dn] scroll  [Esc] close ",
                    Style::default().fg(DIMMER),
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(CYAN))
                .style(Style::default().bg(BG)),
        )
        .wrap(Wrap { trim: false })
        .scroll((scroll as u16, 0));

    f.render_widget(p, overlay);
}
