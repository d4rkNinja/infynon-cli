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
    let overlay = Rect {
        x,
        y,
        width: w,
        height: h,
    };
    f.render_widget(Clear, overlay);

    let mut lines: Vec<Line> = vec![];

    // ── Header: status code + method + time ────────────────────────────────
    let status_str = step
        .status_code
        .map(|s| s.to_string())
        .unwrap_or_else(|| "ERR".to_string());
    let sc = status_code_color(step.status_code);
    let mc = method_color(&step.method);

    lines.push(blank_line());
    lines.push(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(
            &status_str,
            Style::default().fg(sc).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ", Style::default()),
        Span::styled("[", Style::default().fg(DIMMER)),
        Span::styled(
            &step.method,
            Style::default().fg(mc).add_modifier(Modifier::BOLD),
        ),
        Span::styled("]", Style::default().fg(DIMMER)),
        Span::styled("  ", Style::default()),
        Span::styled(
            format!("{}ms", step.duration_ms),
            Style::default().fg(TEXT_DIM),
        ),
    ]));

    // ── URL ────────────────────────────────────────────────────────────────
    lines.push(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(
            truncate(&step.url, (w as usize).saturating_sub(4)),
            Style::default().fg(CYAN),
        ),
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
        let key_w = ((w as usize) / 3).clamp(8, 24);
        for (k, v) in &step.extracted {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    format!("{:<w$}", truncate(k, key_w), w = key_w),
                    Style::default().fg(CYAN),
                ),
                Span::styled("= ", Style::default().fg(DIMMER)),
                Span::styled(v.to_string(), Style::default().fg(WHITE)),
            ]));
        }
        lines.push(blank_line());
    }

    // ── Request body (pretty JSON, responsive line limit) ────────────────
    let req_max_lines = ((h as usize) / 4).clamp(6, 20);
    if let Some(req_body) = &step.request_body {
        if !req_body.is_empty() {
            lines.push(section_header("Request Body", w as usize));
            let pretty = serde_json::from_str::<serde_json::Value>(req_body)
                .map(|v| serde_json::to_string_pretty(&v).unwrap_or_else(|_| req_body.clone()))
                .unwrap_or_else(|_| req_body.clone());
            for owned_line in pretty
                .lines()
                .take(req_max_lines)
                .map(|l| l.to_string())
                .collect::<Vec<_>>()
            {
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
    let resp_max_lines = ((h as usize) / 2).clamp(8, 40);
    if let Some(resp_body) = &step.response_body {
        if !resp_body.is_empty() {
            lines.push(section_header("Response Body", w as usize));
            let pretty = serde_json::from_str::<serde_json::Value>(resp_body)
                .map(|v| serde_json::to_string_pretty(&v).unwrap_or_else(|_| resp_body.clone()))
                .unwrap_or_else(|_| resp_body.clone());
            for owned_line in pretty
                .lines()
                .take(resp_max_lines)
                .map(|l| l.to_string())
                .collect::<Vec<_>>()
            {
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
