use crate::cli::args::ApiCommands;
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
}
