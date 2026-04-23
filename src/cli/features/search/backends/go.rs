use reqwest::blocking::Client;
use regex::Regex;
use serde::Deserialize;

use super::super::http::{build_query_url, fetch_json, fetch_text};
use super::super::signals::{add_release_age_qualifiers, add_version_qualifiers, push_qualifier};
use super::super::signals_extra::{build_signal, version_base_score};
use super::super::types::SearchHit;

#[derive(Deserialize)]
struct LatestResponse {
    #[serde(rename = "Version")]
    version: Option<String>,
    #[serde(rename = "Time")]
    time: Option<String>,
}

pub(super) fn search(client: &Client, query: &str) -> Vec<SearchHit> {
    let url = build_query_url("https://pkg.go.dev/search", &[("q", query), ("m", "package")]);
    let html = match fetch_text(client, &url) {
        Some(value) => value,
        None => return Vec::new(),
    };

    let path_re = Regex::new(r#"<span class="SearchSnippet-header-path">\(([^<]+)\)</span>"#).expect("valid Go path regex");
    let synopsis_re = Regex::new(r#"<p class="SearchSnippet-synopsis"[^>]*>\s*(.*?)\s*</p>"#).expect("valid Go synopsis regex");
    let imported_by_re = Regex::new(r#"Imported by </span><strong>([0-9,]+)</strong>"#).expect("valid Go imported-by regex");
    let license_re = Regex::new(r#"\?tab=licenses"[^>]*>\s*([^<]+?)\s*</a>"#).expect("valid Go license regex");

    html.split(r#"<div class="SearchSnippet""#)
        .skip(1)
        .filter_map(|block| to_hit(client, query, block, &path_re, &synopsis_re, &imported_by_re, &license_re))
        .take(8)
        .collect()
}

fn to_hit(
    client: &Client,
    query: &str,
    block: &str,
    path_re: &Regex,
    synopsis_re: &Regex,
    imported_by_re: &Regex,
    license_re: &Regex,
) -> Option<SearchHit> {
    let path = strip_html(path_re.captures(block)?.get(1)?.as_str());
    if path.is_empty() {
        return None;
    }

    let synopsis = synopsis_re
        .captures(block)
        .and_then(|capture| capture.get(1))
        .map(|value| strip_html(value.as_str()))
        .unwrap_or_default();
    let imported_by = imported_by_re
        .captures(block)
        .and_then(|capture| capture.get(1))
        .map(|value| value.as_str().replace(',', "").parse::<u64>().unwrap_or(0))
        .unwrap_or(0);
    let license = license_re
        .captures(block)
        .and_then(|capture| capture.get(1))
        .map(|value| strip_html(value.as_str()))
        .unwrap_or_default();

    let latest_url = format!("https://proxy.golang.org/{}/@latest", escape_go_module_path(&path));
    let latest = fetch_json::<LatestResponse>(client, &latest_url);
    let version = latest.as_ref().and_then(|value| value.version.as_deref()).unwrap_or("-").to_string();

    let mut qualifiers = Vec::new();
    if imported_by >= 500 {
        push_qualifier(&mut qualifiers, "popular");
    }
    if !license.is_empty() {
        push_qualifier(&mut qualifiers, "licensed");
    }
    if path.starts_with("github.com/") || path.starts_with("gitlab.com/") || path.starts_with("bitbucket.org/") {
        push_qualifier(&mut qualifiers, "repo");
    }
    add_release_age_qualifiers(&mut qualifiers, latest.as_ref().and_then(|value| value.time.as_deref()));
    add_version_qualifiers(&mut qualifiers, &version);

    Some(SearchHit {
        eco: "Go".into(),
        signal: build_signal(query, &path, &qualifiers),
        pkg: path,
        ver: version.clone(),
        desc: synopsis,
        score: (imported_by.min(50_000) / 50) as i32 + version_base_score(&version),
    })
}

pub(crate) fn escape_go_module_path(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '!' => escaped.push_str("!!"),
            'A'..='Z' => {
                escaped.push('!');
                escaped.push(ch.to_ascii_lowercase());
            }
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn strip_html(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&#34;", "\"")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("<em>", "")
        .replace("</em>", "")
        .replace('\n', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
