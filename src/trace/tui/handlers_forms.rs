fn handle_form(app: &mut App, code: KeyCode) {
    let AppMode::EditForm(ref mut form) = app.mode else {
        return;
    };
    match code {
        KeyCode::Esc => {
            app.mode = AppMode::Browse;
        }
        KeyCode::Tab => {
            let next = form.active_field.next();
            if form.is_edit && next == EditField::Id {
                form.active_field = next.next(); // skip ID when editing
            } else {
                form.active_field = next;
            }
        }
        KeyCode::BackTab => {
            let prev = form.active_field.prev();
            if form.is_edit && prev == EditField::Id {
                form.active_field = prev.prev();
            } else {
                form.active_field = prev;
            }
        }
        KeyCode::Backspace => {
            let field = form.active_field;
            form.get_field_mut(field).pop();
        }
        KeyCode::Char(c) => {
            let field = form.active_field;
            form.get_field_mut(field).push(c);
        }
        KeyCode::Enter => {
            // Extract values before dropping borrow
            let is_edit = form.is_edit;
            let id = form.id.trim().to_string();
            let title = form.title.trim().to_string();
            let body = form.body.trim().to_string();
            let layer_s = form.layer.trim().to_string();
            let scope_s = form.scope.trim().to_string();
            let target = form.target.trim().to_string();
            let author_s = form.author.trim().to_string();
            let tags_s = form.tags.trim().to_string();

            if id.is_empty() && !is_edit {
                app.err("ID is required");
                return;
            }
            if title.is_empty() {
                app.err("Title is required");
                return;
            }
            let layer: TraceLayer = match layer_s.parse() {
                Ok(v) => v,
                Err(_e) => {
                    app.err(format!(
                        "Invalid layer '{}'. Use canonical | team | user",
                        layer_s
                    ));
                    return;
                }
            };
            let scope: TraceScope = match scope_s.parse() {
                Ok(v) => v,
                Err(_) => {
                    app.err(format!("Invalid scope '{}'. Use repo | branch | pr | file | user | session | package", scope_s));
                    return;
                }
            };
            let tags: Vec<String> = if tags_s.is_empty() {
                Vec::new()
            } else {
                tags_s
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            };
            let author = if author_s.is_empty() {
                storage::configured_user()
                    .or_else(storage::detect_user_name)
                    .unwrap_or_else(|| "unknown".to_string())
            } else {
                author_s
            };

            let (action, summary) = if is_edit {
                match storage::update_note_details(
                    &id,
                    Some(&title),
                    Some(&body),
                    None,
                    Some(layer),
                    Some(scope),
                    Some(&target),
                    Some(&author),
                    Some(tags.clone()),
                ) {
                    Ok(()) => (
                        "edit",
                        format!(
                            "title={} layer={} scope={} tags={}",
                            title,
                            layer_s,
                            scope_s,
                            tags.len()
                        ),
                    ),
                    Err(e) => {
                        app.err(e);
                        return;
                    }
                }
            } else {
                let note = TraceNote {
                    id: id.clone(),
                    title: title.clone(),
                    body: body.clone(),
                    layer,
                    scope,
                    target: target.clone(),
                    files: Vec::new(),
                    tags,
                    related_pr: None,
                    author: author.clone(),
                    actor: None,
                    status: NoteStatus::Active,
                    created_at: String::new(),
                    updated_at: String::new(),
                };
                match storage::create_note(note) {
                    Ok(()) => (
                        "create",
                        format!("title={} layer={} scope={}", title, layer_s, scope_s),
                    ),
                    Err(e) => {
                        app.err(e);
                        return;
                    }
                }
            };

            append_audit(AuditEntry {
                timestamp: Utc::now().to_rfc3339(),
                action: action.to_string(),
                note_id: id.clone(),
                author,
                summary,
            });

            app.reload_notes();
            if let Some(pos) = app.notes.iter().position(|n| n.id == id) {
                app.list_state.select(Some(pos));
            }
            app.ok(format!(
                "Note '{}' {}",
                id,
                if is_edit { "updated" } else { "created" }
            ));
            app.mode = AppMode::Browse;
        }
        _ => {}
    }
}

fn handle_delete_confirm(app: &mut App, code: KeyCode) {
    let id = match &app.mode {
        AppMode::DeleteConfirm(id) => id.clone(),
        _ => return,
    };
    match code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            let author = storage::configured_user()
                .or_else(storage::detect_user_name)
                .unwrap_or_else(|| "unknown".to_string());
            match storage::delete_note(&id) {
                Ok(()) => {
                    append_audit(AuditEntry {
                        timestamp: Utc::now().to_rfc3339(),
                        action: "delete".to_string(),
                        note_id: id.clone(),
                        author,
                        summary: format!("deleted note '{}'", id),
                    });
                    app.ok(format!("Note '{}' deleted", id));
                    app.reload_notes();
                    app.clamp_selection();
                }
                Err(e) => app.err(e),
            }
            app.mode = AppMode::Browse;
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.mode = AppMode::Browse;
        }
        _ => {}
    }
}

fn handle_source_delete_confirm(
    app: &mut App,
    cfg: &mut crate::trace::types::TraceConfig,
    code: KeyCode,
) {
    let id = match &app.mode {
        AppMode::SourceDeleteConfirm(id) => id.clone(),
        _ => return,
    };
    match code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            match storage::remove_source(&id) {
                Ok(()) => {
                    *cfg = storage::load_config().unwrap_or_default();
                    app.ok(format!("Source '{}' removed", id));
                    app.reload_sources(cfg);
                }
                Err(e) => app.err(e),
            }
            app.mode = AppMode::Browse;
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.mode = AppMode::Browse;
        }
        _ => {}
    }
}

fn handle_kg_entity_form(app: &mut App, code: KeyCode) {
    let AppMode::KgEntityForm(ref mut form) = app.mode else {
        return;
    };
    match code {
        KeyCode::Esc => {
            app.mode = AppMode::Browse;
        }
        KeyCode::Tab => {
            form.active_field = form.active_field.next();
        }
        KeyCode::BackTab => {
            form.active_field = form.active_field.prev();
        }
        KeyCode::Backspace => {
            let field = form.active_field;
            form.get_field_mut(field).pop();
        }
        KeyCode::Char(c) => {
            let field = form.active_field;
            form.get_field_mut(field).push(c);
        }
        KeyCode::Enter => {
            let is_edit = form.is_edit;
            let original_id = form.original_id.clone();
            let name = form.name.trim().to_string();
            let kind_s = form.kind.trim().to_string();
            let branch = form.branch.trim().to_string();
            let meta_s = form.meta.trim().to_string();

            if name.is_empty() {
                app.err("Name is required");
                return;
            }
            let kind = match EntityKind::from_str(&kind_s) {
                Ok(v) => v,
                Err(e) => {
                    app.err(e);
                    return;
                }
            };
            let mut metadata = std::collections::HashMap::new();
            if !meta_s.is_empty() {
                for pair in meta_s.split(',') {
                    let pair = pair.trim();
                    if let Some((k, v)) = pair.split_once('=') {
                        metadata.insert(k.trim().to_string(), v.trim().to_string());
                    }
                }
            }
            let id = name
                .chars()
                .map(|c| {
                    if c.is_alphanumeric() {
                        c.to_ascii_lowercase()
                    } else {
                        '-'
                    }
                })
                .collect::<String>();
            let now = Utc::now().to_rfc3339();
            let entity = KgEntity {
                id: id.clone(),
                kind,
                name: name.clone(),
                metadata,
                branch: branch.clone(),
                created_at: now.clone(),
                updated_at: now,
            };

            if is_edit {
                let _ = storage::delete_entity(&original_id);
            }
            match storage::create_entity(entity) {
                Ok(()) => {
                    app.ok(format!(
                        "Entity '{}' {}",
                        name,
                        if is_edit { "updated" } else { "created" }
                    ));
                    if app.kg_branch == "*" {
                        app.reload_kg_all();
                    } else {
                        app.reload_kg();
                    }
                    app.reload_kg_branches();
                }
                Err(e) => app.err(e),
            }
            app.mode = AppMode::Browse;
        }
        _ => {}
    }
}

fn handle_kg_edge_form(app: &mut App, code: KeyCode) {
    let AppMode::KgEdgeForm(ref mut form) = app.mode else {
        return;
    };
    match code {
        KeyCode::Esc => {
            app.mode = AppMode::Browse;
        }
        KeyCode::Tab => {
            form.active_field = form.active_field.next();
        }
        KeyCode::BackTab => {
            form.active_field = form.active_field.prev();
        }
        KeyCode::Backspace => {
            let field = form.active_field;
            form.get_field_mut(field).pop();
        }
        KeyCode::Char(c) => {
            let field = form.active_field;
            form.get_field_mut(field).push(c);
        }
        KeyCode::Enter => {
            let is_edit = form.is_edit;
            let original_id = form.original_id.clone();
            let from_s = form.from.trim().to_string();
            let to_s = form.to.trim().to_string();
            let rel_s = form.relation.trim().to_string();
            let weight_s = form.weight.trim().to_string();
            let branch = form.branch.trim().to_string();
            let evidence = form.evidence.trim().to_string();

            if from_s.is_empty() || to_s.is_empty() {
                app.err("From and To are required");
                return;
            }
            let relation = match RelationType::from_str(&rel_s) {
                Ok(v) => v,
                Err(e) => {
                    app.err(e);
                    return;
                }
            };
            let weight: f64 = match weight_s.parse() {
                Ok(v) => v,
                Err(_) => {
                    app.err("Invalid weight (use a number like 0.5)");
                    return;
                }
            };

            // Resolve from/to — use the string as-is (it may be an entity ID or name)
            let source = match storage::find_entity_by_name(&from_s, &branch) {
                Ok(Some(e)) => e.id,
                _ => from_s.clone(),
            };
            let target = match storage::find_entity_by_name(&to_s, &branch) {
                Ok(Some(e)) => e.id,
                _ => to_s.clone(),
            };

            let edge_id = format!("{}-{}-{}", source, relation.as_str(), target);
            let now = Utc::now().to_rfc3339();
            let edge = KgEdge {
                id: edge_id,
                source,
                target,
                relation,
                weight,
                branch: branch.clone(),
                evidence,
                created_at: now,
            };

            if is_edit {
                let _ = storage::delete_edge(&original_id);
            }
            match storage::create_edge(edge) {
                Ok(()) => {
                    app.ok(format!(
                        "Edge {}",
                        if is_edit { "updated" } else { "created" }
                    ));
                    if app.kg_branch == "*" {
                        app.reload_kg_all();
                    } else {
                        app.reload_kg();
                    }
                }
                Err(e) => app.err(e),
            }
            app.mode = AppMode::Browse;
        }
        _ => {}
    }
}

fn handle_kg_entity_delete(app: &mut App, code: KeyCode) {
    let id = match &app.mode {
        AppMode::KgEntityDelete(id) => id.clone(),
        _ => return,
    };
    match code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            match storage::delete_entity(&id) {
                Ok(()) => {
                    app.ok(format!("Entity '{}' deleted", id));
                    if app.kg_branch == "*" {
                        app.reload_kg_all();
                    } else {
                        app.reload_kg();
                    }
                    app.reload_kg_branches();
                    // clamp selection
                    let len = app.kg_entities.len();
                    match app.kg_entity_state.selected() {
                        Some(i) if i >= len && len > 0 => app.kg_entity_state.select(Some(len - 1)),
                        Some(_) if len == 0 => app.kg_entity_state.select(None),
                        None if len > 0 => app.kg_entity_state.select(Some(0)),
                        _ => {}
                    }
                }
                Err(e) => app.err(e),
            }
            app.mode = AppMode::Browse;
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.mode = AppMode::Browse;
        }
        _ => {}
    }
}

fn handle_kg_edge_delete(app: &mut App, code: KeyCode) {
    let id = match &app.mode {
        AppMode::KgEdgeDelete(id) => id.clone(),
        _ => return,
    };
    match code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            match storage::delete_edge(&id) {
                Ok(()) => {
                    app.ok(format!("Edge '{}' deleted", id));
                    if app.kg_branch == "*" {
                        app.reload_kg_all();
                    } else {
                        app.reload_kg();
                    }
                    let len = app.kg_edges.len();
                    match app.kg_entity_state.selected() {
                        Some(i) if i >= len && len > 0 => app.kg_entity_state.select(Some(len - 1)),
                        Some(_) if len == 0 => app.kg_entity_state.select(None),
                        None if len > 0 => app.kg_entity_state.select(Some(0)),
                        _ => {}
                    }
                }
                Err(e) => app.err(e),
            }
            app.mode = AppMode::Browse;
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.mode = AppMode::Browse;
        }
        _ => {}
    }
}

fn handle_kg_branch_picker(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Down | KeyCode::Char('j') => {
            if !app.kg_branches.is_empty() {
                app.kg_branch_idx = (app.kg_branch_idx + 1) % app.kg_branches.len();
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if !app.kg_branches.is_empty() {
                app.kg_branch_idx =
                    (app.kg_branch_idx + app.kg_branches.len() - 1) % app.kg_branches.len();
            }
        }
        KeyCode::Enter => {
            if let Some(branch) = app.kg_branches.get(app.kg_branch_idx).cloned() {
                app.kg_branch = branch.clone();
                app.reload_kg();
                app.ok(format!("Switched to branch '{}'", branch));
            }
            app.mode = AppMode::Browse;
        }
        KeyCode::Char('a') => {
            app.kg_branch = "*".to_string();
            app.reload_kg_all();
            app.ok("Showing all branches");
            app.mode = AppMode::Browse;
        }
        KeyCode::Esc => {
            app.mode = AppMode::Browse;
        }
        _ => {}
    }
}

// ─── Drawing ──────────────────────────────────────────────────────────────────

