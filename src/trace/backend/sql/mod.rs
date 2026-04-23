use crate::trace::types::{
    KgEdge, KgEntity, PackageRisk, SourceKind, SyncRun, TraceNote, TraceSource,
};
use mysql::{prelude::Queryable, Opts, Pool};
use postgres::{Client, NoTls};
use rusqlite::Connection;

mod store;

pub fn validate_and_prepare(source: &TraceSource) -> Result<(), String> {
    match source.kind {
        SourceKind::Sqlite => {
            let conn = sqlite_connection(source)?;
            init_sqlite(&conn)?;
            migrate_sqlite(&conn)?;
            store::upsert_source_sqlite(&conn, source)
        }
        SourceKind::Postgres => {
            let mut client = postgres_connection(source)?;
            init_postgres(&mut client)?;
            migrate_postgres(&mut client)?;
            store::upsert_source_postgres(&mut client, source)
        }
        SourceKind::Mysql => {
            let pool = mysql_pool(source)?;
            let mut conn = pool.get_conn().map_err(|e| e.to_string())?;
            init_mysql(&mut conn)?;
            migrate_mysql(&mut conn)?;
            store::upsert_source_mysql(&mut conn, source)
        }
        SourceKind::Redis => unreachable!(),
    }
}

pub fn push_all(
    source: &TraceSource,
    notes: &[TraceNote],
    package_findings: &[PackageRisk],
    sync_run: &SyncRun,
) -> Result<(), String> {
    match source.kind {
        SourceKind::Sqlite => push_all_sqlite(source, notes, package_findings, sync_run),
        SourceKind::Postgres => push_all_postgres(source, notes, package_findings, sync_run),
        SourceKind::Mysql => push_all_mysql(source, notes, package_findings, sync_run),
        SourceKind::Redis => unreachable!(),
    }
}

pub fn pull_notes(source: &TraceSource) -> Result<Vec<TraceNote>, String> {
    match source.kind {
        SourceKind::Sqlite => store::pull_notes_sqlite(&sqlite_connection(source)?),
        SourceKind::Postgres => {
            let mut client = postgres_connection(source)?;
            store::pull_notes_postgres(&mut client)
        }
        SourceKind::Mysql => {
            let pool = mysql_pool(source)?;
            let mut conn = pool.get_conn().map_err(|e| e.to_string())?;
            store::pull_notes_mysql(&mut conn)
        }
        SourceKind::Redis => unreachable!(),
    }
}

pub fn push_kg(
    source: &TraceSource,
    entities: &[KgEntity],
    edges: &[KgEdge],
) -> Result<(), String> {
    match source.kind {
        SourceKind::Sqlite => {
            let conn = sqlite_connection(source)?;
            init_sqlite(&conn)?;
            for e in entities {
                store::upsert_kg_entity_sqlite(&conn, e)?;
            }
            for e in edges {
                store::upsert_kg_edge_sqlite(&conn, e)?;
            }
            Ok(())
        }
        SourceKind::Postgres => {
            let mut client = postgres_connection(source)?;
            init_postgres(&mut client)?;
            for e in entities {
                store::upsert_kg_entity_postgres(&mut client, e)?;
            }
            for e in edges {
                store::upsert_kg_edge_postgres(&mut client, e)?;
            }
            Ok(())
        }
        SourceKind::Mysql => {
            let pool = mysql_pool(source)?;
            let mut conn = pool.get_conn().map_err(|e| e.to_string())?;
            init_mysql(&mut conn)?;
            for e in entities {
                store::upsert_kg_entity_mysql(&mut conn, e)?;
            }
            for e in edges {
                store::upsert_kg_edge_mysql(&mut conn, e)?;
            }
            Ok(())
        }
        _ => unreachable!(),
    }
}

pub fn pull_kg(source: &TraceSource) -> Result<(Vec<KgEntity>, Vec<KgEdge>), String> {
    match source.kind {
        SourceKind::Sqlite => {
            let conn = sqlite_connection(source)?;
            Ok((
                store::pull_kg_entities_sqlite(&conn)?,
                store::pull_kg_edges_sqlite(&conn)?,
            ))
        }
        SourceKind::Postgres => {
            let mut client = postgres_connection(source)?;
            Ok((
                store::pull_kg_entities_postgres(&mut client)?,
                store::pull_kg_edges_postgres(&mut client)?,
            ))
        }
        SourceKind::Mysql => {
            let pool = mysql_pool(source)?;
            let mut conn = pool.get_conn().map_err(|e| e.to_string())?;
            Ok((
                store::pull_kg_entities_mysql(&mut conn)?,
                store::pull_kg_edges_mysql(&mut conn)?,
            ))
        }
        _ => unreachable!(),
    }
}

pub fn record_sync(source: &TraceSource, run: &SyncRun) -> Result<(), String> {
    match source.kind {
        SourceKind::Sqlite => store::insert_sync_sqlite(&sqlite_connection(source)?, run),
        SourceKind::Postgres => store::insert_sync_postgres(&mut postgres_connection(source)?, run),
        SourceKind::Mysql => {
            let pool = mysql_pool(source)?;
            let mut conn = pool.get_conn().map_err(|e| e.to_string())?;
            store::insert_sync_mysql(&mut conn, run)
        }
        SourceKind::Redis => unreachable!(),
    }
}

fn push_all_sqlite(
    source: &TraceSource,
    notes: &[TraceNote],
    package_findings: &[PackageRisk],
    sync_run: &SyncRun,
) -> Result<(), String> {
    let conn = sqlite_connection(source)?;
    init_sqlite(&conn)?;
    migrate_sqlite(&conn)?;
    for note in notes {
        store::upsert_note_sqlite(&conn, note)?;
    }
    store::refresh_package_findings_sqlite(&conn, package_findings)?;
    store::insert_sync_sqlite(&conn, sync_run)
}

fn push_all_postgres(
    source: &TraceSource,
    notes: &[TraceNote],
    package_findings: &[PackageRisk],
    sync_run: &SyncRun,
) -> Result<(), String> {
    let mut client = postgres_connection(source)?;
    init_postgres(&mut client)?;
    migrate_postgres(&mut client)?;
    for note in notes {
        store::upsert_note_postgres(&mut client, note)?;
    }
    store::refresh_package_findings_postgres(&mut client, package_findings)?;
    store::insert_sync_postgres(&mut client, sync_run)
}

fn push_all_mysql(
    source: &TraceSource,
    notes: &[TraceNote],
    package_findings: &[PackageRisk],
    sync_run: &SyncRun,
) -> Result<(), String> {
    let pool = mysql_pool(source)?;
    let mut conn = pool.get_conn().map_err(|e| e.to_string())?;
    init_mysql(&mut conn)?;
    migrate_mysql(&mut conn)?;
    for note in notes {
        store::upsert_note_mysql(&mut conn, note)?;
    }
    store::refresh_package_findings_mysql(&mut conn, package_findings)?;
    store::insert_sync_mysql(&mut conn, sync_run)
}

fn sqlite_connection(source: &TraceSource) -> Result<Connection, String> {
    let path = source
        .url
        .strip_prefix("sqlite://")
        .unwrap_or(source.url.as_str());
    Connection::open(path).map_err(|e| e.to_string())
}

fn postgres_connection(source: &TraceSource) -> Result<Client, String> {
    Client::connect(&source.url, NoTls).map_err(|e| e.to_string())
}

fn mysql_pool(source: &TraceSource) -> Result<Pool, String> {
    let opts = Opts::from_url(&source.url).map_err(|e| e.to_string())?;
    Pool::new(opts).map_err(|e| e.to_string())
}

fn init_sqlite(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(&crate::trace::storage::sql_schema_for(SourceKind::Sqlite))
        .map_err(|e| e.to_string())
}

fn init_postgres(client: &mut Client) -> Result<(), String> {
    client
        .batch_execute(&crate::trace::storage::sql_schema_for(SourceKind::Postgres))
        .map_err(|e| e.to_string())
}

fn init_mysql(conn: &mut mysql::PooledConn) -> Result<(), String> {
    for stmt in crate::trace::storage::sql_schema_for(SourceKind::Mysql).split(";\n") {
        let trimmed = stmt.trim();
        if !trimmed.is_empty() {
            conn.query_drop(trimmed).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn migrate_sqlite(conn: &Connection) -> Result<(), String> {
    let columns = sqlite_columns(conn)?;
    if !columns.iter().any(|column| column == "owner_user") {
        conn.execute(
            "ALTER TABLE trace_sources ADD COLUMN owner_user TEXT NULL",
            [],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn migrate_postgres(client: &mut Client) -> Result<(), String> {
    client
        .batch_execute("ALTER TABLE trace_sources ADD COLUMN IF NOT EXISTS owner_user TEXT NULL;")
        .map_err(|e| e.to_string())
}

fn migrate_mysql(conn: &mut mysql::PooledConn) -> Result<(), String> {
    match conn.query_drop("ALTER TABLE trace_sources ADD COLUMN owner_user TEXT NULL") {
        Ok(()) => Ok(()),
        Err(error) => {
            let message = error.to_string();
            if message.contains("Duplicate column name") {
                Ok(())
            } else {
                Err(message)
            }
        }
    }
}

fn sqlite_columns(conn: &Connection) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare("PRAGMA table_info(trace_sources)")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| e.to_string())?;
    rows.map(|value| value.map_err(|e| e.to_string())).collect()
}
