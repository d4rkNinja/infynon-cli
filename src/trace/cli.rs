use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum TraceAction {
    /// Show the Trace command surface and backend guidance.
    Overview,

    /// Initialize local Trace state for this repository.
    Init {
        /// Repo name stored in `.infynon/trace/config.toml`
        #[arg(long)]
        repo: Option<String>,
        /// Owner or team name for this memory space
        #[arg(long)]
        owner: Option<String>,
        /// Default Trace user for notes and sync ownership
        #[arg(long)]
        user: Option<String>,
    },

    /// Manage Trace database backends.
    #[command(name = "source")]
    Source {
        #[command(subcommand)]
        action: SourceAction,
    },

    /// Create, update, list, and delete Trace notes.
    #[command(name = "note")]
    Note {
        #[command(subcommand)]
        action: NoteAction,
    },

    /// Retrieve Trace notes by layer, scope, user, file, or tag.
    Retrieve {
        /// Filter by layer: canonical | team | user
        #[arg(long)]
        layer: Option<String>,
        /// Filter by scope: repo | branch | pr | file | user | session | package
        #[arg(long)]
        scope: Option<String>,
        /// Filter by target value
        #[arg(long)]
        target: Option<String>,
        /// Filter by author/user
        #[arg(long)]
        author: Option<String>,
        /// Filter by file path mention
        #[arg(long)]
        file: Option<String>,
        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,
        /// Output format: table | markdown | json
        #[arg(long, default_value = "table")]
        format: String,
        /// Limit the number of notes returned
        #[arg(long)]
        limit: Option<usize>,
    },

    /// Record a pull, push, or bidirectional sync against a configured backend.
    Sync {
        /// Source ID from `trace source add ...`
        #[arg(long)]
        source: Option<String>,
        /// Direction: pull | push | both
        #[arg(long, default_value = "both")]
        direction: String,
    },

    /// Compact stale and session-scoped notes.
    Compact,

    /// Print the uniform Trace schema for SQL or Redis backends.
    Schema {
        /// Backend family: sql | redis
        backend: String,
    },

    /// Open the Trace terminal UI.
    Tui,

    /// Knowledge graph — branch-wise entity-relationship explorer.
    #[command(name = "graph")]
    Graph {
        #[command(subcommand)]
        action: GraphAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum SourceAction {
    /// Add a Redis backend.
    /// Benefit: best for fast live retrieval, active sessions, user presence, and conflict checks.
    AddRedis {
        /// Source ID, e.g. `team-redis`
        id: String,
        /// Redis connection URL, e.g. `redis://localhost:6379/0`
        #[arg(long)]
        url: String,
        /// Logical namespace prefix for all Trace keys
        #[arg(long, default_value = "trace")]
        namespace: String,
        /// Optional notes for operators and teammates
        #[arg(long)]
        notes: Option<String>,
        /// Owner user for this backend, usually the operator or team lead
        #[arg(long)]
        user: Option<String>,
        /// Make this the default source
        #[arg(long)]
        default: bool,
    },

    /// Add a SQL backend.
    /// Benefit: best for durable storage, structured queries, reporting, and long-term canonical memory.
    AddSql {
        /// Source ID, e.g. `team-postgres`
        id: String,
        /// SQL engine: postgres | mysql | sqlite
        #[arg(long)]
        engine: String,
        /// SQL connection URL, e.g. `postgres://user:pass@host:5432/db`
        #[arg(long)]
        url: String,
        /// Optional logical database/schema name
        #[arg(long)]
        database: Option<String>,
        /// Optional username if you don't want it embedded in the URL
        #[arg(long)]
        username: Option<String>,
        /// Optional env var name containing the password/token
        #[arg(long)]
        password_env: Option<String>,
        /// Optional notes for operators and teammates
        #[arg(long)]
        notes: Option<String>,
        /// Owner user for this backend, usually the operator or team lead
        #[arg(long)]
        user: Option<String>,
        /// Make this the default source
        #[arg(long)]
        default: bool,
    },

    /// List configured Trace backends.
    List,

    /// Remove a configured Trace backend by ID.
    Remove { id: String },

    /// Set the default backend by ID.
    Default { id: String },
}

#[derive(Subcommand, Debug)]
pub enum NoteAction {
    /// Create a Trace note.
    Add {
        id: String,
        #[arg(long)]
        title: String,
        #[arg(long)]
        body: String,
        #[arg(long, default_value = "team")]
        layer: String,
        #[arg(long, default_value = "repo")]
        scope: String,
        #[arg(long, default_value = "current")]
        target: String,
        #[arg(long)]
        author: Option<String>,
        #[arg(long)]
        actor: Option<String>,
        #[arg(long, value_delimiter = ',')]
        files: Vec<String>,
        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,
        #[arg(long)]
        related_pr: Option<u64>,
    },

    /// Update a Trace note.
    Update {
        id: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        body: Option<String>,
        #[arg(long)]
        status: Option<String>,
    },

    /// Delete a Trace note.
    Remove { id: String },

    /// List Trace notes.
    List,
}

#[derive(Subcommand, Debug)]
pub enum GraphAction {
    /// Add an entity (node) to the knowledge graph.
    #[command(name = "entity")]
    Entity {
        #[command(subcommand)]
        action: GraphEntityAction,
    },

    /// Add an edge (relationship) between two entities.
    #[command(name = "edge")]
    Edge {
        #[command(subcommand)]
        action: GraphEdgeAction,
    },

    /// Show the knowledge graph for a branch.
    Show {
        /// Branch name (defaults to current git branch)
        #[arg(long)]
        branch: Option<String>,
        /// Filter by entity kind
        #[arg(long)]
        kind: Option<String>,
    },

    /// Auto-build the knowledge graph from git history, notes, and lock files.
    Build {
        /// Branch to build for (defaults to current git branch)
        #[arg(long)]
        branch: Option<String>,
        /// Build for all local branches
        #[arg(long)]
        all_branches: bool,
    },

    /// Diff knowledge graphs between two branches.
    Diff {
        /// First branch
        branch_a: String,
        /// Second branch
        branch_b: String,
    },

    /// Find the shortest path between two entities.
    Path {
        /// Source entity name
        from: String,
        /// Target entity name
        to: String,
        /// Branch (defaults to current git branch)
        #[arg(long)]
        branch: Option<String>,
    },

    /// Show all entities connected to a given entity (impact analysis).
    Impact {
        /// Entity name
        entity: String,
        /// Branch (defaults to current git branch)
        #[arg(long)]
        branch: Option<String>,
    },

    /// List entities with no connections.
    Orphans {
        /// Branch (defaults to current git branch)
        #[arg(long)]
        branch: Option<String>,
    },

    /// Export the knowledge graph.
    Export {
        /// Format: json | dot
        #[arg(long, default_value = "json")]
        format: String,
        /// Branch (defaults to current git branch)
        #[arg(long)]
        branch: Option<String>,
        /// Output file path (prints to stdout if omitted)
        #[arg(long, short)]
        output: Option<String>,
    },

    /// Import a knowledge graph from a file.
    Import {
        /// File path to import from
        file: String,
        /// Format: json (auto-detected from extension if omitted)
        #[arg(long)]
        format: Option<String>,
        /// Target branch (defaults to current git branch)
        #[arg(long)]
        branch: Option<String>,
    },

    /// Open the knowledge graph TUI.
    Tui {
        /// Branch (defaults to current git branch)
        #[arg(long)]
        branch: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum GraphEntityAction {
    /// Add an entity.
    Add {
        /// Entity name (e.g. src/auth.rs, chrono, alice)
        name: String,
        /// Entity kind: file | package | person | decision | endpoint | module | pr | branch | note | vulnerability
        #[arg(long)]
        kind: String,
        /// Branch (defaults to current git branch)
        #[arg(long)]
        branch: Option<String>,
        /// Key=value metadata pairs
        #[arg(long, value_delimiter = ',')]
        meta: Vec<String>,
    },

    /// Remove an entity and its edges.
    Remove {
        /// Entity ID or name
        id: String,
    },

    /// List entities.
    List {
        /// Filter by branch
        #[arg(long)]
        branch: Option<String>,
        /// Filter by kind
        #[arg(long)]
        kind: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum GraphEdgeAction {
    /// Add a relationship between two entities.
    Add {
        /// Source entity name or ID
        #[arg(long)]
        from: String,
        /// Target entity name or ID
        #[arg(long)]
        to: String,
        /// Relation type: depends_on | introduced_by | modified_by | affects | decided_by | relates_to | supersedes | conflicts_with | documents | exposes | owns
        #[arg(long)]
        relation: String,
        /// Edge weight 0.0-1.0
        #[arg(long, default_value = "1.0")]
        weight: f64,
        /// Branch (defaults to current git branch)
        #[arg(long)]
        branch: Option<String>,
        /// Evidence (note ID, commit hash, or description)
        #[arg(long)]
        evidence: Option<String>,
    },

    /// Remove an edge.
    Remove {
        /// Edge ID
        id: String,
    },

    /// List edges.
    List {
        /// Filter by branch
        #[arg(long)]
        branch: Option<String>,
        /// Filter by relation type
        #[arg(long)]
        relation: Option<String>,
    },
}
