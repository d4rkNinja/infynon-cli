use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum FlowAction {
    Create {
        name: String,
        #[arg(long, value_name = "DESCRIPTION")]
        ai: Option<String>,
    },
    List,
    Show { id: String },
    Run {
        id: String,
        #[arg(long, value_name = "URL")]
        base_url: Option<String>,
        #[arg(long, value_name = "KEY=VALUE", value_parser = crate::cli::args::parse_key_val)]
        set: Vec<(String, String)>,
        #[arg(long, value_name = "FORMAT")]
        output: Option<String>,
    },
    RunAll {
        #[arg(long, value_name = "URL")]
        base_url: Option<String>,
        #[arg(long, value_name = "KEY=VALUE", value_parser = crate::cli::args::parse_key_val)]
        set: Vec<(String, String)>,
        #[arg(long, value_name = "FORMAT")]
        output: Option<String>,
    },
    Remove { id: String },
    Merge {
        flow1: String,
        flow2: String,
        #[arg(long, value_name = "NODE_ID")]
        join_at: String,
        #[arg(long, default_value = "merged-flow")]
        name: String,
    },
}
