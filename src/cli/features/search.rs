use super::*;

mod backends;
mod display;
mod http;
mod matching;
mod resolve;
mod signals;
mod signals_extra;
mod types;

#[cfg(test)]
mod tests;

use backends::search_backend;
use display::render_results;
use matching::normalize_pkg_name;
use resolve::resolve_search_ecosystems;
use signals_extra::compute_match_score;
use types::SearchHit;

pub fn cmd_search(query: &str, ecosystem: Option<&str>) {
    println!();
    Logger::title("INFYNON Package Search", "blue");
    Logger::step(&format!("Searching '{}'...", query));

    let client = http_client();
    let ecosystems = match resolve_search_ecosystems(ecosystem) {
        Ok(value) => value,
        Err(message) => {
            Logger::error(&message);
            println!();
            return;
        }
    };

    let query_norm = normalize_pkg_name(query);
    let mut results: Vec<SearchHit> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    let sp = spinner();
    for eco in ecosystems {
        sp.set_message(format!("Searching {}...", eco));
        let (mut hits, note) = search_backend(eco, client, query);
        if let Some(note) = note {
            notes.push(note);
        }
        for hit in &mut hits {
            hit.score += compute_match_score(query, &query_norm, &hit.pkg, &hit.signal);
        }
        results.extend(hits);
    }
    sp.finish_and_clear();

    if !render_results(query, &mut results, &notes) {
        println!();
    }
}
