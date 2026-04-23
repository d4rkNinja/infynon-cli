use crate::cli::args::{AssertionAction, PromptAction};
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum NodeAction {
    Create {
        #[arg(long, value_name = "DESCRIPTION")]
        ai: Option<String>,
    },
    Get { id: String },
    List,
    Remove { id: String },
    Clone { id: String, new_id: String },
    Run {
        id: String,
        #[arg(long)]
        base_url: Option<String>,
        #[arg(long, value_name = "KEY=VALUE", value_parser = crate::cli::args::parse_key_val)]
        set: Vec<(String, String)>,
        #[arg(long)]
        prompt: bool,
    },
    Export {
        id: String,
        #[arg(long, default_value = "curl")]
        format: String,
        #[arg(long, value_name = "URL")]
        base_url: Option<String>,
    },
    #[command(name = "assertion")]
    Assertion {
        node_id: String,
        #[command(subcommand)]
        action: AssertionAction,
    },
    #[command(name = "prompt")]
    Prompt {
        node_id: String,
        #[command(subcommand)]
        action: PromptAction,
    },
}
