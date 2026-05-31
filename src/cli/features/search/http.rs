use reqwest::blocking::Client;
use serde::de::DeserializeOwned;

pub(super) fn build_query_url(base: &str, params: &[(&str, &str)]) -> String {
    reqwest::Url::parse(base)
        .map(|mut url| {
            url.query_pairs_mut().extend_pairs(params.iter().copied());
            url.to_string()
        })
        .unwrap_or_else(|_| base.to_string())
}

pub(super) fn fetch_json<T: DeserializeOwned>(client: &Client, url: &str) -> Option<T> {
    client.get(url).send().ok()?.json::<T>().ok()
}

pub(super) fn fetch_text(client: &Client, url: &str) -> Option<String> {
    client.get(url).send().ok()?.text().ok()
}
