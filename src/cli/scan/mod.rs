mod agent;
mod autofix;
mod human;
mod install_gate;
mod report;
mod runner;
mod specs;
#[cfg(test)]
mod tests;
mod types;

pub use autofix::upgrade_cmd;
pub use install_gate::{check_packages_before_install, escalate_severity, tool_to_osv_ecosystem};
pub use report::severity_colored;
pub use runner::run_scan;
pub use specs::parse_pkg_spec;
pub use types::{FixLevel, OutputFormat, VulnHit};
