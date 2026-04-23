use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum FlowAction {
    Create {
        name: String,
        #[arg(long, value_name = "DESCRIPTION")]
        ai: Option<String>,
    },
    List,
    Show {
        id: String,
    },
    #[command(
        about = "Run one saved API flow",
        after_help = "Examples:\n  infynon weave flow run auth-refresh\n  infynon weave flow run auth-refresh --format json --no-input\n  infynon weave flow run auth-refresh --format junit --no-input\n\nExit codes:\n  0   all assertions passed\n  20  flow execution failed\n  21  required runtime input missing in non-interactive mode\n  22  invalid flow definition or missing node"
    )]
    Run {
        id: String,
        #[arg(long, value_name = "URL")]
        base_url: Option<String>,
        #[arg(long, value_name = "KEY=VALUE", value_parser = crate::cli::args::parse_key_val)]
        set: Vec<(String, String)>,
        #[arg(long, value_name = "FORMAT")]
        format: Option<String>,
        #[arg(long, value_name = "FORMAT")]
        output: Option<String>,
        #[arg(long)]
        no_input: bool,
    },
    #[command(
        about = "Run all saved API flows",
        after_help = "Examples:\n  infynon weave flow run-all\n  infynon weave flow run-all --format json --no-input\n  infynon weave flow run-all --format junit --no-input\n\nExit codes:\n  0   all flows passed\n  20  at least one flow failed\n  21  runtime input missing in non-interactive mode\n  22  invalid flow definition or missing node"
    )]
    RunAll {
        #[arg(long, value_name = "URL")]
        base_url: Option<String>,
        #[arg(long, value_name = "KEY=VALUE", value_parser = crate::cli::args::parse_key_val)]
        set: Vec<(String, String)>,
        #[arg(long, value_name = "FORMAT")]
        format: Option<String>,
        #[arg(long, value_name = "FORMAT")]
        output: Option<String>,
        #[arg(long)]
        no_input: bool,
    },
    Remove {
        id: String,
    },
    Merge {
        flow1: String,
        flow2: String,
        #[arg(long, value_name = "NODE_ID")]
        join_at: String,
        #[arg(long, default_value = "merged-flow")]
        name: String,
    },
}
