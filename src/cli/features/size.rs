use super::*;
use crate::cli::scan;

pub fn cmd_size(packages: &[String], ecosystem: Option<&str>) {
    use tabled::settings::{object::Rows, Color, Padding, Style};
    use tabled::{Table, Tabled};

    println!();
    Logger::title("INFYNON Package Size", "blue");

    let eco = match ecosystem {
        Some(e) => e,
        None => detect_ecosystem(),
    };
    let client = http_client();

    #[derive(Tabled)]
    struct Row {
        #[tabled(rename = " Package ")]
        pkg: String,
        #[tabled(rename = " Version ")]
        ver: String,
        #[tabled(rename = " Size ")]
        size: String,
        #[tabled(rename = " Dependencies ")]
        deps: String,
    }

    let sp = spinner();
    let mut rows = Vec::new();

    for spec in packages {
        let (name, _) = scan::parse_pkg_spec(spec);
        sp.set_message(format!("Fetching {}...", name));
        let info: Option<(String, String, String)> = match eco {
            "npm" => size_npm(client, &name),
            "crates.io" | "cargo" => size_crates(client, &name),
            "PyPI" | "pip" => size_pypi(client, &name),
            _ => {
                registry::fetch_latest_version(&name, eco).map(|v| (v, "N/A".into(), "N/A".into()))
            }
        };
        match info {
            Some((ver, size, deps)) => rows.push(Row {
                pkg: name,
                ver,
                size,
                deps,
            }),
            None => Logger::error(&format!("Could not fetch info for '{}'", name)),
        }
    }
    sp.finish_and_clear();

    if rows.is_empty() {
        Logger::error("No package info retrieved.");
        println!();
        return;
    }

    let mut table = Table::new(rows);
    table
        .with(Style::modern())
        .with(Padding::new(1, 1, 0, 0))
        .modify(Rows::first(), Color::BOLD | Color::FG_BRIGHT_CYAN);
    println!();
    println!("{}", table);
    println!();
}

fn size_npm(client: &reqwest::blocking::Client, name: &str) -> Option<(String, String, String)> {
    use serde::Deserialize;
    #[derive(Deserialize)]
    struct R {
        version: String,
        dist: Option<D>,
        dependencies: Option<HashMap<String, String>>,
    }
    #[derive(Deserialize)]
    struct D {
        #[serde(rename = "unpackedSize")]
        unpacked_size: Option<u64>,
    }
    let url = format!(
        "https://registry.npmjs.org/{}/latest",
        registry::urlenc(name)
    );
    let r: R = client.get(&url).send().ok()?.json().ok()?;
    Some((
        r.version,
        r.dist
            .and_then(|d| d.unpacked_size)
            .map(format_bytes)
            .unwrap_or_else(|| "unknown".into()),
        r.dependencies
            .map(|d| d.len().to_string())
            .unwrap_or_else(|| "0".into()),
    ))
}

fn size_crates(client: &reqwest::blocking::Client, name: &str) -> Option<(String, String, String)> {
    use serde::Deserialize;
    #[derive(Deserialize)]
    struct R {
        #[serde(rename = "crate")]
        krate: C,
        versions: Vec<V>,
    }
    #[derive(Deserialize)]
    struct C {
        newest_version: String,
    }
    #[derive(Deserialize)]
    struct V {
        num: String,
        crate_size: Option<u64>,
    }
    #[derive(Deserialize)]
    struct DR {
        dependencies: Vec<serde_json::Value>,
    }

    let url = format!("https://crates.io/api/v1/crates/{}", name);
    let r: R = client.get(&url).send().ok()?.json().ok()?;
    let ver = &r.krate.newest_version;
    let size = r
        .versions
        .iter()
        .find(|v| v.num == *ver)
        .and_then(|v| v.crate_size)
        .map(format_bytes)
        .unwrap_or_else(|| "unknown".into());
    let dep_url = format!(
        "https://crates.io/api/v1/crates/{}/{}/dependencies",
        name, ver
    );
    let deps = client
        .get(&dep_url)
        .send()
        .ok()
        .and_then(|r| r.json::<DR>().ok())
        .map(|d| d.dependencies.len().to_string())
        .unwrap_or_else(|| "?".into());
    Some((ver.clone(), size, deps))
}

fn size_pypi(client: &reqwest::blocking::Client, name: &str) -> Option<(String, String, String)> {
    use serde::Deserialize;
    #[derive(Deserialize)]
    struct R {
        info: I,
        urls: Vec<U>,
    }
    #[derive(Deserialize)]
    struct I {
        version: String,
        requires_dist: Option<Vec<String>>,
    }
    #[derive(Deserialize)]
    struct U {
        size: Option<u64>,
        packagetype: Option<String>,
    }

    let url = format!("https://pypi.org/pypi/{}/json", name);
    let r: R = client.get(&url).send().ok()?.json().ok()?;
    let size = r
        .urls
        .iter()
        .find(|u| u.packagetype.as_deref() == Some("bdist_wheel"))
        .or(r.urls.first())
        .and_then(|u| u.size)
        .map(format_bytes)
        .unwrap_or_else(|| "unknown".into());
    let deps = r
        .info
        .requires_dist
        .map(|d| d.len().to_string())
        .unwrap_or_else(|| "0".into());
    Some((r.info.version, size, deps))
}
