use crate::cli::args::{AiAction, EnvAction, FlowAction, NodeAction};
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum ApiCommands {
    #[command(name = "tui")]
    Tui {
        #[arg(value_name = "FLOW_ID")]
        flow_id: Option<String>,
    },
    #[command(name = "node")]
    Node {
        #[command(subcommand)]
        action: NodeAction,
    },
    #[command(name = "flow")]
    Flow {
        #[command(subcommand)]
        action: FlowAction,
    },
    #[command(name = "attach")]
    Attach {
        from: String,
        to: String,
        #[arg(long, value_delimiter = ',', value_name = "VAR,...")]
        carry: Vec<String>,
        #[arg(long, value_name = "EXPR")]
        condition: Option<String>,
        #[arg(long)]
        ai: bool,
    },
    #[command(name = "detach")]
    Detach { from: String, to: String },
    #[command(name = "ai")]
    Ai {
        #[command(subcommand)]
        action: AiAction,
    },
    #[command(name = "env")]
    Env {
        #[command(subcommand)]
        action: EnvAction,
    },
    Validate,
    #[command(name = "import")]
    Import {
        spec: String,
        #[arg(long, value_name = "FLOW_NAME")]
        flow: Option<String>,
        #[arg(long, value_name = "URL")]
        base_url: Option<String>,
        #[arg(long, value_name = "PREFIX")]
        prefix: Option<String>,
        #[arg(long)]
        dry_run: bool,
    },
}
