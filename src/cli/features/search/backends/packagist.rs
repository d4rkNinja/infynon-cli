use std::collections::HashMap;

use reqwest::blocking::Client;
use serde::Deserialize;

use super::super::http::{build_query_url, fetch_json};
use super::super::signals::{
    add_release_age_qualifiers, add_version_qualifiers, has_non_empty, licenses_present,
    push_qualifier,
};
use super::super::signals_extra::{build_signal, version_base_score};
use super::super::types::SearchHit;

#[derive(Deserialize)]
struct SearchResponse {
    results: Vec<SearchPackage>,
}

#[derive(Deserialize)]
struct SearchPackage {
    name: String,
    description: Option<String>,
    downloads: Option<u64>,
    favers: Option<u64>,
    repository: Option<String>,
}

#[derive(Deserialize)]
struct DetailResponse {
    package: PackageDetail,
}

#[derive(Deserialize)]
struct PackageDetail {
    #[serde(default)]
    maintainers: Vec<Maintainer>,
    #[serde(default)]
    versions: HashMap<String, PackageVersion>,
}

#[derive(Deserialize)]
struct Maintainer {
    name: Option<String>,
}

#[derive(Deserialize, Clone)]
struct PackageVersion {
    version: Option<String>,
    time: Option<String>,
    #[serde(default)]
    license: Vec<String>,
    support: Option<HashMap<String, String>>,
    source: Option<PackageSource>,
    homepage: Option<String>,
}

#[derive(Deserialize, Clone)]
struct PackageSource {
    url: Option<String>,
}

pub(super) fn search(client: &Client, query: &str) -> Vec<SearchHit> {
    let url = build_query_url(
        "https://packagist.org/search.json",
        &[("q", query), ("per_page", "8")],
    );
    fetch_json::<SearchResponse>(client, &url)
        .map(|response| {
            response
                .results
                .into_iter()
                .take(8)
                .map(|item| to_hit(client, query, item))
                .collect()
        })
        .unwrap_or_default()
}

fn to_hit(client: &Client, query: &str, item: SearchPackage) -> SearchHit {
    let detail_url = format!("https://packagist.org/packages/{}.json", item.name);
    let mut qualifiers = Vec::new();
    let mut version = "-".to_string();
    let mut latest_release: Option<String> = None;
    let mut score = ((item.downloads.unwrap_or(0) / 1_000_000).min(40)) as i32
        + ((item.favers.unwrap_or(0) / 100).min(40)) as i32;

    if item.downloads.unwrap_or(0) >= 1_000_000 || item.favers.unwrap_or(0) >= 100 {
        push_qualifier(&mut qualifiers, "popular");
    }
    if has_non_empty(item.repository.as_deref()) {
        push_qualifier(&mut qualifiers, "repo");
    }

    if let Some(detail) = fetch_json::<DetailResponse>(client, &detail_url) {
        if detail
            .package
            .maintainers
            .iter()
            .any(|m| has_non_empty(m.name.as_deref()))
        {
            push_qualifier(&mut qualifiers, "owners");
        }
        if let Some(chosen) = detail
            .package
            .versions
            .values()
            .filter(|meta| {
                meta.version
                    .as_deref()
                    .map(|value| !value.contains("dev"))
                    .unwrap_or(false)
            })
            .max_by_key(|meta| meta.time.clone().unwrap_or_default())
            .cloned()
            .or_else(|| {
                detail
                    .package
                    .versions
                    .values()
                    .max_by_key(|meta| meta.time.clone().unwrap_or_default())
                    .cloned()
            })
        {
            if let Some(chosen_version) = chosen.version.as_deref() {
                version = chosen_version.to_string();
            }
            latest_release = chosen.time.clone();
            if licenses_present(&chosen.license) {
                push_qualifier(&mut qualifiers, "licensed");
            }
            if chosen
                .support
                .as_ref()
                .map(|support| support.values().any(|value| !value.trim().is_empty()))
                .unwrap_or(false)
                || has_non_empty(chosen.homepage.as_deref())
            {
                push_qualifier(&mut qualifiers, "docs");
            }
            if chosen
                .source
                .as_ref()
                .and_then(|source| source.url.as_deref())
                .map(|value| !value.trim().is_empty())
                .unwrap_or(false)
            {
                push_qualifier(&mut qualifiers, "repo");
            }
        }
    }

    add_release_age_qualifiers(&mut qualifiers, latest_release.as_deref());
    add_version_qualifiers(&mut qualifiers, &version);
    if qualifiers.iter().any(|value| value == "popular")
        && qualifiers.iter().any(|value| value == "licensed")
        && qualifiers.iter().any(|value| value == "repo")
    {
        push_qualifier(&mut qualifiers, "trusted");
    }
    score += version_base_score(&version);

    SearchHit {
        eco: "Packagist".into(),
        signal: build_signal(query, &item.name, &qualifiers),
        pkg: item.name,
        ver: version,
        desc: item.description.unwrap_or_default(),
        score,
    }
}
