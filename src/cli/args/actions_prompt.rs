use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum PromptAction {
    List,
    Add {
        var: String,
        #[arg(long, default_value = "")]
        label: String,
        #[arg(long)]
        secret: bool,
        #[arg(long)]
        default: Option<String>,
        #[arg(long = "type", default_value = "text")]
        prompt_type: String,
        #[arg(long)]
        options: Option<String>,
    },
    Remove { index: usize },
}
