use std::path::{Path, PathBuf};

use owo_colors::OwoColorize;

use crate::tui::logger::Logger;

fn env_file_path() -> PathBuf {
    Path::new(".infynon").join(".env")
}

/// Parse a .env file into an ordered list of (key, value) pairs.
/// `None` value means the line is a comment or blank — preserved verbatim on write.
fn read_env_file() -> Vec<(String, Option<String>)> {
    let path = env_file_path();
    let content = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    content
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with('#') || trimmed.is_empty() {
                (line.to_string(), None)
            } else if let Some(eq) = trimmed.find('=') {
                let key = trimmed[..eq].trim().to_string();
                let value = trimmed[eq + 1..].to_string();
                (key, Some(value))
            } else {
                (trimmed.to_string(), Some(String::new()))
            }
        })
        .collect()
}

fn write_env_file(entries: &[(String, Option<String>)]) -> std::io::Result<()> {
    let path = env_file_path();
    std::fs::create_dir_all(path.parent().unwrap())?;
    let mut content = String::new();
    for (key, val) in entries {
        match val {
            Some(v) => content.push_str(&format!("{}={}\n", key, v)),
            None => {
                content.push_str(key);
                content.push('\n');
            }
        }
    }
    std::fs::write(&path, content)
}

fn looks_sensitive(key: &str) -> bool {
    let upper = key.to_uppercase();
    ["TOKEN", "SECRET", "PASSWORD", "KEY", "PASS", "AUTH", "CREDENTIAL", "PRIVATE"]
        .iter()
        .any(|word| upper.contains(word))
}

fn mask(value: &str) -> String {
    if value.len() <= 6 {
        "***".to_string()
    } else {
        format!("{}***", &value[..4])
    }
}

pub fn cmd_env_list() {
    println!();
    Logger::title("Environment Variables", "cyan");
    println!();

    let entries = read_env_file();
    let pairs: Vec<_> = entries
        .iter()
        .filter_map(|(k, v)| v.as_ref().map(|val| (k, val)))
        .collect();

    if pairs.is_empty() {
        println!("  No variables set. Use: infynon weave env set KEY VALUE");
        println!();
        println!("  File: {}", env_file_path().display().to_string().truecolor(100, 100, 140));
        println!();
        return;
    }

    println!(
        "  {:<32} {}",
        "KEY".truecolor(100, 100, 140),
        "VALUE".truecolor(100, 100, 140),
    );
    println!("  {}", "─".repeat(60).truecolor(50, 50, 80));

    for (key, value) in &pairs {
        let display = if looks_sensitive(key) {
            mask(value).truecolor(120, 120, 160).to_string()
        } else {
            value.truecolor(200, 200, 220).to_string()
        };
        println!("  {:<32} {}", key.bold(), display);
    }

    println!();
    println!("  {} variable(s)", pairs.len().to_string().bright_cyan());
    println!("  File: {}", env_file_path().display().to_string().truecolor(100, 100, 140));
    println!("  Tip: reference as {{$KEY}} in any node path, headers, or body");
    println!();
}

pub fn cmd_env_set(key: &str, value: &str) {
    if key.is_empty() {
        Logger::error("Key cannot be empty.");
        return;
    }

    let mut entries = read_env_file();
    let mut found = false;
    for (k, v) in entries.iter_mut() {
        if v.is_some() && k == key {
            *v = Some(value.to_string());
            found = true;
            break;
        }
    }

    if !found {
        entries.push((key.to_string(), Some(value.to_string())));
    }

    match write_env_file(&entries) {
        Ok(()) => {
            let sensitive = looks_sensitive(key);
            let display = if sensitive { mask(value) } else { value.to_string() };
            let label = if found { "(updated)" } else { "(added)" };
            println!(
                "  {}  {}={} {}",
                "✔".bright_green(),
                key.bold(),
                display.truecolor(200, 200, 220),
                label.truecolor(100, 100, 140),
            );
        }
        Err(e) => Logger::error(&format!("Could not write .env: {}", e)),
    }
}

pub fn cmd_env_delete(key: &str) {
    let mut entries = read_env_file();

    let before = entries.len();
    entries.retain(|(k, v)| !(v.is_some() && k == key));

    if entries.len() == before {
        Logger::error(&format!("Variable '{}' not found.", key));
        return;
    }

    match write_env_file(&entries) {
        Ok(()) => println!("  {}  Deleted: {}", "✔".bright_green(), key.bold()),
        Err(e) => Logger::error(&format!("Could not write .env: {}", e)),
    }
}

pub fn cmd_env_get(key: &str, reveal: bool) {
    let entries = read_env_file();
    let value = entries.iter().find_map(|(k, v)| {
        if k == key { v.as_deref() } else { None }
    });

    match value {
        Some(value) => {
            let sensitive = looks_sensitive(key);
            let display = if !reveal && sensitive { mask(value) } else { value.to_string() };
            println!("  {}  {}={}", "→".bright_cyan(), key.bold(), display.truecolor(200, 200, 220));
            if !reveal && sensitive {
                println!("     (use --reveal to show full value)");
            }
        }
        None => Logger::error(&format!("Variable '{}' not found.", key)),
    }
}
