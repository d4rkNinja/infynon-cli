use std::collections::HashMap;

use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::Value;

use super::super::http::{build_query_url, fetch_json};
use super::super::signals::{has_json_value, push_qualifier};
use super::super::signals_extra::{build_signal, version_base_score};
use super::super::types::SearchHit;

#[derive(Deserialize)]
struct SearchResponse {
    #[serde(default)]
    results: Vec<SearchObject>,
}

#[derive(Deserialize)]
struct SearchObject {
    package: SearchPackage,
    score: Option<SearchScore>,
    flags: Option<SearchFlags>,
}

#[derive(Deserialize)]
struct SearchScore {
    #[serde(rename = "final")]
    final_: Option<f64>,
    detail: Option<SearchScoreDetail>,
}

#[derive(Deserialize)]
struct SearchScoreDetail {
    popularity: Option<f64>,
    quality: Option<f64>,
    maintenance: Option<f64>,
}

#[derive(Deserialize)]
struct SearchFlags {
    unstable: Option<bool>,
}

#[derive(Deserialize)]
struct SearchPackage {
    name: String,
    version: String,
    description: Option<String>,
}

#[derive(Deserialize)]
struct PackageMeta {
    version: Option<String>,
    license: Option<Value>,
    maintainers: Option<Vec<Value>>,
    repository: Option<Value>,
    homepage: Option<Value>,
    bugs: Option<Value>,
    scripts: Option<HashMap<String, String>>,
}

pub(super) fn search(client: &Client, query: &str) -> Vec<SearchHit> {
    let url = build_query_url(
        "https://api.npms.io/v2/search",
        &[("q", query), ("size", "8")],
    );
    fetch_json::<SearchResponse>(client, &url)
        .map(|response| {
            response
                .results
                .into_iter()
                .map(|item| to_hit(client, query, item))
                .collect()
        })
        .unwrap_or_default()
}

fn to_hit(client: &Client, query: &str, item: SearchObject) -> SearchHit {
    let detail = item.score.as_ref().and_then(|score| score.detail.as_ref());
    let mut qualifiers = Vec::new();
    if detail.and_then(|value| value.popularity).unwrap_or(0.0) >= 0.6 {
        push_qualifier(&mut qualifiers, "popular");
    }
    if detail.and_then(|value| value.maintenance).unwrap_or(0.0) >= 0.75 {
        push_qualifier(&mut qualifiers, "maintained");
    }
    if detail.and_then(|value| value.quality).unwrap_or(0.0) >= 0.8
        && detail.and_then(|value| value.maintenance).unwrap_or(0.0) >= 0.7
    {
        push_qualifier(&mut qualifiers, "trusted");
    }
    if item.flags.and_then(|flags| flags.unstable).unwrap_or(false) {
        push_qualifier(&mut qualifiers, "unstable");
    }

    let detail_url = format!("https://registry.npmjs.org/{}/latest", item.package.name);
    let mut version = item.package.version.clone();
    if let Some(meta) = fetch_json::<PackageMeta>(client, &detail_url) {
        if let Some(meta_version) = meta.version.as_deref() {
            version = meta_version.to_string();
        }
        if has_json_value(meta.license.as_ref()) {
            push_qualifier(&mut qualifiers, "licensed");
        }
        if meta
            .scripts
            .as_ref()
            .map(|scripts| {
                ["preinstall", "install", "postinstall", "prepare"]
                    .iter()
                    .any(|key| {
                        scripts
                            .get(*key)
                            .map(|value| !value.trim().is_empty())
                            .unwrap_or(false)
                    })
            })
            .unwrap_or(false)
        {
            push_qualifier(&mut qualifiers, "install-script-risk");
        }
        if meta
            .maintainers
            .as_ref()
            .map(|value| !value.is_empty())
            .unwrap_or(false)
        {
            push_qualifier(&mut qualifiers, "owners");
        }
        if has_json_value(meta.repository.as_ref()) {
            push_qualifier(&mut qualifiers, "repo");
        }
        if has_json_value(meta.homepage.as_ref()) || has_json_value(meta.bugs.as_ref()) {
            push_qualifier(&mut qualifiers, "docs");
        }
    }

    SearchHit {
        eco: "npm".into(),
        signal: build_signal(query, &item.package.name, &qualifiers),
        pkg: item.package.name,
        ver: version.clone(),
        desc: item.package.description.unwrap_or_default(),
        score: version_base_score(&version)
            + (item.score.and_then(|score| score.final_).unwrap_or(0.0) * 100.0) as i32,
    }
}
