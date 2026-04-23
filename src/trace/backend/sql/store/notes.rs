pub fn upsert_note_sqlite(conn: &Connection, note: &TraceNote) -> Result<(), String> {
    conn.execute(
        "INSERT INTO trace_notes (id,title,body,layer,scope,target,files_json,tags_json,related_pr,author,actor,status,created_at,updated_at)
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

pub fn upsert_note_postgres(client: &mut Client, note: &TraceNote) -> Result<(), String> {
    client
        .execute(
            "INSERT INTO trace_notes (id,title,body,layer,scope,target,files_json,tags_json,related_pr,author,actor,status,created_at,updated_at)
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

pub fn upsert_note_mysql(conn: &mut mysql::PooledConn, note: &TraceNote) -> Result<(), String> {
    conn.exec_drop(
        "INSERT INTO trace_notes (id,title,body,layer,scope,target,files_json,tags_json,related_pr,author,actor,status,created_at,updated_at)
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
    conn.execute("DELETE FROM trace_package_findings", [])
        .map_err(|e| e.to_string())?;
    for finding in findings {
        conn.execute(
            "INSERT INTO trace_package_findings (package_name,version,ecosystem,severity,vulnerability_id,source_file,installed_by)
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
        .execute("DELETE FROM trace_package_findings", &[])
        .map_err(|e| e.to_string())?;
    for finding in findings {
        client
            .execute(
                "INSERT INTO trace_package_findings (package_name,version,ecosystem,severity,vulnerability_id,source_file,installed_by)
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
    conn.query_drop("DELETE FROM trace_package_findings")
        .map_err(|e| e.to_string())?;
    for finding in findings {
        conn.exec_drop(
            "INSERT INTO trace_package_findings (package_name,version,ecosystem,severity,vulnerability_id,source_file,installed_by)
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

