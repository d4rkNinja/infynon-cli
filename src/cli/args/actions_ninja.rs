use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum NinjaAction {
    /// Open the INFYNON workspace/task management TUI.
    #[command(name = "tui", hide = true)]
    Tui,

    /// Run the project-local Codex launcher command.
    #[command(name = "codex", hide = true)]
    Codex {
        #[arg(
            long,
            default_value_t = false,
            default_missing_value = "true",
            num_args = 0..=1,
            require_equals = false
        )]
        background: bool,
        #[arg(long)]
        cwd: Option<String>,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Run the project-local Claude launcher command.
    #[command(name = "claude", hide = true)]
    Claude {
        #[arg(
            long,
            default_value_t = false,
            default_missing_value = "true",
            num_args = 0..=1,
            require_equals = false
        )]
        background: bool,
        #[arg(long)]
        cwd: Option<String>,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Run the project-local Gemini launcher command.
    #[command(name = "gemini", hide = true)]
    Gemini {
        #[arg(
            long,
            default_value_t = false,
            default_missing_value = "true",
            num_args = 0..=1,
            require_equals = false
        )]
        background: bool,
        #[arg(long)]
        cwd: Option<String>,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}
