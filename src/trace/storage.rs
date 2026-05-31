use crate::trace::types::{
    EntityKind, KgEdge, KgEntity, KgGraph, NoteStatus, PackageRisk, RelationType, SyncDirection,
    SyncRun, SyncState, TraceConfig, TraceLayer, TraceNote, TraceScope, TraceSource,
};
use crate::{engine, trace::types::SourceKind};
use chrono::Utc;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn normalize_user(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

pub fn trace_dir() -> PathBuf {
    crate::utils::project_infynon_path(&["trace"])
}

pub fn ensure_layout() -> Result<(), String> {
    for dir in [
        trace_dir(),
        trace_dir().join("notes"),
        trace_dir().join("notes").join("canonical"),
        trace_dir().join("notes").join("team"),
        trace_dir().join("notes").join("user"),
        trace_dir().join("state"),
    ] {
        fs::create_dir_all(&dir)
            .map_err(|e| format!("failed to create {}: {}", dir.display(), e))?;
    }
    Ok(())
}

pub fn config_path() -> PathBuf {
    trace_dir().join("config.toml")
}

pub fn sync_state_path() -> PathBuf {
    trace_dir().join("state").join("sync.json")
}

fn stable_trace_id(prefix: &str, raw: &str) -> String {
    format!("{}-{}", prefix, crate::utils::storage_key(raw))
}

fn storage_key(input: &str) -> String {
    crate::utils::storage_key(input)
}

fn edge_dedupe_key(source: &str, target: &str, relation: RelationType) -> String {
    format!("{}|{}|{}", source, target, relation.as_str())
}

pub fn sanitize(input: &str) -> String {
    input
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect()
}

fn remove_legacy_alias(path: &Path, legacy_path: &Path) -> Result<(), String> {
    if path != legacy_path {
        match fs::remove_file(legacy_path) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(())
}

pub fn kg_dir() -> PathBuf {
    trace_dir().join("kg")
}

pub fn ensure_kg_layout() -> Result<(), String> {
    for dir in [kg_dir(), kg_dir().join("entities"), kg_dir().join("edges")] {
        fs::create_dir_all(&dir)
            .map_err(|e| format!("failed to create {}: {}", dir.display(), e))?;
    }
    Ok(())
}

include!("storage/config.rs");
include!("storage/notes.rs");
include!("storage/schema.rs");
include!("storage/packages.rs");
include!("storage/graph.rs");
