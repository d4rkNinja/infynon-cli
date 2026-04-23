use reqwest::blocking::Client;
use serde::Deserialize;

use super::super::http::{build_query_url, fetch_json};
use super::super::signals::{
    add_release_age_qualifiers, add_version_qualifiers, has_non_empty, push_qualifier,
};
use super::super::signals_extra::{build_signal, version_base_score};
use super::super::types::SearchHit;

#[derive(Deserialize)]
struct SearchResponse {
    packages: Vec<SearchPackage>,
}

#[derive(Deserialize)]
struct SearchPackage {
    package: String,
}

#[derive(Deserialize)]
struct DetailResponse {
    latest: Latest,
}

#[derive(Deserialize)]
struct Latest {
    version: String,
    published: Option<String>,
    pubspec: Pubspec,
}

#[derive(Deserialize)]
struct Pubspec {
    repository: Option<String>,
    homepage: Option<String>,
    issue_tracker: Option<String>,
}

pub(super) fn search(client: &Client, query: &str) -> Vec<SearchHit> {
    let url = build_query_url("https://pub.dev/api/search", &[("q", query)]);
    fetch_json::<SearchResponse>(client, &url)
        .map(|response| {
            response
                .packages
                .into_iter()
                .take(8)
                .map(|item| to_hit(client, query, item))
                .collect()
        })
        .unwrap_or_default()
}

fn to_hit(client: &Client, query: &str, item: SearchPackage) -> SearchHit {
    let detail_url = format!("https://pub.dev/api/packages/{}", item.package);
    let mut qualifiers = Vec::new();
    let mut version = "-".to_string();
    if let Some(detail) = fetch_json::<DetailResponse>(client, &detail_url) {
        version = detail.latest.version.clone();
        if has_non_empty(detail.latest.pubspec.repository.as_deref()) {
            push_qualifier(&mut qualifiers, "repo");
        }
        if has_non_empty(detail.latest.pubspec.homepage.as_deref())
            || has_non_empty(detail.latest.pubspec.issue_tracker.as_deref())
        {
            push_qualifier(&mut qualifiers, "docs");
        }
        add_release_age_qualifiers(&mut qualifiers, detail.latest.published.as_deref());
    }

    add_version_qualifiers(&mut qualifiers, &version);
    SearchHit {
        eco: "pub.dev".into(),
        signal: build_signal(query, &item.package, &qualifiers),
        pkg: item.package,
        ver: version.clone(),
        desc: String::new(),
        score: version_base_score(&version),
    }
}
