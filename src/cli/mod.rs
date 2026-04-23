pub mod args;
pub mod commands;
pub mod features;
pub mod scan;
pub mod validate;

fn quote_pkg_arg(arg: &str) -> String {
    if arg.is_empty()
        || arg
            .chars()
            .any(|c| c.is_whitespace() || matches!(c, '"' | '\''))
    {
        format!("{:?}", arg)
    } else {
        arg.to_string()
    }
}

pub(crate) fn format_pkg_cmd(program: &str, args: &[String]) -> String {
    std::iter::once(program.to_string())
        .chain(args.iter().map(|arg| quote_pkg_arg(arg)))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Build a direct process invocation for a package-manager binary and its argument list.
fn make_pkg_command(program: &str, args: &[String]) -> std::io::Result<std::process::Command> {
    use std::process::Command;
    let program = program.trim();
    if program.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "empty program",
        ));
    }
    let mut c = Command::new(program);
    c.args(args);
    Ok(c)
}

pub(crate) fn proxy_pkg_invocation(
    ecosystem: &str,
    actual_binary: &str,
    args: &[String],
) -> (String, Vec<String>) {
    match ecosystem {
        "pub" => {
            let mut actual_args = Vec::with_capacity(args.len() + 1);
            actual_args.push("pub".to_string());
            actual_args.extend(args.iter().cloned());
            (actual_binary.to_string(), actual_args)
        }
        _ => (actual_binary.to_string(), args.to_vec()),
    }
}

/// Run a package-manager command and capture its output (for fix/clean/migrate).
pub(crate) fn run_pkg_cmd(cmd: &str) -> std::io::Result<std::process::Output> {
    #[cfg(windows)]
    let (program, args): (&str, Vec<String>) = ("cmd", vec!["/C".to_string(), cmd.to_string()]);
    #[cfg(not(windows))]
    let (program, args): (&str, Vec<String>) = ("sh", vec!["-c".to_string(), cmd.to_string()]);

    make_pkg_command(program, &args)?.output()
}

/// Run a package-manager command with inherited stdio (for proxy installs).
pub(crate) fn proxy_pkg_cmd(
    ecosystem: &str,
    actual_binary: &str,
    args: &[String],
) -> std::io::Result<std::process::ExitStatus> {
    let (program, actual_args) = proxy_pkg_invocation(ecosystem, actual_binary, args);
    make_pkg_command(&program, &actual_args)?.status()
}

pub fn run_package_manager() -> Result<(), crate::error::types::InfynonError> {
    commands::execute_pkg_mode()
}

pub fn run_root() -> Result<(), crate::error::types::InfynonError> {
    commands::execute_root_mode()
}
