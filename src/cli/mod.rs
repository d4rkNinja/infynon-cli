pub mod args;
pub mod commands;
pub mod features;
pub mod scan;
pub mod validate;

/// Build a platform-appropriate `Command` for a package-manager shell string.
///
/// Windows routes through `cmd /C` so .cmd/.bat wrappers resolve correctly.
/// Unix splits on whitespace and execs the binary directly.
fn make_pkg_command(cmd: &str) -> std::io::Result<std::process::Command> {
    use std::process::Command;
    let cmd = cmd.trim();
    if cmd.is_empty() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "empty command"));
    }
    #[cfg(windows)]
    {
        let mut c = Command::new("cmd");
        c.args(["/C", cmd]);
        Ok(c)
    }
    #[cfg(not(windows))]
    {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let mut c = Command::new(parts[0]);
        c.args(&parts[1..]);
        Ok(c)
    }
}

/// Run a package-manager command and capture its output (for fix/clean/migrate).
pub(crate) fn run_pkg_cmd(cmd: &str) -> std::io::Result<std::process::Output> {
    make_pkg_command(cmd)?.output()
}

/// Run a package-manager command with inherited stdio (for proxy installs).
pub(crate) fn proxy_pkg_cmd(cmd: &str) -> std::io::Result<std::process::ExitStatus> {
    make_pkg_command(cmd)?.status()
}

pub fn run_package_manager() -> Result<(), crate::error::types::InfynonError> {
    commands::execute_pkg_mode()
}

pub fn run_root() -> Result<(), crate::error::types::InfynonError> {
    commands::execute_root_mode()
}
