fn kg_entity_fields(entity: &KgEntity) -> Result<(String, String), String> {
    let metadata_json = serde_json::to_string(&entity.metadata).map_err(|e| e.to_string())?;
    Ok((entity.kind.as_str().to_owned(), metadata_json))
}

/// Parses raw column values into a `KgEntity`.
fn kg_entity_from_row(
    id: String,
    kind_str: String,
    name: String,
    metadata_json: String,
    branch: String,
    created_at: String,
    updated_at: String,
) -> Result<KgEntity, String> {
    Ok(KgEntity {
        id,
        kind: EntityKind::from_str(&kind_str).map_err(|e| e.to_string())?,
        name,
        metadata: serde_json::from_str(&metadata_json).map_err(|e| e.to_string())?,
        branch,
        created_at,
        updated_at,
    })
}

/// Parses raw column values into a `KgEdge`.
#[allow(clippy::too_many_arguments)]
fn kg_edge_from_row(
    id: String,
    source: String,
    target: String,
    relation_str: String,
    weight: f64,
    branch: String,
    evidence: String,
    created_at: String,
) -> Result<KgEdge, String> {
    Ok(KgEdge {
        id,
        source,
        target,
        relation: RelationType::from_str(&relation_str).map_err(|e| e.to_string())?,
        weight,
        branch,
        evidence,
        created_at,
    })
}

// ── Knowledge Graph: Entities ──────────────────────────────────────────

pub fn upsert_kg_entity_sqlite(conn: &Connection, entity: &KgEntity) -> Result<(), String> {
    let (kind_str, metadata_json) = kg_entity_fields(entity)?;
    conn.execute(
        "INSERT INTO trace_kg_entities (id,kind,name,metadata_json,branch,created_at,updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7)
         ON CONFLICT(id) DO UPDATE SET kind=excluded.kind, name=excluded.name,
         metadata_json=excluded.metadata_json, branch=excluded.branch,
         created_at=excluded.created_at, updated_at=excluded.updated_at",
        rusqlite::params![
            entity.id,
            kind_str,
            entity.name,
            metadata_json,
            entity.branch,
            entity.created_at,
            entity.updated_at
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn upsert_kg_entity_postgres(client: &mut Client, entity: &KgEntity) -> Result<(), String> {
    let (kind_str, metadata_json) = kg_entity_fields(entity)?;
    client
        .execute(
            "INSERT INTO trace_kg_entities (id,kind,name,metadata_json,branch,created_at,updated_at)
             VALUES ($1,$2,$3,$4,$5,$6,$7)
             ON CONFLICT (id) DO UPDATE SET kind=EXCLUDED.kind, name=EXCLUDED.name,
             metadata_json=EXCLUDED.metadata_json, branch=EXCLUDED.branch,
             created_at=EXCLUDED.created_at, updated_at=EXCLUDED.updated_at",
            &[
                &entity.id, &kind_str, &entity.name, &metadata_json,
                &entity.branch, &entity.created_at, &entity.updated_at,
            ],
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn upsert_kg_entity_mysql(
    conn: &mut mysql::PooledConn,
    entity: &KgEntity,
) -> Result<(), String> {
    let (kind_str, metadata_json) = kg_entity_fields(entity)?;
    conn.exec_drop(
        "INSERT INTO trace_kg_entities (id,kind,name,metadata_json,branch,created_at,updated_at)
         VALUES (:id,:kind,:name,:metadata_json,:branch,:created_at,:updated_at)
         ON DUPLICATE KEY UPDATE kind=VALUES(kind), name=VALUES(name),
         metadata_json=VALUES(metadata_json), branch=VALUES(branch),
         created_at=VALUES(created_at), updated_at=VALUES(updated_at)",
        params! {
            "id" => &entity.id,
            "kind" => &kind_str,
            "name" => &entity.name,
            "metadata_json" => &metadata_json,
            "branch" => &entity.branch,
            "created_at" => &entity.created_at,
            "updated_at" => &entity.updated_at,
        },
    )
    .map_err(|e| e.to_string())
}

pub fn pull_kg_entities_sqlite(conn: &Connection) -> Result<Vec<KgEntity>, String> {
    let mut stmt = conn
        .prepare("SELECT id,kind,name,metadata_json,branch,created_at,updated_at FROM trace_kg_entities ORDER BY updated_at DESC")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            kg_entity_from_row(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
            )
            .map_err(to_sql_err)
        })
        .map_err(|e| e.to_string())?;
    rows.map(|r| r.map_err(|e| e.to_string())).collect()
}

pub fn pull_kg_entities_postgres(client: &mut Client) -> Result<Vec<KgEntity>, String> {
    let rows = client
        .query("SELECT id,kind,name,metadata_json,branch,created_at,updated_at FROM trace_kg_entities ORDER BY updated_at DESC", &[])
        .map_err(|e| e.to_string())?;
    rows.into_iter()
        .map(|row| {
            kg_entity_from_row(
                row.get(0),
                row.get(1),
                row.get(2),
                row.get(3),
                row.get(4),
                row.get(5),
                row.get(6),
            )
        })
        .collect()
}

pub fn pull_kg_entities_mysql(conn: &mut mysql::PooledConn) -> Result<Vec<KgEntity>, String> {
    let rows: Vec<Row> = conn
        .query("SELECT id,kind,name,metadata_json,branch,created_at,updated_at FROM trace_kg_entities ORDER BY updated_at DESC")
        .map_err(|e| e.to_string())?;
    rows.into_iter()
        .map(|row| {
            kg_entity_from_row(
                row_value(&row, 0, "id")?,
                row_value(&row, 1, "kind")?,
                row_value(&row, 2, "name")?,
                row_value(&row, 3, "metadata_json")?,
                row_value(&row, 4, "branch")?,
                row_value(&row, 5, "created_at")?,
                row_value(&row, 6, "updated_at")?,
            )
        })
        .collect()
}

// ── Knowledge Graph: Edges ─────────────────────────────────────────────

pub fn upsert_kg_edge_sqlite(conn: &Connection, edge: &KgEdge) -> Result<(), String> {
    conn.execute(
        "INSERT INTO trace_kg_edges (id,source_entity,target_entity,relation,weight,branch,evidence,created_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8)
         ON CONFLICT(id) DO UPDATE SET source_entity=excluded.source_entity, target_entity=excluded.target_entity,
         relation=excluded.relation, weight=excluded.weight, branch=excluded.branch,
         evidence=excluded.evidence, created_at=excluded.created_at",
        rusqlite::params![
            edge.id, edge.source, edge.target, edge.relation.as_str(),
            edge.weight, edge.branch, edge.evidence, edge.created_at
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn upsert_kg_edge_postgres(client: &mut Client, edge: &KgEdge) -> Result<(), String> {
    client
        .execute(
            "INSERT INTO trace_kg_edges (id,source_entity,target_entity,relation,weight,branch,evidence,created_at)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
             ON CONFLICT (id) DO UPDATE SET source_entity=EXCLUDED.source_entity, target_entity=EXCLUDED.target_entity,
             relation=EXCLUDED.relation, weight=EXCLUDED.weight, branch=EXCLUDED.branch,
             evidence=EXCLUDED.evidence, created_at=EXCLUDED.created_at",
            &[
                &edge.id, &edge.source, &edge.target, &edge.relation.as_str(),
                &edge.weight, &edge.branch, &edge.evidence, &edge.created_at,
            ],
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn upsert_kg_edge_mysql(conn: &mut mysql::PooledConn, edge: &KgEdge) -> Result<(), String> {
    conn.exec_drop(
        "INSERT INTO trace_kg_edges (id,source_entity,target_entity,relation,weight,branch,evidence,created_at)
         VALUES (:id,:source_entity,:target_entity,:relation,:weight,:branch,:evidence,:created_at)
         ON DUPLICATE KEY UPDATE source_entity=VALUES(source_entity), target_entity=VALUES(target_entity),
         relation=VALUES(relation), weight=VALUES(weight), branch=VALUES(branch),
         evidence=VALUES(evidence), created_at=VALUES(created_at)",
        params! {
            "id" => &edge.id,
            "source_entity" => &edge.source,
            "target_entity" => &edge.target,
            "relation" => edge.relation.as_str(),
            "weight" => edge.weight,
            "branch" => &edge.branch,
            "evidence" => &edge.evidence,
            "created_at" => &edge.created_at,
        },
    )
    .map_err(|e| e.to_string())
}

pub fn pull_kg_edges_sqlite(conn: &Connection) -> Result<Vec<KgEdge>, String> {
    let mut stmt = conn
        .prepare("SELECT id,source_entity,target_entity,relation,weight,branch,evidence,created_at FROM trace_kg_edges ORDER BY created_at DESC")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            kg_edge_from_row(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
            )
            .map_err(to_sql_err)
        })
        .map_err(|e| e.to_string())?;
    rows.map(|r| r.map_err(|e| e.to_string())).collect()
}

pub fn pull_kg_edges_postgres(client: &mut Client) -> Result<Vec<KgEdge>, String> {
    let rows = client
        .query("SELECT id,source_entity,target_entity,relation,weight,branch,evidence,created_at FROM trace_kg_edges ORDER BY created_at DESC", &[])
        .map_err(|e| e.to_string())?;
    rows.into_iter()
        .map(|row| {
            kg_edge_from_row(
                row.get(0),
                row.get(1),
                row.get(2),
                row.get(3),
                row.get(4),
                row.get(5),
                row.get(6),
                row.get(7),
            )
        })
        .collect()
}

pub fn pull_kg_edges_mysql(conn: &mut mysql::PooledConn) -> Result<Vec<KgEdge>, String> {
    let rows: Vec<Row> = conn
        .query("SELECT id,source_entity,target_entity,relation,weight,branch,evidence,created_at FROM trace_kg_edges ORDER BY created_at DESC")
        .map_err(|e| e.to_string())?;
    rows.into_iter()
        .map(|row| {
            kg_edge_from_row(
                row_value(&row, 0, "id")?,
                row_value(&row, 1, "source_entity")?,
                row_value(&row, 2, "target_entity")?,
                row_value(&row, 3, "relation")?,
                row_value(&row, 4, "weight")?,
                row_value(&row, 5, "branch")?,
                row_value(&row, 6, "evidence")?,
                row_value(&row, 7, "created_at")?,
            )
        })
        .collect()
}
