use std::collections::HashMap;

use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::Value;

use super::super::http::{build_query_url, fetch_json};
use super::super::signals::{
    add_release_age_qualifiers, add_version_qualifiers, licenses_present, push_qualifier,
};
use super::super::signals_extra::{build_signal, version_base_score};
use super::super::types::SearchHit;

#[derive(Deserialize)]
struct SearchPackage {
    name: String,
    #[serde(default)]
    latest_stable_version: Option<String>,
    meta: Option<SearchMeta>,
    downloads: Option<SearchDownloads>,
}

#[derive(Deserialize)]
struct SearchMeta {
    description: Option<String>,
}

#[derive(Deserialize)]
struct SearchDownloads {
    all: Option<u64>,
}

#[derive(Deserialize)]
struct PackageDetail {
    meta: Option<DetailMeta>,
    #[serde(default)]
    owners: Vec<Value>,
    downloads: Option<DetailDownloads>,
    inserted_at: Option<String>,
    updated_at: Option<String>,
    latest_stable_version: Option<String>,
}

#[derive(Deserialize)]
struct DetailMeta {
    #[serde(default)]
    licenses: Vec<String>,
    #[serde(default)]
    links: HashMap<String, String>,
}

#[derive(Deserialize)]
struct DetailDownloads {
    all: Option<u64>,
    recent: Option<u64>,
}

pub(super) fn search(client: &Client, query: &str) -> Vec<SearchHit> {
    let url = build_query_url("https://hex.pm/api/packages", &[("search", query)]);
    fetch_json::<Vec<SearchPackage>>(client, &url)
        .map(|response| response.into_iter().take(8).map(|item| to_hit(client, query, item)).collect())
        .unwrap_or_default()
}

fn to_hit(client: &Client, query: &str, item: SearchPackage) -> SearchHit {
    let detail_url = format!("https://hex.pm/api/packages/{}", item.name);
    let mut qualifiers = Vec::new();
    let mut version = item.latest_stable_version.clone().unwrap_or_else(|| "-".to_string());
    let mut score = ((item.downloads.as_ref().and_then(|value| value.all).unwrap_or(0) / 100_000).min(30)) as i32;
    if item.downloads.as_ref().and_then(|value| value.all).unwrap_or(0) >= 100_000 {
        push_qualifier(&mut qualifiers, "popular");
    }

    if let Some(detail) = fetch_json::<PackageDetail>(client, &detail_url) {
        if let Some(latest) = detail.latest_stable_version.as_deref() {
            version = latest.to_string();
        }
        if detail.downloads.as_ref().and_then(|value| value.recent).unwrap_or(0) >= 5_000 {
            push_qualifier(&mut qualifiers, "maintained");
        }
        if detail.downloads.as_ref().and_then(|value| value.all).unwrap_or(0) >= 100_000 {
            push_qualifier(&mut qualifiers, "popular");
        }
        if !detail.owners.is_empty() {
            push_qualifier(&mut qualifiers, "owners");
        }
        if detail.meta.as_ref().map(|meta| licenses_present(&meta.licenses)).unwrap_or(false) {
            push_qualifier(&mut qualifiers, "licensed");
        }
        if detail.meta.as_ref().map(|meta| meta.links.iter().any(|(key, value)| {
            !value.trim().is_empty()
                && ["github", "gitlab", "source", "repo"]
                    .iter()
                    .any(|needle| key.to_ascii_lowercase().contains(needle))
        })).unwrap_or(false)
        {
            push_qualifier(&mut qualifiers, "repo");
        }
        if detail.meta.as_ref().map(|meta| !meta.links.is_empty()).unwrap_or(false) {
            push_qualifier(&mut qualifiers, "docs");
        }
        add_release_age_qualifiers(&mut qualifiers, detail.updated_at.as_deref().or(detail.inserted_at.as_deref()));
    }

    add_version_qualifiers(&mut qualifiers, &version);
    score += version_base_score(&version);
    SearchHit {
        eco: "Hex".into(),
        signal: build_signal(query, &item.name, &qualifiers),
        pkg: item.name,
        ver: version,
        desc: item.meta.and_then(|meta| meta.description).unwrap_or_default(),
        score,
    }
}
