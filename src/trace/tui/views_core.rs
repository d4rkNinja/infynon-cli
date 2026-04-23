fn draw_ui(f: &mut ratatui::Frame, app: &mut App, cfg: &crate::trace::types::TraceConfig) {
    let area = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(area);

    // Tab bar
    let titles: Vec<Line> = TraceTab::all()
        .iter()
        .map(|t| Line::from(t.title()))
        .collect();
    let tabs = Tabs::new(titles)
        .select(app.tab.index())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Trace Memory "),
        )
        .style(Style::default().fg(Color::Gray))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(tabs, chunks[0]);

    // Main content area
    match &app.mode {
        AppMode::Browse => match app.tab {
            TraceTab::Overview => draw_overview(f, chunks[1], cfg, app),
            TraceTab::Sources => draw_sources(f, chunks[1], cfg, &mut app.sources_state),
            TraceTab::Notes => draw_notes_panel(f, chunks[1], app),
            TraceTab::Packages => draw_packages(f, chunks[1], app),
            TraceTab::EditLog => draw_edit_log(f, chunks[1], &app.audit, app.audit_scroll),
            TraceTab::Graph => draw_graph_panel(f, chunks[1], app),
        },
        AppMode::ViewDetail => draw_note_detail(f, chunks[1], app),
        AppMode::PackageDetail => draw_package_detail(f, chunks[1], app),
        AppMode::EditForm(_) | AppMode::DeleteConfirm(_) | AppMode::SourceDeleteConfirm(_) => {
            // Draw the current tab as background, then overlay the modal
            match app.tab {
                TraceTab::Notes => draw_notes_panel(f, chunks[1], app),
                TraceTab::Sources => draw_sources(f, chunks[1], cfg, &mut app.sources_state),
                _ => {}
            }
            match &app.mode {
                AppMode::EditForm(form) => {
                    let is_edit = form.is_edit;
                    let active = form.active_field;
                    let fields: Vec<(EditField, String)> = EditField::all()
                        .iter()
                        .map(|&fld| (fld, form.get_field(fld).to_string()))
                        .collect();
                    let status_clone = app.status.clone();
                    draw_form_modal(f, area, is_edit, active, &fields, status_clone.as_ref());
                }
                AppMode::DeleteConfirm(id) => {
                    let id = id.clone();
                    draw_delete_modal(f, area, &id);
                }
                AppMode::SourceDeleteConfirm(id) => {
                    let id = id.clone();
                    draw_source_delete_modal(f, area, &id);
                }
                _ => {}
            }
        }
        AppMode::KgEntityForm(_)
        | AppMode::KgEdgeForm(_)
        | AppMode::KgEntityDelete(_)
        | AppMode::KgEdgeDelete(_)
        | AppMode::KgBranchPicker => {
            draw_graph_panel(f, chunks[1], app);
            match &app.mode {
                AppMode::KgEntityForm(form) => {
                    let status_clone = app.status.clone();
                    draw_kg_entity_form_modal(f, area, form, status_clone.as_ref());
                }
                AppMode::KgEdgeForm(form) => {
                    let status_clone = app.status.clone();
                    draw_kg_edge_form_modal(f, area, form, status_clone.as_ref());
                }
                AppMode::KgEntityDelete(id) => {
                    let id = id.clone();
                    draw_kg_delete_modal(f, area, "entity", &id);
                }
                AppMode::KgEdgeDelete(id) => {
                    let id = id.clone();
                    draw_kg_delete_modal(f, area, "edge", &id);
                }
                AppMode::KgBranchPicker => {
                    let branches = app.kg_branches.clone();
                    let idx = app.kg_branch_idx;
                    let current = app.kg_branch.clone();
                    draw_kg_branch_picker(f, area, &branches, idx, &current);
                }
                _ => {}
            }
        }
    }

    // Status bar
    let help = match &app.mode {
        AppMode::ViewDetail => " ↑↓/jk: nav   e: edit   d: delete   Esc/q: back",
        AppMode::PackageDetail => " ↑↓/jk: nav   Esc/q: back to list",
        AppMode::EditForm(_) => " Tab: next field   Shift+Tab: prev   Enter: save   Esc: cancel",
        AppMode::DeleteConfirm(_) | AppMode::SourceDeleteConfirm(_)
        | AppMode::KgEntityDelete(_) | AppMode::KgEdgeDelete(_) => {
            " y: confirm delete   n/Esc: cancel"
        }
        AppMode::KgEntityForm(_) | AppMode::KgEdgeForm(_) => {
            " Tab: next  Shift+Tab: prev  Enter: save  Esc: cancel"
        }
        AppMode::KgBranchPicker => {
            " up/down: select  Enter: switch  a: all branches  Esc: cancel"
        }
        AppMode::Browse => match app.tab {
            TraceTab::Notes => {
                " ↑↓/jk: nav   Enter: view   n: new   e: edit   d: delete   r: reload   h/l: tabs   q: quit"
            }
            TraceTab::Sources => {
                " ↑↓/jk: nav   s: set default   d: remove   r: reload   h/l: tabs   q: quit"
            }
            TraceTab::Packages => {
                " ↑↓/jk: nav   Enter: detail   r: reload   h/l: tabs   q: quit"
            }
            TraceTab::EditLog => {
                " ↑↓/jk: scroll   g: top   G: bottom   r: reload   h/l: tabs   q: quit"
            }
            TraceTab::Graph => match app.kg_view {
                KgView::Entities => {
                    " up/down: nav  n: new  Enter: edit  d: delete  b: branch  a: all  B: build  Tab: view  r: reload  q: quit"
                }
                KgView::Edges => {
                    " up/down: nav  n: new  Enter: edit  d: delete  b: branch  a: all  Tab: view  r: reload  q: quit"
                }
                KgView::Visual => {
                    " up/down: nav  b: branch  a: all  B: build  Tab: view  e/w/v: switch  r: reload  q: quit"
                }
            }
            _ => " 1-6: tabs   h/l: switch tab   q: quit",
        },
    };
    let status_text = match &app.status {
        Some((msg, is_err)) => {
            let prefix = if *is_err { "✗ " } else { "✓ " };
            format!(" {} {}  │{}", prefix, msg, help)
        }
        None => help.to_string(),
    };
    let status_style = match &app.status {
        Some((_, true)) => Style::default().fg(Color::Red),
        Some((_, false)) => Style::default().fg(Color::Green),
        None => Style::default().fg(Color::DarkGray),
    };
    let status_bar = Paragraph::new(status_text)
        .style(status_style)
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(status_bar, chunks[2]);
}

fn draw_overview(
    f: &mut ratatui::Frame,
    area: Rect,
    cfg: &crate::trace::types::TraceConfig,
    app: &App,
) {
    let src_count = cfg.sources.len().to_string();
    let note_count = app.notes.len().to_string();
    let pkg_count = app.packages.len().to_string();
    let lines = vec![
        Line::from(""),
        kv("  Repo          ", &cfg.repo_name),
        kv("  Owner         ", &cfg.owner),
        kv(
            "  Default user  ",
            cfg.default_user.as_deref().unwrap_or("-"),
        ),
        kv("  Sources       ", &src_count),
        kv("  Notes         ", &note_count),
        kv("  Pkg findings  ", &pkg_count),
        Line::from(""),
        Line::from(Span::styled(
            "  Tab 5 → Edit Log shows a full audit trail of every create / edit / delete via TUI.",
            Style::default().fg(Color::DarkGray),
        )),
    ];
    let p = Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(" Overview "));
    f.render_widget(p, area);
}

fn draw_sources(
    f: &mut ratatui::Frame,
    area: Rect,
    cfg: &crate::trace::types::TraceConfig,
    state: &mut ListState,
) {
    let items: Vec<ListItem> = if cfg.sources.is_empty() {
        vec![ListItem::new(
            "  No sources configured. Run: infynon trace source add-redis / add-sql",
        )]
    } else {
        cfg.sources
            .iter()
            .map(|src| {
                let is_default = cfg.default_source.as_deref() == Some(src.id.as_str());
                let def_span = if is_default {
                    Span::styled(" ★", Style::default().fg(Color::Green))
                } else {
                    Span::raw("  ")
                };
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("  {:<18} ", src.id),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(
                        format!("[{:<8}]", src.kind.as_str()),
                        Style::default().fg(Color::Yellow),
                    ),
                    def_span,
                    Span::raw(format!(
                        "  user: {:<14}  {}",
                        src.owner_user.as_deref().unwrap_or("-"),
                        src.url
                    )),
                ]))
            })
            .collect()
    };
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Sources  [ s=set default  d=remove  r=reload ] "),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    if cfg.sources.is_empty() {
        f.render_widget(list, area);
    } else {
        f.render_stateful_widget(list, area, state);
    }
}

fn draw_notes_panel(f: &mut ratatui::Frame, area: Rect, app: &mut App) {
    let items: Vec<ListItem> = if app.notes.is_empty() {
        vec![ListItem::new("  No notes yet.  Press  n  to create one.")]
    } else {
        app.notes
            .iter()
            .map(|n| {
                let sc = match n.status {
                    NoteStatus::Active => Color::Green,
                    NoteStatus::Stale => Color::Yellow,
                    NoteStatus::Archived => Color::DarkGray,
                };
                ListItem::new(Line::from(vec![
                    Span::styled(format!("  {:<16} ", n.id), Style::default().fg(Color::Cyan)),
                    Span::styled(
                        format!("[{:<9}]", n.layer.as_str()),
                        Style::default().fg(Color::Blue),
                    ),
                    Span::styled(
                        format!(" {:<9}", n.scope.as_str()),
                        Style::default().fg(Color::Magenta),
                    ),
                    Span::styled(
                        format!(" {:<10}", n.status.as_str()),
                        Style::default().fg(sc),
                    ),
                    Span::styled(
                        format!(" {:<12} ", n.author),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw(&n.title),
                ]))
            })
            .collect()
    };
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Notes  [ n=new  e=edit  d=delete  Enter=view ] "),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    f.render_stateful_widget(list, area, &mut app.list_state);
}

fn draw_note_detail(f: &mut ratatui::Frame, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(38), Constraint::Percentage(62)])
        .split(area);

    draw_notes_panel(f, chunks[0], app);

    if let Some(note) = app.selected_note().cloned() {
        let tags_joined = note.tags.join(", ");
        let updated_short = note.updated_at[..note.updated_at.len().min(19)].to_string();
        let lines = vec![
            Line::from(""),
            kv("  ID:       ", &note.id),
            kv("  Title:    ", &note.title),
            kv("  Layer:    ", note.layer.as_str()),
            kv("  Scope:    ", note.scope.as_str()),
            kv("  Target:   ", &note.target),
            kv("  Author:   ", &note.author),
            kv("  Status:   ", note.status.as_str()),
            kv("  Tags:     ", &tags_joined),
            kv("  Updated:  ", &updated_short),
            Line::from(""),
            Line::from(Span::styled(
                "  Body:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from("  ─────────────────────────────────────────────"),
            Line::from(format!("  {}", note.body)),
        ];
        let p = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Note Detail  [ Esc: back ] "),
            )
            .wrap(Wrap { trim: false });
        f.render_widget(p, chunks[1]);
    }
}

fn draw_packages(f: &mut ratatui::Frame, area: Rect, app: &mut App) {
    let items: Vec<ListItem> = if app.packages.is_empty() {
        vec![ListItem::new(
            "  No vulnerable packages found, or no lock files detected in this directory.",
        )]
    } else {
        app.packages
            .iter()
            .map(|r| {
                let sev_color = sev_color(r.severity.as_str());
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("  {:<9}", r.severity),
                        Style::default().fg(sev_color).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("{}@{}  ", r.package, r.version),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(
                        format!("{:<22}", r.vulnerability_id),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw(format!(
                        "owner: {}",
                        r.installed_by.as_deref().unwrap_or("unknown")
                    )),
                ]))
            })
            .collect()
    };
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Package Risks  [ Enter=detail  r=reload ] "),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    if app.packages.is_empty() {
        f.render_widget(list, area);
    } else {
        f.render_stateful_widget(list, area, &mut app.packages_state);
    }
}

fn draw_package_detail(f: &mut ratatui::Frame, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    draw_packages(f, chunks[0], app);

    if let Some(r) = app.selected_package().cloned() {
        let sc = sev_color(r.severity.as_str());
        let pkg_ver = format!("{}@{}", r.package, r.version);
        let lines = vec![
            Line::from(""),
            kv("  Vulnerability:  ", &r.vulnerability_id),
            kv("  Package:        ", &pkg_ver),
            kv("  Ecosystem:      ", &r.ecosystem),
            Line::from(vec![
                Span::styled("  Severity:       ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    r.severity.clone(),
                    Style::default().fg(sc).add_modifier(Modifier::BOLD),
                ),
            ]),
            kv("  Source file:    ", &r.source_file),
            kv(
                "  Installed by:   ",
                r.installed_by.as_deref().unwrap_or("unknown"),
            ),
            Line::from(""),
            Line::from(Span::styled(
                "  Press Esc to go back.",
                Style::default().fg(Color::DarkGray),
            )),
        ];
        let p = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Package Detail  [ Esc: back ] "),
            )
            .wrap(Wrap { trim: false });
        f.render_widget(p, chunks[1]);
    }
}

fn sev_color(sev: &str) -> Color {
    match sev {
        "CRITICAL" => Color::Red,
        "HIGH" => Color::LightRed,
        "MEDIUM" => Color::Yellow,
        _ => Color::Gray,
    }
}

fn draw_edit_log(f: &mut ratatui::Frame, area: Rect, entries: &[AuditEntry], scroll: usize) {
    let title = if entries.is_empty() {
        " Edit Log ".to_string()
    } else {
        format!(
            " Edit Log  — {} entries  ({}/{}) ",
            entries.len(),
            scroll + 1,
            entries.len()
        )
    };

    let items: Vec<ListItem> = if entries.is_empty() {
        vec![ListItem::new(
            "  No TUI edits recorded yet. Create, edit, or delete a note to begin the audit trail.",
        )]
    } else {
        entries
            .iter()
            .skip(scroll)
            .map(|e| {
                let ac = match e.action.as_str() {
                    "create" => Color::Green,
                    "edit" => Color::Yellow,
                    "delete" => Color::Red,
                    _ => Color::Gray,
                };
                let ts = &e.timestamp[..e.timestamp.len().min(19)];
                ListItem::new(Line::from(vec![
                    Span::styled(format!("  {} ", ts), Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{:<7} ", e.action.to_uppercase()),
                        Style::default().fg(ac).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("{:<18} ", e.note_id),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(
                        format!("by {:<14} ", e.author),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw(&e.summary),
                ]))
            })
            .collect()
    };
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(list, area);
}

