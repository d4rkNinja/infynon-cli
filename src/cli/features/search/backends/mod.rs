use reqwest::blocking::Client;

use super::types::SearchHit;

mod crates_io;
mod go;
mod hex;
mod npm;
mod nuget;
mod packagist;
mod pubdev;
mod pypi;
mod rubygems;

pub(super) use go::escape_go_module_path;

pub(super) fn search_backend(
    eco: &str,
    client: &Client,
    query: &str,
) -> (Vec<SearchHit>, Option<String>) {
    match eco {
        "npm" => (npm::search(client, query), None),
        "crates.io" => (crates_io::search(client, query), None),
        "PyPI" => pypi::search(client, query),
        "RubyGems" => (rubygems::search(client, query), None),
        "Packagist" => (packagist::search(client, query), None),
        "pub.dev" => (pubdev::search(client, query), None),
        "NuGet" => (nuget::search(client, query), None),
        "Hex" => (hex::search(client, query), None),
        "Go" => (go::search(client, query), None),
        _ => (Vec::new(), None),
    }
}
