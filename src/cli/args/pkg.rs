use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "infynon pkg",
    version,
    about = "INFYNON Package Manager - universal secure package intelligence",
    styles = crate::cli::args::get_styles()
)]
pub struct PkgArgs {
    #[arg(long, global = true, value_name = "LEVEL", default_missing_value = "all", num_args = 0..=1, require_equals = false)]
    pub strict: Option<String>,
    #[arg(
        long,
        global = true,
        value_name = "FILE",
        help = "Path to a specific lock/manifest file to scan or install from"
    )]
    pub pkg_file: Option<String>,
    #[arg(long, global = true, help = "Emit machine-readable JSON to stdout")]
    pub json: bool,
    #[arg(
        long,
        global = true,
        help = "Disable interactive prompts and fail when input is required"
    )]
    pub no_input: bool,
    #[arg(
        long,
        global = true,
        help = "Non-interactive: install all packages, even vulnerable ones"
    )]
    pub yes: bool,
    #[arg(
        long,
        global = true,
        help = "Non-interactive: skip vulnerable packages, install only safe ones"
    )]
    pub skip_vulnerable: bool,
    #[arg(
        long,
        global = true,
        help = "Non-interactive: auto-install fixed versions, skip unfixable packages"
    )]
    pub auto_fix: bool,
    #[arg(long, global = true, hide = true, help = "Deprecated alias for --json")]
    pub agent: bool,
    #[command(subcommand)]
    pub command: Option<PkgCommands>,
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub passthrough_args: Vec<String>,
}

impl PkgArgs {
    pub fn machine_output(&self) -> bool {
        self.json || self.agent
    }

    pub fn non_interactive(&self) -> bool {
        self.no_input || self.machine_output() || std::env::var_os("CI").is_some()
    }
}

#[derive(Subcommand, Debug)]
pub enum PkgCommands {
    Scan {
        #[arg(long, value_name = "FORMAT")]
        output: Option<String>,
        #[arg(long, value_name = "LEVEL", default_missing_value = "all", num_args = 0..=1, require_equals = false)]
        fix: Option<String>,
        #[arg(long, value_name = "FILE")]
        pkg_file: Option<String>,
    },
    Audit {
        #[arg(long, value_name = "FILE")]
        pkg_file: Option<String>,
    },
    Why {
        package: String,
        #[arg(long, value_name = "FILE")]
        pkg_file: Option<String>,
    },
    #[command(
        about = "Explain why one installed package matters and what to do next",
        after_help = "Examples:\n  infynon pkg explain serde_json\n  infynon pkg explain requests --ecosystem pip\n  infynon pkg explain tokio --pkg-file Cargo.lock"
    )]
    Explain {
        package: String,
        #[arg(long)]
        ecosystem: Option<String>,
        #[arg(long, value_name = "FILE")]
        pkg_file: Option<String>,
    },
    Outdated {
        #[arg(long, value_name = "FILE")]
        pkg_file: Option<String>,
    },
    Diff {
        package: String,
        v1: String,
        v2: String,
        #[arg(long)]
        ecosystem: Option<String>,
    },
    Doctor {
        #[arg(long, value_name = "FILE")]
        pkg_file: Option<String>,
    },
    Size {
        packages: Vec<String>,
        #[arg(long)]
        ecosystem: Option<String>,
    },
    Search {
        query: String,
        #[arg(long)]
        ecosystem: Option<String>,
    },
    Fix {
        #[arg(long)]
        auto: bool,
        #[arg(long, value_name = "FILE")]
        pkg_file: Option<String>,
    },
    Clean {
        #[arg(long, value_name = "FILE")]
        pkg_file: Option<String>,
    },
    Migrate {
        from: String,
        to: String,
    },
    #[command(name = "eagle-eye")]
    EagleEye {
        #[command(subcommand)]
        action: EagleEyeAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum EagleEyeAction {
    Setup,
    Start,
    Status,
    Enable,
    Disable,
}
