use clap::{Parser, Subcommand};
use clap::builder::styling::{AnsiColor, Effects, Styles};

fn get_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::BrightCyan.on_default() | Effects::BOLD)
        .usage(AnsiColor::BrightGreen.on_default() | Effects::BOLD)
        .literal(AnsiColor::BrightMagenta.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::BrightBlue.on_default())
        .error(AnsiColor::Red.on_default() | Effects::BOLD)
}

#[derive(Parser, Debug)]
#[command(
    name = "infynon pkg",
    version,
    about = "INFYNON Package Manager — Universal secure package installation",
    styles = get_styles()
)]
pub struct PkgArgs {
    /// Block vulnerable packages. Optionally specify severity level: critical | high | medium | low | all (default: all)
    #[arg(
        long,
        global = true,
        value_name = "LEVEL",
        default_missing_value = "all",
        num_args = 0..=1,
        require_equals = false,
    )]
    pub strict: Option<String>,

    /// Override lock/manifest file path (e.g. --pkg-file ./subdir/Cargo.lock)
    #[arg(long, global = true, value_name = "FILE", help = "Path to a specific lock/manifest file to scan or install from")]
    pub pkg_file: Option<String>,

    #[command(subcommand)]
    pub command: Option<PkgCommands>,

    /// Native wrapper command passthrough: npm install express, cargo add serde, go get ...
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub passthrough_args: Vec<String>,
}

#[derive(Subcommand, Debug)]
pub enum PkgCommands {
    /// Scan project lock/manifest files for known CVEs
    Scan {
        /// Save report to file: markdown | pdf | both  (omit to show inline only)
        #[arg(long, value_name = "FORMAT")]
        output: Option<String>,

        /// Only report/fix at or above this severity: critical | high | medium | low | informational | all
        #[arg(
            long,
            value_name = "LEVEL",
            default_missing_value = "all",
            num_args = 0..=1,
            require_equals = false,
        )]
        fix: Option<String>,

        /// Override lock/manifest file (e.g. --pkg-file ./Cargo.lock)
        #[arg(long, value_name = "FILE")]
        pkg_file: Option<String>,
    },

    /// Deep recursive dependency audit with tree visualization
    Audit {
        /// Override lock/manifest file
        #[arg(long, value_name = "FILE")]
        pkg_file: Option<String>,
    },

    /// Show why a package is in your dependency tree
    Why {
        /// Package name to trace
        package: String,
        /// Override lock/manifest file
        #[arg(long, value_name = "FILE")]
        pkg_file: Option<String>,
    },

    /// Check for outdated dependencies across all ecosystems
    Outdated {
        /// Override lock/manifest file
        #[arg(long, value_name = "FILE")]
        pkg_file: Option<String>,
    },

    /// Show what changed between two versions of a package
    Diff {
        /// Package name
        package: String,
        /// First version
        v1: String,
        /// Second version
        v2: String,
        /// Ecosystem (auto-detected if omitted)
        #[arg(long)]
        ecosystem: Option<String>,
    },

    /// Health check: duplicates, unused deps, phantom deps, lock files
    Doctor {
        /// Override lock/manifest file
        #[arg(long, value_name = "FILE")]
        pkg_file: Option<String>,
    },

    /// Show package size, install weight, and dependency count
    Size {
        /// Package name(s) to check
        packages: Vec<String>,
        /// Ecosystem (auto-detected if omitted)
        #[arg(long)]
        ecosystem: Option<String>,
    },

    /// Search packages across ecosystems
    Search {
        /// Search query
        query: String,
        /// Limit to a specific ecosystem
        #[arg(long)]
        ecosystem: Option<String>,
    },

    /// Auto-fix all vulnerable dependencies to their nearest safe version
    Fix {
        /// Automatically fix all vulnerabilities
        #[arg(long)]
        auto: bool,
        /// Override lock/manifest file
        #[arg(long, value_name = "FILE")]
        pkg_file: Option<String>,
    },

    /// Find and remove unused dependencies
    Clean {
        /// Override lock/manifest file
        #[arg(long, value_name = "FILE")]
        pkg_file: Option<String>,
    },

    /// Migrate between package managers (e.g. npm → pnpm, pip → uv)
    Migrate {
        /// Source package manager
        from: String,
        /// Target package manager
        to: String,
    },
}

#[derive(Parser, Debug)]
#[command(
    name = "infynon",
    version,
    about = "INFYNON — Universal Security Manager For Your Any Backend",
    styles = get_styles()
)]
pub struct FirewallArgs {
    #[command(subcommand)]
    pub command: Option<FirewallCommands>,
}

#[derive(Subcommand, Debug)]
pub enum FirewallCommands {
    /// Start the background nightly intelligence pipeline
    Daemon,
    /// Open the real-time TUI dashboard
    Dashboard,
    /// Manually trigger nightly intelligence pipeline immediately
    UpdateIntel,
}
