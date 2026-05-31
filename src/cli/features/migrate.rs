use super::*;
use crate::cli::scan;

pub fn cmd_migrate(from: &str, to: &str) {
    use std::io::{self, Write};

    println!();
    Logger::title("INFYNON Migrate", "blue");
    Logger::step(&format!("Migrating from {} → {}...", from, to));

    // Validate migration path
    let valid_js = ["npm", "yarn", "pnpm", "bun"];
    let valid_py = ["pip", "uv", "poetry"];

    let is_js = valid_js.contains(&from) && valid_js.contains(&to);
    let is_py = valid_py.contains(&from) && valid_py.contains(&to);

    if !is_js && !is_py {
        Logger::error(&format!(
            "Migration from '{}' to '{}' is not supported.",
            from, to
        ));
        println!();
        println!("  {}  Supported migrations:", "ℹ".bright_cyan());
        println!("     JavaScript: npm, yarn, pnpm, bun");
        println!("     Python:     pip, uv, poetry");
        println!();
        return;
    }

    if from == to {
        Logger::error("Source and target are the same.");
        println!();
        return;
    }

    enum MigrateStep {
        Delete {
            desc: String,
            path: String,
        },
        Run {
            desc: String,
            invocation: crate::cli::PkgInvocation,
        },
    }
    impl MigrateStep {
        fn desc(&self) -> &str {
            match self {
                Self::Delete { desc, .. } | Self::Run { desc, .. } => desc,
            }
        }
        fn display_cmd(&self) -> String {
            match self {
                Self::Delete { path, .. } => format!("delete {}", path),
                Self::Run { invocation, .. } => invocation.display(),
            }
        }
    }

    let mut steps: Vec<MigrateStep> = Vec::new();

    if is_js {
        let old_lock = match from {
            "npm" => "package-lock.json",
            "yarn" => "yarn.lock",
            "pnpm" => "pnpm-lock.yaml",
            "bun" => "bun.lockb",
            _ => "",
        };
        if !old_lock.is_empty() && Path::new(old_lock).exists() {
            steps.push(MigrateStep::Delete {
                desc: format!("Remove {}", old_lock),
                path: old_lock.to_string(),
            });
        }
        if Path::new("node_modules").is_dir() {
            steps.push(MigrateStep::Delete {
                desc: "Remove node_modules".into(),
                path: "node_modules".into(),
            });
        }
        let invocation = match to {
            "npm" => Some(crate::cli::PkgInvocation::from_args("npm", &["install"])),
            "yarn" => Some(crate::cli::PkgInvocation::from_args("yarn", &["install"])),
            "pnpm" => Some(crate::cli::PkgInvocation::from_args("pnpm", &["install"])),
            "bun" => Some(crate::cli::PkgInvocation::from_args("bun", &["install"])),
            _ => None,
        };
        if let Some(invocation) = invocation {
            steps.push(MigrateStep::Run {
                desc: format!("Install with {}", to),
                invocation,
            });
        }
    }

    if is_py {
        let dep_file = if Path::new("requirements.txt").exists() {
            Some("requirements.txt")
        } else if Path::new("pyproject.toml").exists() {
            Some("pyproject.toml")
        } else {
            None
        };

        let Some(dep_file) = dep_file else {
            Logger::error("No requirements.txt or pyproject.toml found.");
            println!();
            return;
        };

        match to {
            "uv" => {
                let cmd = if dep_file == "requirements.txt" {
                    crate::cli::PkgInvocation::from_args(
                        "uv",
                        &["pip", "install", "-r", "requirements.txt"],
                    )
                } else {
                    crate::cli::PkgInvocation::from_args("uv", &["pip", "install", "."])
                };
                steps.push(MigrateStep::Run {
                    desc: "Install with uv".into(),
                    invocation: cmd,
                });
            }
            "poetry" => {
                if !Path::new("pyproject.toml").exists() {
                    steps.push(MigrateStep::Run {
                        desc: "Initialize poetry".into(),
                        invocation: crate::cli::PkgInvocation::from_args(
                            "poetry",
                            &["init", "--no-interaction"],
                        ),
                    });
                }
                steps.push(MigrateStep::Run {
                    desc: "Install with poetry".into(),
                    invocation: crate::cli::PkgInvocation::from_args("poetry", &["install"]),
                });
            }
            "pip" => {
                let pip = crate::ecosystems::detector::resolve_binary("pip");
                let cmd = if dep_file == "requirements.txt" {
                    crate::cli::PkgInvocation::from_args(
                        &pip,
                        &["install", "-r", "requirements.txt"],
                    )
                } else {
                    crate::cli::PkgInvocation::from_args(&pip, &["install", "."])
                };
                steps.push(MigrateStep::Run {
                    desc: "Install with pip".into(),
                    invocation: cmd,
                });
            }
            _ => {}
        }
    }

    // Show plan
    println!();
    println!("  {}  Migration plan:\n", "→".truecolor(100, 100, 140));
    for (i, step) in steps.iter().enumerate() {
        println!(
            "     {}  {}  {}",
            format!("{}.", i + 1).bold().truecolor(0, 210, 255),
            step.desc().bold(),
            format!("({})", step.display_cmd()).truecolor(100, 100, 120)
        );
    }

    println!();
    print!("  Proceed? (y/N): ");
    io::stdout().flush().ok();
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).ok();

    if choice.trim().to_lowercase() != "y" {
        Logger::raw_dim("  Migration cancelled.");
        println!();
        return;
    }

    // Execute
    println!();
    for step in &steps {
        match step {
            MigrateStep::Delete { desc, path } => {
                let p = Path::new(path);
                let result = if p.is_dir() {
                    fs::remove_dir_all(p)
                } else {
                    fs::remove_file(p)
                };
                match result {
                    Ok(_) => println!("  {}  {}", "✔".bright_green(), desc.bold()),
                    Err(e) => println!("  {}  {} — {}", "✘".bright_red(), desc.bold(), e),
                }
            }
            MigrateStep::Run { desc, invocation } => {
                let sp = spinner();
                sp.set_message(desc.clone());
                let result = invocation.output();
                sp.finish_and_clear();
                match result {
                    Ok(out) if out.status.success() => {
                        println!("  {}  {}", "✔".bright_green(), desc.bold())
                    }
                    Ok(out) => {
                        println!(
                            "  {}  {} — exit {}",
                            "✘".bright_red(),
                            desc.bold(),
                            out.status.code().unwrap_or(-1)
                        );
                        let stderr = String::from_utf8_lossy(&out.stderr);
                        for line in stderr.lines().take(4) {
                            println!(
                                "       {} {}",
                                "│".truecolor(80, 80, 100),
                                line.truecolor(200, 80, 80)
                            );
                        }
                    }
                    Err(e) => println!("  {}  {} — {}", "✘".bright_red(), desc.bold(), e),
                }
            }
        }
    }

    // Run vulnerability scan on new setup
    println!();
    Logger::step("Running post-migration vulnerability scan...");
    scan::run_scan(None, None, None, false);
}
