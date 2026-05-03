use crate::cli::args::{ApiCommands, NinjaAction, SoulAction, TaskAction, WorkspaceAction};
use crate::trace::cli::TraceAction;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "infynon",
    version,
    about = "INFYNON - package intelligence, API flow testing, and shared coding memory",
    styles = crate::cli::args::get_styles()
)]
pub struct RootArgs {
    #[command(subcommand)]
    pub command: Option<RootCommands>,
}

#[derive(Subcommand, Debug)]
pub enum RootCommands {
    #[command(name = "pkg")]
    Pkg {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "weave")]
    Weave {
        #[command(subcommand)]
        action: ApiCommands,
    },
    #[command(name = "trace")]
    Trace {
        #[command(subcommand)]
        action: TraceAction,
    },
    #[command(name = "workspace")]
    Workspace {
        #[command(subcommand)]
        action: WorkspaceAction,
    },
    #[command(name = "task")]
    Task {
        #[command(subcommand)]
        action: TaskAction,
    },
    #[command(name = "soul")]
    Soul {
        #[command(subcommand)]
        action: SoulAction,
    },
    #[command(name = "ninja", hide = true)]
    Ninja {
        #[command(subcommand)]
        action: NinjaAction,
    },
    #[command(name = "coding", hide = true)]
    Coding {
        #[command(subcommand)]
        action: NinjaAction,
    },
}
