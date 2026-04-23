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

pub fn execute(action: TraceAction) {
    match action {
        TraceAction::Overview => Logger::trace_overview(),
        TraceAction::Init { repo, owner, user } => {
            cmd_init(repo.as_deref(), owner.as_deref(), user.as_deref())
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
        } => cmd_retrieve(
            layer.as_deref(),
            scope.as_deref(),
            target.as_deref(),
            author.as_deref(),
            file.as_deref(),
            tag.as_deref(),
        ),
        TraceAction::Sync { source, direction } => cmd_sync(source.as_deref(), &direction),
        TraceAction::Compact => cmd_compact(),
        TraceAction::Schema { backend } => cmd_schema(&backend),
        TraceAction::Tui => crate::trace::tui::run(),
        TraceAction::Graph { action } => execute_graph(action),
    }
}

include!("commands/source.rs");
include!("commands/notes.rs");
include!("commands/graph.rs");