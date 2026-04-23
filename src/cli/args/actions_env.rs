use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum EnvAction {
    List,
    Set { key: String, value: String },
    Delete { key: String },
    Get {
        key: String,
        #[arg(long)]
        reveal: bool,
    },
}
