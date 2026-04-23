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

