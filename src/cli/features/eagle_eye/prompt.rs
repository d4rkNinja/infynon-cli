use owo_colors::OwoColorize;
use std::io::{self, Write};

pub(super) fn prompt_line(label: &str, default: &str) -> String {
    print!("  {} [{}]: ", label, default);
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    let trimmed = input.trim();
    if trimmed.is_empty() {
        default.to_string()
    } else {
        trimmed.to_string()
    }
}

pub(super) fn prompt_optional_line(label: &str, current: &str) -> Option<String> {
    print!("  {} [{}]: ", label, current);
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    let trimmed = input.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub(super) fn prompt_u16(label: &str, default: u16) -> u16 {
    print!("  {} [{}]: ", label, default);
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    input.trim().parse::<u16>().unwrap_or(default)
}

pub(super) fn prompt_u32(label: &str, default: u32) -> u32 {
    print!("  {} [{}]: ", label, default);
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    input.trim().parse::<u32>().unwrap_or(default).max(1)
}

pub(super) fn prompt_bool(label: &str, current: bool) -> bool {
    let default = if current { "yes" } else { "no" };
    print!("  {} [{}]: ", label, default);
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    match input.trim().to_ascii_lowercase().as_str() {
        "no" | "false" | "n" => false,
        "" => current,
        _ => true,
    }
}

pub(super) fn parse_csv_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
        .collect()
}

pub(super) fn collect_scan_paths(existing: &mut Vec<String>) {
    println!("  Enter full paths to project directories (one per line).");
    println!("  Eagle Eye will scan for lock files in each path.");
    println!("  Enter an empty line when done.\n");

    if !existing.is_empty() {
        println!("  Current paths:");
        for path in existing.iter() {
            println!(
                "    {} {}",
                "·".truecolor(60, 60, 80),
                path.truecolor(180, 180, 200)
            );
        }
        print!("\n  Keep existing paths? [Y/n]: ");
        io::stdout().flush().ok();
        let mut keep = String::new();
        io::stdin().read_line(&mut keep).ok();
        if keep.trim().eq_ignore_ascii_case("n") {
            existing.clear();
        }
    }

    loop {
        print!("  Path {}: ", existing.len() + 1);
        io::stdout().flush().ok();
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        let trimmed = input.trim();
        if trimmed.is_empty() {
            break;
        }
        existing.push(trimmed.to_string());
    }
}
