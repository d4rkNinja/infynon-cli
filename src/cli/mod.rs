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

#[derive(Debug, Clone)]
pub(crate) struct PkgInvocation {
    pub program: String,
    pub args: Vec<String>,
}

impl PkgInvocation {
    pub(crate) fn new(program: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            program: program.into(),
            args,
        }
    }

    pub(crate) fn from_args(program: &str, args: &[&str]) -> Self {
        Self::new(program, args.iter().map(|arg| arg.to_string()).collect())
    }

    pub(crate) fn display(&self) -> String {
        format_pkg_cmd(&self.program, &self.args)
    }

    pub(crate) fn output(&self) -> std::io::Result<std::process::Output> {
        make_pkg_command(&self.program, &self.args)?.output()
    }
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
    let resolved_program = crate::ecosystems::detector::resolve_binary(program);
    #[cfg(windows)]
    if is_windows_command_script(&resolved_program) {
        let shell = std::env::var_os("COMSPEC").unwrap_or_else(|| "cmd.exe".into());
        let mut c = Command::new(shell);
        c.arg("/C").arg(&resolved_program).args(args);
        return Ok(c);
    }
    let mut c = Command::new(resolved_program);
    c.args(args);
    Ok(c)
}

#[cfg(windows)]
fn is_windows_command_script(program: &str) -> bool {
    std::path::Path::new(program)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("cmd") || ext.eq_ignore_ascii_case("bat"))
        .unwrap_or(false)
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
