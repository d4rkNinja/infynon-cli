use crate::trace::types::{TraceNote, TraceSource, PackageRisk, SourceKind, SyncDirection, SyncRun, KgEntity, KgEdge};

mod redis;
mod sql;

pub fn validate_and_prepare(source: &TraceSource) -> Result<(), String> {
    match source.kind {
        SourceKind::Redis => redis::validate_and_prepare(source),
        SourceKind::Postgres | SourceKind::Mysql | SourceKind::Sqlite => {
            sql::validate_and_prepare(source)
        }
    }
}

pub fn push_all(
    source: &TraceSource,
    notes: &[TraceNote],
    package_findings: &[PackageRisk],
    sync_run: &SyncRun,
) -> Result<(), String> {
    match source.kind {
        SourceKind::Redis => redis::push_all(source, notes, package_findings, sync_run),
        SourceKind::Postgres | SourceKind::Mysql | SourceKind::Sqlite => {
            sql::push_all(source, notes, package_findings, sync_run)
        }
    }
}

pub fn pull_notes(source: &TraceSource) -> Result<Vec<TraceNote>, String> {
    match source.kind {
        SourceKind::Redis => redis::pull_notes(source),
        SourceKind::Postgres | SourceKind::Mysql | SourceKind::Sqlite => sql::pull_notes(source),
    }
}

pub fn push_kg(source: &TraceSource, entities: &[KgEntity], edges: &[KgEdge]) -> Result<(), String> {
    match source.kind {
        SourceKind::Redis => redis::push_kg(source, entities, edges),
        SourceKind::Postgres | SourceKind::Mysql | SourceKind::Sqlite => {
            sql::push_kg(source, entities, edges)
        }
    }
}

pub fn pull_kg(source: &TraceSource) -> Result<(Vec<KgEntity>, Vec<KgEdge>), String> {
    match source.kind {
        SourceKind::Redis => redis::pull_kg(source),
        SourceKind::Postgres | SourceKind::Mysql | SourceKind::Sqlite => sql::pull_kg(source),
    }
}

pub fn record_sync(
    source: &TraceSource,
    direction: SyncDirection,
    summary: &str,
) -> Result<(), String> {
    let run = SyncRun {
        timestamp: chrono::Utc::now().to_rfc3339(),
        direction,
        source_id: Some(source.id.clone()),
        summary: summary.to_string(),
    };
    match source.kind {
        SourceKind::Redis => redis::record_sync(source, &run),
        SourceKind::Postgres | SourceKind::Mysql | SourceKind::Sqlite => {
            sql::record_sync(source, &run)
        }
    }
}
