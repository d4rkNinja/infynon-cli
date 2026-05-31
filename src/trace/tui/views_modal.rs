fn draw_form_modal(
    f: &mut ratatui::Frame,
    area: Rect,
    is_edit: bool,
    active: EditField,
    fields: &[(EditField, String)],
    status: Option<&(String, bool)>,
) {
    let popup = centered_rect(78, 90, area);
    f.render_widget(Clear, popup);

    let title = if is_edit {
        " Edit Note  [ Tab: next   Enter: save   Esc: cancel ] "
    } else {
        " New Note   [ Tab: next   Enter: save   Esc: cancel ] "
    };

    let mut lines: Vec<Line> = vec![Line::from("")];

    for (fld, val) in fields {
        let fld = *fld;
        // Skip ID field when editing (it's immutable)
        if is_edit && fld == EditField::Id {
            lines.push(Line::from(vec![Span::styled(
                format!("  ID (locked): {}", val),
                Style::default().fg(Color::DarkGray),
            )]));
            lines.push(Line::from(""));
            continue;
        }

        let is_active = fld == active;
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
            format!("  {}█ ", val),
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
            format!("  ⚠  {} ", msg),
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

fn draw_delete_modal(f: &mut ratatui::Frame, area: Rect, id: &str) {
    let popup = centered_rect(48, 22, area);
    f.render_widget(Clear, popup);
    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Delete note: ", Style::default().fg(Color::Yellow)),
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

fn draw_source_delete_modal(f: &mut ratatui::Frame, area: Rect, id: &str) {
    let popup = centered_rect(48, 22, area);
    f.render_widget(Clear, popup);
    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Remove source: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                id,
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  This will remove the backend from the local config.",
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  [y]  confirm remove",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw("      "),
            Span::styled("[n / Esc]  cancel", Style::default().fg(Color::Green)),
        ]),
    ];
    let p = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Confirm Remove Source ")
            .border_style(Style::default().fg(Color::Red)),
    );
    f.render_widget(p, popup);
}

// ─── Knowledge graph panels ──────────────────────────────────────────────────

fn draw_graph_panel(f: &mut ratatui::Frame, area: Rect, app: &mut App) {
    match app.kg_view {
        KgView::Entities => draw_kg_entities(f, area, app),
        KgView::Edges => draw_kg_edges(f, area, app),
        KgView::Visual => draw_kg_visual(f, area, app),
    }
}

fn entity_kind_color(kind: &EntityKind) -> Color {
    match kind {
        EntityKind::File => Color::Cyan,
        EntityKind::Package => Color::Yellow,
        EntityKind::Person => Color::Green,
        EntityKind::Decision => Color::Magenta,
        EntityKind::Vulnerability => Color::Red,
        EntityKind::Endpoint => Color::LightBlue,
        EntityKind::Module => Color::LightCyan,
        EntityKind::Pr => Color::LightYellow,
        EntityKind::Branch => Color::LightGreen,
        EntityKind::Note => Color::Gray,
    }
}

fn draw_kg_entities(f: &mut ratatui::Frame, area: Rect, app: &mut App) {
    let branch_label = if app.kg_branch == "*" {
        "all".to_string()
    } else {
        app.kg_branch.clone()
    };
    let title = format!(
        " Graph Entities [{}]  [ n=new  Enter=edit  d=delete  b=branch  a=all  B=build ] ",
        branch_label
    );
    let items: Vec<ListItem> = if app.kg_entities.is_empty() {
        vec![ListItem::new(
            "  No entities in knowledge graph. Add entities with: infynon trace kg entity add",
        )]
    } else {
        app.kg_entities
            .iter()
            .map(|e| {
                let kc = entity_kind_color(&e.kind);
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("  {:<14} ", e.kind.as_str()),
                        Style::default().fg(kc).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("{:<30} ", e.name),
                        Style::default().fg(Color::White),
                    ),
                    Span::styled(
                        e.branch.to_string(),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]))
            })
            .collect()
    };
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    if app.kg_entities.is_empty() {
        f.render_widget(list, area);
    } else {
        f.render_stateful_widget(list, area, &mut app.kg_entity_state);
    }
}

fn draw_kg_edges(f: &mut ratatui::Frame, area: Rect, app: &mut App) {
    let branch_label = if app.kg_branch == "*" {
        "all".to_string()
    } else {
        app.kg_branch.clone()
    };
    let title = format!(
        " Graph Edges [{}]  [ n=new  Enter=edit  d=delete  b=branch  a=all ] ",
        branch_label
    );
    let items: Vec<ListItem> = if app.kg_edges.is_empty() {
        vec![ListItem::new(
            "  No edges in knowledge graph. Add edges with: infynon trace kg edge add",
        )]
    } else {
        app.kg_edges
            .iter()
            .map(|e| {
                let rel_color = match e.relation.as_str() {
                    "depends_on" => Color::Yellow,
                    "modified_by" => Color::Green,
                    "introduced_by" => Color::Red,
                    "exposes" => Color::LightRed,
                    "tested_by" => Color::Cyan,
                    _ => Color::Gray,
                };
                let evidence_short = if e.evidence.len() > 30 {
                    format!("{}...", &e.evidence[..27])
                } else {
                    e.evidence.clone()
                };
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("  {:<20}", e.source),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(" \u{2192} ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{:<20} ", e.target),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(
                        format!("({}) ", e.relation.as_str()),
                        Style::default().fg(rel_color).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("[{:.1}] ", e.weight),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::styled(evidence_short, Style::default().fg(Color::DarkGray)),
                ]))
            })
            .collect()
    };
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    if app.kg_edges.is_empty() {
        f.render_widget(list, area);
    } else {
        f.render_stateful_widget(list, area, &mut app.kg_entity_state);
    }
}

