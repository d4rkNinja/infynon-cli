use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum WorkspaceAction {
    /// Create a user-global workspace entry in ~/.infynon.
    Create {
        name: String,
        #[arg(long)]
        mutate: bool,
        #[arg(long)]
        folder_name: Option<String>,
        #[arg(long)]
        path: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        default: bool,
        #[arg(long)]
        lite_model: Option<String>,
        #[arg(long)]
        lite_thinking: Option<String>,
        #[arg(long)]
        frontier_model: Option<String>,
        #[arg(long)]
        frontier_thinking: Option<String>,
        #[arg(long)]
        highest_frontier_model: Option<String>,
        #[arg(long)]
        highest_frontier_thinking: Option<String>,
        #[arg(long)]
        super_lite_model: Option<String>,
        #[arg(long)]
        super_lite_thinking: Option<String>,
    },

    /// List user-global workspaces.
    List,

    /// Show one workspace definition.
    Show { name: String },

    /// Show the user-global INFYNON agent root folder.
    #[command(name = "agent-root-show")]
    AgentRootShow,

    /// Set the user-global INFYNON agent root folder.
    #[command(name = "agent-root-set")]
    AgentRootSet {
        #[arg(long)]
        mutate: bool,
        #[arg(long)]
        path: String,
    },

    /// Update an existing workspace.
    Update {
        name: String,
        #[arg(long)]
        mutate: bool,
        #[arg(long)]
        folder_name: Option<String>,
        #[arg(long)]
        path: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        default: bool,
        #[arg(long)]
        lite_model: Option<String>,
        #[arg(long)]
        lite_thinking: Option<String>,
        #[arg(long)]
        frontier_model: Option<String>,
        #[arg(long)]
        frontier_thinking: Option<String>,
        #[arg(long)]
        highest_frontier_model: Option<String>,
        #[arg(long)]
        highest_frontier_thinking: Option<String>,
        #[arg(long)]
        super_lite_model: Option<String>,
        #[arg(long)]
        super_lite_thinking: Option<String>,
    },

    /// Add a folder entry to a workspace.
    AddFolder {
        name: String,
        #[arg(long)]
        mutate: bool,
        #[arg(long)]
        folder_name: String,
        #[arg(long)]
        path: String,
    },

    /// Remove a folder entry from a workspace.
    RemoveFolder {
        name: String,
        #[arg(long)]
        mutate: bool,
        #[arg(long)]
        folder_name: String,
    },

    /// Remove a workspace definition.
    Remove {
        name: String,
        #[arg(long)]
        mutate: bool,
    },
}
