use crate::trace::cli::{
    GraphAction, GraphEdgeAction, GraphEntityAction, NoteAction, SourceAction, TraceAction,
};
use crate::trace::storage;
use crate::trace::types::{
    EntityKind, KgEdge, KgEntity, NoteStatus, RelationType, SourceKind, SyncDirection, TraceLayer,
    TraceNote, TraceScope, TraceSource,
};
use crate::tui::logger::Logger;
use chrono::Utc;
use std::collections::HashMap;
use std::str::FromStr;

const EXIT_TRACE_INVALID_INPUT: i32 = 30;
const EXIT_TRACE_STORAGE_ERROR: i32 = 31;

pub fn execute(action: TraceAction) -> i32 {
    match action {
        TraceAction::Overview => {
            Logger::trace_overview();
            0
        }
        TraceAction::Init { repo, owner, user } => {
            cmd_init(repo.as_deref(), owner.as_deref(), user.as_deref());
            0
        }
        TraceAction::Source { action } => execute_source(action),
        TraceAction::Note { action } => execute_note(action),
        TraceAction::Retrieve {
            layer,
            scope,
            target,
            author,
            file,
            tag,
            format,
            limit,
        } => cmd_retrieve(
            layer.as_deref(),
            scope.as_deref(),
            target.as_deref(),
            author.as_deref(),
            file.as_deref(),
            tag.as_deref(),
            &format,
            limit,
        ),
        TraceAction::Sync { source, direction } => cmd_sync(source.as_deref(), &direction),
        TraceAction::Compact => {
            cmd_compact();
            0
        }
        TraceAction::Schema { backend } => {
            cmd_schema(&backend);
            0
        }
        TraceAction::Tui => {
            crate::trace::tui::run();
            0
        }
        TraceAction::Graph { action } => {
            execute_graph(action);
            0
        }
    }
}

include!("commands/source.rs");
include!("commands/notes.rs");
include!("commands/graph.rs");
