use crate::loom::cli::{LoomAction, NoteAction, SourceAction};
use crate::loom::storage;
use crate::loom::types::{
    LoomLayer, LoomNote, LoomScope, LoomSource, NoteStatus, SourceKind, SyncDirection,
};
use crate::tui::logger::Logger;
use chrono::Utc;

pub fn execute(action: LoomAction) {
    match action {
        LoomAction::Overview => Logger::loom_overview(),
        LoomAction::Init { repo, owner, user } => {
            cmd_init(repo.as_deref(), owner.as_deref(), user.as_deref())
        }
        LoomAction::Source { action } => execute_source(action),
        LoomAction::Note { action } => execute_note(action),
        LoomAction::Retrieve {
            layer,
            scope,
            target,
            author,
            file,
            tag,
        } => cmd_retrieve(
            layer.as_deref(),
            scope.as_deref(),
            target.as_deref(),
            author.as_deref(),
            file.as_deref(),
            tag.as_deref(),
        ),
        LoomAction::Sync { source, direction } => cmd_sync(source.as_deref(), &direction),
        LoomAction::Compact => cmd_compact(),
        LoomAction::Schema { backend } => cmd_schema(&backend),
        LoomAction::Tui => crate::loom::tui::run(),
    }
}

fn execute_source(action: SourceAction) {
    match action {
        SourceAction::AddRedis {
            id,
            url,
            namespace,
            notes,
            user,
            default,
        } => cmd_source_add_redis(
            &id,
            &url,
            &namespace,
            notes.as_deref(),
            user.as_deref(),
            default,
        ),
        SourceAction::AddSql {
            id,
            engine,
            url,
            database,
            username,
            password_env,
            notes,
            user,
            default,
        } => cmd_source_add_sql(
            &id,
            &engine,
            &url,
            database.as_deref(),
            username.as_deref(),
            password_env.as_deref(),
            notes.as_deref(),
            user.as_deref(),
            default,
        ),
        SourceAction::List => cmd_source_list(),
        SourceAction::Remove { id } => cmd_source_remove(&id),
        SourceAction::Default { id } => cmd_source_default(&id),
    }
}

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

fn cmd_init(repo: Option<&str>, owner: Option<&str>, user: Option<&str>) {
    let repo_name = repo.map(|s| s.to_string()).unwrap_or_else(storage::detect_repo_name);
    let owner_name = owner.unwrap_or("team");
    let detected_user = storage::detect_user_name();
    let default_user = user.or(detected_user.as_deref());
    match storage::init_config(&repo_name, owner_name, default_user) {
        Ok(()) => {
            Logger::success(&format!("Initialized Loom for '{}'", repo_name));
            Logger::detail("Owner:", owner_name);
            if let Some(user) = default_user {
                Logger::detail("Default user:", user);
            }
            Logger::detail("Path:", &storage::loom_dir().display().to_string());
        }
        Err(e) => Logger::error(&e),
    }
}

fn cmd_source_add_redis(
    id: &str,
    url: &str,
    namespace: &str,
    notes: Option<&str>,
    user: Option<&str>,
    make_default: bool,
) {
    let source = LoomSource {
        id: id.to_string(), kind: SourceKind::Redis, url: url.to_string(), enabled: true,
        owner_user: user.map(|value| value.to_string()).or_else(storage::configured_user),
        database: None, namespace: Some(namespace.to_string()), username: None,
        password_env: None, notes: notes.map(|s| s.to_string()),
    };
    if let Err(e) = crate::loom::backend::validate_and_prepare(&source) {
        return Logger::error(&format!("Redis validation failed: {}", e));
    }
    match storage::add_source(source, make_default) {
        Ok(()) => {
            Logger::success(&format!("Added Redis source '{}'", id));
            Logger::raw_dim("Benefit: low-latency lookups, live presence, and fast overlap detection.");
        }
        Err(e) => Logger::error(&e),
    }
}

fn cmd_source_add_sql(
    id: &str,
    engine: &str,
    url: &str,
    database: Option<&str>,
    username: Option<&str>,
    password_env: Option<&str>,
    notes: Option<&str>,
    user: Option<&str>,
    make_default: bool,
) {
    let kind = match engine.to_ascii_lowercase().as_str() {
        "postgres" | "postgresql" => SourceKind::Postgres,
        "mysql" => SourceKind::Mysql,
        "sqlite" => SourceKind::Sqlite,
        other => { Logger::error(&format!("Unsupported SQL engine '{}'. Use postgres | mysql | sqlite.", other)); return; }
    };
    let source = LoomSource {
        id: id.to_string(), kind, url: url.to_string(), enabled: true,
        owner_user: user.map(|value| value.to_string()).or_else(storage::configured_user),
        database: database.map(|s| s.to_string()), namespace: None,
        username: username.map(|s| s.to_string()), password_env: password_env.map(|s| s.to_string()),
        notes: notes.map(|s| s.to_string()),
    };
    if let Err(e) = crate::loom::backend::validate_and_prepare(&source) {
        return Logger::error(&format!("SQL validation failed: {}", e));
    }
    match storage::add_source(source, make_default) {
        Ok(()) => { Logger::success(&format!("Added {} source '{}'", kind.as_str(), id)); Logger::raw_dim("Benefit: durable structured storage, better filtering, reporting, and canonical memory."); }
        Err(e) => Logger::error(&e),
    }
}

fn cmd_source_list() {
    match storage::load_config() {
        Ok(cfg) => {
            if cfg.sources.is_empty() {
                Logger::info("No Loom backends configured.");
                return;
            }
            println!("  {:<18} {:<10} {:<8} {:<16} {}", "ID", "KIND", "DEFAULT", "USER", "URL");
            println!("  {}", "-".repeat(80));
            for source in cfg.sources {
                let is_default = cfg.default_source.as_deref() == Some(source.id.as_str());
                println!(
                    "  {:<18} {:<10} {:<8} {:<16} {}",
                    source.id,
                    source.kind.as_str(),
                    if is_default { "yes" } else { "no" },
                    source.owner_user.clone().unwrap_or_else(|| "-".to_string()),
                    source.url
                );
            }
        }
        Err(e) => Logger::error(&e),
    }
}

fn cmd_source_remove(id: &str) {
    match storage::remove_source(id) {
        Ok(()) => Logger::success(&format!("Removed source '{}'", id)),
        Err(e) => Logger::error(&e),
    }
}

fn cmd_source_default(id: &str) {
    match storage::set_default_source(id) {
        Ok(()) => Logger::success(&format!("Default source set to '{}'", id)),
        Err(e) => Logger::error(&e),
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
    let note = LoomNote {
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
            let run = crate::loom::types::SyncRun {
                timestamp: Utc::now().to_rfc3339(),
                direction,
                source_id: Some(source.id.clone()),
                summary: format!("push {} notes", local_notes.len()),
            };
            match crate::loom::backend::push_all(&source, &local_notes, &package_findings, &run) {
                Ok(()) => {
                    let _ = storage::record_sync(direction, Some(&source.id), &run.summary);
                    Logger::success("Push sync completed");
                }
                Err(e) => Logger::error(&e),
            }
        }
        SyncDirection::Pull => match crate::loom::backend::pull_notes(&source) {
            Ok(notes) => match storage::merge_remote_notes(notes) {
                Ok(merged) => {
                    let summary = format!("pull merged {} notes", merged);
                    let _ = crate::loom::backend::record_sync(&source, direction, &summary);
                    let _ = storage::record_sync(direction, Some(&source.id), &summary);
                    Logger::success(&format!("Pull sync completed, merged {}", merged));
                }
                Err(e) => Logger::error(&e),
            },
            Err(e) => Logger::error(&e),
        },
        SyncDirection::Both => {
            let run = crate::loom::types::SyncRun {
                timestamp: Utc::now().to_rfc3339(),
                direction,
                source_id: Some(source.id.clone()),
                summary: format!("push {} notes", local_notes.len()),
            };
            if let Err(e) = crate::loom::backend::push_all(&source, &local_notes, &package_findings, &run) {
                return Logger::error(&e);
            }
            match crate::loom::backend::pull_notes(&source) {
                Ok(notes) => match storage::merge_remote_notes(notes) {
                    Ok(merged) => {
                        let summary = format!("push/pull merged {} notes", merged);
                        let _ = crate::loom::backend::record_sync(&source, direction, &summary);
                        let _ = storage::record_sync(direction, Some(&source.id), &summary);
                        Logger::success(&format!("Bidirectional sync completed, merged {}", merged));
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
            Logger::success("Loom compaction finished");
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
        other => Logger::error(&format!("Unsupported backend '{}'. Use sql | redis.", other)),
    }
}

fn print_notes(notes: &[LoomNote]) {
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

fn parse_layer(value: &str) -> Result<LoomLayer, String> {
    value.parse().map_err(|_| format!("Invalid layer '{}'. Use canonical | team | user.", value))
}

fn parse_scope(value: &str) -> Result<LoomScope, String> {
    value.parse().map_err(|_| format!("Invalid scope '{}'. Use repo | branch | pr | file | user | session | package.", value))
}

fn parse_status(value: &str) -> Result<NoteStatus, String> {
    value.parse().map_err(|_| format!("Invalid status '{}'. Use active | stale | archived.", value))
}

fn parse_direction(value: &str) -> Result<SyncDirection, String> {
    value.parse().map_err(|_| format!("Invalid direction '{}'. Use pull | push | both.", value))
}
