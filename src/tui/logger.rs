use owo_colors::OwoColorize;
use std::time::Instant;

pub struct Logger;

const LABEL_W: usize = 18;
const CMD_W: usize   = 40;

impl Logger {
    pub fn title(text: &str, bg: &str) {
        let final_text = match bg {
            "blue" => format!(" {} ", text).bold().black().on_bright_cyan().to_string(),
            "red"  => format!(" {} ", text).bold().black().on_bright_red().to_string(),
            _      => format!(" {} ", text).bold().black().on_bright_black().to_string(),
        };
        println!("\n{}\n", final_text);
    }

    pub fn info(msg: &str) {
        println!("  {} {}", "::".cyan(), msg.white());
    }

    pub fn success(msg: &str) {
        println!("  {} {}", "✔".green(), msg.bold().green());
    }

    pub fn error(msg: &str) {
        println!("  {} {}", "✘".red(), msg.bold().red());
    }

    pub fn subtitle(icon: &str, label: &str, value: &str) {
        println!("\n{} {} {}", icon.cyan(), label.bold().cyan(), value.bold().green());
    }

    pub fn step(msg: &str) {
        println!("  {} {}", ">>".truecolor(255, 100, 100).bold(), msg.white());
    }

    pub fn detail(label: &str, value: &str) {
        println!("  {} {}", label.white(), value.bold().purple());
    }

    pub fn raw_dim(msg: &str) {
        println!("  {}", msg.truecolor(180, 180, 180));
    }

    fn divider() {
        println!("  {}", "─".repeat(66).truecolor(40, 40, 60));
    }

    fn section(icon: &str, title: &str) {
        println!("\n  {}  {}\n", icon, title.bold().truecolor(0, 210, 255));
    }

    fn row(icon: &str, label: &str, value: &str) {
        let padded = format!("{:<width$}", label, width = LABEL_W - 2);
        println!("  {}  {}  {}", icon, padded.bold().truecolor(255, 170, 50), value.truecolor(180, 180, 200));
    }

    fn cont(value: &str) {
        let pad = " ".repeat(LABEL_W + 5);
        println!("  {}{}", pad, value.truecolor(180, 180, 200));
    }

    fn cmd_row(cmd: &str, desc: &str) {
        let padded = format!("{:<width$}", cmd, width = CMD_W);
        println!("  {}  {}", padded.bold().truecolor(120, 220, 120), desc.truecolor(140, 140, 160));
    }

    // ── infynon (firewall) splash ────────────────────────────────────────────
    pub fn splash(start: Instant) {
        let banner = r#"
  ██╗███╗   ██╗███████╗██╗   ██╗███╗   ██╗ ██████╗ ███╗   ██╗
  ██║████╗  ██║██╔════╝╚██╗ ██╔╝████╗  ██║██╔═══██╗████╗  ██║
  ██║██╔██╗ ██║█████╗   ╚████╔╝ ██╔██╗ ██║██║   ██║██╔██╗ ██║
  ██║██║╚██╗██║██╔══╝    ╚██╔╝  ██║╚██╗██║██║   ██║██║╚██╗██║
  ██║██║ ╚████║██║        ██║   ██║ ╚████║╚██████╔╝██║ ╚████║
  ╚═╝╚═╝  ╚═══╝╚═╝        ╚═╝   ╚═╝  ╚═══╝ ╚═════╝ ╚═╝  ╚═══╝"#;
        println!("{}", banner.truecolor(0, 210, 255).bold());
        println!("  {}\n", format!("Network Firewall & Security CLI  ·  v{}", env!("CARGO_PKG_VERSION")).truecolor(120, 120, 140).italic());

        Self::divider();
        Self::section("⚡", "What is INFYNON?");

        Self::row("🛡️", "Firewall",     "Real-time reverse proxy WAF — sits between internet and your backend.");
        Self::cont(                      "Inspects, filters, blocks HTTP traffic with a live TUI dashboard.");
        Self::row("🔎", "Pipeline",     "4-stage request evaluation on every request:");
        Self::row("  ", "Stage 1",      "IP Filter — blocklist, allowlist, CIDR, auto-reputation banning");
        Self::row("  ", "Stage 2",      "Rate Limiter — per-IP, per-path, global sliding window");
        Self::row("  ", "Stage 3",      "WAF Engine — SQLi, XSS, path traversal, cmd injection detection");
        Self::row("  ", "Stage 4",      "Custom Rules — user-defined IF-THEN rules with priority ordering");
        Self::row("🖥️", "TUI",         "7 live views: Dashboard, Feed, Blocked, IP Inspector, Rules, Stats, Config");
        Self::row("🔧", "Config",       "TOML config editable from TUI + file. Hot-reload on file change.");
        Self::row("🌐", "Routing",      "Multi-upstream path-based routing to different backend services.");
        Self::row("🔒", "Maintenance",  "Toggle maintenance mode from TUI (m key) or config file.");

        Self::divider();
        Self::section("📟", "Firewall Commands");

        Self::cmd_row("infynon init",                   "Create default infynon.toml config");
        Self::cmd_row("infynon start",                  "Start firewall + TUI dashboard");
        Self::cmd_row("infynon start --headless",       "Start firewall without TUI (background)");
        Self::cmd_row("infynon monitor",                "Open TUI monitor (starts proxy too)");
        Self::cmd_row("infynon status",                 "Show current firewall configuration");
        Self::cmd_row("infynon block <IP>",             "Block an IP address immediately");
        Self::cmd_row("infynon unblock <IP>",           "Remove an IP from blocklist");
        Self::cmd_row("infynon rules list",             "List all custom rules with hit counts");
        Self::cmd_row("infynon rules enable <name>",    "Enable a rule by name");
        Self::cmd_row("infynon rules disable <name>",   "Disable a rule by name");
        Self::cmd_row("infynon logs",                   "View recent firewall log entries");
        Self::cmd_row("infynon logs --verdict block",   "Filter logs by verdict (block/allow/flag)");
        Self::cmd_row("infynon config check",           "Validate config file");
        Self::cmd_row("infynon config show",            "Print effective config with defaults");

        Self::divider();
        Self::section("📦", "Package Security  (infynon pkg)");

        Self::cmd_row("infynon pkg scan",               "Scan lock files for known CVEs");
        Self::cmd_row("infynon pkg npm install <pkg>",  "Secure install via any package manager");
        Self::cmd_row("infynon pkg audit",              "Deep recursive dependency audit");
        Self::cmd_row("infynon pkg fix --auto",         "Auto-fix all vulnerable dependencies");

        Self::footer(start, "Network Firewall + Package Security", "d4rkninja", "whit3ninj4");
    }

    // ── infynon pkg splash ───────────────────────────────────────────────────
    pub fn splash_pkg() {
        println!();
        println!(
            "  {} {}",
            "infynon pkg".bold().truecolor(120, 80, 255),
            format!("·  Package Security Manager  ·  v{}", env!("CARGO_PKG_VERSION")).truecolor(120, 120, 140).italic()
        );
        println!("  {}\n", "─".repeat(52).truecolor(80, 50, 160));

        Self::divider();
        Self::section("📦", "What is infynon pkg?");

        Self::row("🎯", "Purpose",      "Secure proxy between you and any upstream package registry.");
        Self::cont(                      "Every install passes through a 3-layer verification pipeline.");
        Self::row("🛡️", "Layer 1",      "Blocklist trie lookup — known-bad packages blocked in <1ms");
        Self::row("🔎", "Layer 2",      "Heuristic scan — preinstall scripts, typosquatting, pkg age");
        Self::row("🤖", "Layer 3",      "LLM source analysis via local Ollama — flagged packages only");
        Self::row("🌍", "Ecosystems",   "npm · yarn · pnpm · bun · pip · uv · poetry · cargo · go");
        Self::cont(                      "gem · composer · nuget · hex · pub");
        Self::row("🔍", "Auto-Detect",  "No ecosystem flag? Scans package.json / Cargo.toml / go.mod etc.");
        Self::row("📄", "SBOM",         "Writes .infynon/sbom.json (CycloneDX) after every install.");

        Self::divider();
        Self::section("📟", "Usage  —  all supported package managers");

        println!("  {}\n", "── JavaScript ─────────────────────────────────────────────────".truecolor(60, 60, 80));
        Self::cmd_row("infynon pkg npm install express",      "npm");
        Self::cmd_row("infynon pkg yarn add lodash",          "yarn");
        Self::cmd_row("infynon pkg pnpm add react",           "pnpm");
        Self::cmd_row("infynon pkg bun add axios",            "bun");

        println!("\n  {}\n", "── Python ─────────────────────────────────────────────────────".truecolor(60, 60, 80));
        Self::cmd_row("infynon pkg pip install requests",     "pip");
        Self::cmd_row("infynon pkg uv pip install fastapi",   "uv");
        Self::cmd_row("infynon pkg poetry add django",        "poetry");

        println!("\n  {}\n", "── Rust ───────────────────────────────────────────────────────".truecolor(60, 60, 80));
        Self::cmd_row("infynon pkg cargo add serde",          "cargo");

        println!("\n  {}\n", "── Go ─────────────────────────────────────────────────────────".truecolor(60, 60, 80));
        Self::cmd_row("infynon pkg go get golang.org/x/crypto", "go");

        println!("\n  {}\n", "── Ruby ───────────────────────────────────────────────────────".truecolor(60, 60, 80));
        Self::cmd_row("infynon pkg gem install rails",        "gem");

        println!("\n  {}\n", "── PHP ────────────────────────────────────────────────────────".truecolor(60, 60, 80));
        Self::cmd_row("infynon pkg composer require laravel/framework", "composer");

        println!("\n  {}\n", "── .NET ───────────────────────────────────────────────────────".truecolor(60, 60, 80));
        Self::cmd_row("infynon pkg nuget add Newtonsoft.Json", "nuget / dotnet");

        println!("\n  {}\n", "── Elixir ─────────────────────────────────────────────────────".truecolor(60, 60, 80));
        Self::cmd_row("infynon pkg hex deps.get",             "hex / mix");

        println!("\n  {}\n", "── Dart ───────────────────────────────────────────────────────".truecolor(60, 60, 80));
        Self::cmd_row("infynon pkg pub add http",             "pub / dart");

        println!("\n  {}\n", "── Auto-detect ────────────────────────────────────────────────".truecolor(60, 60, 80));
        Self::cmd_row("infynon pkg install <pkg>",            "Detects from package.json / Cargo.toml / go.mod …");
        Self::cmd_row("infynon pkg --strict install <pkg>",   "Treat WARN as BLOCKED (CI mode)");

        println!("\n  {}\n", "── Eagle Eye  (scheduled monitoring) ──────────────────────────".truecolor(60, 60, 80));
        Self::cmd_row("infynon pkg eagle-eye setup",          "Interactive setup: SMTP, paths, risk level");
        Self::cmd_row("infynon pkg eagle-eye start",          "Start scheduled vulnerability monitoring");
        Self::cmd_row("infynon pkg eagle-eye status",         "Show current Eagle Eye config and status");
        Self::cmd_row("infynon pkg eagle-eye enable",         "Enable Eagle Eye monitoring");
        Self::cmd_row("infynon pkg eagle-eye disable",        "Disable Eagle Eye monitoring");

        println!("\n  {}\n", "── Security & Analysis ────────────────────────────────────────".truecolor(60, 60, 80));
        Self::cmd_row("infynon pkg scan",                     "Scan lock files for known CVEs");
        Self::cmd_row("infynon pkg audit",                    "Deep recursive dependency scan with tree");
        Self::cmd_row("infynon pkg why <package>",            "Trace why a package is in your tree");
        Self::cmd_row("infynon pkg outdated",                 "Check for outdated dependencies");
        Self::cmd_row("infynon pkg diff <pkg> <v1> <v2>",     "Compare two versions of a package");
        Self::cmd_row("infynon pkg doctor",                   "Health check: dupes, unused, phantoms");
        Self::cmd_row("infynon pkg size <package>",           "Show package size and dep count");
        Self::cmd_row("infynon pkg search <query>",           "Cross-ecosystem package search");
        Self::cmd_row("infynon pkg fix --auto",               "Auto-fix all vulnerable deps");
        Self::cmd_row("infynon pkg clean",                    "Find & remove unused dependencies");
        Self::cmd_row("infynon pkg migrate <from> <to>",      "Migrate between package managers");

        Self::divider();
        println!();
    }

    fn footer(start: Instant, mode: &str, author1: &str, author2: &str) {
        let elapsed = start.elapsed();
        Self::divider();
        println!(
            "\n  {} {} {} {}  ·  {} & {} are sweating on this — {}\n",
            "⚡".yellow(),
            format!("Arrived in {}ms", elapsed.as_millis()).bold().truecolor(0, 210, 255),
            "·".truecolor(60, 60, 80),
            mode.truecolor(100, 100, 120).italic(),
            author1.bold().truecolor(255, 80, 80),
            author2.bold().truecolor(180, 80, 255),
            "shipping soon 🚀".bold().truecolor(0, 255, 160)
        );
    }
}
