use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum LoomAction {
    /// Show the Loom command surface and backend guidance.
    Overview,

    /// Initialize local Loom state for this repository.
    Init {
        /// Repo name stored in `.infynon/loom/config.toml`
        #[arg(long)]
        repo: Option<String>,
        /// Owner or team name for this memory space
        #[arg(long)]
        owner: Option<String>,
        /// Default Loom user for notes and sync ownership
        #[arg(long)]
        user: Option<String>,
    },

    /// Manage Loom database backends.
    #[command(name = "source")]
    Source {
        #[command(subcommand)]
        action: SourceAction,
    },

    /// Create, update, list, and delete Loom notes.
    #[command(name = "note")]
    Note {
        #[command(subcommand)]
        action: NoteAction,
    },

    /// Retrieve Loom notes by layer, scope, user, file, or tag.
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
    },

    /// Record a pull, push, or bidirectional sync against a configured backend.
    Sync {
        /// Source ID from `loom source add ...`
        #[arg(long)]
        source: Option<String>,
        /// Direction: pull | push | both
        #[arg(long, default_value = "both")]
        direction: String,
    },

    /// Compact stale and session-scoped notes.
    Compact,

    /// Print the uniform Loom schema for SQL or Redis backends.
    Schema {
        /// Backend family: sql | redis
        backend: String,
    },

    /// Open the Loom terminal UI.
    Tui,
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
        /// Logical namespace prefix for all Loom keys
        #[arg(long, default_value = "loom")]
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

    /// List configured Loom backends.
    List,

    /// Remove a configured Loom backend by ID.
    Remove {
        id: String,
    },

    /// Set the default backend by ID.
    Default {
        id: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum NoteAction {
    /// Create a Loom note.
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

    /// Update a Loom note.
    Update {
        id: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        body: Option<String>,
        #[arg(long)]
        status: Option<String>,
    },

    /// Delete a Loom note.
    Remove {
        id: String,
    },

    /// List Loom notes.
    List,
}
