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
    crates: Vec<SearchCrate>,
}

#[derive(Deserialize)]
struct SearchCrate {
    name: String,
    newest_version: String,
    description: Option<String>,
}

#[derive(Deserialize)]
struct DetailResponse {
    #[serde(rename = "crate")]
    krate: CrateMeta,
    #[serde(default)]
    versions: Vec<CrateVersion>,
}

#[derive(Deserialize)]
struct CrateMeta {
    downloads: Option<u64>,
    recent_downloads: Option<u64>,
    max_stable_version: Option<String>,
    repository: Option<String>,
    documentation: Option<String>,
    homepage: Option<String>,
    updated_at: Option<String>,
}

#[derive(Deserialize)]
struct CrateVersion {
    num: String,
    license: Option<String>,
    created_at: Option<String>,
    yanked: Option<bool>,
}

pub(super) fn search(client: &Client, query: &str) -> Vec<SearchHit> {
    let url = build_query_url("https://crates.io/api/v1/crates", &[("q", query), ("per_page", "8")]);
    fetch_json::<SearchResponse>(client, &url)
        .map(|response| response.crates.into_iter().map(|item| to_hit(client, query, item)).collect())
        .unwrap_or_default()
}

fn to_hit(client: &Client, query: &str, item: SearchCrate) -> SearchHit {
    let detail_url = format!("https://crates.io/api/v1/crates/{}", item.name);
    let mut qualifiers = Vec::new();
    let mut version = item.newest_version.clone();
    if let Some(detail) = fetch_json::<DetailResponse>(client, &detail_url) {
        if let Some(stable) = detail.krate.max_stable_version.as_deref() {
            version = stable.to_string();
        }
        if detail.krate.downloads.unwrap_or(0) >= 1_000_000
            || detail.krate.recent_downloads.unwrap_or(0) >= 50_000
        {
            push_qualifier(&mut qualifiers, "popular");
        }
        if detail.krate.recent_downloads.unwrap_or(0) >= 10_000 {
            push_qualifier(&mut qualifiers, "maintained");
        }
        if has_non_empty(detail.krate.repository.as_deref()) || has_non_empty(detail.krate.homepage.as_deref()) {
            push_qualifier(&mut qualifiers, "repo");
        }
        if has_non_empty(detail.krate.documentation.as_deref()) || has_non_empty(detail.krate.homepage.as_deref()) {
            push_qualifier(&mut qualifiers, "docs");
        }
        add_release_age_qualifiers(&mut qualifiers, detail.krate.updated_at.as_deref());

        if let Some(best_version) = detail
            .versions
            .iter()
            .find(|meta| meta.num == version || (!meta.yanked.unwrap_or(false) && !meta.num.contains("alpha")))
            .or_else(|| detail.versions.first())
        {
            if license_present(best_version.license.as_deref()) {
                push_qualifier(&mut qualifiers, "licensed");
            }
            if best_version.yanked.unwrap_or(false) {
                push_qualifier(&mut qualifiers, "yanked");
            }
            add_release_age_qualifiers(&mut qualifiers, best_version.created_at.as_deref());
        }
    }

    add_version_qualifiers(&mut qualifiers, &version);
    SearchHit {
        eco: "crates.io".into(),
        signal: build_signal(query, &item.name, &qualifiers),
        pkg: item.name,
        ver: version.clone(),
        desc: item.description.unwrap_or_default(),
        score: version_base_score(&version),
    }
}
