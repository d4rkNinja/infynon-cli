use crate::loom::types::{LoomNote, LoomSource, PackageRisk, SourceKind, SyncDirection, SyncRun};

mod redis;
mod sql;

pub fn validate_and_prepare(source: &LoomSource) -> Result<(), String> {
    match source.kind {
        SourceKind::Redis => redis::validate_and_prepare(source),
        SourceKind::Postgres | SourceKind::Mysql | SourceKind::Sqlite => {
            sql::validate_and_prepare(source)
        }
    }
}

pub fn push_all(
    source: &LoomSource,
    notes: &[LoomNote],
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

pub fn pull_notes(source: &LoomSource) -> Result<Vec<LoomNote>, String> {
    match source.kind {
        SourceKind::Redis => redis::pull_notes(source),
        SourceKind::Postgres | SourceKind::Mysql | SourceKind::Sqlite => sql::pull_notes(source),
    }
}

pub fn record_sync(
    source: &LoomSource,
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
