use crate::loom::types::{
    LoomConfig, LoomLayer, LoomNote, LoomScope, LoomSource, NoteStatus, PackageRisk, SyncDirection,
    SyncRun, SyncState,
};
use crate::{engine, loom::types::SourceKind};
use chrono::Utc;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn normalize_user(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() { None } else { Some(t.to_string()) }
}

pub fn loom_dir() -> PathBuf {
    PathBuf::from(".infynon").join("loom")
}

pub fn ensure_layout() -> Result<(), String> {
    for dir in [
        loom_dir(),
        loom_dir().join("notes"),
        loom_dir().join("notes").join("canonical"),
        loom_dir().join("notes").join("team"),
        loom_dir().join("notes").join("user"),
        loom_dir().join("state"),
    ] {
        fs::create_dir_all(&dir).map_err(|e| format!("failed to create {}: {}", dir.display(), e))?;
    }
    Ok(())
}

pub fn config_path() -> PathBuf {
    loom_dir().join("config.toml")
}

pub fn sync_state_path() -> PathBuf {
    loom_dir().join("state").join("sync.json")
}

pub fn init_config(repo_name: &str, owner: &str, default_user: Option<&str>) -> Result<(), String> {
    ensure_layout()?;
    let cfg = LoomConfig {
        repo_name: repo_name.to_string(),
        owner: owner.to_string(),
        default_user: default_user.and_then(normalize_user),
        default_source: None,
        sources: Vec::new(),
    };
    save_config(&cfg)
}

pub fn load_config() -> Result<LoomConfig, String> {
    match fs::read_to_string(config_path()) {
        Ok(content) => toml::from_str(&content).map_err(|e| format!("invalid loom config: {}", e)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(LoomConfig::default()),
        Err(e) => Err(e.to_string()),
    }
}

pub fn save_config(cfg: &LoomConfig) -> Result<(), String> {
    ensure_layout()?;
    let content = toml::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    fs::write(config_path(), content).map_err(|e| e.to_string())
}

pub fn add_source(source: LoomSource, make_default: bool) -> Result<(), String> {
    let mut cfg = load_config()?;
    if cfg.sources.iter().any(|s| s.id == source.id) {
        return Err(format!("source '{}' already exists", source.id));
    }
    if make_default {
        cfg.default_source = Some(source.id.clone());
    }
    cfg.sources.push(source);
    save_config(&cfg)
}

pub fn configured_user() -> Option<String> {
    load_config().ok().and_then(|cfg| cfg.default_user.and_then(|v| normalize_user(&v)))
}

pub fn get_source(id: Option<&str>) -> Result<LoomSource, String> {
    let cfg = load_config()?;
    let wanted = match id {
        Some(id) => id.to_string(),
        None => cfg
            .default_source
            .clone()
            .ok_or_else(|| "No default Loom source configured.".to_string())?,
    };
    cfg.sources
        .into_iter()
        .find(|s| s.id == wanted)
        .ok_or_else(|| format!("source '{}' not found", wanted))
}

pub fn remove_source(id: &str) -> Result<(), String> {
    let mut cfg = load_config()?;
    let before = cfg.sources.len();
    cfg.sources.retain(|s| s.id != id);
    if before == cfg.sources.len() {
        return Err(format!("source '{}' not found", id));
    }
    if cfg.default_source.as_deref() == Some(id) {
        cfg.default_source = cfg.sources.first().map(|s| s.id.clone());
    }
    save_config(&cfg)
}

pub fn set_default_source(id: &str) -> Result<(), String> {
    let mut cfg = load_config()?;
    if !cfg.sources.iter().any(|s| s.id == id) {
        return Err(format!("source '{}' not found", id));
    }
    cfg.default_source = Some(id.to_string());
    save_config(&cfg)
}

fn note_path(layer: LoomLayer, id: &str) -> PathBuf {
    loom_dir()
        .join("notes")
        .join(layer.as_str())
        .join(format!("{}.json", sanitize(id)))
}

fn sanitize(input: &str) -> String {
    input.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect()
}

pub fn create_note(mut note: LoomNote) -> Result<(), String> {
    ensure_layout()?;
    let now = Utc::now().to_rfc3339();
    if note.created_at.is_empty() {
        note.created_at = now.clone();
    }
    note.updated_at = now;
    let content = serde_json::to_string_pretty(&note).map_err(|e| e.to_string())?;
    fs::write(note_path(note.layer, &note.id), content).map_err(|e| e.to_string())
}

pub fn update_note(
    id: &str,
    title: Option<&str>,
    body: Option<&str>,
    status: Option<NoteStatus>,
) -> Result<(), String> {
    let note = load_note(id)?.ok_or_else(|| format!("note '{}' not found", id))?;
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
    next.updated_at = Utc::now().to_rfc3339();
    create_note(next)
}

pub fn delete_note(id: &str) -> Result<(), String> {
    for layer in [LoomLayer::Canonical, LoomLayer::Team, LoomLayer::User] {
        match fs::remove_file(note_path(layer, id)) {
            Ok(()) => return Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => continue,
            Err(e) => return Err(e.to_string()),
        }
    }
    Err(format!("note '{}' not found", id))
}

pub fn load_note(id: &str) -> Result<Option<LoomNote>, String> {
    for layer in [LoomLayer::Canonical, LoomLayer::Team, LoomLayer::User] {
        match fs::read_to_string(note_path(layer, id)) {
            Ok(content) => {
                let note = serde_json::from_str(&content).map_err(|e| e.to_string())?;
                return Ok(Some(note));
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => continue,
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(None)
}

pub fn list_notes() -> Result<Vec<LoomNote>, String> {
    ensure_layout()?;
    let mut notes = Vec::new();
    for layer in [LoomLayer::Canonical, LoomLayer::Team, LoomLayer::User] {
        let dir = loom_dir().join("notes").join(layer.as_str());
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
            if let Ok(note) = serde_json::from_str::<LoomNote>(&content) {
                notes.push(note);
            }
        }
    }
    notes.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(notes)
}

pub fn retrieve_notes(
    layer: Option<LoomLayer>,
    scope: Option<LoomScope>,
    target: Option<&str>,
    author: Option<&str>,
    file: Option<&str>,
    tag: Option<&str>,
) -> Result<Vec<LoomNote>, String> {
    let mut notes = list_notes()?;
    notes.retain(|n| {
        layer.map(|v| n.layer == v).unwrap_or(true)
            && scope.map(|v| n.scope == v).unwrap_or(true)
            && target.map(|v| n.target.contains(v)).unwrap_or(true)
            && author.map(|v| n.author.eq_ignore_ascii_case(v)).unwrap_or(true)
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

pub fn record_sync(direction: SyncDirection, source_id: Option<&str>, summary: &str) -> Result<(), String> {
    append_sync_run(SyncRun { timestamp: Utc::now().to_rfc3339(), direction, source_id: source_id.map(|s| s.to_string()), summary: summary.to_string() })
}

pub fn compact_notes() -> Result<(usize, usize), String> {
    let notes = list_notes()?;
    let mut archived = 0usize;
    let mut kept = 0usize;
    for note in notes {
        let should_archive = note.status == NoteStatus::Stale
            || (note.scope == LoomScope::Session && note.layer != LoomLayer::Canonical);
        if should_archive {
            let _ = update_note(&note.id, None, None, Some(NoteStatus::Archived));
            archived += 1;
        } else {
            kept += 1;
        }
    }
    Ok((kept, archived))
}

pub fn supported_schema_sql() -> String {
    sql_schema_for(SourceKind::Sqlite)
}

pub fn sql_schema_for(kind: SourceKind) -> String {
    let (id_auto, ts_default, index_prefix) = match kind {
        SourceKind::Postgres => (
            "BIGSERIAL PRIMARY KEY",
            "TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP",
            "CREATE INDEX IF NOT EXISTS",
        ),
        SourceKind::Mysql => (
            "BIGINT PRIMARY KEY AUTO_INCREMENT",
            "TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP",
            "CREATE INDEX",
        ),
        _ => (
            "INTEGER PRIMARY KEY AUTOINCREMENT",
            "TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP",
            "CREATE INDEX IF NOT EXISTS",
        ),
    };
    let mut sql = format!(
        "
CREATE TABLE IF NOT EXISTS loom_sources (
  id TEXT PRIMARY KEY,
  kind TEXT NOT NULL,
  url TEXT NOT NULL,
  enabled BOOLEAN NOT NULL DEFAULT TRUE,
  owner_user TEXT NULL,
  database_name TEXT NULL,
  namespace TEXT NULL,
  username TEXT NULL,
  password_env TEXT NULL,
  notes TEXT NULL,
  created_at {ts_default}
);

CREATE TABLE IF NOT EXISTS loom_notes (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  body TEXT NOT NULL,
  layer TEXT NOT NULL,
  scope TEXT NOT NULL,
  target TEXT NOT NULL,
  files_json TEXT NOT NULL,
  tags_json TEXT NOT NULL,
  related_pr BIGINT NULL,
  author TEXT NOT NULL,
  actor TEXT NULL,
  status TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL,
  updated_at TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS loom_sync_runs (
  id {id_auto},
  timestamp TEXT NOT NULL,
  direction TEXT NOT NULL,
  source_id TEXT NULL,
  summary TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS loom_package_findings (
  id {id_auto},
  package_name TEXT NOT NULL,
  version TEXT NOT NULL,
  ecosystem TEXT NOT NULL,
  severity TEXT NOT NULL,
  vulnerability_id TEXT NOT NULL,
  source_file TEXT NOT NULL,
  installed_by TEXT NULL,
  observed_at {ts_default}
);
"
    );
    if kind == SourceKind::Mysql {
        sql.push_str(
            "CREATE INDEX idx_loom_notes_layer_scope ON loom_notes(layer, scope);\n\
CREATE INDEX idx_loom_notes_target ON loom_notes(target);\n\
CREATE INDEX idx_loom_notes_author ON loom_notes(author);\n\
CREATE INDEX idx_loom_notes_status ON loom_notes(status);\n",
        );
    } else {
        sql.push_str(&format!(
            "{idx} idx_loom_notes_layer_scope ON loom_notes(layer, scope);\n\
{idx} idx_loom_notes_target ON loom_notes(target);\n\
{idx} idx_loom_notes_author ON loom_notes(author);\n\
{idx} idx_loom_notes_status ON loom_notes(status);\n",
            idx = index_prefix
        ));
    }
    sql.trim().to_string()
}

pub fn supported_schema_redis() -> String {
    let schema = r#"
loom:source:{id} -> hash
  kind
  url
  enabled
  owner_user
  database
  namespace
  username
  password_env
  notes

loom:note:{id} -> hash
  title
  body
  layer
  scope
  target
  files_json
  tags_json
  related_pr
  author
  actor
  status
  created_at
  updated_at

loom:index:layer:{layer} -> set(note_id)
loom:index:scope:{scope} -> set(note_id)
loom:index:target:{target} -> set(note_id)
loom:index:author:{author} -> set(note_id)
loom:index:status:{status} -> set(note_id)

loom:sync:runs -> list(json)
loom:package:finding:{package}:{vuln_id} -> hash
loom:package:index:severity:{severity} -> set(package:vuln_id)
"#;
    schema.trim().to_string()
}

pub fn package_risks() -> Result<Vec<PackageRisk>, String> {
    let packages = engine::scanner::detect_locked_packages(None);
    if packages.is_empty() {
        return Ok(Vec::new());
    }
    let queries: Vec<(String, String, String)> = packages
        .iter()
        .map(|pkg| {
            (
                pkg.name.clone(),
                map_osv_ecosystem(&pkg.ecosystem).to_string(),
                pkg.version.clone(),
            )
        })
        .collect();

    let results = engine::osv::batch_query(&queries)?;
    let notes = retrieve_notes(None, Some(LoomScope::Package), None, None, None, None).unwrap_or_default();

    let mut out = Vec::new();
    for (pkg, refs) in packages.iter().zip(results.iter()) {
        for vuln in refs {
            let severity = engine::osv::fetch_vuln_detail(&vuln.id)
                .ok()
                .map(|d| engine::osv::severity_label(&d))
                .unwrap_or("UNKNOWN")
                .to_string();
            let installed_by = notes
                .iter()
                .find(|n| n.target.eq_ignore_ascii_case(&pkg.name))
                .map(|n| n.author.clone());
            out.push(PackageRisk {
                package: pkg.name.clone(),
                version: pkg.version.clone(),
                ecosystem: pkg.ecosystem.clone(),
                severity,
                vulnerability_id: vuln.id.clone(),
                source_file: pkg.source.clone(),
                installed_by,
            });
        }
    }
    out.sort_by(|a, b| b.severity.cmp(&a.severity).then(a.package.cmp(&b.package)));
    Ok(out)
}

fn map_osv_ecosystem(eco: &str) -> &'static str {
    match eco {
        "pip" | "uv" | "poetry" => "PyPI",
        "cargo" => "crates.io",
        "go" => "Go",
        "composer" => "Packagist",
        "gem" => "RubyGems",
        "nuget" => "NuGet",
        "hex" => "Hex",
        "pub" => "Pub",
        _ => "npm",
    }
}

pub fn detect_repo_name() -> String {
    std::env::current_dir()
        .ok()
        .as_deref()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("repo")
        .to_string()
}

pub fn detect_user_name() -> Option<String> {
    for key in ["INFYNON_USER", "USER", "USERNAME"] {
        if let Ok(value) = std::env::var(key) {
            if let Some(s) = normalize_user(&value) { return Some(s); }
        }
    }
    None
}

pub fn merge_remote_notes(remote: Vec<LoomNote>) -> Result<usize, String> {
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
