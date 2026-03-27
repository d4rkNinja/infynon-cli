pub mod args;
pub mod commands;
pub mod features;
pub mod scan;

use std::time::Instant;

pub fn run_package_manager() -> Result<(), crate::error::types::InfynonError> {
    commands::execute_pkg_mode()
}

pub fn run_firewall(start: Instant) -> Result<(), crate::error::types::InfynonError> {
    commands::execute_firewall_mode(start)
}
