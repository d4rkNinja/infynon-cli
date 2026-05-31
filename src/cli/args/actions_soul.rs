use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum SoulAction {
    /// Show the user-global soul profile.
    Show,

    /// Replace the user-global soul profile.
    Update {
        #[arg(long)]
        text: Option<String>,
        #[arg(long)]
        file: Option<String>,
    },
}
