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

    /// Weave — node-based API flow testing, context-threading, and security probe TUI.
    /// Model your API as a directed graph of HTTP nodes, thread variables between requests,
    /// run security probes, and inspect results live. All state stored in .infynon/api/.
    #[command(name = "weave")]
    Api {
        #[command(subcommand)]
        action: ApiCommands,
    },
}

// ── API Testing commands ──────────────────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum ApiCommands {
    /// Open the Weave interactive TUI dashboard with 10 tabs:
    /// 1=Overview  2=Flow Graph  3=Live Execution  4=Latency Profiler
    /// 5=Security Probes  6=Coverage Map  7=State Inspector  8=Run Diff
    /// 9=Node Library  0=Config.
    /// From the Overview (tab 1) press Enter/a to run a flow.
    /// From the Node Library (tab 9) press Enter/r to run a single node.
    #[command(name = "tui")]
    Tui {
        /// Flow ID to open directly (skips overview). Omit to start at the overview.
        #[arg(value_name = "FLOW_ID")]
        flow_id: Option<String>,
    },

    /// Manage individual HTTP request nodes — create, inspect, run, clone, export,
    /// and control assertions. Nodes are the building blocks of flows.
    #[command(name = "node")]
    Node {
        #[command(subcommand)]
        action: NodeAction,
    },

    /// Manage flows — ordered graphs of connected nodes that run end-to-end.
    /// Create flows manually, via AI description, or by importing an OpenAPI spec.
    #[command(name = "flow")]
    Flow {
        #[command(subcommand)]
        action: FlowAction,
    },

    /// Connect two nodes with a directed edge so the flow executes FROM → TO.
    /// Extracted variables from FROM are available in TO via {var_name} placeholders.
    /// Use --carry to forward specific variables; --condition to make the edge conditional.
    /// Example: infynon weave attach login get-profile --carry token,user_id
    #[command(name = "attach")]
    Attach {
        /// ID of the source node (executes first)
        from: String,
        /// ID of the target node (executes after FROM completes)
        to: String,
        /// Comma-separated list of variable names to carry from FROM into TO's context.
        /// If omitted, all extracted variables from FROM are available in TO.
        #[arg(long, value_delimiter = ',', value_name = "VAR,...")]
        carry: Vec<String>,
        /// Only traverse this edge when the expression is true (e.g. "status == 201").
        /// Supports: status ==/</>= N, body.field == value, header.name contains text.
        #[arg(long, value_name = "EXPR")]
        condition: Option<String>,
        /// Ask AI to infer which variables should be carried across this edge
        /// based on the FROM node's extractions and the TO node's placeholders.
        #[arg(long)]
        ai: bool,
    },

    /// Remove the directed edge between two nodes, breaking their execution link.
    /// The nodes themselves are NOT deleted — only the connection is removed.
    /// Example: infynon weave detach login get-profile
    #[command(name = "detach")]
    Detach {
        /// ID of the source node
        from: String,
        /// ID of the target node
        to: String,
    },

    /// AI-powered assistance: suggest next nodes, auto-build flows, run security probes,
    /// generate assertions, explain failures, and suggest conditional branches.
    #[command(name = "ai")]
    Ai {
        #[command(subcommand)]
        action: AiAction,
    },

    /// Manage environment variables for this project.
    /// Variables set here are written to the .env file in the current directory and
    /// available as {$VAR_NAME} in any node's path, headers, or body.
    /// Useful for setting AUTH_TOKEN, API_KEY, BASE_URL, and other shared values.
    /// Example: infynon weave env set AUTH_TOKEN eyJhbGci...
    #[command(name = "env")]
    Env {
        #[command(subcommand)]
        action: EnvAction,
    },

    /// Validate all nodes and flows in .infynon/api/ and report any issues.
    /// Checks: node IDs, HTTP methods, path format, body JSON validity, extraction prefixes,
    /// flow entry node existence, edge node existence, and circular references.
    /// Exits with code 1 if any errors are found — safe to use in CI pipelines.
    /// Example: infynon weave validate
    Validate,

    /// Import HTTP nodes from an OpenAPI 3.x or Swagger 2.x spec file.
    /// Reads .yaml, .yml, or .json. Auto-generates node IDs, body templates with
    /// {field_name} placeholders, variable extractions from response schemas,
    /// status assertions, and Authorization headers for non-auth endpoints.
    /// Use --dry-run to preview without writing any files.
    /// Example: infynon weave import openapi.yaml --flow "My Flow" --prefix /api/v1
    #[command(name = "import")]
    Import {
        /// Path to the OpenAPI 3.x or Swagger 2.x spec file (.yaml, .yml, or .json)
        spec: String,
        /// If provided, automatically create a flow containing all imported nodes,
        /// with AI-inferred execution order. Value is the human-readable flow name.
        #[arg(long, value_name = "FLOW_NAME")]
        flow: Option<String>,
        /// Override the base URL from the spec (default: spec's servers[0].url).
        /// Useful when the spec points to production but you want to test staging.
        #[arg(long, value_name = "URL")]
        base_url: Option<String>,
        /// Only import endpoints whose path starts with this prefix.
        /// Example: --prefix /api/v1  (skips /health, /internal, etc.)
        #[arg(long, value_name = "PREFIX")]
        prefix: Option<String>,
        /// Preview all endpoints that would be imported without saving any files.
        /// Shows node IDs, methods, paths, assertion counts, and extraction counts.
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum NodeAction {
    /// Create a new HTTP request node interactively or from an AI description.
    /// Interactive wizard prompts for: ID, name, method, path, body, headers,
    /// variable extractions (e.g. token=body.access_token), and assertions.
    /// Use --ai to skip the wizard and let AI generate the node from plain English.
    /// Example: infynon weave node create --ai "POST /api/users with name and email fields"
    Create {
        /// Generate the node from a plain-English description instead of the interactive wizard.
        /// AI will infer method, path, body schema, extractions, and assertions automatically.
        #[arg(long, value_name = "DESCRIPTION")]
        ai: Option<String>,
    },
    /// Show the full definition of a node: method, path, headers, body template,
    /// variable extractions, assertions (with enabled/disabled status), and description.
    /// Example: infynon weave node get login
    Get {
        /// Node ID to inspect
        id: String,
    },
    /// List all nodes saved in .infynon/api/nodes/ with their method, path, and ID.
    /// Reads both .toml and .yaml/.yml files automatically.
    List,
    /// Permanently delete a node from .infynon/api/nodes/.
    /// Any flows referencing this node will become invalid — run `infynon weave validate` afterward.
    Remove {
        /// Node ID to delete
        id: String,
    },
    /// Duplicate an existing node under a new ID. All fields (headers, body, assertions,
    /// extractions) are copied. Useful for creating variations of an existing request.
    /// Example: infynon weave node clone create-user create-admin-user
    Clone {
        /// ID of the node to copy
        id: String,
        /// ID for the new duplicate node (kebab-case recommended)
        new_id: String,
    },
    /// Execute a single node in isolation against a live server and print the result.
    /// Shows status code, response body, assertion pass/fail, and extracted variables.
    /// Use --set to inject known values, or --prompt to interactively fill any unresolved
    /// {placeholder} variables found in the node's path, headers, or body.
    /// Example: infynon weave node run get-profile --base-url http://localhost:4000 --set token=abc123
    /// Example: infynon weave node run verify-otp --prompt  (asks for any missing {vars} interactively)
    Run {
        /// Node ID to execute
        id: String,
        /// Base URL prepended to the node's path (e.g. http://localhost:3000)
        #[arg(long, default_value = "http://localhost:3000")]
        base_url: String,
        /// Inject context variables as KEY=VALUE pairs. Used to fill {placeholder} fields
        /// in the request path, headers, or body. Repeat for multiple variables.
        /// Example: --set token=abc123 --set user_id=42
        #[arg(long, value_name = "KEY=VALUE", value_parser = parse_key_val)]
        set: Vec<(String, String)>,
        /// Interactively prompt for any unresolved {placeholder} variables found in the
        /// node's path, headers, or body that weren't supplied via --set or .env.
        /// Combines with --set: pre-fill known values and prompt only for the rest.
        #[arg(long)]
        prompt: bool,
    },
    /// Export a node as a ready-to-run curl command or raw JSON definition.
    /// curl format includes all headers and body. json format outputs the full node schema.
    /// Example: infynon weave node export login --format curl --base-url http://localhost:3000
    Export {
        /// Node ID to export
        id: String,
        /// Export format: curl (ready-to-run shell command) | json (raw node definition)
        #[arg(long, default_value = "curl")]
        format: String,
        /// Base URL to use in the exported curl command (omit to use a placeholder)
        #[arg(long, value_name = "URL")]
        base_url: Option<String>,
    },
    /// Manage test assertions on a node — list, add, remove, enable, disable, or toggle.
    /// Assertions verify the response (status code, body fields, headers) after each run.
    /// Disabled assertions are skipped at runtime but preserved for later re-enabling.
    /// Example: infynon weave node assertion login list
    #[command(name = "assertion")]
    Assertion {
        /// Node ID whose assertions you want to manage
        node_id: String,
        #[command(subcommand)]
        action: AssertionAction,
    },
    /// Manage runtime prompt inputs on a node — variables the user is asked to supply
    /// interactively when the node executes (OTPs, passwords, dynamic values).
    #[command(name = "prompt")]
    Prompt {
        /// Node ID
        node_id: String,
        #[command(subcommand)]
        action: PromptAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum PromptAction {
    /// List all prompt inputs defined on the node.
    List,
    /// Add a new prompt input — a variable the user is asked to supply at runtime.
    /// The entered value is injected as {var} in the request path, headers, and body.
    /// Example: infynon weave node prompt login add otp --label "OTP Code" --secret
    Add {
        /// Variable name (used as {var} placeholder in the node's path/headers/body)
        var: String,
        /// Human-readable label shown to the user at runtime (e.g. "OTP Code", "Admin password")
        #[arg(long, default_value = "")]
        label: String,
        /// Mask user input with * characters (for passwords and tokens)
        #[arg(long)]
        secret: bool,
        /// Pre-filled default value the user can accept or override
        #[arg(long)]
        default: Option<String>,
    },
    /// Remove a prompt input by its index (use `list` to find the index).
    Remove { index: usize },
}

#[derive(Subcommand, Debug)]
pub enum AssertionAction {
    /// Show all assertions on the node with their index, enabled/disabled status,
    /// expression, and on_fail action (stop | warn).
    /// Example: infynon weave node assertion login list
    List,
    /// Re-enable a previously disabled assertion so it runs again during flow execution.
    /// Use `list` first to find the index.
    /// Example: infynon weave node assertion login enable 2
    Enable {
        /// Zero-based index of the assertion to enable (see `list` for indices)
        index: usize
    },
    /// Disable an assertion so it is skipped during flow execution without deleting it.
    /// Useful for temporarily bypassing a failing check while debugging.
    /// Example: infynon weave node assertion login disable 1
    Disable {
        /// Zero-based index of the assertion to disable (see `list` for indices)
        index: usize
    },
    /// Flip the enabled/disabled state of an assertion.
    /// If it was enabled it becomes disabled, and vice versa.
    /// Example: infynon weave node assertion login toggle 0
    Toggle {
        /// Zero-based index of the assertion to toggle (see `list` for indices)
        index: usize
    },
    /// Add a new assertion to the node. Expressions support:
    ///   status == 200 | status >= 200 | body exists | body.field == "value"
    ///   body.count > 0 | header.content-type contains application/json
    /// on_fail controls flow behavior: stop (halt the flow) | warn (log and continue).
    /// Example: infynon weave node assertion login add "status == 200" --on-fail stop
    Add {
        /// Assertion expression to evaluate against the response (e.g. "status == 200")
        check: String,
        /// What to do when this assertion fails: stop (halt execution) | warn (log and continue)
        #[arg(long, default_value = "stop")]
        on_fail: String,
    },
    /// Permanently delete an assertion from the node by its index.
    /// Use `list` first to confirm the correct index before removing.
    /// Example: infynon weave node assertion login remove 2
    Remove {
        /// Zero-based index of the assertion to delete (see `list` for indices)
        index: usize
    },
}

#[derive(Subcommand, Debug)]
pub enum FlowAction {
    /// Create a new named flow (empty graph). Add nodes to it with `infynon weave attach`.
    /// Use --ai to describe the flow in plain English and let AI build the node graph.
    /// Example: infynon weave flow create "User Onboarding" --ai "register, verify email, login"
    Create {
        /// Human-readable name for the flow (e.g. "User Onboarding Flow")
        name: String,
        /// Build the flow automatically from a plain-English description.
        /// AI will select nodes from your library, connect them, and infer carry variables.
        #[arg(long, value_name = "DESCRIPTION")]
        ai: Option<String>,
    },
    /// List all flows saved in .infynon/api/flows/ with their ID, name, entry node,
    /// and total node count.
    List,
    /// Print the directed graph of a flow: node IDs, edges, carry variables, and conditions.
    /// Useful for understanding execution order before running.
    /// Example: infynon weave flow show onboarding-flow
    Show {
        /// Flow ID to display
        id: String,
    },
    /// Execute all nodes in a flow in order, threading extracted variables between them.
    /// Prints a live step-by-step table: node, status, assertions passed/failed, extracted vars.
    /// Use --set to pre-seed context variables, --output to save a report.
    /// Example: infynon weave flow run onboarding-flow --base-url http://localhost:4000 --set token=abc123
    Run {
        /// Flow ID to execute
        id: String,
        /// Base URL prepended to every node's path (e.g. http://staging.example.com).
        /// Overrides the flow's stored base_url if set.
        #[arg(long, value_name = "URL")]
        base_url: Option<String>,
        /// Pre-seed context variables before the flow starts. Repeat for multiple values.
        /// Variables are available as {key} in all node paths, headers, and bodies.
        /// Example: --set token=eyJhbG --set user_id=42
        #[arg(long, value_name = "KEY=VALUE", value_parser = parse_key_val)]
        set: Vec<(String, String)>,
        /// Save a run report to ./reports/<flow-id>-<timestamp>.<ext>.
        /// Formats: markdown | pdf | both
        #[arg(long, value_name = "FORMAT")]
        output: Option<String>,
    },
    /// Execute every flow in .infynon/api/flows/ in sequence and summarize results.
    /// Useful for full regression testing or CI checks across all API scenarios.
    /// Example: infynon weave flow run-all --base-url $API_URL --set token=abc123 --output markdown
    RunAll {
        /// Base URL prepended to every node's path in every flow.
        #[arg(long, value_name = "URL")]
        base_url: Option<String>,
        /// Pre-seed context variables for all flows. Repeat for multiple values.
        #[arg(long, value_name = "KEY=VALUE", value_parser = parse_key_val)]
        set: Vec<(String, String)>,
        /// Save a combined run report for all flows: markdown | pdf | both
        #[arg(long, value_name = "FORMAT")]
        output: Option<String>,
    },
    /// Delete a flow definition from .infynon/api/flows/.
    /// The individual nodes referenced by this flow are NOT deleted.
    /// Example: infynon weave flow remove old-flow
    Remove {
        /// Flow ID to delete
        id: String,
    },
    /// Combine two flows into a single new flow by connecting FLOW2's entry node
    /// to a node in FLOW1. The resulting merged flow runs FLOW1 then FLOW2.
    /// Example: infynon weave flow merge auth-flow data-flow --join-at login --name full-journey
    Merge {
        /// ID of the first (upstream) flow
        flow1: String,
        /// ID of the second (downstream) flow — its entry is attached to --join-at
        flow2: String,
        /// Node ID in FLOW1 that FLOW2's entry node will be connected to
        #[arg(long, value_name = "NODE_ID")]
        join_at: String,
        /// Name for the resulting merged flow
        #[arg(long, default_value = "merged-flow")]
        name: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum EnvAction {
    /// List all environment variables stored in the project's .env file.
    /// Shows KEY=VALUE pairs (sensitive values are masked with ***).
    /// Example: infynon weave env list
    List,
    /// Add or update an environment variable in the project's .env file.
    /// Variables are available as {$KEY} in any node's path, headers, or body.
    /// Example: infynon weave env set AUTH_TOKEN eyJhbGciOiJIUzI1NiJ9...
    /// Example: infynon weave env set BASE_URL http://staging.example.com
    Set {
        /// Variable name (e.g. AUTH_TOKEN, BASE_URL)
        key: String,
        /// Variable value
        value: String,
    },
    /// Delete an environment variable from the .env file.
    /// Example: infynon weave env delete OLD_TOKEN
    Delete {
        /// Variable name to remove
        key: String,
    },
    /// Show the current value of a single environment variable.
    /// Sensitive-looking keys (TOKEN, SECRET, PASSWORD, KEY) are masked unless --reveal is set.
    /// Example: infynon weave env get AUTH_TOKEN
    /// Example: infynon weave env get AUTH_TOKEN --reveal
    Get {
        /// Variable name to look up
        key: String,
        /// Show the full value even if it looks sensitive
        #[arg(long)]
        reveal: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum AiAction {
    /// Analyze a node's role in the flow and suggest the best next node to add after it.
    /// AI considers the node's extractions, path, and method to recommend a logical continuation.
    /// Example: infynon weave ai suggest --after login
    Suggest {
        /// Node ID to suggest a follow-up for
        #[arg(long, value_name = "NODE_ID")]
        after: String,
    },
    /// Automatically pick the best next node from your library and attach it after the given node.
    /// AI infers which variables to carry across the new edge. Optionally scoped to one flow.
    /// Example: infynon weave ai attach --after login --flow onboarding-flow
    Attach {
        /// Node ID to attach a new node after
        #[arg(long, value_name = "NODE_ID")]
        after: String,
        /// Scope the attachment to a specific flow (omit to update all flows)
        #[arg(long, value_name = "FLOW_ID")]
        flow: Option<String>,
    },
    /// Find nodes in your library that are not yet connected to any flow and add them
    /// to the specified flow in a logical order inferred by AI.
    /// Example: infynon weave ai complete onboarding-flow
    Complete {
        /// Flow ID to add unconnected nodes into
        flow_id: String,
    },
    /// Run automated security probes against a flow: SQL injection, XSS, auth bypass,
    /// IDOR, path traversal, rate-limit testing, and more. Results shown in tab 5 of the TUI.
    /// Example: infynon weave ai probe onboarding-flow --base-url http://localhost:3000
    Probe {
        /// Flow ID to probe
        flow_id: String,
        /// Base URL to send probe requests to
        #[arg(long, value_name = "URL")]
        base_url: Option<String>,
    },
    /// Build a new flow from an explicit ordered list of node IDs. AI will infer carry variables
    /// and edge conditions between them. Useful when you know the order but want AI to wire them up.
    /// Example: infynon weave ai build-flow --nodes login,get-profile,create-post --name user-journey
    BuildFlow {
        /// Comma-separated ordered list of node IDs to connect into a flow
        #[arg(long, value_delimiter = ',', value_name = "NODE_IDS")]
        nodes: Vec<String>,
        /// Name for the generated flow (used as both ID and display name)
        #[arg(long, default_value = "ai-generated-flow")]
        name: String,
    },
    /// Analyze a flow's run output and explain in plain English why it failed,
    /// which assertion broke, what the actual vs expected values were, and how to fix it.
    /// Example: infynon weave ai explain onboarding-flow --run 0
    Explain {
        /// Flow ID whose run history to explain
        flow_id: String,
        /// Index of the run to explain (0 = most recent, 1 = second most recent, etc.)
        #[arg(long, default_value = "0")]
        run: usize,
    },
    /// Inspect a node's response schema and generate meaningful assertions automatically.
    /// AI checks for common fields (id, token, status, error) and adds typed assertions.
    /// Example: infynon weave ai assert login
    Assert {
        /// Node ID to generate assertions for
        node_id: String,
    },
    /// Analyze a node's possible response codes and suggest conditional edges for each branch.
    /// For example: a 201 edge to the next step and a 409 edge to a conflict-handler node.
    /// Example: infynon weave ai branch create-user
    Branch {
        /// Node ID to suggest branches for
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
