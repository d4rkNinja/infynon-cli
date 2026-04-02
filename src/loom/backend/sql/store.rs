use crate::loom::types::{
    LoomLayer, LoomNote, LoomScope, LoomSource, NoteStatus, PackageRisk, SyncRun,
};
use mysql::{params, prelude::FromValue, prelude::Queryable, Row};
use postgres::Client;
use rusqlite::Connection;

pub fn upsert_source_sqlite(conn: &Connection, source: &LoomSource) -> Result<(), String> {
    conn.execute(
        "INSERT INTO loom_sources (id,kind,url,enabled,owner_user,database_name,namespace,username,password_env,notes)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)
         ON CONFLICT(id) DO UPDATE SET kind=excluded.kind, url=excluded.url, enabled=excluded.enabled,
         owner_user=excluded.owner_user, database_name=excluded.database_name, namespace=excluded.namespace, username=excluded.username,
         password_env=excluded.password_env, notes=excluded.notes",
        rusqlite::params![
            source.id,
            source.kind.as_str(),
            source.url,
            source.enabled,
            source.owner_user,
            source.database,
            source.namespace,
            source.username,
            source.password_env,
            source.notes
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn upsert_source_postgres(client: &mut Client, source: &LoomSource) -> Result<(), String> {
    client
        .execute(
            "INSERT INTO loom_sources (id,kind,url,enabled,owner_user,database_name,namespace,username,password_env,notes)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
             ON CONFLICT (id) DO UPDATE SET kind=EXCLUDED.kind, url=EXCLUDED.url, enabled=EXCLUDED.enabled,
             owner_user=EXCLUDED.owner_user, database_name=EXCLUDED.database_name, namespace=EXCLUDED.namespace, username=EXCLUDED.username,
             password_env=EXCLUDED.password_env, notes=EXCLUDED.notes",
            &[
                &source.id,
                &source.kind.as_str(),
                &source.url,
                &source.enabled,
                &source.owner_user,
                &source.database,
                &source.namespace,
                &source.username,
                &source.password_env,
                &source.notes,
            ],
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn upsert_source_mysql(
    conn: &mut mysql::PooledConn,
    source: &LoomSource,
) -> Result<(), String> {
    conn.exec_drop(
        "INSERT INTO loom_sources (id,kind,url,enabled,owner_user,database_name,namespace,username,password_env,notes)
         VALUES (:id,:kind,:url,:enabled,:owner_user,:database_name,:namespace,:username,:password_env,:notes)
         ON DUPLICATE KEY UPDATE kind=VALUES(kind), url=VALUES(url), enabled=VALUES(enabled),
         owner_user=VALUES(owner_user), database_name=VALUES(database_name), namespace=VALUES(namespace), username=VALUES(username),
         password_env=VALUES(password_env), notes=VALUES(notes)",
        params! {
            "id" => &source.id,
            "kind" => source.kind.as_str(),
            "url" => &source.url,
            "enabled" => source.enabled,
            "owner_user" => &source.owner_user,
            "database_name" => &source.database,
            "namespace" => &source.namespace,
            "username" => &source.username,
            "password_env" => &source.password_env,
            "notes" => &source.notes,
        },
    )
    .map_err(|e| e.to_string())
}

pub fn upsert_note_sqlite(conn: &Connection, note: &LoomNote) -> Result<(), String> {
    conn.execute(
        "INSERT INTO loom_notes (id,title,body,layer,scope,target,files_json,tags_json,related_pr,author,actor,status,created_at,updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)
         ON CONFLICT(id) DO UPDATE SET title=excluded.title, body=excluded.body, layer=excluded.layer, scope=excluded.scope,
         target=excluded.target, files_json=excluded.files_json, tags_json=excluded.tags_json, related_pr=excluded.related_pr,
         author=excluded.author, actor=excluded.actor, status=excluded.status, created_at=excluded.created_at, updated_at=excluded.updated_at",
        rusqlite::params![
            note.id,
            note.title,
            note.body,
            note.layer.as_str(),
            note.scope.as_str(),
            note.target,
            serde_json::to_string(&note.files).map_err(|e| e.to_string())?,
            serde_json::to_string(&note.tags).map_err(|e| e.to_string())?,
            note.related_pr,
            note.author,
            note.actor,
            note.status.as_str(),
            note.created_at,
            note.updated_at
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn upsert_note_postgres(client: &mut Client, note: &LoomNote) -> Result<(), String> {
    client
        .execute(
            "INSERT INTO loom_notes (id,title,body,layer,scope,target,files_json,tags_json,related_pr,author,actor,status,created_at,updated_at)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)
             ON CONFLICT (id) DO UPDATE SET title=EXCLUDED.title, body=EXCLUDED.body, layer=EXCLUDED.layer, scope=EXCLUDED.scope,
             target=EXCLUDED.target, files_json=EXCLUDED.files_json, tags_json=EXCLUDED.tags_json, related_pr=EXCLUDED.related_pr,
             author=EXCLUDED.author, actor=EXCLUDED.actor, status=EXCLUDED.status, created_at=EXCLUDED.created_at, updated_at=EXCLUDED.updated_at",
            &[
                &note.id,
                &note.title,
                &note.body,
                &note.layer.as_str(),
                &note.scope.as_str(),
                &note.target,
                &serde_json::to_string(&note.files).map_err(|e| e.to_string())?,
                &serde_json::to_string(&note.tags).map_err(|e| e.to_string())?,
                &note.related_pr.map(|v| v as i64),
                &note.author,
                &note.actor,
                &note.status.as_str(),
                &note.created_at,
                &note.updated_at,
            ],
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn upsert_note_mysql(conn: &mut mysql::PooledConn, note: &LoomNote) -> Result<(), String> {
    conn.exec_drop(
        "INSERT INTO loom_notes (id,title,body,layer,scope,target,files_json,tags_json,related_pr,author,actor,status,created_at,updated_at)
         VALUES (:id,:title,:body,:layer,:scope,:target,:files_json,:tags_json,:related_pr,:author,:actor,:status,:created_at,:updated_at)
         ON DUPLICATE KEY UPDATE title=VALUES(title), body=VALUES(body), layer=VALUES(layer), scope=VALUES(scope),
         target=VALUES(target), files_json=VALUES(files_json), tags_json=VALUES(tags_json), related_pr=VALUES(related_pr),
         author=VALUES(author), actor=VALUES(actor), status=VALUES(status), created_at=VALUES(created_at), updated_at=VALUES(updated_at)",
        params! {
            "id" => &note.id,
            "title" => &note.title,
            "body" => &note.body,
            "layer" => note.layer.as_str(),
            "scope" => note.scope.as_str(),
            "target" => &note.target,
            "files_json" => serde_json::to_string(&note.files).map_err(|e| e.to_string())?,
            "tags_json" => serde_json::to_string(&note.tags).map_err(|e| e.to_string())?,
            "related_pr" => note.related_pr,
            "author" => &note.author,
            "actor" => &note.actor,
            "status" => note.status.as_str(),
            "created_at" => &note.created_at,
            "updated_at" => &note.updated_at,
        },
    )
    .map_err(|e| e.to_string())
}

pub fn refresh_package_findings_sqlite(
    conn: &Connection,
    findings: &[PackageRisk],
) -> Result<(), String> {
    conn.execute("DELETE FROM loom_package_findings", [])
        .map_err(|e| e.to_string())?;
    for finding in findings {
        conn.execute(
            "INSERT INTO loom_package_findings (package_name,version,ecosystem,severity,vulnerability_id,source_file,installed_by)
             VALUES (?1,?2,?3,?4,?5,?6,?7)",
            rusqlite::params![
                finding.package,
                finding.version,
                finding.ecosystem,
                finding.severity,
                finding.vulnerability_id,
                finding.source_file,
                finding.installed_by
            ],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn refresh_package_findings_postgres(
    client: &mut Client,
    findings: &[PackageRisk],
) -> Result<(), String> {
    client
        .execute("DELETE FROM loom_package_findings", &[])
        .map_err(|e| e.to_string())?;
    for finding in findings {
        client
            .execute(
                "INSERT INTO loom_package_findings (package_name,version,ecosystem,severity,vulnerability_id,source_file,installed_by)
                 VALUES ($1,$2,$3,$4,$5,$6,$7)",
                &[
                    &finding.package,
                    &finding.version,
                    &finding.ecosystem,
                    &finding.severity,
                    &finding.vulnerability_id,
                    &finding.source_file,
                    &finding.installed_by,
                ],
            )
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn refresh_package_findings_mysql(
    conn: &mut mysql::PooledConn,
    findings: &[PackageRisk],
) -> Result<(), String> {
    conn.query_drop("DELETE FROM loom_package_findings")
        .map_err(|e| e.to_string())?;
    for finding in findings {
        conn.exec_drop(
            "INSERT INTO loom_package_findings (package_name,version,ecosystem,severity,vulnerability_id,source_file,installed_by)
             VALUES (?,?,?,?,?,?,?)",
            (
                &finding.package,
                &finding.version,
                &finding.ecosystem,
                &finding.severity,
                &finding.vulnerability_id,
                &finding.source_file,
                &finding.installed_by,
            ),
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn insert_sync_sqlite(conn: &Connection, run: &SyncRun) -> Result<(), String> {
    conn.execute(
        "INSERT INTO loom_sync_runs (timestamp,direction,source_id,summary) VALUES (?1,?2,?3,?4)",
        rusqlite::params![run.timestamp, run.direction.as_str(), run.source_id, run.summary],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn insert_sync_postgres(client: &mut Client, run: &SyncRun) -> Result<(), String> {
    client
        .execute(
            "INSERT INTO loom_sync_runs (timestamp,direction,source_id,summary) VALUES ($1,$2,$3,$4)",
            &[&run.timestamp, &run.direction.as_str(), &run.source_id, &run.summary],
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn insert_sync_mysql(conn: &mut mysql::PooledConn, run: &SyncRun) -> Result<(), String> {
    conn.exec_drop(
        "INSERT INTO loom_sync_runs (timestamp,direction,source_id,summary) VALUES (?,?,?,?)",
        (
            &run.timestamp,
            run.direction.as_str(),
            &run.source_id,
            &run.summary,
        ),
    )
    .map_err(|e| e.to_string())
}

pub fn pull_notes_sqlite(conn: &Connection) -> Result<Vec<LoomNote>, String> {
    let mut stmt = conn
        .prepare("SELECT id,title,body,layer,scope,target,files_json,tags_json,related_pr,author,actor,status,created_at,updated_at FROM loom_notes ORDER BY updated_at DESC")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(LoomNote {
                id: row.get(0)?,
                title: row.get(1)?,
                body: row.get(2)?,
                layer: parse_layer(row.get::<_, String>(3)?.as_str()).map_err(to_sql_err)?,
                scope: parse_scope(row.get::<_, String>(4)?.as_str()).map_err(to_sql_err)?,
                target: row.get(5)?,
                files: serde_json::from_str::<Vec<String>>(&row.get::<_, String>(6)?)
                    .map_err(to_sql_err)?,
                tags: serde_json::from_str::<Vec<String>>(&row.get::<_, String>(7)?)
                    .map_err(to_sql_err)?,
                related_pr: row.get::<_, Option<i64>>(8)?.map(|v| v as u64),
                author: row.get(9)?,
                actor: row.get(10)?,
                status: parse_status(row.get::<_, String>(11)?.as_str()).map_err(to_sql_err)?,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.map(|r| r.map_err(|e| e.to_string())).collect()
}

pub fn pull_notes_postgres(client: &mut Client) -> Result<Vec<LoomNote>, String> {
    let rows = client
        .query("SELECT id,title,body,layer,scope,target,files_json,tags_json,related_pr,author,actor,status,created_at,updated_at FROM loom_notes ORDER BY updated_at DESC", &[])
        .map_err(|e| e.to_string())?;
    rows.into_iter()
        .map(|row| {
            Ok(LoomNote {
                id: row.get(0),
                title: row.get(1),
                body: row.get(2),
                layer: parse_layer(row.get::<_, String>(3).as_str())?,
                scope: parse_scope(row.get::<_, String>(4).as_str())?,
                target: row.get(5),
                files: serde_json::from_str(&row.get::<_, String>(6)).map_err(|e| e.to_string())?,
                tags: serde_json::from_str(&row.get::<_, String>(7)).map_err(|e| e.to_string())?,
                related_pr: row.get::<_, Option<i64>>(8).map(|v| v as u64),
                author: row.get(9),
                actor: row.get(10),
                status: parse_status(row.get::<_, String>(11).as_str())?,
                created_at: row.get(12),
                updated_at: row.get(13),
            })
        })
        .collect()
}

pub fn pull_notes_mysql(conn: &mut mysql::PooledConn) -> Result<Vec<LoomNote>, String> {
    let rows: Vec<Row> = conn
        .query("SELECT id,title,body,layer,scope,target,files_json,tags_json,related_pr,author,actor,status,created_at,updated_at FROM loom_notes ORDER BY updated_at DESC")
        .map_err(|e| e.to_string())?;
    rows.into_iter().map(note_from_mysql_row).collect()
}

fn note_from_mysql_row(row: Row) -> Result<LoomNote, String> {
    let id = row_value::<String>(&row, 0, "id")?;
    let title = row_value::<String>(&row, 1, "title")?;
    let body = row_value::<String>(&row, 2, "body")?;
    let layer = row_value::<String>(&row, 3, "layer")?;
    let scope = row_value::<String>(&row, 4, "scope")?;
    let target = row_value::<String>(&row, 5, "target")?;
    let files_json = row_value::<String>(&row, 6, "files_json")?;
    let tags_json = row_value::<String>(&row, 7, "tags_json")?;
    let related_pr = row.get::<Option<u64>, _>(8).unwrap_or(None);
    let author = row_value::<String>(&row, 9, "author")?;
    let actor = row.get::<Option<String>, _>(10).unwrap_or(None);
    let status = row_value::<String>(&row, 11, "status")?;
    let created_at = row_value::<String>(&row, 12, "created_at")?;
    let updated_at = row_value::<String>(&row, 13, "updated_at")?;

    Ok(LoomNote {
        id,
        title,
        body,
        layer: parse_layer(&layer)?,
        scope: parse_scope(&scope)?,
        target,
        files: serde_json::from_str(&files_json).map_err(|e| e.to_string())?,
        tags: serde_json::from_str(&tags_json).map_err(|e| e.to_string())?,
        related_pr,
        author,
        actor,
        status: parse_status(&status)?,
        created_at,
        updated_at,
    })
}

fn row_value<T: FromValue>(row: &Row, index: usize, field: &str) -> Result<T, String> {
    row.get(index)
        .ok_or_else(|| format!("missing mysql field '{}'", field))
}

fn parse_layer(v: &str) -> Result<LoomLayer, String> { v.parse() }
fn parse_scope(v: &str) -> Result<LoomScope, String> { v.parse() }
fn parse_status(v: &str) -> Result<NoteStatus, String> { v.parse() }

fn to_sql_err<E: std::fmt::Display>(e: E) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        0,
        rusqlite::types::Type::Text,
        Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            e.to_string(),
        )),
    )
}
