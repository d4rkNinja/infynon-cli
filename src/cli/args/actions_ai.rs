use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum AiAction {
    Suggest {
        #[arg(long, value_name = "NODE_ID")]
        after: String,
    },
    Attach {
        #[arg(long, value_name = "NODE_ID")]
        after: String,
        #[arg(long, value_name = "FLOW_ID")]
        flow: Option<String>,
    },
    Complete { flow_id: String },
    Probe {
        flow_id: String,
        #[arg(long, value_name = "URL")]
        base_url: Option<String>,
    },
    BuildFlow {
        #[arg(long, value_delimiter = ',', value_name = "NODE_IDS")]
        nodes: Vec<String>,
        #[arg(long, default_value = "ai-generated-flow")]
        name: String,
    },
    Explain {
        flow_id: String,
        #[arg(long, default_value = "0")]
        run: usize,
    },
    Assert { node_id: String },
    Branch { node_id: String },
}
