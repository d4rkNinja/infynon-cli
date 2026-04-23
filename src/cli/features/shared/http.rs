use reqwest::blocking::Client;
use std::sync::OnceLock;
use std::time::Duration;

static HTTP_CLIENT: OnceLock<Client> = OnceLock::new();

pub(crate) fn http_client() -> &'static Client {
    HTTP_CLIENT.get_or_init(|| {
        let ua = format!(
            "infynon/{} (https://github.com/d4rkNinja/infynon-cli)",
            env!("CARGO_PKG_VERSION")
        );
        Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent(ua)
            .build()
            .unwrap_or_default()
    })
}
