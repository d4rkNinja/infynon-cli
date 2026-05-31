pub fn upsert_source_sqlite(conn: &Connection, source: &TraceSource) -> Result<(), String> {
    conn.execute(
        "INSERT INTO trace_sources (id,kind,url,enabled,owner_user,database_name,namespace,username,password_env,notes)
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

pub fn upsert_source_postgres(client: &mut Client, source: &TraceSource) -> Result<(), String> {
    client
        .execute(
            "INSERT INTO trace_sources (id,kind,url,enabled,owner_user,database_name,namespace,username,password_env,notes)
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
    source: &TraceSource,
) -> Result<(), String> {
    conn.exec_drop(
        "INSERT INTO trace_sources (id,kind,url,enabled,owner_user,database_name,namespace,username,password_env,notes)
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
