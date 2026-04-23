use crate::trace::types::{
    EntityKind, KgEdge, KgEntity, NoteStatus, PackageRisk, RelationType, SyncRun, TraceLayer,
    TraceNote, TraceScope, TraceSource,
};
use redis::{Commands, Connection};
use std::collections::HashMap;
use std::str::FromStr;

pub fn validate_and_prepare(source: &TraceSource) -> Result<(), String> {
    let mut conn = connection(source)?;
    let _: String = redis::cmd("PING")
        .query(&mut conn)
        .map_err(|e| e.to_string())?;
    let key = key(source, "meta:schema");
    let _: () = conn.set(key, "v1").map_err(|e| e.to_string())?;
    upsert_source(&mut conn, source)?;
    Ok(())
}

pub fn push_all(
    source: &TraceSource,
    notes: &[TraceNote],
    package_findings: &[PackageRisk],
    sync_run: &SyncRun,
) -> Result<(), String> {
    let mut conn = connection(source)?;
    for note in notes {
        upsert_note(&mut conn, source, note)?;
    }
    for finding in package_findings {
        upsert_package_finding(&mut conn, source, finding)?;
    }
    record_sync(source, sync_run)
}

pub fn pull_notes(source: &TraceSource) -> Result<Vec<TraceNote>, String> {
    let mut conn = connection(source)?;
    let ids: Vec<String> = conn
        .smembers(key(source, "notes:all"))
        .map_err(|e| e.to_string())?;
    let mut notes = Vec::new();
    for id in ids {
        let hash: HashMap<String, String> = conn
            .hgetall(key(source, &format!("note:{}", id)))
            .map_err(|e| e.to_string())?;
        if hash.is_empty() {
            continue;
        }
        notes.push(note_from_hash(&hash)?);
    }
    notes.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(notes)
}

pub fn record_sync(source: &TraceSource, run: &SyncRun) -> Result<(), String> {
    let mut conn = connection(source)?;
    let payload = serde_json::to_string(run).map_err(|e| e.to_string())?;
    let _: () = conn
        .lpush(key(source, "sync:runs"), payload)
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn push_kg(
    source: &TraceSource,
    entities: &[KgEntity],
    edges: &[KgEdge],
) -> Result<(), String> {
    let mut conn = connection(source)?;
    for entity in entities {
        upsert_kg_entity(&mut conn, source, entity)?;
    }
    for edge in edges {
        upsert_kg_edge(&mut conn, source, edge)?;
    }
    Ok(())
}

pub fn pull_kg(source: &TraceSource) -> Result<(Vec<KgEntity>, Vec<KgEdge>), String> {
    let mut conn = connection(source)?;

    let entity_ids: Vec<String> = conn
        .smembers(key(source, "kg:entities:all"))
        .map_err(|e| e.to_string())?;
    let mut entities = Vec::new();
    for id in entity_ids {
        let hash: HashMap<String, String> = conn
            .hgetall(key(source, &format!("kg:entity:{}", id)))
            .map_err(|e| e.to_string())?;
        if hash.is_empty() {
            continue;
        }
        entities.push(entity_from_hash(&hash)?);
    }

    let edge_ids: Vec<String> = conn
        .smembers(key(source, "kg:edges:all"))
        .map_err(|e| e.to_string())?;
    let mut edges = Vec::new();
    for id in edge_ids {
        let hash: HashMap<String, String> = conn
            .hgetall(key(source, &format!("kg:edge:{}", id)))
            .map_err(|e| e.to_string())?;
        if hash.is_empty() {
            continue;
        }
        edges.push(edge_from_hash(&hash)?);
    }

    Ok((entities, edges))
}

fn upsert_kg_entity(
    conn: &mut Connection,
    source: &TraceSource,
    entity: &KgEntity,
) -> Result<(), String> {
    let entity_key = key(source, &format!("kg:entity:{}", entity.id));
    let metadata_json = serde_json::to_string(&entity.metadata).map_err(|e| e.to_string())?;
    let _: () = redis::cmd("HSET")
        .arg(&entity_key)
        .arg("id")
        .arg(&entity.id)
        .arg("kind")
        .arg(entity.kind.as_str())
        .arg("name")
        .arg(&entity.name)
        .arg("metadata_json")
        .arg(metadata_json)
        .arg("branch")
        .arg(&entity.branch)
        .arg("created_at")
        .arg(&entity.created_at)
        .arg("updated_at")
        .arg(&entity.updated_at)
        .query(conn)
        .map_err(|e| e.to_string())?;
    let _: () = conn
        .sadd(key(source, "kg:entities:all"), &entity.id)
        .map_err(|e| e.to_string())?;
    let _: () = conn
        .sadd(
            key(
                source,
                &format!("kg:index:branch:{}:entities", entity.branch),
            ),
            &entity.id,
        )
        .map_err(|e| e.to_string())?;
    let _: () = conn
        .sadd(
            key(source, &format!("kg:index:kind:{}", entity.kind.as_str())),
            &entity.id,
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn upsert_kg_edge(
    conn: &mut Connection,
    source: &TraceSource,
    edge: &KgEdge,
) -> Result<(), String> {
    let edge_key = key(source, &format!("kg:edge:{}", edge.id));
    let _: () = redis::cmd("HSET")
        .arg(&edge_key)
        .arg("id")
        .arg(&edge.id)
        .arg("source_entity")
        .arg(&edge.source)
        .arg("target_entity")
        .arg(&edge.target)
        .arg("relation")
        .arg(edge.relation.as_str())
        .arg("weight")
        .arg(edge.weight.to_string())
        .arg("branch")
        .arg(&edge.branch)
        .arg("evidence")
        .arg(&edge.evidence)
        .arg("created_at")
        .arg(&edge.created_at)
        .query(conn)
        .map_err(|e| e.to_string())?;
    let _: () = conn
        .sadd(key(source, "kg:edges:all"), &edge.id)
        .map_err(|e| e.to_string())?;
    let _: () = conn
        .sadd(
            key(source, &format!("kg:index:branch:{}:edges", edge.branch)),
            &edge.id,
        )
        .map_err(|e| e.to_string())?;
    let _: () = conn
        .sadd(
            key(
                source,
                &format!("kg:index:relation:{}", edge.relation.as_str()),
            ),
            &edge.id,
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn entity_from_hash(hash: &HashMap<String, String>) -> Result<KgEntity, String> {
    Ok(KgEntity {
        id: value(hash, "id")?,
        kind: EntityKind::from_str(&value(hash, "kind")?).map_err(|e| e.to_string())?,
        name: value(hash, "name")?,
        metadata: serde_json::from_str(&value(hash, "metadata_json")?)
            .map_err(|e| e.to_string())?,
        branch: value(hash, "branch")?,
        created_at: value(hash, "created_at")?,
        updated_at: value(hash, "updated_at")?,
    })
}

fn edge_from_hash(hash: &HashMap<String, String>) -> Result<KgEdge, String> {
    Ok(KgEdge {
        id: value(hash, "id")?,
        source: value(hash, "source_entity")?,
        target: value(hash, "target_entity")?,
        relation: RelationType::from_str(&value(hash, "relation")?).map_err(|e| e.to_string())?,
        weight: value(hash, "weight")?
            .parse::<f64>()
            .map_err(|e| e.to_string())?,
        branch: value(hash, "branch")?,
        evidence: value(hash, "evidence")?,
        created_at: value(hash, "created_at")?,
    })
}

fn upsert_note(
    conn: &mut Connection,
    source: &TraceSource,
    note: &TraceNote,
) -> Result<(), String> {
    let note_key = key(source, &format!("note:{}", note.id));
    let files = serde_json::to_string(&note.files).map_err(|e| e.to_string())?;
    let tags = serde_json::to_string(&note.tags).map_err(|e| e.to_string())?;
    let related_pr = note.related_pr.map(|v| v.to_string()).unwrap_or_default();
    let actor = note.actor.clone().unwrap_or_default();
    let _: () = redis::cmd("HSET")
        .arg(&note_key)
        .arg("id")
        .arg(&note.id)
        .arg("title")
        .arg(&note.title)
        .arg("body")
        .arg(&note.body)
        .arg("layer")
        .arg(note.layer.as_str())
        .arg("scope")
        .arg(note.scope.as_str())
        .arg("target")
        .arg(&note.target)
        .arg("files_json")
        .arg(files)
        .arg("tags_json")
        .arg(tags)
        .arg("related_pr")
        .arg(related_pr)
        .arg("author")
        .arg(&note.author)
        .arg("actor")
        .arg(actor)
        .arg("status")
        .arg(note.status.as_str())
        .arg("created_at")
        .arg(&note.created_at)
        .arg("updated_at")
        .arg(&note.updated_at)
        .query(conn)
        .map_err(|e| e.to_string())?;
    let _: () = conn
        .sadd(key(source, "notes:all"), &note.id)
        .map_err(|e| e.to_string())?;
    let _: () = conn
        .sadd(
            key(source, &format!("index:layer:{}", note.layer.as_str())),
            &note.id,
        )
        .map_err(|e| e.to_string())?;
    let _: () = conn
        .sadd(
            key(source, &format!("index:scope:{}", note.scope.as_str())),
            &note.id,
        )
        .map_err(|e| e.to_string())?;
    let _: () = conn
        .sadd(
            key(source, &format!("index:author:{}", note.author)),
            &note.id,
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn upsert_source(conn: &mut Connection, source: &TraceSource) -> Result<(), String> {
    let source_key = key(source, &format!("source:{}", source.id));
    let database = source.database.clone().unwrap_or_default();
    let namespace = source.namespace.clone().unwrap_or_default();
    let username = source.username.clone().unwrap_or_default();
    let password_env = source.password_env.clone().unwrap_or_default();
    let notes = source.notes.clone().unwrap_or_default();
    let owner_user = source.owner_user.clone().unwrap_or_default();
    let _: () = redis::cmd("HSET")
        .arg(&source_key)
        .arg("id")
        .arg(&source.id)
        .arg("kind")
        .arg(source.kind.as_str())
        .arg("url")
        .arg(&source.url)
        .arg("enabled")
        .arg(source.enabled as u8)
        .arg("owner_user")
        .arg(owner_user)
        .arg("database")
        .arg(database)
        .arg("namespace")
        .arg(namespace)
        .arg("username")
        .arg(username)
        .arg("password_env")
        .arg(password_env)
        .arg("notes")
        .arg(notes)
        .query(conn)
        .map_err(|e| e.to_string())?;
    let _: () = conn
        .sadd(key(source, "sources:all"), &source.id)
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn upsert_package_finding(
    conn: &mut Connection,
    source: &TraceSource,
    finding: &PackageRisk,
) -> Result<(), String> {
    let id = format!("{}:{}", finding.package, finding.vulnerability_id);
    let finding_key = key(source, &format!("package:finding:{}", id));
    let owner = finding.installed_by.clone().unwrap_or_default();
    let _: () = redis::cmd("HSET")
        .arg(&finding_key)
        .arg("package_name")
        .arg(&finding.package)
        .arg("version")
        .arg(&finding.version)
        .arg("ecosystem")
        .arg(&finding.ecosystem)
        .arg("severity")
        .arg(&finding.severity)
        .arg("vulnerability_id")
        .arg(&finding.vulnerability_id)
        .arg("source_file")
        .arg(&finding.source_file)
        .arg("installed_by")
        .arg(owner)
        .query(conn)
        .map_err(|e| e.to_string())?;
    let _: () = conn
        .sadd(
            key(
                source,
                &format!("package:index:severity:{}", finding.severity),
            ),
            id,
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn note_from_hash(hash: &HashMap<String, String>) -> Result<TraceNote, String> {
    Ok(TraceNote {
        id: value(hash, "id")?,
        title: value(hash, "title")?,
        body: value(hash, "body")?,
        layer: value(hash, "layer")?.parse()?,
        scope: value(hash, "scope")?.parse()?,
        target: value(hash, "target")?,
        files: serde_json::from_str(&value(hash, "files_json")?).map_err(|e| e.to_string())?,
        tags: serde_json::from_str(&value(hash, "tags_json")?).map_err(|e| e.to_string())?,
        related_pr: hash.get("related_pr").and_then(|v| {
            if v.is_empty() {
                None
            } else {
                v.parse().ok()
            }
        }),
        author: value(hash, "author")?,
        actor: hash.get("actor").cloned().filter(|v| !v.is_empty()),
        status: value(hash, "status")?.parse()?,
        created_at: value(hash, "created_at")?,
        updated_at: value(hash, "updated_at")?,
    })
}

fn value(hash: &HashMap<String, String>, key: &str) -> Result<String, String> {
    hash.get(key)
        .cloned()
        .ok_or_else(|| format!("missing field '{}'", key))
}

fn connection(source: &TraceSource) -> Result<Connection, String> {
    let client = redis::Client::open(source.url.as_str()).map_err(|e| e.to_string())?;
    client.get_connection().map_err(|e| e.to_string())
}

fn key(source: &TraceSource, suffix: &str) -> String {
    let ns = source.namespace.as_deref().unwrap_or("trace");
    format!("{}:{}", ns, suffix)
}
