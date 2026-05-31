use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum DoctorAction {
    /// Diagnose npm/global wrapper installation issues.
    #[command(name = "npm")]
    Npm,
}
