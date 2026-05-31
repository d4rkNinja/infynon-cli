use std::collections::HashSet;

use super::matching::best_follow_up_hint;
use super::types::SearchHit;
use crate::tui::logger::Logger;
use owo_colors::OwoColorize;
use tabled::settings::{object::Rows, Color, Padding, Style};
use tabled::{Table, Tabled};

pub(super) fn render_results(query: &str, results: &mut Vec<SearchHit>, notes: &[String]) -> bool {
    dedupe_hits(results);
    results.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.eco.cmp(&b.eco))
            .then_with(|| a.pkg.cmp(&b.pkg))
    });

    if results.is_empty() {
        Logger::info(&format!("No packages found for '{}'", query));
        for note in notes {
            Logger::raw_dim(&format!("  {}", note));
        }
        return false;
    }

    #[derive(Tabled)]
    struct Row {
        #[tabled(rename = " Ecosystem ")]
        eco: String,
        #[tabled(rename = " Package ")]
        pkg: String,
        #[tabled(rename = " Version ")]
        ver: String,
        #[tabled(rename = " Signal ")]
        signal: String,
        #[tabled(rename = " Description ")]
        desc: String,
    }

    let rows: Vec<Row> = results
        .iter()
        .take(20)
        .map(|hit| Row {
            eco: hit.eco.clone(),
            pkg: hit.pkg.clone(),
            ver: hit.ver.clone(),
            signal: hit.signal.clone(),
            desc: truncate_description(&hit.desc, 52),
        })
        .collect();
    let count = rows.len();

    let mut table = Table::new(rows);
    table
        .with(Style::modern())
        .with(Padding::new(1, 1, 0, 0))
        .modify(Rows::first(), Color::BOLD | Color::FG_BRIGHT_CYAN);

    println!();
    println!("{}", table);
    if let Some(best) = best_follow_up_hint(query, results) {
        println!(
            "\n  {}  Did you mean {} ({})?",
            "->".truecolor(100, 100, 140),
            best.pkg.bold(),
            best.eco.truecolor(120, 120, 140)
        );
    }
    for note in notes {
        println!("  {}", note.truecolor(120, 120, 140));
    }
    println!(
        "\n  {}  {} packages matching '{}'\n",
        "*".truecolor(0, 210, 255),
        count.to_string().bold(),
        query.bold()
    );
    true
}

fn truncate_description(description: &str, max_chars: usize) -> String {
    let trimmed = description.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.chars().count() <= max_chars {
        return trimmed.to_string();
    }
    trimmed.chars().take(max_chars).collect::<String>()
}

fn dedupe_hits(results: &mut Vec<SearchHit>) {
    let mut seen = HashSet::new();
    results.retain(|hit| seen.insert((hit.eco.clone(), hit.pkg.to_ascii_lowercase())));
}
