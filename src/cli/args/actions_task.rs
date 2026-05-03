use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum TaskAction {
    /// Create a task entry in ~/.infynon/tasks.
    Create {
        id: String,
        #[arg(long)]
        mutate: bool,
        #[arg(long)]
        workspace: Option<String>,
        #[arg(long)]
        folder_name: Option<String>,
        #[arg(long)]
        agent: Option<String>,
        #[arg(long)]
        model: Option<String>,
        #[arg(long)]
        thinking: Option<String>,
        #[arg(long)]
        prompt: Option<String>,
        #[arg(long)]
        command: Option<String>,
        #[arg(long)]
        pid: Option<u32>,
        #[arg(long)]
        session_id: Option<String>,
        #[arg(long)]
        notes: Option<String>,
        #[arg(long)]
        result: Option<String>,
        #[arg(long)]
        blocked_by: Option<String>,
        #[arg(long)]
        blocked_reason: Option<String>,
        #[arg(long, default_value = "draft")]
        status: String,
    },

    /// List saved tasks.
    List {
        #[arg(long)]
        workspace: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        agent: Option<String>,
    },

    /// Show one task definition.
    Show { id: String },

    /// Update task metadata.
    Update {
        id: String,
        #[arg(long)]
        mutate: bool,
        #[arg(long)]
        workspace: Option<String>,
        #[arg(long)]
        folder_name: Option<String>,
        #[arg(long)]
        agent: Option<String>,
        #[arg(long)]
        model: Option<String>,
        #[arg(long)]
        thinking: Option<String>,
        #[arg(long)]
        prompt: Option<String>,
        #[arg(long)]
        command: Option<String>,
        #[arg(long)]
        pid: Option<u32>,
        #[arg(long)]
        session_id: Option<String>,
        #[arg(long)]
        notes: Option<String>,
        #[arg(long)]
        result: Option<String>,
        #[arg(long)]
        blocked_by: Option<String>,
        #[arg(long)]
        blocked_reason: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        parent_task_id: Option<String>,
    },

    /// Append a note to the task tracker.
    Note {
        id: String,
        #[arg(long)]
        mutate: bool,
        #[arg(long)]
        text: String,
    },

    /// Append a result update to the task tracker.
    Result {
        id: String,
        #[arg(long)]
        mutate: bool,
        #[arg(long)]
        text: String,
    },

    /// Fork a subtask from an existing task.
    Fork {
        new_id: String,
        #[arg(long)]
        from: String,
        #[arg(long)]
        mutate: bool,
        #[arg(long)]
        workspace: Option<String>,
        #[arg(long)]
        folder_name: Option<String>,
        #[arg(long)]
        agent: Option<String>,
        #[arg(long)]
        model: Option<String>,
        #[arg(long)]
        thinking: Option<String>,
        #[arg(long)]
        prompt: Option<String>,
        #[arg(long)]
        notes: Option<String>,
        #[arg(long)]
        result: Option<String>,
        #[arg(long)]
        session_id: Option<String>,
        #[arg(long)]
        blocked_by: Option<String>,
        #[arg(long)]
        blocked_reason: Option<String>,
        #[arg(long, default_value = "draft")]
        status: String,
    },

    /// Mark a task as running.
    Start {
        id: String,
        #[arg(long)]
        mutate: bool,
        #[arg(long)]
        pid: Option<u32>,
        #[arg(long)]
        session_id: Option<String>,
    },

    /// Resume an existing agent session for a task.
    Resume {
        id: String,
        #[arg(long)]
        mutate: bool,
        #[arg(long)]
        session_id: Option<String>,
        #[arg(long)]
        prompt: Option<String>,
    },

    /// Kill a task and optionally terminate its process.
    Kill {
        id: String,
        #[arg(long)]
        mutate: bool,
        #[arg(long)]
        pid: Option<u32>,
        #[arg(long)]
        reason: Option<String>,
        #[arg(long)]
        force: bool,
    },

    /// Mark a task as completed.
    Complete {
        id: String,
        #[arg(long)]
        mutate: bool,
        #[arg(long)]
        notes: Option<String>,
        #[arg(long)]
        result: Option<String>,
        #[arg(long)]
        close_terminal: bool,
        #[arg(long)]
        keep_terminal: bool,
    },

    /// Mark a task as failed.
    Fail {
        id: String,
        #[arg(long)]
        mutate: bool,
        #[arg(long)]
        reason: Option<String>,
        #[arg(long)]
        result: Option<String>,
        #[arg(long)]
        close_terminal: bool,
        #[arg(long)]
        keep_terminal: bool,
    },

    /// Remove a task definition.
    Remove {
        id: String,
        #[arg(long)]
        mutate: bool,
    },
}
