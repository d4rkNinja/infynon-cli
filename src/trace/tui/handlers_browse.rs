fn handle_browse(app: &mut App, cfg: &mut crate::trace::types::TraceConfig, code: KeyCode) -> bool {
    app.status = None; // clear previous status on any key

    match code {
        // ── Back from detail views ──────────────────────────────────────────
        KeyCode::Char('q') | KeyCode::Esc
            if matches!(app.mode, AppMode::ViewDetail | AppMode::PackageDetail) =>
        {
            app.mode = AppMode::Browse;
        }
        KeyCode::Char('q') => return true,

        // ── Tab switching ────────────────────────────────────────────────────
        KeyCode::Char('1') => {
            app.tab = TraceTab::Overview;
            app.mode = AppMode::Browse;
        }
        KeyCode::Char('2') => {
            app.tab = TraceTab::Sources;
            app.mode = AppMode::Browse;
        }
        KeyCode::Char('3') => {
            app.tab = TraceTab::Notes;
            app.mode = AppMode::Browse;
        }
        KeyCode::Char('4') => {
            app.tab = TraceTab::Packages;
            app.mode = AppMode::Browse;
        }
        KeyCode::Char('5') => {
            app.tab = TraceTab::EditLog;
            app.mode = AppMode::Browse;
            app.reload_audit();
        }
        KeyCode::Char('6') => {
            app.tab = TraceTab::Graph;
            app.mode = AppMode::Browse;
            app.reload_kg();
        }
        KeyCode::Right | KeyCode::Char('l')
            if !matches!(app.mode, AppMode::ViewDetail | AppMode::PackageDetail) =>
        {
            let next = (app.tab.index() + 1) % 6;
            app.tab = TraceTab::all()[next];
            app.mode = AppMode::Browse;
            if app.tab == TraceTab::EditLog {
                app.reload_audit();
            }
            if app.tab == TraceTab::Graph {
                app.reload_kg();
            }
        }
        KeyCode::Left | KeyCode::Char('h')
            if !matches!(app.mode, AppMode::ViewDetail | AppMode::PackageDetail) =>
        {
            let prev = (app.tab.index() + 6 - 1) % 6;
            app.tab = TraceTab::all()[prev];
            app.mode = AppMode::Browse;
            if app.tab == TraceTab::Graph {
                app.reload_kg();
            }
        }

        // ── Notes tab ────────────────────────────────────────────────────────
        KeyCode::Down | KeyCode::Char('j') if app.tab == TraceTab::Notes => {
            let len = app.notes.len();
            if len > 0 {
                let next = app.selected_idx().map(|i| (i + 1) % len).unwrap_or(0);
                app.list_state.select(Some(next));
            }
        }
        KeyCode::Up | KeyCode::Char('k') if app.tab == TraceTab::Notes => {
            let len = app.notes.len();
            if len > 0 {
                let prev = app
                    .selected_idx()
                    .map(|i| if i == 0 { len - 1 } else { i - 1 })
                    .unwrap_or(0);
                app.list_state.select(Some(prev));
            }
        }
        KeyCode::Enter if app.tab == TraceTab::Notes => {
            if app.selected_note().is_some() {
                app.mode = AppMode::ViewDetail;
            }
        }
        KeyCode::Char('n') if app.tab == TraceTab::Notes => {
            let author = storage::configured_user()
                .or_else(storage::detect_user_name)
                .unwrap_or_else(|| "unknown".to_string());
            app.mode = AppMode::EditForm(NoteForm::new_create(author));
        }
        KeyCode::Char('e') if app.tab == TraceTab::Notes => {
            if let Some(note) = app.selected_note() {
                let form = NoteForm::from_note(note);
                app.mode = AppMode::EditForm(form);
            }
        }
        KeyCode::Char('d') if app.tab == TraceTab::Notes => {
            if let Some(note) = app.selected_note() {
                let id = note.id.clone();
                app.mode = AppMode::DeleteConfirm(id);
            }
        }
        KeyCode::Char('r') if app.tab == TraceTab::Notes => {
            app.reload_notes();
            app.clamp_selection();
            app.ok("Notes reloaded");
        }

        // ── Sources tab ──────────────────────────────────────────────────────
        KeyCode::Down | KeyCode::Char('j') if app.tab == TraceTab::Sources => {
            let len = cfg.sources.len();
            if len > 0 {
                let next = app
                    .sources_state
                    .selected()
                    .map(|i| (i + 1) % len)
                    .unwrap_or(0);
                app.sources_state.select(Some(next));
            }
        }
        KeyCode::Up | KeyCode::Char('k') if app.tab == TraceTab::Sources => {
            let len = cfg.sources.len();
            if len > 0 {
                let prev = app
                    .sources_state
                    .selected()
                    .map(|i| if i == 0 { len - 1 } else { i - 1 })
                    .unwrap_or(0);
                app.sources_state.select(Some(prev));
            }
        }
        KeyCode::Char('d') if app.tab == TraceTab::Sources => {
            if let Some(idx) = app.sources_state.selected() {
                if let Some(src) = cfg.sources.get(idx) {
                    app.mode = AppMode::SourceDeleteConfirm(src.id.clone());
                }
            }
        }
        KeyCode::Char('s') if app.tab == TraceTab::Sources => {
            if let Some(idx) = app.sources_state.selected() {
                if let Some(src) = cfg.sources.get(idx) {
                    let id = src.id.clone();
                    match storage::set_default_source(&id) {
                        Ok(()) => {
                            *cfg = storage::load_config().unwrap_or_default();
                            app.reload_sources(cfg);
                            if let Some(selected_idx) =
                                cfg.sources.iter().position(|source| source.id == id)
                            {
                                app.sources_state.select(Some(selected_idx));
                            }
                            app.ok(format!("Default source set to '{}'", id));
                        }
                        Err(e) => app.err(e),
                    }
                }
            }
        }
        KeyCode::Char('r') if app.tab == TraceTab::Sources => {
            *cfg = storage::load_config().unwrap_or_default();
            app.reload_sources(cfg);
            app.ok("Sources reloaded");
        }

        // ── Packages tab ─────────────────────────────────────────────────────
        KeyCode::Down | KeyCode::Char('j') if app.tab == TraceTab::Packages => {
            let len = app.packages.len();
            if len > 0 {
                let next = app
                    .packages_state
                    .selected()
                    .map(|i| (i + 1) % len)
                    .unwrap_or(0);
                app.packages_state.select(Some(next));
            }
        }
        KeyCode::Up | KeyCode::Char('k') if app.tab == TraceTab::Packages => {
            let len = app.packages.len();
            if len > 0 {
                let prev = app
                    .packages_state
                    .selected()
                    .map(|i| if i == 0 { len - 1 } else { i - 1 })
                    .unwrap_or(0);
                app.packages_state.select(Some(prev));
            }
        }
        KeyCode::Enter if app.tab == TraceTab::Packages => {
            if app.selected_package().is_some() {
                app.mode = AppMode::PackageDetail;
            }
        }
        KeyCode::Char('r') if app.tab == TraceTab::Packages => {
            app.reload_packages();
            app.ok("Package risks reloaded");
        }

        // ── EditLog tab ──────────────────────────────────────────────────────
        KeyCode::Down | KeyCode::Char('j') if app.tab == TraceTab::EditLog => {
            if app.audit_scroll + 1 < app.audit.len() {
                app.audit_scroll += 1;
            }
        }
        KeyCode::Up | KeyCode::Char('k') if app.tab == TraceTab::EditLog => {
            app.audit_scroll = app.audit_scroll.saturating_sub(1);
        }
        KeyCode::Char('g') if app.tab == TraceTab::EditLog => {
            app.audit_scroll = 0;
        }
        KeyCode::Char('G') if app.tab == TraceTab::EditLog => {
            app.audit_scroll = app.audit.len().saturating_sub(1);
        }
        KeyCode::Char('r') if app.tab == TraceTab::EditLog => {
            app.reload_audit();
            app.ok("Edit log reloaded");
        }

        // ── Graph tab ───────────────────────────────────────────────────────
        KeyCode::Char('n') if app.tab == TraceTab::Graph && app.kg_view == KgView::Entities => {
            app.mode = AppMode::KgEntityForm(KgEntityForm::new_create(&app.kg_branch));
        }
        KeyCode::Char('n') if app.tab == TraceTab::Graph && app.kg_view == KgView::Edges => {
            app.mode = AppMode::KgEdgeForm(KgEdgeForm::new_create(&app.kg_branch));
        }
        KeyCode::Enter if app.tab == TraceTab::Graph && app.kg_view == KgView::Entities => {
            if let Some(idx) = app.kg_entity_state.selected() {
                if let Some(ent) = app.kg_entities.get(idx) {
                    app.mode = AppMode::KgEntityForm(KgEntityForm::from_entity(ent));
                }
            }
        }
        KeyCode::Enter if app.tab == TraceTab::Graph && app.kg_view == KgView::Edges => {
            if let Some(idx) = app.kg_entity_state.selected() {
                if let Some(edge) = app.kg_edges.get(idx) {
                    app.mode = AppMode::KgEdgeForm(KgEdgeForm::from_edge(edge));
                }
            }
        }
        KeyCode::Char('d') if app.tab == TraceTab::Graph && app.kg_view == KgView::Entities => {
            if let Some(idx) = app.kg_entity_state.selected() {
                if let Some(ent) = app.kg_entities.get(idx) {
                    app.mode = AppMode::KgEntityDelete(ent.id.clone());
                }
            }
        }
        KeyCode::Char('d') if app.tab == TraceTab::Graph && app.kg_view == KgView::Edges => {
            if let Some(idx) = app.kg_entity_state.selected() {
                if let Some(edge) = app.kg_edges.get(idx) {
                    app.mode = AppMode::KgEdgeDelete(edge.id.clone());
                }
            }
        }
        KeyCode::Char('b') if app.tab == TraceTab::Graph => {
            app.reload_kg_branches();
            app.mode = AppMode::KgBranchPicker;
        }
        KeyCode::Char('a') if app.tab == TraceTab::Graph => {
            if app.kg_branch == "*" {
                app.kg_branch = storage::detect_current_branch();
                app.reload_kg();
                app.ok("Showing current branch");
            } else {
                app.kg_branch = "*".to_string();
                app.reload_kg_all();
                app.ok("Showing all branches");
            }
        }
        KeyCode::Char('B') if app.tab == TraceTab::Graph => {
            let _ = storage::ensure_kg_layout();
            let branch = if app.kg_branch == "*" {
                storage::detect_current_branch()
            } else {
                app.kg_branch.clone()
            };
            match storage::auto_build_graph(&branch) {
                Ok((ents, edges)) => {
                    app.ok(format!("Built: {} entities, {} edges", ents, edges));
                    app.reload_kg();
                    app.reload_kg_branches();
                }
                Err(e) => app.err(e),
            }
        }
        KeyCode::Down | KeyCode::Char('j') if app.tab == TraceTab::Graph => match app.kg_view {
            KgView::Entities => {
                let len = app.kg_entities.len();
                if len > 0 {
                    let next = app
                        .kg_entity_state
                        .selected()
                        .map(|i| (i + 1) % len)
                        .unwrap_or(0);
                    app.kg_entity_state.select(Some(next));
                    app.kg_selected_entity = Some(next);
                }
            }
            KgView::Edges | KgView::Visual => {
                let len = app.kg_edges.len();
                if len > 0 {
                    let next = app
                        .kg_entity_state
                        .selected()
                        .map(|i| (i + 1) % len)
                        .unwrap_or(0);
                    app.kg_entity_state.select(Some(next));
                }
            }
        },
        KeyCode::Up | KeyCode::Char('k') if app.tab == TraceTab::Graph => match app.kg_view {
            KgView::Entities => {
                let len = app.kg_entities.len();
                if len > 0 {
                    let prev = app
                        .kg_entity_state
                        .selected()
                        .map(|i| if i == 0 { len - 1 } else { i - 1 })
                        .unwrap_or(0);
                    app.kg_entity_state.select(Some(prev));
                    app.kg_selected_entity = Some(prev);
                }
            }
            KgView::Edges | KgView::Visual => {
                let len = app.kg_edges.len();
                if len > 0 {
                    let prev = app
                        .kg_entity_state
                        .selected()
                        .map(|i| if i == 0 { len - 1 } else { i - 1 })
                        .unwrap_or(0);
                    app.kg_entity_state.select(Some(prev));
                }
            }
        },
        KeyCode::Tab if app.tab == TraceTab::Graph => {
            app.kg_view = match app.kg_view {
                KgView::Entities => KgView::Edges,
                KgView::Edges => KgView::Visual,
                KgView::Visual => KgView::Entities,
            };
            app.kg_entity_state = ListState::default();
            let len = match app.kg_view {
                KgView::Entities => app.kg_entities.len(),
                KgView::Edges | KgView::Visual => app.kg_edges.len(),
            };
            if len > 0 {
                app.kg_entity_state.select(Some(0));
            }
        }
        KeyCode::Char('r') if app.tab == TraceTab::Graph => {
            app.reload_kg();
            app.ok("Knowledge graph reloaded");
        }
        KeyCode::Char('v') if app.tab == TraceTab::Graph => {
            app.kg_view = KgView::Visual;
        }
        KeyCode::Char('e') if app.tab == TraceTab::Graph => {
            app.kg_view = KgView::Entities;
        }
        KeyCode::Char('w') if app.tab == TraceTab::Graph => {
            app.kg_view = KgView::Edges;
        }

        _ => {}
    }
    false
}

