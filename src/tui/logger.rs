use owo_colors::OwoColorize;

pub struct Logger;

const LABEL_W: usize = 18;
const CMD_W: usize = 40;

impl Logger {
    pub fn title(text: &str, bg: &str) {
        let final_text = match bg {
            "blue" => format!(" {} ", text)
                .bold()
                .black()
                .on_bright_cyan()
                .to_string(),
            "red" => format!(" {} ", text)
                .bold()
                .black()
                .on_bright_red()
                .to_string(),
            _ => format!(" {} ", text)
                .bold()
                .black()
                .on_bright_black()
                .to_string(),
        };
        println!("\n{}\n", final_text);
    }

    pub fn info(msg: &str) {
        println!("  {} {}", "::".cyan(), msg.white());
    }

    pub fn success(msg: &str) {
        println!("  {} {}", "x".green(), msg.bold().green());
    }

    pub fn error(msg: &str) {
        println!("  {} {}", "!".red(), msg.bold().red());
    }

    pub fn subtitle(icon: &str, label: &str, value: &str) {
        println!(
            "\n{} {} {}",
            icon.cyan(),
            label.bold().cyan(),
            value.bold().green()
        );
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
        println!("  {}", "-".repeat(66).truecolor(40, 40, 60));
    }

    fn section(icon: &str, title: &str) {
        println!("\n  {}  {}\n", icon, title.bold().truecolor(0, 210, 255));
    }

    fn row(icon: &str, label: &str, value: &str) {
        let padded = format!("{:<width$}", label, width = LABEL_W - 2);
        println!(
            "  {}  {}  {}",
            icon,
            padded.bold().truecolor(255, 170, 50),
            value.truecolor(180, 180, 200)
        );
    }

    fn cont(value: &str) {
        let pad = " ".repeat(LABEL_W + 5);
        println!("  {}{}", pad, value.truecolor(180, 180, 200));
    }

    fn cmd_row(cmd: &str, desc: &str) {
        let padded = format!("{:<width$}", cmd, width = CMD_W);
        println!(
            "  {}  {}",
            padded.bold().truecolor(120, 220, 120),
            desc.truecolor(140, 140, 160)
        );
    }

    pub fn splash_root() {
        println!();
        println!(
            "  {} {}",
            "infynon".bold().truecolor(0, 210, 255),
            format!("· v{}", env!("CARGO_PKG_VERSION"))
                .truecolor(120, 120, 140)
                .italic()
        );
        println!("  {}\n", "-".repeat(52).truecolor(60, 60, 90));

        Self::divider();
        Self::section("::", "Main Commands");

        Self::row("1", "pkg", "Package intelligence across 14 ecosystems");
        Self::cont("Secure installs, CVE scanning, audit, diff, doctor, and remediation.");
        Self::row("2", "weave", "API flow testing and security probes");
        Self::cont("Node graphs, context threading, run diff, prompts, and OpenAPI import.");
        Self::row("3", "trace", "Repo memory & provenance");
        Self::cont(
            "Redis or SQL-backed memory for teams, agents, package ownership, and retrieval.",
        );
        Self::row("4", "workspace", "User-global workspace registry");
        Self::cont("Manage ~/.infynon workspaces, defaults, paths, and folder mappings.");
        Self::row("5", "task", "User-global task orchestration");
        Self::cont("Track agent, model, prompt, pid, status, and kill/complete task lifecycle.");

        Self::divider();
        Self::section(">", "Style Guide");

        Self::cmd_row("infynon pkg scan", "Scan lock files for known CVEs");
        Self::cmd_row(
            "infynon pkg audit",
            "Deep dependency audit with risk breakdown",
        );
        Self::cmd_row(
            "infynon weave flow run <id>",
            "Run a saved API flow end-to-end",
        );
        Self::cmd_row(
            "infynon weave ai probe <id>",
            "Run built-in flow security probes",
        );
        Self::cmd_row(
            "infynon trace overview",
            "Show Trace commands and backend guidance",
        );
        Self::cmd_row(
            "infynon workspace create app --mutate",
            "Create a user-global workspace entry",
        );
        Self::cmd_row(
            "infynon task create build-api --mutate",
            "Create a user-global task entry",
        );

        Self::divider();
        println!();
    }

    pub fn trace_overview() {
        println!();
        println!(
            "  {} {}",
            "infynon trace".bold().truecolor(120, 220, 120),
            "· repo memory & provenance"
                .truecolor(120, 120, 140)
                .italic()
        );
        println!("  {}\n", "-".repeat(52).truecolor(60, 90, 60));

        Self::divider();
        Self::section("::", "Main Commands");

        Self::cmd_row("infynon trace init", "Initialize local Trace state");
        Self::cmd_row(
            "infynon trace source add-redis",
            "Add Redis for fast live retrieval",
        );
        Self::cmd_row(
            "infynon trace source add-sql",
            "Add SQL for durable structured memory",
        );
        Self::cmd_row(
            "infynon trace retrieve",
            "Query notes by user, file, tag, or scope",
        );
        Self::cmd_row(
            "infynon trace note add",
            "Create notes for repo, PR, file, branch, or package",
        );
        Self::cmd_row(
            "infynon trace tui",
            "Open Trace TUI with notes, sources, and package risk",
        );

        Self::divider();
        println!();
    }

    pub fn splash_pkg() {
        println!();
        println!(
            "  {} {}",
            "infynon pkg".bold().truecolor(120, 80, 255),
            format!("· Package Intelligence · v{}", env!("CARGO_PKG_VERSION"))
                .truecolor(120, 120, 140)
                .italic()
        );
        println!("  {}\n", "-".repeat(52).truecolor(80, 50, 160));

        Self::divider();
        Self::section("::", "What is infynon pkg?");

        Self::row(
            ">",
            "Purpose",
            "Dependency risk scanning before install and during maintenance.",
        );
        Self::cont(
            "Focus: CVEs, dependency drift, install script risk, and ecosystem-wide visibility.",
        );
        Self::row(
            ">",
            "Coverage",
            "npm · yarn · pnpm · bun · pip · uv · poetry · cargo · go",
        );
        Self::cont("gem · composer · nuget · hex · pub");
        Self::row(
            ">",
            "Auto-Detect",
            "Scans package.json / Cargo.toml / go.mod / pyproject.toml and more.",
        );

        Self::divider();
        Self::section(">", "Common Commands");

        Self::cmd_row("infynon pkg scan", "Scan lock files for known CVEs");
        Self::cmd_row(
            "infynon pkg audit",
            "Deep recursive dependency scan with tree",
        );
        Self::cmd_row(
            "infynon pkg why <package>",
            "Trace why a package is in your tree",
        );
        Self::cmd_row("infynon pkg outdated", "Check for outdated dependencies");
        Self::cmd_row(
            "infynon pkg diff <pkg> <v1> <v2>",
            "Compare two versions of a package",
        );
        Self::cmd_row(
            "infynon pkg doctor",
            "Health check: dupes, unused, phantoms",
        );
        Self::cmd_row("infynon pkg fix --auto", "Auto-fix vulnerable dependencies");
        Self::cmd_row(
            "infynon pkg eagle-eye setup",
            "Configure scheduled vulnerability monitoring",
        );

        Self::divider();
        println!();
    }
}
