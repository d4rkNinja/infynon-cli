use owo_colors::OwoColorize;
use std::time::Instant;

pub struct Logger;

// Consistent column widths
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
        // emoji is 2-wide, pad label to LABEL_W - 2 to visually align
        let padded = format!("{:<width$}", label, width = LABEL_W - 2);
        println!("  {}  {}  {}", icon, padded.bold().truecolor(255, 170, 50), value.truecolor(180, 180, 200));
    }

    fn cont(value: &str) {
        // continuation line — align to same value column (icon=1 + 2sp + label + 2sp = ~23)
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
        println!("  {}\n", "Universal Package Security Manager  ·  v0.1.0-beta.1".truecolor(120, 120, 140).italic());

        Self::divider();
        Self::section("⚡", "What is INFYNON?");

        Self::row("🛡️", "Mission",      "Drop-in replacement for any package manager (npm, pip, cargo,");
        Self::cont(                      "gem, go, composer, nuget, hex, pub) — intercepts every install.");
        Self::row("🔎", "How it works", "Runs a 3-layer security pipeline before any package touches disk:");
        Self::row("  ", "Layer 1",      "In-memory blocklist trie lookup     (<1ms, never skipped)");
        Self::row("  ", "Layer 2",      "Static heuristic scan — scripts, age, typosquatting  (<50ms)");
        Self::row("  ", "Layer 3",      "LLM deep-code analysis via local Ollama  (<8s, flagged only)");
        Self::row("📦", "Ecosystems",   "npm · yarn · pnpm · bun · pip · uv · poetry · cargo · go");
        Self::cont(                      "gem · composer · nuget · hex · pub  — auto-detected from project");
        Self::row("🌙", "Night Daemon", "Nightly CVE/GitHub feed crawl → LLM extraction → hot-swap");
        Self::row("🔒", "Privacy",      "Local-first. Zero data leaves machine unless API mode is opted in.");

        Self::divider();
        Self::section("📟", "Commands");

        Self::cmd_row("infynon daemon",              "Start nightly intelligence pipeline daemon");
        Self::cmd_row("infynon dashboard",           "Open real-time TUI security dashboard");
        Self::cmd_row("infynon update-intel",        "Manually force a CVE intel refresh now");
        Self::cmd_row("infynon pkg <pm> install <pkg>", "Secure install via any package manager");

        Self::footer(start, "Firewall Engine", "d4rkninja", "whit3ninj4");
    }

    // ── infynon pkg splash ───────────────────────────────────────────────────
    pub fn splash_pkg() {
        // Simple styled wordmark — no ASCII art
        println!();
        println!(
            "  {} {}",
            "infynon pkg".bold().truecolor(120, 80, 255),
            "·  Package Security Manager  ·  v0.1.0-beta.1".truecolor(120, 120, 140).italic()
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
