use reqwest::blocking::Client;

pub(crate) fn http_client() -> &'static Client {
    crate::utils::http_client()
}
