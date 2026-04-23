fn draw_kg_visual(f: &mut ratatui::Frame, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // Left: entity list
    draw_kg_entities(f, chunks[0], app);

    // Right: visual graph
    let selected_id = app
        .kg_selected_entity
        .and_then(|i| app.kg_entities.get(i))
        .map(|e| e.id.clone());

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));

    // Group entities by kind
    let mut grouped: std::collections::BTreeMap<String, Vec<&KgEntity>> =
        std::collections::BTreeMap::new();
    for ent in &app.kg_entities {
        grouped
            .entry(ent.kind.as_str().to_string())
            .or_default()
            .push(ent);
    }

    let max_lines = chunks[1].height.saturating_sub(4) as usize;
    let mut line_count = 0;

    for (kind, entities) in &grouped {
        if line_count >= max_lines {
            break;
        }
        // Section header
        let header = format!("  \u{250c}\u{2500} {} ", kind);
        let pad = (chunks[1].width as usize).saturating_sub(header.len() + 3);
        lines.push(Line::from(Span::styled(
            format!("{}{}\u{2510}", header, "\u{2500}".repeat(pad)),
            Style::default().fg(Color::DarkGray),
        )));
        line_count += 1;

        for ent in entities {
            if line_count >= max_lines {
                break;
            }
            let kc = entity_kind_color(&ent.kind);
            let is_selected = selected_id.as_deref() == Some(&ent.id);
            let name_style = if is_selected {
                Style::default()
                    .fg(kc)
                    .add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else {
                Style::default().fg(kc)
            };

            // Find outgoing edges for this entity
            let outgoing: Vec<&KgEdge> = app
                .kg_edges
                .iter()
                .filter(|edge| edge.source == ent.id)
                .collect();

            if outgoing.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("  \u{2502}  ", Style::default().fg(Color::DarkGray)),
                    Span::styled(format!("[{}]", ent.name), name_style),
                ]));
                line_count += 1;
            } else {
                for edge in &outgoing {
                    if line_count >= max_lines {
                        break;
                    }
                    let rel_color = match edge.relation.as_str() {
                        "depends_on" => Color::Yellow,
                        "modified_by" => Color::Green,
                        "introduced_by" => Color::Red,
                        "exposes" => Color::LightRed,
                        "tested_by" => Color::Cyan,
                        _ => Color::Gray,
                    };
                    // Find target entity name
                    let target_name = app
                        .kg_entities
                        .iter()
                        .find(|e| e.id == edge.target)
                        .map(|e| e.name.as_str())
                        .unwrap_or(&edge.target);
                    let target_kind = app
                        .kg_entities
                        .iter()
                        .find(|e| e.id == edge.target)
                        .map(|e| &e.kind);
                    let tc = target_kind.map(entity_kind_color).unwrap_or(Color::White);

                    lines.push(Line::from(vec![
                        Span::styled("  \u{2502}  ", Style::default().fg(Color::DarkGray)),
                        Span::styled(format!("[{}]", ent.name), name_style),
                        Span::styled(
                            format!(
                                " \u{2500}\u{2500}{}\u{2500}\u{2500}\u{25b6} ",
                                edge.relation.as_str()
                            ),
                            Style::default().fg(rel_color),
                        ),
                        Span::styled(format!("[{}]", target_name), Style::default().fg(tc)),
                    ]));
                    line_count += 1;
                }
            }
        }

        if line_count < max_lines {
            let footer_pad = (chunks[1].width as usize).saturating_sub(5);
            lines.push(Line::from(Span::styled(
                format!("  \u{2514}{}\u{2518}", "\u{2500}".repeat(footer_pad)),
                Style::default().fg(Color::DarkGray),
            )));
            line_count += 1;
        }
    }

    if app.kg_entities.is_empty() && app.kg_edges.is_empty() {
        lines.push(Line::from(Span::styled(
            "  Empty knowledge graph.",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let vg_branch_label = if app.kg_branch == "*" {
        "all".to_string()
    } else {
        app.kg_branch.clone()
    };
    let title = format!(" Visual Graph [{}] ", vg_branch_label);
    let p = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: false });
    f.render_widget(p, chunks[1]);
}

// ─── KG modals ──────────────────────────────────────────────────────────────

fn draw_kg_entity_form_modal(
    f: &mut ratatui::Frame,
    area: Rect,
    form: &KgEntityForm,
    status: Option<&(String, bool)>,
) {
    let popup = centered_rect(78, 70, area);
    f.render_widget(Clear, popup);

    let title = if form.is_edit {
        " Edit Entity  [ Tab: next   Enter: save   Esc: cancel ] "
    } else {
        " New Entity   [ Tab: next   Enter: save   Esc: cancel ] "
    };

    let mut lines: Vec<Line> = vec![Line::from("")];

    for &fld in &KgEntityField::all() {
        let is_active = fld == form.active_field;
        let lbl_style = if is_active {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Yellow)
        };
        let val_style = if is_active {
            Style::default().fg(Color::White).bg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Gray)
        };

        lines.push(Line::from(vec![Span::styled(
            format!("  {} ", fld.label()),
            lbl_style,
        )]));
        lines.push(Line::from(vec![Span::styled(
            format!("  {}\u{2588} ", form.get_field(fld)),
            val_style,
        )]));
        lines.push(Line::from(""));
    }

    if let Some((msg, is_err)) = status {
        let style = if *is_err {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Green)
        };
        lines.push(Line::from(vec![Span::styled(
            format!("  !  {} ", msg),
            style,
        )]));
    }

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(p, popup);
}

fn draw_kg_edge_form_modal(
    f: &mut ratatui::Frame,
    area: Rect,
    form: &KgEdgeForm,
    status: Option<&(String, bool)>,
) {
    let popup = centered_rect(78, 90, area);
    f.render_widget(Clear, popup);

    let title = if form.is_edit {
        " Edit Edge  [ Tab: next   Enter: save   Esc: cancel ] "
    } else {
        " New Edge   [ Tab: next   Enter: save   Esc: cancel ] "
    };

    let mut lines: Vec<Line> = vec![Line::from("")];

    for &fld in &KgEdgeField::all() {
        let is_active = fld == form.active_field;
        let lbl_style = if is_active {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Yellow)
        };
        let val_style = if is_active {
            Style::default().fg(Color::White).bg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Gray)
        };

        lines.push(Line::from(vec![Span::styled(
            format!("  {} ", fld.label()),
            lbl_style,
        )]));
        lines.push(Line::from(vec![Span::styled(
            format!("  {}\u{2588} ", form.get_field(fld)),
            val_style,
        )]));
        lines.push(Line::from(""));
    }

    if let Some((msg, is_err)) = status {
        let style = if *is_err {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Green)
        };
        lines.push(Line::from(vec![Span::styled(
            format!("  !  {} ", msg),
            style,
        )]));
    }

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(p, popup);
}

fn draw_kg_delete_modal(f: &mut ratatui::Frame, area: Rect, entity_type: &str, id: &str) {
    let popup = centered_rect(48, 22, area);
    f.render_widget(Clear, popup);
    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("  Delete {}: ", entity_type),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                id,
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  This action cannot be undone.",
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  [y]  confirm delete",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw("      "),
            Span::styled("[n / Esc]  cancel", Style::default().fg(Color::Green)),
        ]),
    ];
    let p = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Confirm Delete ")
            .border_style(Style::default().fg(Color::Red)),
    );
    f.render_widget(p, popup);
}

fn draw_kg_branch_picker(
    f: &mut ratatui::Frame,
    area: Rect,
    branches: &[String],
    selected: usize,
    current: &str,
) {
    let popup = centered_rect(50, 60, area);
    f.render_widget(Clear, popup);

    let items: Vec<ListItem> = branches
        .iter()
        .enumerate()
        .map(|(i, b)| {
            let is_current = b == current;
            let marker = if is_current { " *" } else { "" };
            let style = if i == selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else if is_current {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Line::from(Span::styled(
                format!("  {}{}", b, marker),
                style,
            )))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Branch Picker  [ Enter: switch   a: all   Esc: cancel ] ")
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(list, popup);
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn kv<'a>(label: &'a str, value: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled(label, Style::default().fg(Color::Yellow)),
        Span::raw(value),
    ])
}

fn centered_rect(pct_x: u16, pct_y: u16, r: Rect) -> Rect {
    let margin_v = (100 - pct_y) / 2;
    let margin_h = (100 - pct_x) / 2;
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(margin_v),
            Constraint::Percentage(pct_y),
            Constraint::Percentage(margin_v),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(margin_h),
            Constraint::Percentage(pct_x),
            Constraint::Percentage(margin_h),
        ])
        .split(vert[1])[1]
}
