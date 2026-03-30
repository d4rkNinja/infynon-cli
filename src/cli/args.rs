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

    /// CI mode: install all packages even if vulnerable (no interactive prompts)
    #[arg(long, global = true, help = "Non-interactive: install all packages, even vulnerable ones")]
    pub yes: bool,

    /// CI mode: skip vulnerable packages silently, install only clean ones (no interactive prompts)
    #[arg(long, global = true, help = "Non-interactive: skip vulnerable packages, install only safe ones")]
    pub skip_vulnerable: bool,

    /// CI mode: auto-upgrade vulnerable packages to their safe version; skip if no fix is available (no interactive prompts)
    #[arg(long, global = true, help = "Non-interactive: auto-install fixed versions, skip unfixable packages")]
    pub auto_fix: bool,

    /// Agent/AI mode: emit machine-readable JSON instead of human-formatted output.
    /// Exit codes — 0: clean  1: warnings (low/info)  2: vulnerabilities found  3: blocked by --strict
    #[arg(long, global = true, help = "Output machine-readable JSON for AI agents and CI pipelines")]
    pub agent: bool,

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

    /// Eagle Eye — scheduled vulnerability monitoring with email alerts
    #[command(name = "eagle-eye")]
    EagleEye {
        #[command(subcommand)]
        action: EagleEyeAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum EagleEyeAction {
    /// Interactive setup — configure SMTP, paths, risk level, schedule
    Setup,
    /// Start Eagle Eye monitoring (runs in foreground)
    Start,
    /// Show current Eagle Eye configuration and status
    Status,
    /// Enable Eagle Eye monitoring
    Enable,
    /// Disable Eagle Eye monitoring
    Disable,
}

#[derive(Parser, Debug)]
#[command(
    name = "infynon",
    version,
    about = "INFYNON — Universal Security Firewall & Package Manager",
    styles = get_styles()
)]
pub struct FirewallArgs {
    #[command(subcommand)]
    pub command: Option<FirewallCommands>,
}

#[derive(Subcommand, Debug)]
pub enum FirewallCommands {
    /// Initialize firewall configuration (creates infynon.toml)
    Init {
        /// Listen port for the firewall proxy
        #[arg(long, default_value = "8080")]
        port: u16,
        /// Upstream backend address
        #[arg(long, default_value = "127.0.0.1")]
        upstream: String,
        /// Upstream backend port
        #[arg(long, default_value = "3000")]
        upstream_port: u16,
    },

    /// Start the firewall reverse proxy
    Start {
        /// Path to configuration file
        #[arg(long, value_name = "FILE")]
        config: Option<String>,
        /// Override listen port
        #[arg(long)]
        port: Option<u16>,
        /// Override upstream address:port
        #[arg(long, value_name = "HOST:PORT")]
        upstream: Option<String>,
        /// Start without TUI (headless mode)
        #[arg(long)]
        headless: bool,
    },

    /// Open the real-time TUI monitor (connects to running firewall)
    Monitor {
        /// Path to configuration file
        #[arg(long, value_name = "FILE")]
        config: Option<String>,
        /// Start on a specific view: dashboard | feed | blocked | ips | rules | stats | config
        #[arg(long, default_value = "dashboard")]
        view: String,
    },

    /// Show firewall status
    Status {
        /// Path to configuration file
        #[arg(long, value_name = "FILE")]
        config: Option<String>,
    },

    /// Block an IP address immediately
    #[command(name = "block")]
    BlockIp {
        /// IP address to block
        ip: String,
    },

    /// Unblock an IP address
    #[command(name = "unblock")]
    UnblockIp {
        /// IP address to unblock
        ip: String,
    },

    /// Manage firewall rules
    Rules {
        #[command(subcommand)]
        action: RulesAction,
    },

    /// View and export firewall logs
    Logs {
        /// Follow mode (stream new events)
        #[arg(long)]
        follow: bool,
        /// Filter by verdict: allow | block | flag | rate_limited
        #[arg(long)]
        verdict: Option<String>,
        /// Filter by source IP
        #[arg(long)]
        ip: Option<String>,
        /// Show events from last duration (e.g. 1h, 24h, 7d)
        #[arg(long)]
        since: Option<String>,
        /// Number of recent entries to show
        #[arg(long, default_value = "50")]
        count: usize,
    },

    /// Validate and display the current configuration
    #[command(name = "config")]
    ConfigCmd {
        #[command(subcommand)]
        action: Option<ConfigAction>,
    },

    /// Start the background nightly intelligence pipeline
    Daemon,

    /// Manually trigger nightly intelligence pipeline immediately
    UpdateIntel,

    /// Weave — node-based API flow testing & security probes TUI
    #[command(name = "weave")]
    Api {
        #[command(subcommand)]
        action: ApiCommands,
    },
}

// ── API Testing commands ──────────────────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum ApiCommands {
    /// Open the TUI dashboard (flow graph, live execution, security probes)
    #[command(name = "tui")]
    Tui {
        /// Flow ID to open (optional — defaults to overview)
        #[arg(value_name = "FLOW_ID")]
        flow_id: Option<String>,
    },

    /// Node management
    #[command(name = "node")]
    Node {
        #[command(subcommand)]
        action: NodeAction,
    },

    /// Flow management
    #[command(name = "flow")]
    Flow {
        #[command(subcommand)]
        action: FlowAction,
    },

    /// Attach two nodes (creates an edge in all relevant flows)
    #[command(name = "attach")]
    Attach {
        /// Source node ID
        from: String,
        /// Target node ID
        to: String,
        /// Variables to carry across the edge (comma-separated)
        #[arg(long, value_delimiter = ',')]
        carry: Vec<String>,
        /// Only follow this edge if condition is true (e.g. "status == 201")
        #[arg(long, value_name = "EXPR")]
        condition: Option<String>,
        /// Let AI infer what to carry
        #[arg(long)]
        ai: bool,
    },

    /// Remove edge between two nodes
    #[command(name = "detach")]
    Detach {
        from: String,
        to: String,
    },

    /// AI-powered operations
    #[command(name = "ai")]
    Ai {
        #[command(subcommand)]
        action: AiAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum NodeAction {
    /// Create a new node (interactive or from --ai description)
    Create {
        /// Let AI generate the node from a description
        #[arg(long, value_name = "DESCRIPTION")]
        ai: Option<String>,
    },
    /// Show full details of a node
    Get {
        id: String,
    },
    /// List all nodes in the library
    List,
    /// Remove a node
    Remove {
        id: String,
    },
    /// Clone a node with a new ID
    Clone {
        id: String,
        new_id: String,
    },
    /// Execute a single node in isolation
    Run {
        id: String,
        /// Base URL for the request
        #[arg(long, default_value = "http://localhost:3000")]
        base_url: String,
        /// Set context variables (key=value)
        #[arg(long, value_name = "KEY=VALUE", value_parser = parse_key_val)]
        set: Vec<(String, String)>,
    },
    /// Export a node as curl command or JSON
    Export {
        id: String,
        /// Output format: curl | json
        #[arg(long, default_value = "curl")]
        format: String,
        /// Base URL for the exported command
        #[arg(long)]
        base_url: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum FlowAction {
    /// Create a new flow
    Create {
        /// Flow name
        name: String,
        /// Let AI build the flow from a description
        #[arg(long, value_name = "DESCRIPTION")]
        ai: Option<String>,
    },
    /// List all flows
    List,
    /// Show a flow's graph
    Show {
        id: String,
    },
    /// Run a specific flow
    Run {
        id: String,
        /// Override base URL
        #[arg(long)]
        base_url: Option<String>,
    },
    /// Run all flows
    RunAll {
        #[arg(long)]
        base_url: Option<String>,
    },
    /// Delete a flow (nodes are not deleted)
    Remove {
        id: String,
    },
    /// Merge two flows into one
    Merge {
        flow1: String,
        flow2: String,
        /// Node ID where flow2 is attached to flow1
        #[arg(long)]
        join_at: String,
        /// Name for the merged flow
        #[arg(long, default_value = "merged-flow")]
        name: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum AiAction {
    /// Suggest the next node after a given node
    Suggest {
        #[arg(long, value_name = "NODE_ID")]
        after: String,
    },
    /// Automatically attach the best next node
    Attach {
        #[arg(long, value_name = "NODE_ID")]
        after: String,
        /// Only update this flow
        #[arg(long)]
        flow: Option<String>,
    },
    /// Add unconnected nodes to a flow automatically
    Complete {
        flow_id: String,
    },
    /// Run security probes on a flow
    Probe {
        flow_id: String,
        #[arg(long)]
        base_url: Option<String>,
    },
    /// Build a flow from a list of node IDs
    BuildFlow {
        #[arg(long, value_delimiter = ',', value_name = "NODE_IDS")]
        nodes: Vec<String>,
        #[arg(long, default_value = "ai-generated-flow")]
        name: String,
    },
    /// Explain why the last run of a flow failed
    Explain {
        flow_id: String,
        /// Which run to explain (0 = most recent)
        #[arg(long, default_value = "0")]
        run: usize,
    },
    /// Generate assertions for a node
    Assert {
        node_id: String,
    },
    /// Suggest conditional branches for a node
    Branch {
        node_id: String,
    },
}

fn parse_key_val(s: &str) -> Result<(String, String), String> {
    let pos = s.find('=').ok_or_else(|| format!("Expected KEY=VALUE, got '{}'", s))?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}

#[derive(Subcommand, Debug)]
pub enum RulesAction {
    /// List all active rules with hit counts
    List,
    /// Enable a rule by name
    Enable { name: String },
    /// Disable a rule by name
    Disable { name: String },
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Validate the configuration file
    Check,
    /// Show the effective configuration (with defaults)
    Show,
}
