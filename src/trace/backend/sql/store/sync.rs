pub fn insert_sync_sqlite(conn: &Connection, run: &SyncRun) -> Result<(), String> {
    conn.execute(
        "INSERT INTO trace_sync_runs (timestamp,direction,source_id,summary) VALUES (?1,?2,?3,?4)",
        rusqlite::params![
            run.timestamp,
            run.direction.as_str(),
            run.source_id,
            run.summary
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn insert_sync_postgres(client: &mut Client, run: &SyncRun) -> Result<(), String> {
    client
        .execute(
            "INSERT INTO trace_sync_runs (timestamp,direction,source_id,summary) VALUES ($1,$2,$3,$4)",
            &[&run.timestamp, &run.direction.as_str(), &run.source_id, &run.summary],
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn insert_sync_mysql(conn: &mut mysql::PooledConn, run: &SyncRun) -> Result<(), String> {
    conn.exec_drop(
        "INSERT INTO trace_sync_runs (timestamp,direction,source_id,summary) VALUES (?,?,?,?)",
        (
            &run.timestamp,
            run.direction.as_str(),
            &run.source_id,
            &run.summary,
        ),
    )
    .map_err(|e| e.to_string())
}

pub fn pull_notes_sqlite(conn: &Connection) -> Result<Vec<TraceNote>, String> {
    let mut stmt = conn
        .prepare("SELECT id,title,body,layer,scope,target,files_json,tags_json,related_pr,author,actor,status,created_at,updated_at FROM trace_notes ORDER BY updated_at DESC")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(TraceNote {
                id: row.get(0)?,
                title: row.get(1)?,
                body: row.get(2)?,
                layer: row
                    .get::<_, String>(3)?
                    .as_str()
                    .parse()
                    .map_err(to_sql_err)?,
                scope: row
                    .get::<_, String>(4)?
                    .as_str()
                    .parse()
                    .map_err(to_sql_err)?,
                target: row.get(5)?,
                files: serde_json::from_str::<Vec<String>>(&row.get::<_, String>(6)?)
                    .map_err(to_sql_err)?,
                tags: serde_json::from_str::<Vec<String>>(&row.get::<_, String>(7)?)
                    .map_err(to_sql_err)?,
                related_pr: row.get::<_, Option<i64>>(8)?.map(|v| v as u64),
                author: row.get(9)?,
                actor: row.get(10)?,
                status: row
                    .get::<_, String>(11)?
                    .as_str()
                    .parse()
                    .map_err(to_sql_err)?,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.map(|r| r.map_err(|e| e.to_string())).collect()
}

pub fn pull_notes_postgres(client: &mut Client) -> Result<Vec<TraceNote>, String> {
    let rows = client
        .query("SELECT id,title,body,layer,scope,target,files_json,tags_json,related_pr,author,actor,status,created_at,updated_at FROM trace_notes ORDER BY updated_at DESC", &[])
        .map_err(|e| e.to_string())?;
    rows.into_iter()
        .map(|row| {
            Ok(TraceNote {
                id: row.get(0),
                title: row.get(1),
                body: row.get(2),
                layer: row.get::<_, String>(3).as_str().parse()?,
                scope: row.get::<_, String>(4).as_str().parse()?,
                target: row.get(5),
                files: serde_json::from_str(&row.get::<_, String>(6)).map_err(|e| e.to_string())?,
                tags: serde_json::from_str(&row.get::<_, String>(7)).map_err(|e| e.to_string())?,
                related_pr: row.get::<_, Option<i64>>(8).map(|v| v as u64),
                author: row.get(9),
                actor: row.get(10),
                status: row.get::<_, String>(11).as_str().parse()?,
                created_at: row.get(12),
                updated_at: row.get(13),
            })
        })
        .collect()
}

pub fn pull_notes_mysql(conn: &mut mysql::PooledConn) -> Result<Vec<TraceNote>, String> {
    let rows: Vec<Row> = conn
        .query("SELECT id,title,body,layer,scope,target,files_json,tags_json,related_pr,author,actor,status,created_at,updated_at FROM trace_notes ORDER BY updated_at DESC")
        .map_err(|e| e.to_string())?;
    rows.into_iter().map(note_from_mysql_row).collect()
}

fn note_from_mysql_row(row: Row) -> Result<TraceNote, String> {
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

    Ok(TraceNote {
        id,
        title,
        body,
        layer: layer.parse()?,
        scope: scope.parse()?,
        target,
        files: serde_json::from_str(&files_json).map_err(|e| e.to_string())?,
        tags: serde_json::from_str(&tags_json).map_err(|e| e.to_string())?,
        related_pr,
        author,
        actor,
        status: status.parse()?,
        created_at,
        updated_at,
    })
}

fn row_value<T: FromValue>(row: &Row, index: usize, field: &str) -> Result<T, String> {
    row.get(index)
        .ok_or_else(|| format!("missing mysql field '{}'", field))
}

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
