fn execute_note(action: NoteAction) {
    match action {
        NoteAction::Add {
            id,
            title,
            body,
            layer,
            scope,
            target,
            author,
            actor,
            files,
            tags,
            related_pr,
        } => cmd_note_add(
            &id,
            &title,
            &body,
            &layer,
            &scope,
            &target,
            author.as_deref(),
            actor.as_deref(),
            &files,
            &tags,
            related_pr,
        ),
        NoteAction::Update {
            id,
            title,
            body,
            status,
        } => cmd_note_update(&id, title.as_deref(), body.as_deref(), status.as_deref()),
        NoteAction::Remove { id } => cmd_note_remove(&id),
        NoteAction::List => cmd_note_list(),
    }
}


#[allow(clippy::too_many_arguments)]
fn cmd_note_add(
    id: &str,
    title: &str,
    body: &str,
    layer: &str,
    scope: &str,
    target: &str,
    author: Option<&str>,
    actor: Option<&str>,
    files: &[String],
    tags: &[String],
    related_pr: Option<u64>,
) {
    let layer = match parse_layer(layer) {
        Ok(v) => v,
        Err(e) => return Logger::error(&e),
    };
    let scope = match parse_scope(scope) {
        Ok(v) => v,
        Err(e) => return Logger::error(&e),
    };
    let now = Utc::now().to_rfc3339();
    let note = TraceNote {
        id: id.to_string(),
        title: title.to_string(),
        body: body.to_string(),
        layer,
        scope,
        target: target.to_string(),
        files: files.to_vec(),
        tags: tags.to_vec(),
        related_pr,
        author: resolve_author(author),
        actor: actor.map(|s| s.to_string()),
        status: NoteStatus::Active,
        created_at: now.clone(),
        updated_at: now,
    };
    match storage::create_note(note) {
        Ok(()) => Logger::success(&format!("Saved note '{}'", id)),
        Err(e) => Logger::error(&e),
    }
}

fn resolve_author(author: Option<&str>) -> String {
    author
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(storage::configured_user)
        .or_else(storage::detect_user_name)
        .unwrap_or_else(|| "unknown".to_string())
}

fn cmd_note_update(id: &str, title: Option<&str>, body: Option<&str>, status: Option<&str>) {
    let status = match status {
        Some(value) => match parse_status(value) {
            Ok(v) => Some(v),
            Err(e) => return Logger::error(&e),
        },
        None => None,
    };
    match storage::update_note(id, title, body, status) {
        Ok(()) => Logger::success(&format!("Updated note '{}'", id)),
        Err(e) => Logger::error(&e),
    }
}

fn cmd_note_remove(id: &str) {
    match storage::delete_note(id) {
        Ok(()) => Logger::success(&format!("Removed note '{}'", id)),
        Err(e) => Logger::error(&e),
    }
}

fn cmd_note_list() {
    match storage::list_notes() {
        Ok(notes) => print_notes(&notes),
        Err(e) => Logger::error(&e),
    }
}

fn cmd_retrieve(
    layer: Option<&str>,
    scope: Option<&str>,
    target: Option<&str>,
    author: Option<&str>,
    file: Option<&str>,
    tag: Option<&str>,
) {
    let layer = match layer {
        Some(v) => match parse_layer(v) {
            Ok(v) => Some(v),
            Err(e) => return Logger::error(&e),
        },
        None => None,
    };
    let scope = match scope {
        Some(v) => match parse_scope(v) {
            Ok(v) => Some(v),
            Err(e) => return Logger::error(&e),
        },
        None => None,
    };
    match storage::retrieve_notes(layer, scope, target, author, file, tag) {
        Ok(notes) => print_notes(&notes),
        Err(e) => Logger::error(&e),
    }
}

fn cmd_sync(source: Option<&str>, direction: &str) {
    let direction = match parse_direction(direction) {
        Ok(v) => v,
        Err(e) => return Logger::error(&e),
    };
    let source = match storage::get_source(source) {
        Ok(v) => v,
        Err(e) => return Logger::error(&e),
    };

    let local_notes = match storage::list_notes() {
        Ok(v) => v,
        Err(e) => return Logger::error(&e),
    };
    let package_findings = storage::package_risks().unwrap_or_default();

    match direction {
        SyncDirection::Push => {
            let run = crate::trace::types::SyncRun {
                timestamp: Utc::now().to_rfc3339(),
                direction,
                source_id: Some(source.id.clone()),
                summary: format!("push {} notes", local_notes.len()),
            };
            match crate::trace::backend::push_all(&source, &local_notes, &package_findings, &run) {
                Ok(()) => {
                    let _ = storage::record_sync(direction, Some(&source.id), &run.summary);
                    Logger::success("Push sync completed");
                }
                Err(e) => Logger::error(&e),
            }
        }
        SyncDirection::Pull => match crate::trace::backend::pull_notes(&source) {
            Ok(notes) => match storage::merge_remote_notes(notes) {
                Ok(merged) => {
                    let summary = format!("pull merged {} notes", merged);
                    let _ = crate::trace::backend::record_sync(&source, direction, &summary);
                    let _ = storage::record_sync(direction, Some(&source.id), &summary);
                    Logger::success(&format!("Pull sync completed, merged {}", merged));
                }
                Err(e) => Logger::error(&e),
            },
            Err(e) => Logger::error(&e),
        },
        SyncDirection::Both => {
            let run = crate::trace::types::SyncRun {
                timestamp: Utc::now().to_rfc3339(),
                direction,
                source_id: Some(source.id.clone()),
                summary: format!("push {} notes", local_notes.len()),
            };
            if let Err(e) =
                crate::trace::backend::push_all(&source, &local_notes, &package_findings, &run)
            {
                return Logger::error(&e);
            }
            match crate::trace::backend::pull_notes(&source) {
                Ok(notes) => match storage::merge_remote_notes(notes) {
                    Ok(merged) => {
                        let summary = format!("push/pull merged {} notes", merged);
                        let _ = crate::trace::backend::record_sync(&source, direction, &summary);
                        let _ = storage::record_sync(direction, Some(&source.id), &summary);
                        Logger::success(&format!(
                            "Bidirectional sync completed, merged {}",
                            merged
                        ));
                    }
                    Err(e) => Logger::error(&e),
                },
                Err(e) => Logger::error(&e),
            }
        }
    }
}

fn cmd_compact() {
    match storage::compact_notes() {
        Ok((kept, archived)) => {
            Logger::success("Trace compaction finished");
            Logger::detail("Kept:", &kept.to_string());
            Logger::detail("Archived:", &archived.to_string());
        }
        Err(e) => Logger::error(&e),
    }
}

fn cmd_schema(backend: &str) {
    match backend.to_ascii_lowercase().as_str() {
        "sql" => println!("{}", storage::supported_schema_sql()),
        "redis" => println!("{}", storage::supported_schema_redis()),
        other => Logger::error(&format!(
            "Unsupported backend '{}'. Use sql | redis.",
            other
        )),
    }
}

fn print_notes(notes: &[TraceNote]) {
    if notes.is_empty() {
        Logger::info("No notes matched.");
        return;
    }
    println!(
        "  {:<16} {:<10} {:<10} {:<10} {:<16} {}",
        "ID", "LAYER", "SCOPE", "STATUS", "AUTHOR", "TITLE"
    );
    println!("  {}", "-".repeat(90));
    for note in notes {
        println!(
            "  {:<16} {:<10} {:<10} {:<10} {:<16} {}",
            note.id,
            note.layer.as_str(),
            note.scope.as_str(),
            note.status.as_str(),
            note.author,
            note.title
        );
    }
}

fn parse_layer(value: &str) -> Result<TraceLayer, String> {
    value
        .parse()
        .map_err(|_| format!("Invalid layer '{}'. Use canonical | team | user.", value))
}

fn parse_scope(value: &str) -> Result<TraceScope, String> {
    value.parse().map_err(|_| {
        format!(
            "Invalid scope '{}'. Use repo | branch | pr | file | user | session | package.",
            value
        )
    })
}

fn parse_status(value: &str) -> Result<NoteStatus, String> {
    value
        .parse()
        .map_err(|_| format!("Invalid status '{}'. Use active | stale | archived.", value))
}

fn parse_direction(value: &str) -> Result<SyncDirection, String> {
    value
        .parse()
        .map_err(|_| format!("Invalid direction '{}'. Use pull | push | both.", value))
}

// ─── Knowledge Graph commands ───────────────────────────────────────────────

