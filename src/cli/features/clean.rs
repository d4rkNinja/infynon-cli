use super::*;
use super::doctor::{find_unused_deps, collect_source};

pub fn cmd_clean(pkg_file: Option<&str>) {
    use std::io::{self, Write};

    println!();
    Logger::title("INFYNON Clean", "blue");
    Logger::step("Scanning for unused dependencies...");

    let source_js = collect_source(&["js", "ts", "jsx", "tsx", "mjs", "cjs"]);
    let source_rs = collect_source(&["rs"]);
    let unused = find_unused_deps(&source_js, &source_rs);
    if unused.is_empty() {
        println!();
        Logger::success("No unused dependencies found. Your project is clean!");
        println!();
        return;
    }

    println!();
    println!(
        "  {}  Found {} potentially unused dependencies:\n",
        "⚠".bright_yellow(), unused.len()
    );
    for (i, (name, eco)) in unused.iter().enumerate() {
        println!(
            "     {}  {} {}",
            format!("[{}]", i + 1).bold().truecolor(100, 100, 140),
            name.bold(), format!("({})", eco).truecolor(120, 120, 140)
        );
    }

    println!();
    print!("  Remove all unused? (y/N): ");
    io::stdout().flush().ok();
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).ok();

    if choice.trim().to_lowercase() != "y" {
        Logger::raw_dim("  Skipped. No changes made.");
        println!();
        return;
    }

    // Generate and execute uninstall commands
    for (name, eco) in &unused {
        let cmd = match eco.as_str() {
            "npm" => {
                if Path::new("yarn.lock").exists() { format!("yarn remove {}", name) }
                else if Path::new("pnpm-lock.yaml").exists() { format!("pnpm remove {}", name) }
                else { format!("npm uninstall {}", name) }
            }
            "cargo" => format!("cargo remove {}", name),
            "pip" => format!("pip uninstall -y {}", name),
            _ => continue,
        };

        let result = crate::cli::run_pkg_cmd(&cmd);
        match result {
            Ok(out) if out.status.success() => {
                println!("  {}  {} removed ({})", "✔".bright_green(), name.bold(), cmd.truecolor(100, 100, 120));
            }
            Ok(out) => {
                println!("  {}  {} failed: exit {}", "✘".bright_red(), name.bold(), out.status.code().unwrap_or(-1));
            }
            Err(e) => {
                println!("  {}  {} error: {}", "✘".bright_red(), name.bold(), e);
            }
        }
    }
    println!();
}
