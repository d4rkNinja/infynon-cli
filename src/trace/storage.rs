use crate::trace::types::{
    EntityKind, KgEdge, KgEntity, KgGraph, RelationType, TraceConfig, TraceLayer, TraceNote,
    TraceScope, TraceSource, NoteStatus, PackageRisk, SyncDirection, SyncRun, SyncState,
};
use crate::{engine, trace::types::SourceKind};
use chrono::Utc;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn normalize_user(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() { None } else { Some(t.to_string()) }
}

pub fn trace_dir() -> PathBuf {
    PathBuf::from(".infynon").join("trace")
}

pub fn ensure_layout() -> Result<(), String> {
    for dir in [
        trace_dir(),
        trace_dir().join("notes"),
        trace_dir().join("notes").join("canonical"),
        trace_dir().join("notes").join("team"),
        trace_dir().join("notes").join("user"),
        trace_dir().join("state"),
    ] {
        fs::create_dir_all(&dir).map_err(|e| format!("failed to create {}: {}", dir.display(), e))?;
    }
    Ok(())
}

pub fn config_path() -> PathBuf {
    trace_dir().join("config.toml")
}

pub fn sync_state_path() -> PathBuf {
    trace_dir().join("state").join("sync.json")
}

pub fn init_config(repo_name: &str, owner: &str, default_user: Option<&str>) -> Result<(), String> {
    ensure_layout()?;
    let cfg = TraceConfig {
        repo_name: repo_name.to_string(),
        owner: owner.to_string(),
        default_user: default_user.and_then(normalize_user),
        default_source: None,
        sources: Vec::new(),
    };
    save_config(&cfg)
}

pub fn load_config() -> Result<TraceConfig, String> {
    match fs::read_to_string(config_path()) {
        Ok(content) => toml::from_str(&content).map_err(|e| format!("invalid trace config: {}", e)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(TraceConfig::default()),
        Err(e) => Err(e.to_string()),
    }
}

pub fn save_config(cfg: &TraceConfig) -> Result<(), String> {
    ensure_layout()?;
    let content = toml::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    fs::write(config_path(), content).map_err(|e| e.to_string())
}

pub fn add_source(source: TraceSource, make_default: bool) -> Result<(), String> {
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

pub fn get_source(id: Option<&str>) -> Result<TraceSource, String> {
    let cfg = load_config()?;
    let wanted = match id {
        Some(id) => id.to_string(),
        None => cfg
            .default_source
            .clone()
            .ok_or_else(|| "No default Trace source configured.".to_string())?,
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

fn note_path(layer: TraceLayer, id: &str) -> PathBuf {
    trace_dir()
        .join("notes")
        .join(layer.as_str())
        .join(format!("{}.json", sanitize(id)))
}

pub fn sanitize(input: &str) -> String {
    input.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect()
}

pub fn create_note(mut note: TraceNote) -> Result<(), String> {
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
    for layer in [TraceLayer::Canonical, TraceLayer::Team, TraceLayer::User] {
        match fs::remove_file(note_path(layer, id)) {
            Ok(()) => return Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => continue,
            Err(e) => return Err(e.to_string()),
        }
    }
    Err(format!("note '{}' not found", id))
}

pub fn load_note(id: &str) -> Result<Option<TraceNote>, String> {
    for layer in [TraceLayer::Canonical, TraceLayer::Team, TraceLayer::User] {
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
CREATE TABLE IF NOT EXISTS trace_sources (
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

CREATE TABLE IF NOT EXISTS trace_notes (
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

CREATE TABLE IF NOT EXISTS trace_sync_runs (
  id {id_auto},
  timestamp TEXT NOT NULL,
  direction TEXT NOT NULL,
  source_id TEXT NULL,
  summary TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS trace_package_findings (
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
            "CREATE INDEX idx_trace_notes_layer_scope ON trace_notes(layer, scope);\n\
CREATE INDEX idx_trace_notes_target ON trace_notes(target);\n\
CREATE INDEX idx_trace_notes_author ON trace_notes(author);\n\
CREATE INDEX idx_trace_notes_status ON trace_notes(status);\n",
        );
    } else {
        sql.push_str(&format!(
            "{idx} idx_trace_notes_layer_scope ON trace_notes(layer, scope);\n\
{idx} idx_trace_notes_target ON trace_notes(target);\n\
{idx} idx_trace_notes_author ON trace_notes(author);\n\
{idx} idx_trace_notes_status ON trace_notes(status);\n",
            idx = index_prefix
        ));
    }

    // Knowledge Graph tables
    sql.push_str(&format!(
        "
CREATE TABLE IF NOT EXISTS trace_kg_entities (
  id TEXT PRIMARY KEY,
  kind TEXT NOT NULL,
  name TEXT NOT NULL,
  metadata_json TEXT NOT NULL DEFAULT '{{}}',
  branch TEXT NOT NULL DEFAULT 'main',
  created_at TIMESTAMP NOT NULL,
  updated_at TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS trace_kg_edges (
  id TEXT PRIMARY KEY,
  source_entity TEXT NOT NULL,
  target_entity TEXT NOT NULL,
  relation TEXT NOT NULL,
  weight REAL NOT NULL DEFAULT 1.0,
  branch TEXT NOT NULL DEFAULT 'main',
  evidence TEXT NOT NULL DEFAULT '',
  created_at TIMESTAMP NOT NULL
);
"
    ));

    if kind == SourceKind::Mysql {
        sql.push_str(
            "CREATE INDEX idx_trace_kg_entities_kind ON trace_kg_entities(kind);\n\
CREATE INDEX idx_trace_kg_entities_branch ON trace_kg_entities(branch);\n\
CREATE INDEX idx_trace_kg_entities_name ON trace_kg_entities(name);\n\
CREATE INDEX idx_trace_kg_edges_source ON trace_kg_edges(source_entity);\n\
CREATE INDEX idx_trace_kg_edges_target ON trace_kg_edges(target_entity);\n\
CREATE INDEX idx_trace_kg_edges_branch ON trace_kg_edges(branch);\n\
CREATE INDEX idx_trace_kg_edges_relation ON trace_kg_edges(relation);\n",
        );
    } else {
        sql.push_str(&format!(
            "{idx} idx_trace_kg_entities_kind ON trace_kg_entities(kind);\n\
{idx} idx_trace_kg_entities_branch ON trace_kg_entities(branch);\n\
{idx} idx_trace_kg_entities_name ON trace_kg_entities(name);\n\
{idx} idx_trace_kg_edges_source ON trace_kg_edges(source_entity);\n\
{idx} idx_trace_kg_edges_target ON trace_kg_edges(target_entity);\n\
{idx} idx_trace_kg_edges_branch ON trace_kg_edges(branch);\n\
{idx} idx_trace_kg_edges_relation ON trace_kg_edges(relation);\n",
            idx = index_prefix
        ));
    }

    sql.trim().to_string()
}

pub fn supported_schema_redis() -> String {
    let schema = r#"
trace:source:{id} -> hash
  kind
  url
  enabled
  owner_user
  database
  namespace
  username
  password_env
  notes

trace:note:{id} -> hash
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

trace:index:layer:{layer} -> set(note_id)
trace:index:scope:{scope} -> set(note_id)
trace:index:target:{target} -> set(note_id)
trace:index:author:{author} -> set(note_id)
trace:index:status:{status} -> set(note_id)

trace:sync:runs -> list(json)
trace:package:finding:{package}:{vuln_id} -> hash
trace:package:index:severity:{severity} -> set(package:vuln_id)

trace:kg:entity:{id} -> hash (id, kind, name, metadata_json, branch, created_at, updated_at)
trace:kg:edge:{id} -> hash (id, source_entity, target_entity, relation, weight, branch, evidence, created_at)
trace:kg:entities:all -> set(entity_id)
trace:kg:edges:all -> set(edge_id)
trace:kg:index:branch:{branch}:entities -> set(entity_id)
trace:kg:index:branch:{branch}:edges -> set(edge_id)
trace:kg:index:kind:{kind} -> set(entity_id)
trace:kg:index:relation:{relation} -> set(edge_id)
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
    let notes = retrieve_notes(None, Some(TraceScope::Package), None, None, None, None).unwrap_or_default();

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

pub fn kg_dir() -> PathBuf {
    trace_dir().join("kg")
}

pub fn ensure_kg_layout() -> Result<(), String> {
    for dir in [kg_dir(), kg_dir().join("entities"), kg_dir().join("edges")] {
        fs::create_dir_all(&dir).map_err(|e| format!("failed to create {}: {}", dir.display(), e))?;
    }
    Ok(())
}

fn kg_entity_path(id: &str) -> PathBuf {
    kg_dir().join("entities").join(format!("{}.json", sanitize(id)))
}

fn kg_edge_path(id: &str) -> PathBuf {
    kg_dir().join("edges").join(format!("{}.json", sanitize(id)))
}

pub fn create_entity(entity: KgEntity) -> Result<(), String> {
    ensure_kg_layout()?;
    let content = serde_json::to_string_pretty(&entity).map_err(|e| e.to_string())?;
    fs::write(kg_entity_path(&entity.id), content).map_err(|e| e.to_string())
}

pub fn delete_entity(id: &str) -> Result<(), String> {
    let path = kg_entity_path(id);
    match fs::remove_file(&path) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            return Err(format!("entity '{}' not found", id));
        }
        Err(e) => return Err(e.to_string()),
    }
    // Remove all edges referencing this entity
    let edges = list_edges(None, None)?;
    for edge in edges {
        if edge.source == id || edge.target == id {
            let _ = delete_edge(&edge.id);
        }
    }
    Ok(())
}

pub fn list_entities(branch: Option<&str>, kind: Option<EntityKind>) -> Result<Vec<KgEntity>, String> {
    ensure_kg_layout()?;
    let dir = kg_dir().join("entities");
    let entries = match fs::read_dir(&dir) {
        Ok(e) => e,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e.to_string()),
    };
    let mut entities = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        if let Ok(entity) = serde_json::from_str::<KgEntity>(&content) {
            let branch_ok = branch.map(|b| entity.branch == b).unwrap_or(true);
            let kind_ok = kind.map(|k| entity.kind == k).unwrap_or(true);
            if branch_ok && kind_ok {
                entities.push(entity);
            }
        }
    }
    entities.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(entities)
}

pub fn load_entity(id: &str) -> Result<Option<KgEntity>, String> {
    match fs::read_to_string(kg_entity_path(id)) {
        Ok(content) => {
            let entity = serde_json::from_str(&content).map_err(|e| e.to_string())?;
            Ok(Some(entity))
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

pub fn find_entity_by_name(name: &str, branch: &str) -> Result<Option<KgEntity>, String> {
    let entities = list_entities(Some(branch), None)?;
    Ok(entities.into_iter().find(|e| e.name == name))
}

pub fn create_edge(edge: KgEdge) -> Result<(), String> {
    ensure_kg_layout()?;
    let content = serde_json::to_string_pretty(&edge).map_err(|e| e.to_string())?;
    fs::write(kg_edge_path(&edge.id), content).map_err(|e| e.to_string())
}

pub fn delete_edge(id: &str) -> Result<(), String> {
    let path = kg_edge_path(id);
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Err(format!("edge '{}' not found", id)),
        Err(e) => Err(e.to_string()),
    }
}

pub fn list_edges(branch: Option<&str>, relation: Option<RelationType>) -> Result<Vec<KgEdge>, String> {
    ensure_kg_layout()?;
    let dir = kg_dir().join("edges");
    let entries = match fs::read_dir(&dir) {
        Ok(e) => e,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e.to_string()),
    };
    let mut edges = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        if let Ok(edge) = serde_json::from_str::<KgEdge>(&content) {
            let branch_ok = branch.map(|b| edge.branch == b).unwrap_or(true);
            let relation_ok = relation.map(|r| edge.relation == r).unwrap_or(true);
            if branch_ok && relation_ok {
                edges.push(edge);
            }
        }
    }
    edges.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(edges)
}

pub fn load_edge(id: &str) -> Result<Option<KgEdge>, String> {
    match fs::read_to_string(kg_edge_path(id)) {
        Ok(content) => {
            let edge = serde_json::from_str(&content).map_err(|e| e.to_string())?;
            Ok(Some(edge))
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

pub fn load_graph(branch: Option<&str>) -> Result<KgGraph, String> {
    Ok(KgGraph {
        entities: list_entities(branch, None)?,
        edges: list_edges(branch, None)?,
    })
}

pub fn export_graph_json(graph: &KgGraph) -> Result<String, String> {
    serde_json::to_string_pretty(graph).map_err(|e| e.to_string())
}

pub fn export_graph_dot(graph: &KgGraph) -> Result<String, String> {
    let mut dot = String::from("digraph trace_kg {\n  rankdir=LR;\n  node [fontname=\"Helvetica\"];\n\n");

    // Group entities by branch into subgraph clusters
    let mut branches: HashMap<String, Vec<&KgEntity>> = HashMap::new();
    for entity in &graph.entities {
        branches.entry(entity.branch.clone()).or_default().push(entity);
    }

    for (branch, entities) in &branches {
        dot.push_str(&format!("  subgraph cluster_{} {{\n", sanitize(branch)));
        dot.push_str(&format!("    label=\"{}\";\n", branch));
        dot.push_str("    style=dashed;\n");
        for entity in entities {
            let shape = match entity.kind {
                EntityKind::File => "box",
                EntityKind::Person => "ellipse",
                EntityKind::Package => "hexagon",
                EntityKind::Decision => "diamond",
                EntityKind::Endpoint => "component",
                EntityKind::Module => "folder",
                EntityKind::Pr => "note",
                EntityKind::Branch => "tab",
                EntityKind::Note => "rect",
                EntityKind::Vulnerability => "octagon",
            };
            dot.push_str(&format!(
                "    \"{}\" [label=\"{}\" shape={}];\n",
                sanitize(&entity.id),
                entity.name.replace('"', "\\\""),
                shape,
            ));
        }
        dot.push_str("  }\n\n");
    }

    for edge in &graph.edges {
        dot.push_str(&format!(
            "  \"{}\" -> \"{}\" [label=\"{}\"];\n",
            sanitize(&edge.source),
            sanitize(&edge.target),
            edge.relation.as_str(),
        ));
    }

    dot.push_str("}\n");
    Ok(dot)
}

pub fn import_graph_json(content: &str, target_branch: Option<&str>) -> Result<(usize, usize), String> {
    let mut graph: KgGraph = serde_json::from_str(content).map_err(|e| e.to_string())?;
    if let Some(branch) = target_branch {
        for entity in &mut graph.entities {
            entity.branch = branch.to_string();
        }
        for edge in &mut graph.edges {
            edge.branch = branch.to_string();
        }
    }
    let entity_count = graph.entities.len();
    let edge_count = graph.edges.len();
    for entity in graph.entities {
        create_entity(entity)?;
    }
    for edge in graph.edges {
        create_edge(edge)?;
    }
    Ok((entity_count, edge_count))
}

pub fn detect_current_branch() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok().map(|s| s.trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "main".to_string())
}

pub fn auto_build_graph(branch: &str) -> Result<(usize, usize), String> {
    ensure_kg_layout()?;
    let now = Utc::now().to_rfc3339();
    let mut entities_created = 0usize;
    let mut edges_created = 0usize;

    // Pre-load all existing entities and edges into HashSets to avoid N+1 filesystem scans
    let existing = list_entities(Some(branch), None).unwrap_or_default();
    let mut known_entities: std::collections::HashSet<String> = existing.iter().map(|e| e.name.clone()).collect();
    let existing_edges = list_edges(Some(branch), None).unwrap_or_default();
    let mut known_edges: std::collections::HashSet<String> = existing_edges.iter().map(|e| e.id.clone()).collect();

    // Run git log to extract person->file relationships
    let output = std::process::Command::new("git")
        .args(["log", "--name-only", "--format=%an", "--no-merges", "-100", branch])
        .output()
        .map_err(|e| format!("failed to run git log: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "git log failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut current_person: Option<String> = None;

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // If the line doesn't look like a file path, treat it as a person name
        if !line.contains('/') && !line.contains('.') {
            current_person = Some(line.to_string());
            // Create person entity if not exists
            let person_id = format!("person-{}", sanitize(line));
            if !known_entities.contains(line) {
                create_entity(KgEntity {
                    id: person_id,
                    kind: EntityKind::Person,
                    name: line.to_string(),
                    metadata: HashMap::new(),
                    branch: branch.to_string(),
                    created_at: now.clone(),
                    updated_at: now.clone(),
                })?;
                known_entities.insert(line.to_string());
                entities_created += 1;
            }
            continue;
        }

        // It's a file path
        if let Some(ref person) = current_person {
            let file_id = format!("file-{}", sanitize(line));
            if !known_entities.contains(line) {
                create_entity(KgEntity {
                    id: file_id.clone(),
                    kind: EntityKind::File,
                    name: line.to_string(),
                    metadata: HashMap::new(),
                    branch: branch.to_string(),
                    created_at: now.clone(),
                    updated_at: now.clone(),
                })?;
                known_entities.insert(line.to_string());
                entities_created += 1;
            }

            let person_id = format!("person-{}", sanitize(person));
            let edge_id = format!("edge-{}-{}", sanitize(line), sanitize(person));
            if !known_edges.contains(&edge_id) {
                create_edge(KgEdge {
                    id: edge_id.clone(),
                    source: file_id,
                    target: person_id,
                    relation: RelationType::ModifiedBy,
                    weight: 1.0,
                    branch: branch.to_string(),
                    evidence: format!("git log on {}", branch),
                    created_at: now.clone(),
                })?;
                known_edges.insert(edge_id);
                edges_created += 1;
            }
        }
    }

    // Process existing trace notes
    let notes = list_notes().unwrap_or_default();
    for note in &notes {
        let note_entity_id = format!("note-{}", sanitize(&note.id));
        if !known_entities.contains(&note.title) {
            create_entity(KgEntity {
                id: note_entity_id.clone(),
                kind: EntityKind::Note,
                name: note.title.clone(),
                metadata: {
                    let mut m = HashMap::new();
                    m.insert("scope".to_string(), note.scope.as_str().to_string());
                    m.insert("status".to_string(), note.status.as_str().to_string());
                    m
                },
                branch: branch.to_string(),
                created_at: note.created_at.clone(),
                updated_at: note.updated_at.clone(),
            })?;
            known_entities.insert(note.title.clone());
            entities_created += 1;
        }

        // Create Documents edge from note to its target
        if !note.target.is_empty() {
            let target_id = format!("file-{}", sanitize(&note.target));
            let edge_id = format!("edge-note-{}-{}", sanitize(&note.id), sanitize(&note.target));
            if !known_edges.contains(&edge_id) {
                create_edge(KgEdge {
                    id: edge_id.clone(),
                    source: note_entity_id,
                    target: target_id,
                    relation: RelationType::Documents,
                    weight: 1.0,
                    branch: branch.to_string(),
                    evidence: format!("trace note {}", note.id),
                    created_at: note.created_at.clone(),
                })?;
                known_edges.insert(edge_id);
                edges_created += 1;
            }
        }
    }

    Ok((entities_created, edges_created))
}
