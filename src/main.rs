#![allow(dead_code)]

mod api;
mod cli;
mod config;
mod daemon;
mod ecosystems;
mod engine;
mod error;
mod models;
mod ninja;
mod trace;
mod tui;
mod utils;

use std::env;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    let exec_name = if args.is_empty() {
        "infynon".to_string()
    } else {
        let path = Path::new(&args[0]);
        path.file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase()
    };

    // Resolve which mode to run:
    //   "infynon-pkg ..."        -> package manager (symlink/copy)
    //   "infynon pkg ..."        -> package manager (subcommand)
    //   "infynon ..."            -> root command router
    let is_pkg_mode = exec_name.contains("infynon-pkg") || (args.len() > 1 && args[1] == "pkg");

    if is_pkg_mode {
        if let Err(e) = cli::run_package_manager() {
            eprintln!("Fatal Package Manager error: {}", e);
            std::process::exit(1);
        }
    } else if let Err(e) = cli::run_root() {
        eprintln!("Fatal INFYNON error: {}", e);
        std::process::exit(1);
    }
}
