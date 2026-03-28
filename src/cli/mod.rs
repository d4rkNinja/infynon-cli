pub mod args;
pub mod commands;
pub mod features;
pub mod scan;

use std::time::Instant;

/// Execute a package-manager command, capturing its output.
///
/// - **Windows**: routes through `cmd /C <cmd>` so that `.cmd`/`.bat` wrappers
///   (npm.cmd, yarn.cmd, pnpm.cmd, gem.cmd, pip.cmd, poetry.bat, mix.bat …)
///   are resolved by the shell just like they are in a terminal.
/// - **macOS / Linux**: splits on whitespace and execs the binary directly.
///
/// Use this for fix/clean/migrate where you need to inspect the output.
/// For interactive proxy use (where the user should see live output), use `proxy_pkg_cmd`.
pub(crate) fn run_pkg_cmd(cmd: &str) -> std::io::Result<std::process::Output> {
    use std::process::Command;
    let cmd = cmd.trim();
    if cmd.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "empty command",
        ));
    }
    #[cfg(windows)]
    {
        Command::new("cmd").args(["/C", cmd]).output()
    }
    #[cfg(not(windows))]
    {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        Command::new(parts[0]).args(&parts[1..]).output()
    }
}

/// Execute a package-manager command with inherited stdio.
///
/// The child process gets the user's terminal directly, so all live output,
/// progress spinners, and prompts from the real package manager appear normally.
///
/// Use this for proxy mode where `infynon pkg` forwards to npm/cargo/pip/etc.
pub(crate) fn proxy_pkg_cmd(cmd: &str) -> std::io::Result<std::process::ExitStatus> {
    use std::process::Command;
    let cmd = cmd.trim();
    if cmd.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "empty command",
        ));
    }
    #[cfg(windows)]
    {
        Command::new("cmd").args(["/C", cmd]).status()
    }
    #[cfg(not(windows))]
    {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        Command::new(parts[0]).args(&parts[1..]).status()
    }
}

pub fn run_package_manager() -> Result<(), crate::error::types::InfynonError> {
    commands::execute_pkg_mode()
}

pub fn run_firewall(start: Instant) -> Result<(), crate::error::types::InfynonError> {
    commands::execute_firewall_mode(start)
}
