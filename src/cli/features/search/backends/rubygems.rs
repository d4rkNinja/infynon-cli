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
struct SearchGem {
    name: String,
    version: String,
    info: Option<String>,
}

#[derive(Deserialize)]
struct GemDetail {
    version: Option<String>,
    downloads: Option<u64>,
    version_downloads: Option<u64>,
    version_created_at: Option<String>,
    #[serde(default)]
    licenses: Vec<String>,
    yanked: Option<bool>,
    metadata: Option<GemMetadata>,
    homepage_uri: Option<String>,
    documentation_uri: Option<String>,
    source_code_uri: Option<String>,
    bug_tracker_uri: Option<String>,
}

#[derive(Deserialize)]
struct GemMetadata {
    rubygems_mfa_required: Option<String>,
}

pub(super) fn search(client: &Client, query: &str) -> Vec<SearchHit> {
    let url = build_query_url("https://rubygems.org/api/v1/search.json", &[("query", query)]);
    fetch_json::<Vec<SearchGem>>(client, &url)
        .map(|response| response.into_iter().take(8).map(|item| to_hit(client, query, item)).collect())
        .unwrap_or_default()
}

fn to_hit(client: &Client, query: &str, item: SearchGem) -> SearchHit {
    let detail_url = format!("https://rubygems.org/api/v1/gems/{}.json", item.name);
    let mut qualifiers = Vec::new();
    let mut version = item.version.clone();
    if let Some(detail) = fetch_json::<GemDetail>(client, &detail_url) {
        if let Some(detail_version) = detail.version.as_deref() {
            version = detail_version.to_string();
        }
        if detail.downloads.unwrap_or(0) >= 1_000_000 || detail.version_downloads.unwrap_or(0) >= 100_000 {
            push_qualifier(&mut qualifiers, "popular");
        }
        if licenses_present(&detail.licenses) {
            push_qualifier(&mut qualifiers, "licensed");
        }
        if detail
            .metadata
            .as_ref()
            .and_then(|meta| meta.rubygems_mfa_required.as_deref())
            .map(|value| value.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
        {
            push_qualifier(&mut qualifiers, "mfa");
        }
        if has_non_empty(detail.source_code_uri.as_deref()) || has_non_empty(detail.homepage_uri.as_deref()) {
            push_qualifier(&mut qualifiers, "repo");
        }
        if has_non_empty(detail.documentation_uri.as_deref())
            || has_non_empty(detail.bug_tracker_uri.as_deref())
            || has_non_empty(detail.homepage_uri.as_deref())
        {
            push_qualifier(&mut qualifiers, "docs");
        }
        if detail.yanked.unwrap_or(false) {
            push_qualifier(&mut qualifiers, "yanked");
        }
        add_release_age_qualifiers(&mut qualifiers, detail.version_created_at.as_deref());
    }

    add_version_qualifiers(&mut qualifiers, &version);
    SearchHit {
        eco: "RubyGems".into(),
        signal: build_signal(query, &item.name, &qualifiers),
        pkg: item.name,
        ver: version.clone(),
        desc: item.info.unwrap_or_default(),
        score: version_base_score(&version),
    }
}
