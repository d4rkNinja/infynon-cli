pub fn supported_schema_sql() -> String {
    sql_schema_for(SourceKind::Sqlite)
}

pub fn sql_schema_for(kind: SourceKind) -> String {
    let (id_auto, ts_default, index_prefix, pk_text, indexed_text, text_type) = match kind {
        SourceKind::Postgres => (
            "BIGSERIAL PRIMARY KEY",
            "TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP",
            "CREATE INDEX IF NOT EXISTS",
            "TEXT",
            "TEXT",
            "TEXT",
        ),
        SourceKind::Mysql => (
            "BIGINT PRIMARY KEY AUTO_INCREMENT",
            "TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP",
            "CREATE INDEX",
            "VARCHAR(191)",
            "VARCHAR(191)",
            "TEXT",
        ),
        _ => (
            "INTEGER PRIMARY KEY AUTOINCREMENT",
            "TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP",
            "CREATE INDEX IF NOT EXISTS",
            "TEXT",
            "TEXT",
            "TEXT",
        ),
    };
    let mut sql = format!(
        "
CREATE TABLE IF NOT EXISTS trace_sources (
  id {pk_text} PRIMARY KEY,
  kind {indexed_text} NOT NULL,
  url {text_type} NOT NULL,
  enabled BOOLEAN NOT NULL DEFAULT TRUE,
  owner_user {indexed_text} NULL,
  database_name {indexed_text} NULL,
  namespace {indexed_text} NULL,
  username {indexed_text} NULL,
  password_env {indexed_text} NULL,
  notes {text_type} NULL,
  created_at {ts_default}
);

CREATE TABLE IF NOT EXISTS trace_notes (
  id {pk_text} PRIMARY KEY,
  title {text_type} NOT NULL,
  body {text_type} NOT NULL,
  layer {indexed_text} NOT NULL,
  scope {indexed_text} NOT NULL,
  target {indexed_text} NOT NULL,
  files_json {text_type} NOT NULL,
  tags_json {text_type} NOT NULL,
  related_pr BIGINT NULL,
  author {indexed_text} NOT NULL,
  actor {indexed_text} NULL,
  status {indexed_text} NOT NULL,
  created_at TIMESTAMP NOT NULL,
  updated_at TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS trace_sync_runs (
  id {id_auto},
  timestamp {indexed_text} NOT NULL,
  direction {indexed_text} NOT NULL,
  source_id {indexed_text} NULL,
  summary {text_type} NOT NULL
);

CREATE TABLE IF NOT EXISTS trace_package_findings (
  id {id_auto},
  package_name {indexed_text} NOT NULL,
  version {indexed_text} NOT NULL,
  ecosystem {indexed_text} NOT NULL,
  severity {indexed_text} NOT NULL,
  vulnerability_id {indexed_text} NOT NULL,
  source_file {text_type} NOT NULL,
  installed_by {indexed_text} NULL,
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
  id {pk_text} PRIMARY KEY,
  kind {indexed_text} NOT NULL,
  name {indexed_text} NOT NULL,
  metadata_json {text_type} NOT NULL,
  branch {indexed_text} NOT NULL DEFAULT 'main',
  created_at TIMESTAMP NOT NULL,
  updated_at TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS trace_kg_edges (
  id {pk_text} PRIMARY KEY,
  source_entity {indexed_text} NOT NULL,
  target_entity {indexed_text} NOT NULL,
  relation {indexed_text} NOT NULL,
  weight REAL NOT NULL DEFAULT 1.0,
  branch {indexed_text} NOT NULL DEFAULT 'main',
  evidence {text_type} NOT NULL,
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
