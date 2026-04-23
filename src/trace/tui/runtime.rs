pub fn run() {
    run_inner(TraceTab::Notes, storage::detect_current_branch());
}

pub fn run_kg(branch: Option<String>) {
    let b = branch.unwrap_or_else(storage::detect_current_branch);
    run_inner(TraceTab::Graph, b);
}

fn run_inner(initial_tab: TraceTab, kg_branch: String) {
    let _ = enable_raw_mode();
    let mut stdout = io::stdout();
    let _ = execute!(stdout, EnterAlternateScreen);
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = match Terminal::new(backend) {
        Ok(t) => t,
        Err(_) => {
            let _ = disable_raw_mode();
            return;
        }
    };

    let notes = storage::list_notes().unwrap_or_default();
    let packages = storage::package_risks().unwrap_or_default();
    let audit = load_audit_log();

    let mut list_state = ListState::default();
    if !notes.is_empty() {
        list_state.select(Some(0));
    }
    let mut packages_state = ListState::default();
    if !packages.is_empty() {
        packages_state.select(Some(0));
    }
    let cfg_init = storage::load_config().unwrap_or_default();
    let mut sources_state = ListState::default();
    if !cfg_init.sources.is_empty() {
        sources_state.select(Some(0));
    }

    let mut app = App {
        tab: initial_tab,
        notes,
        packages,
        list_state,
        sources_state,
        packages_state,
        audit_scroll: 0,
        mode: AppMode::Browse,
        status: None,
        audit,
        kg_entities: Vec::new(),
        kg_edges: Vec::new(),
        kg_entity_state: ListState::default(),
        kg_selected_entity: None,
        kg_view: KgView::Entities,
        kg_branch,
        kg_branches: Vec::new(),
        kg_branch_idx: 0,
    };

    if app.tab == TraceTab::Graph {
        app.reload_kg();
        app.reload_kg_branches();
    }

    let mut cfg = storage::load_config().unwrap_or_default();

    loop {
        if terminal.draw(|f| draw_ui(f, &mut app, &cfg)).is_err() {
            break;
        }

        match event::poll(std::time::Duration::from_millis(80)) {
            Ok(true) => {}
            _ => continue,
        }
        let Ok(Event::Key(key)) = event::read() else {
            continue;
        };

        let prev_tab = app.tab;
        let quit = match &app.mode {
            AppMode::Browse | AppMode::ViewDetail | AppMode::PackageDetail => {
                if key.kind != KeyEventKind::Press {
                    false
                } else {
                    handle_browse(&mut app, &mut cfg, key.code)
                }
            }
            AppMode::EditForm(_) => {
                if key.kind == KeyEventKind::Press {
                    handle_form(&mut app, key.code);
                }
                false
            }
            AppMode::DeleteConfirm(_) => {
                if key.kind == KeyEventKind::Press {
                    handle_delete_confirm(&mut app, key.code);
                }
                false
            }
            AppMode::SourceDeleteConfirm(_) => {
                if key.kind == KeyEventKind::Press {
                    handle_source_delete_confirm(&mut app, &mut cfg, key.code);
                }
                false
            }
            AppMode::KgEntityForm(_) => {
                if key.kind == KeyEventKind::Press {
                    handle_kg_entity_form(&mut app, key.code);
                }
                false
            }
            AppMode::KgEdgeForm(_) => {
                if key.kind == KeyEventKind::Press {
                    handle_kg_edge_form(&mut app, key.code);
                }
                false
            }
            AppMode::KgEntityDelete(_) => {
                if key.kind == KeyEventKind::Press {
                    handle_kg_entity_delete(&mut app, key.code);
                }
                false
            }
            AppMode::KgEdgeDelete(_) => {
                if key.kind == KeyEventKind::Press {
                    handle_kg_edge_delete(&mut app, key.code);
                }
                false
            }
            AppMode::KgBranchPicker => {
                if key.kind == KeyEventKind::Press {
                    handle_kg_branch_picker(&mut app, key.code);
                }
                false
            }
        };
        if quit {
            break;
        }
        if app.tab != prev_tab
            || (key.kind == KeyEventKind::Press
                && (key.code == KeyCode::Char('r')
                    || (app.tab == TraceTab::Sources
                        && matches!(key.code, KeyCode::Char('s') | KeyCode::Char('d')))))
        {
            cfg = storage::load_config().unwrap_or_default();
        }
    }

    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
    let _ = terminal.show_cursor();
}

// ─── Key handlers ─────────────────────────────────────────────────────────────
