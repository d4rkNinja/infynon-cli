pub(super) fn render_prompt_modal(f: &mut Frame, app: &ApiApp, area: Rect) {
    use crate::api::types::PromptType;
    let modal = match &app.prompt_modal {
        Some(m) => m,
        None => return,
    };

    // ── Calculate dynamic height ───────────────────────────────────────────
    let field_heights: Vec<u16> = modal
        .inputs
        .iter()
        .map(|pi| match pi.prompt_type {
            PromptType::Select | PromptType::Multiselect => (2 + pi.options.len() as u16).max(3),
            _ => 3,
        })
        .collect();
    let total_fields_h: u16 = field_heights.iter().sum();
    let h = (total_fields_h + 8)
        .max(10)
        .min(area.height.saturating_sub(4));
    let w = (area.width * 65 / 100)
        .max(52)
        .min(area.width.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let overlay_area = Rect {
        x,
        y,
        width: w,
        height: h,
    };

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
        Paragraph::new(Line::from(vec![Span::styled(
            " Provide values before the request fires:",
            Style::default().fg(TEXT_DIM),
        )])),
        sections[0],
    );

    // ── Render each prompt field ───────────────────────────────────────────
    for (i, pi) in modal.inputs.iter().enumerate() {
        let field_area = sections[i + 1];
        let is_current = i == modal.current_field;
        let label = if pi.label.is_empty() {
            pi.var.as_str()
        } else {
            pi.label.as_str()
        };

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
                    pi.default
                        .as_deref()
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
                        Style::default()
                            .fg(GREEN)
                            .add_modifier(Modifier::BOLD | Modifier::REVERSED),
                        Style::default().fg(DIM),
                    )
                } else {
                    (
                        Style::default().fg(DIM),
                        Style::default()
                            .fg(RED)
                            .add_modifier(Modifier::BOLD | Modifier::REVERSED),
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

                let items: Vec<ListItem> = pi
                    .options
                    .iter()
                    .enumerate()
                    .map(|(j, opt)| {
                        let is_sel = j == cursor_idx;
                        let style = if is_sel && is_current {
                            Style::default().fg(CYAN).add_modifier(Modifier::BOLD)
                        } else if is_sel {
                            Style::default().fg(WHITE)
                        } else {
                            Style::default().fg(DIM)
                        };
                        let prefix = if is_sel { "> " } else { "  " };
                        ListItem::new(Line::from(vec![Span::styled(
                            format!(" {}{}", prefix, opt),
                            style,
                        )]))
                    })
                    .collect();
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
                let checked = modal
                    .multi_checked
                    .get(i)
                    .map(|v| v.as_slice())
                    .unwrap_or(&[]);
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

                let items: Vec<ListItem> = pi
                    .options
                    .iter()
                    .enumerate()
                    .map(|(j, opt)| {
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
                    })
                    .collect();
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
            Span::styled(
                "Tab",
                Style::default().fg(YELLOW).add_modifier(Modifier::BOLD),
            ),
            Span::styled("] next field  ", Style::default().fg(DIM)),
            Span::styled("[", Style::default().fg(YELLOW)),
            Span::styled(
                "Up/Dn",
                Style::default().fg(YELLOW).add_modifier(Modifier::BOLD),
            ),
            Span::styled("] navigate  ", Style::default().fg(DIM)),
            Span::styled("[", Style::default().fg(YELLOW)),
            Span::styled(
                "Enter",
                Style::default().fg(YELLOW).add_modifier(Modifier::BOLD),
            ),
            Span::styled("] confirm  ", Style::default().fg(DIM)),
            Span::styled("[", Style::default().fg(YELLOW)),
            Span::styled(
                "Esc",
                Style::default().fg(YELLOW).add_modifier(Modifier::BOLD),
            ),
            Span::styled("] cancel", Style::default().fg(DIM)),
        ])),
        footer_area,
    );
}

// ── Body editor modal ─────────────────────────────────────────────────────────
