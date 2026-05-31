use reqwest::blocking::Client;
use serde::Deserialize;

use super::super::http::{build_query_url, fetch_json};
use super::super::signals::{
    add_release_age_qualifiers, add_version_qualifiers, has_non_empty, license_present,
    push_qualifier,
};
use super::super::signals_extra::{build_signal, version_base_score};
use super::super::types::SearchHit;

#[derive(Deserialize)]
struct SearchResponse {
    data: Vec<SearchPackage>,
}

#[derive(Deserialize)]
struct SearchPackage {
    id: String,
    version: String,
    description: Option<String>,
    #[serde(rename = "totalDownloads")]
    total_downloads: Option<u64>,
    verified: Option<bool>,
}

#[derive(Deserialize)]
struct RegistrationIndex {
    #[serde(default)]
    items: Vec<RegistrationPage>,
}

#[derive(Deserialize)]
struct RegistrationPage {
    #[serde(default)]
    items: Vec<RegistrationLeaf>,
}

#[derive(Deserialize)]
struct RegistrationLeaf {
    #[serde(rename = "catalogEntry")]
    catalog_entry: Option<CatalogEntry>,
}

#[derive(Deserialize)]
struct CatalogEntry {
    version: Option<String>,
    #[serde(rename = "licenseExpression")]
    license_expression: Option<String>,
    #[serde(rename = "projectUrl")]
    project_url: Option<String>,
    published: Option<String>,
    listed: Option<bool>,
}

pub(super) fn search(client: &Client, query: &str) -> Vec<SearchHit> {
    let url = build_query_url(
        "https://api-v2v3search-0.nuget.org/query",
        &[("q", query), ("take", "8"), ("prerelease", "false")],
    );
    fetch_json::<SearchResponse>(client, &url)
        .map(|response| {
            response
                .data
                .into_iter()
                .map(|item| to_hit(client, query, item))
                .collect()
        })
        .unwrap_or_default()
}

fn to_hit(client: &Client, query: &str, item: SearchPackage) -> SearchHit {
    let mut qualifiers = Vec::new();
    let mut score = ((item.total_downloads.unwrap_or(0) / 1_000_000).min(40)) as i32;
    if item.verified.unwrap_or(false) {
        push_qualifier(&mut qualifiers, "verified");
    }
    if item.total_downloads.unwrap_or(0) >= 1_000_000 {
        push_qualifier(&mut qualifiers, "popular");
    }

    let detail_url = format!(
        "https://api.nuget.org/v3/registration5-semver1/{}/index.json",
        item.id.to_ascii_lowercase()
    );
    if let Some(detail) = fetch_json::<RegistrationIndex>(client, &detail_url) {
        if let Some(entry) = detail
            .items
            .into_iter()
            .flat_map(|page| page.items.into_iter())
            .filter_map(|leaf| leaf.catalog_entry)
            .find(|entry| {
                entry
                    .version
                    .as_deref()
                    .map(|value| value.eq_ignore_ascii_case(&item.version))
                    .unwrap_or(false)
            })
        {
            if license_present(entry.license_expression.as_deref()) {
                push_qualifier(&mut qualifiers, "licensed");
            }
            if has_non_empty(entry.project_url.as_deref()) {
                push_qualifier(&mut qualifiers, "repo");
            }
            if entry.listed == Some(false) {
                push_qualifier(&mut qualifiers, "unlisted");
            }
            add_release_age_qualifiers(&mut qualifiers, entry.published.as_deref());
        }
    }

    add_version_qualifiers(&mut qualifiers, &item.version);
    score += version_base_score(&item.version);
    SearchHit {
        eco: "NuGet".into(),
        signal: build_signal(query, &item.id, &qualifiers),
        pkg: item.id,
        ver: item.version,
        desc: item.description.unwrap_or_default(),
        score,
    }
}
