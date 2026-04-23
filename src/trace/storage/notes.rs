fn note_path(layer: TraceLayer, id: &str) -> PathBuf {
    trace_dir()
        .join("notes")
        .join(layer.as_str())
        .join(format!("{}.json", storage_key(id)))
}

fn legacy_note_path(layer: TraceLayer, id: &str) -> PathBuf {
    trace_dir()
        .join("notes")
        .join(layer.as_str())
        .join(format!("{}.json", sanitize(id)))
}


pub fn create_note(mut note: TraceNote) -> Result<(), String> {
    ensure_layout()?;
    let now = Utc::now().to_rfc3339();
    if note.created_at.is_empty() {
        note.created_at = now.clone();
    }
    note.updated_at = now;
    let content = serde_json::to_string_pretty(&note).map_err(|e| e.to_string())?;
    let path = note_path(note.layer, &note.id);
    fs::write(&path, content).map_err(|e| e.to_string())?;
    remove_legacy_alias(&path, &legacy_note_path(note.layer, &note.id))
}

pub fn update_note(
    id: &str,
    title: Option<&str>,
    body: Option<&str>,
    status: Option<NoteStatus>,
) -> Result<(), String> {
    update_note_details(id, title, body, status, None, None, None, None, None)
}

pub fn update_note_details(
    id: &str,
    title: Option<&str>,
    body: Option<&str>,
    status: Option<NoteStatus>,
    layer: Option<TraceLayer>,
    scope: Option<TraceScope>,
    target: Option<&str>,
    author: Option<&str>,
    tags: Option<Vec<String>>,
) -> Result<(), String> {
    let note = load_note(id)?.ok_or_else(|| format!("note '{}' not found", id))?;
    let original_layer = note.layer;
    let mut next = note;
    if let Some(title) = title {
        next.title = title.to_string();
    }
    if let Some(body) = body {
        next.body = body.to_string();
    }
    if let Some(status) = status {
        next.status = status;
    }
    if let Some(layer) = layer {
        next.layer = layer;
    }
    if let Some(scope) = scope {
        next.scope = scope;
    }
    if let Some(target) = target {
        next.target = target.to_string();
    }
    if let Some(author) = author {
        next.author = author.to_string();
    }
    if let Some(tags) = tags {
        next.tags = tags;
    }

    create_note(next.clone())?;

    let old_paths = if original_layer == next.layer {
        vec![legacy_note_path(original_layer, id)]
    } else {
        vec![
            note_path(original_layer, id),
            legacy_note_path(original_layer, id),
        ]
    };
    for old_path in old_paths {
        match fs::remove_file(old_path) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.to_string()),
        }
    }

    Ok(())
}

pub fn delete_note(id: &str) -> Result<(), String> {
    for layer in [TraceLayer::Canonical, TraceLayer::Team, TraceLayer::User] {
        let mut deleted = false;
        for path in [note_path(layer, id), legacy_note_path(layer, id)] {
            match fs::remove_file(path) {
                Ok(()) => deleted = true,
                Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                Err(e) => return Err(e.to_string()),
            }
        }
        if deleted {
            return Ok(());
        }
    }
    Err(format!("note '{}' not found", id))
}

pub fn load_note(id: &str) -> Result<Option<TraceNote>, String> {
    for layer in [TraceLayer::Canonical, TraceLayer::Team, TraceLayer::User] {
        for path in [note_path(layer, id), legacy_note_path(layer, id)] {
            match fs::read_to_string(path) {
                Ok(content) => {
                    let note = serde_json::from_str(&content).map_err(|e| e.to_string())?;
                    return Ok(Some(note));
                }
                Err(e) if e.kind() == io::ErrorKind::NotFound => continue,
                Err(e) => return Err(e.to_string()),
            }
        }
    }
    Ok(None)
}

pub fn list_notes() -> Result<Vec<TraceNote>, String> {
    ensure_layout()?;
    let mut notes = Vec::new();
    for layer in [TraceLayer::Canonical, TraceLayer::Team, TraceLayer::User] {
        let dir = trace_dir().join("notes").join(layer.as_str());
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(e) if e.kind() == io::ErrorKind::NotFound => continue,
            Err(e) => return Err(e.to_string()),
        };
        for entry in entries {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
            if let Ok(note) = serde_json::from_str::<TraceNote>(&content) {
                notes.push(note);
            }
        }
    }
    notes.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(notes)
}

pub fn retrieve_notes(
    layer: Option<TraceLayer>,
    scope: Option<TraceScope>,
    target: Option<&str>,
    author: Option<&str>,
    file: Option<&str>,
    tag: Option<&str>,
) -> Result<Vec<TraceNote>, String> {
    let mut notes = list_notes()?;
    notes.retain(|n| {
        layer.map(|v| n.layer == v).unwrap_or(true)
            && scope.map(|v| n.scope == v).unwrap_or(true)
            && target.map(|v| n.target.contains(v)).unwrap_or(true)
            && author
                .map(|v| n.author.eq_ignore_ascii_case(v))
                .unwrap_or(true)
            && file
                .map(|v| n.files.iter().any(|f| f.contains(v)))
                .unwrap_or(true)
            && tag
                .map(|v| n.tags.iter().any(|t| t.eq_ignore_ascii_case(v)))
                .unwrap_or(true)
    });
    Ok(notes)
}

pub fn append_sync_run(run: SyncRun) -> Result<(), String> {
    ensure_layout()?;
    let mut state = match fs::read_to_string(sync_state_path()) {
        Ok(raw) => serde_json::from_str::<SyncState>(&raw).unwrap_or_default(),
        Err(e) if e.kind() == io::ErrorKind::NotFound => SyncState::default(),
        Err(e) => return Err(e.to_string()),
    };
    state.runs.push(run);
    let content = serde_json::to_string_pretty(&state).map_err(|e| e.to_string())?;
    fs::write(sync_state_path(), content).map_err(|e| e.to_string())
}

pub fn record_sync(
    direction: SyncDirection,
    source_id: Option<&str>,
    summary: &str,
) -> Result<(), String> {
    append_sync_run(SyncRun {
        timestamp: Utc::now().to_rfc3339(),
        direction,
        source_id: source_id.map(|s| s.to_string()),
        summary: summary.to_string(),
    })
}

pub fn compact_notes() -> Result<(usize, usize), String> {
    let notes = list_notes()?;
    let mut archived = 0usize;
    let mut kept = 0usize;
    for note in notes {
        let should_archive = note.status == NoteStatus::Stale
            || (note.scope == TraceScope::Session && note.layer != TraceLayer::Canonical);
        if should_archive {
            let _ = update_note(&note.id, None, None, Some(NoteStatus::Archived));
            archived += 1;
        } else {
            kept += 1;
        }
    }
    Ok((kept, archived))
}


pub fn merge_remote_notes(remote: Vec<TraceNote>) -> Result<usize, String> {
    let mut merged = 0usize;
    for note in remote {
        let existing = load_note(&note.id)?;
        let should_write = existing
            .map(|local| note.updated_at > local.updated_at)
            .unwrap_or(true);
        if should_write {
            create_note(note)?;
            merged += 1;
        }
    }
    Ok(merged)
}

// ─── Knowledge Graph local storage ──────────────────────────────────────────

