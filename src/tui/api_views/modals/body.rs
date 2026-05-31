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
    let overlay = Rect {
        x,
        y,
        width: w,
        height: h,
    };

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
        let line_no = Span::styled(format!("{:>3} ", line_idx + 1), Style::default().fg(DIMMER));

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
                Span::styled(cursor_char.to_string(), Style::default().bg(CYAN).fg(BG)),
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
        Span::styled(
            "Ctrl+S",
            Style::default().fg(YELLOW).add_modifier(Modifier::BOLD),
        ),
        Span::styled("] save  ", Style::default().fg(DIM)),
        Span::styled("[", Style::default().fg(YELLOW)),
        Span::styled(
            "Esc",
            Style::default().fg(YELLOW).add_modifier(Modifier::BOLD),
        ),
        Span::styled("] cancel  ", Style::default().fg(DIM)),
        Span::styled("[", Style::default().fg(YELLOW)),
        Span::styled(
            "Arrows",
            Style::default().fg(YELLOW).add_modifier(Modifier::BOLD),
        ),
        Span::styled("] move  ", Style::default().fg(DIM)),
        Span::styled("[", Style::default().fg(YELLOW)),
        Span::styled(
            "Enter",
            Style::default().fg(YELLOW).add_modifier(Modifier::BOLD),
        ),
        Span::styled("] newline", Style::default().fg(DIM)),
    ]));

    let line_count = editor.lines.len();
    let title = format!(" Edit Body - {} ({} lines) ", editor.node_id, line_count);

    let p = Paragraph::new(content_lines).block(
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
