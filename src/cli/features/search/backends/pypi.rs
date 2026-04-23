use std::collections::HashMap;

use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::Value;

use super::super::http::fetch_json;
use super::super::signals::{
    add_release_age_qualifiers, add_version_qualifiers, has_non_empty, license_present,
    push_qualifier,
};
use super::super::signals_extra::{build_signal, version_base_score};
use super::super::types::SearchHit;

#[derive(Deserialize)]
struct Response {
    info: PackageInfo,
    #[serde(default)]
    urls: Vec<PackageFile>,
    ownership: Option<Ownership>,
}

#[derive(Deserialize)]
struct PackageInfo {
    name: String,
    version: String,
    summary: Option<String>,
    license: Option<String>,
    author: Option<String>,
    maintainer: Option<String>,
    #[serde(default)]
    classifiers: Vec<String>,
    #[serde(default)]
    project_urls: HashMap<String, String>,
}

#[derive(Deserialize)]
struct PackageFile {
    upload_time_iso_8601: Option<String>,
    yanked: Option<bool>,
}

#[derive(Deserialize)]
struct Ownership {
    #[serde(default)]
    roles: Vec<Value>,
}

const NOTE: &str = "PyPI search still uses exact package lookup because the public registry search endpoint is not reliably accessible from this CLI.";

pub(super) fn search(client: &Client, query: &str) -> (Vec<SearchHit>, Option<String>) {
    for candidate in [query.to_string(), query.replace('_', "-"), query.replace('-', "_")] {
        let url = format!("https://pypi.org/pypi/{}/json", candidate);
        if let Some(response) = fetch_json::<Response>(client, &url) {
            let hit = to_hit(query, response);
            return (vec![hit], Some(NOTE.to_string()));
        }
    }
    (Vec::new(), Some(NOTE.to_string()))
}

fn to_hit(query: &str, response: Response) -> SearchHit {
    let mut qualifiers = vec!["exact-only".to_string()];
    let latest_upload = response
        .urls
        .iter()
        .filter_map(|file| file.upload_time_iso_8601.as_deref())
        .max()
        .map(str::to_string);

    if license_present(response.info.license.as_deref())
        || response.info.classifiers.iter().any(|value| value.starts_with("License ::"))
    {
        push_qualifier(&mut qualifiers, "licensed");
    }
    if response.info.project_urls.iter().any(|(key, value)| {
        !value.trim().is_empty()
            && ["source", "repository", "github", "code"]
                .iter()
                .any(|needle| key.to_ascii_lowercase().contains(needle))
    }) {
        push_qualifier(&mut qualifiers, "repo");
    }
    if response.info.project_urls.iter().any(|(key, value)| {
        !value.trim().is_empty()
            && ["doc", "home", "issue", "bug"]
                .iter()
                .any(|needle| key.to_ascii_lowercase().contains(needle))
    }) {
        push_qualifier(&mut qualifiers, "docs");
    }
    if has_non_empty(response.info.author.as_deref())
        || has_non_empty(response.info.maintainer.as_deref())
        || response.ownership.as_ref().map(|value| !value.roles.is_empty()).unwrap_or(false)
    {
        push_qualifier(&mut qualifiers, "owners");
    }
    if response.urls.iter().any(|file| file.yanked.unwrap_or(false)) {
        push_qualifier(&mut qualifiers, "yanked");
    }
    add_release_age_qualifiers(&mut qualifiers, latest_upload.as_deref());
    add_version_qualifiers(&mut qualifiers, &response.info.version);
    if qualifiers.iter().any(|value| value == "licensed")
        && qualifiers.iter().any(|value| value == "owners")
        && qualifiers.iter().any(|value| value == "repo")
    {
        push_qualifier(&mut qualifiers, "trusted");
    }

    SearchHit {
        eco: "PyPI".into(),
        signal: build_signal(query, &response.info.name, &qualifiers),
        pkg: response.info.name,
        ver: response.info.version.clone(),
        desc: response.info.summary.unwrap_or_default(),
        score: version_base_score(&response.info.version),
    }
}
