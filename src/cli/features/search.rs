use super::*;

pub fn cmd_search(query: &str, ecosystem: Option<&str>) {
    use tabled::{Table, Tabled};
    use tabled::settings::{Style, Padding, object::Rows, Color};

    println!();
    Logger::title("INFYNON Package Search", "blue");
    Logger::step(&format!("Searching '{}'...", query));

    let client = http_client();
    // (eco, name, version, description)
    let mut results: Vec<(String, String, String, String)> = Vec::new();

    let ecos: Vec<&str> = match ecosystem {
        Some(e) => vec![e],
        None => vec!["npm", "crates.io", "PyPI", "RubyGems", "Packagist", "pub.dev"],
    };

    let sp = spinner();
    for eco in ecos {
        sp.set_message(format!("Searching {}...", eco));
        match eco {
            "npm" => results.extend(search_npm(client, query)),
            "crates.io" | "cargo" => results.extend(search_crates(client, query)),
            "PyPI" | "pip" => results.extend(search_pypi(client, query)),
            "RubyGems" | "gem" => results.extend(search_rubygems(client, query)),
            "Packagist" | "composer" => results.extend(search_packagist(client, query)),
            "pub.dev" | "pub" => results.extend(search_pubdev(client, query)),
            _ => {}
        }
    }
    sp.finish_and_clear();

    if results.is_empty() {
        println!();
        Logger::info(&format!("No packages found for '{}'", query));
        println!();
        return;
    }

    #[derive(Tabled)]
    struct Row {
        #[tabled(rename = " Ecosystem ")]   eco: String,
        #[tabled(rename = " Package ")]     pkg: String,
        #[tabled(rename = " Version ")]     ver: String,
        #[tabled(rename = " Description ")] desc: String,
    }

    let rows: Vec<Row> = results.iter().map(|(e, p, v, d)| Row {
        eco: e.clone(), pkg: p.clone(), ver: v.clone(), desc: d.chars().take(50).collect(),
    }).collect();
    let count = rows.len();

    let mut table = Table::new(rows);
    table.with(Style::modern()).with(Padding::new(1, 1, 0, 0))
        .modify(Rows::first(), Color::BOLD | Color::FG_BRIGHT_CYAN);

    println!();
    println!("{}", table);
    println!(
        "\n  {}  {} packages matching '{}'\n",
        "◆".truecolor(0, 210, 255),
        count.to_string().bold(), query.bold()
    );
}

fn search_npm(client: &reqwest::blocking::Client, query: &str) -> Vec<(String, String, String, String)> {
    use serde::Deserialize;
    #[derive(Deserialize)] struct R { objects: Vec<O> }
    #[derive(Deserialize)] struct O { package: P }
    #[derive(Deserialize)] struct P { name: String, version: String, description: Option<String> }
    let url = format!("https://registry.npmjs.org/-/v1/search?text={}&size=5", query);
    client.get(&url).send().ok().and_then(|r| r.json::<R>().ok())
        .map(|r| r.objects.into_iter().map(|o| ("npm".into(), o.package.name, o.package.version, o.package.description.unwrap_or_default())).collect())
        .unwrap_or_default()
}

fn search_crates(client: &reqwest::blocking::Client, query: &str) -> Vec<(String, String, String, String)> {
    use serde::Deserialize;
    #[derive(Deserialize)] struct R { crates: Vec<C> }
    #[derive(Deserialize)] struct C { name: String, newest_version: String, description: Option<String> }
    let url = format!("https://crates.io/api/v1/crates?q={}&per_page=5", query);
    client.get(&url).send().ok().and_then(|r| r.json::<R>().ok())
        .map(|r| r.crates.into_iter().map(|c| ("crates.io".into(), c.name, c.newest_version, c.description.unwrap_or_default())).collect())
        .unwrap_or_default()
}

fn search_pypi(client: &reqwest::blocking::Client, query: &str) -> Vec<(String, String, String, String)> {
    use serde::Deserialize;
    #[derive(Deserialize)] struct R { info: I }
    #[derive(Deserialize)] struct I { name: String, version: String, summary: Option<String> }
    let url = format!("https://pypi.org/pypi/{}/json", query);
    client.get(&url).send().ok().and_then(|r| r.json::<R>().ok())
        .map(|r| vec![("PyPI".into(), r.info.name, r.info.version, r.info.summary.unwrap_or_default())])
        .unwrap_or_default()
}

fn search_rubygems(client: &reqwest::blocking::Client, query: &str) -> Vec<(String, String, String, String)> {
    use serde::Deserialize;
    #[derive(Deserialize)] struct G { name: String, version: String, info: Option<String> }
    let url = format!("https://rubygems.org/api/v1/search.json?query={}", query);
    client.get(&url).send().ok().and_then(|r| r.json::<Vec<G>>().ok())
        .map(|r| r.into_iter().take(5).map(|g| ("RubyGems".into(), g.name, g.version, g.info.unwrap_or_default())).collect())
        .unwrap_or_default()
}

fn search_packagist(client: &reqwest::blocking::Client, query: &str) -> Vec<(String, String, String, String)> {
    use serde::Deserialize;
    #[derive(Deserialize)] struct R { results: Vec<P> }
    #[derive(Deserialize)] struct P { name: String, description: String }
    let url = format!("https://packagist.org/search.json?q={}&per_page=5", query);
    client.get(&url).send().ok().and_then(|r| r.json::<R>().ok())
        .map(|r| r.results.into_iter().take(5).map(|p| ("Packagist".into(), p.name, "-".into(), p.description)).collect())
        .unwrap_or_default()
}

fn search_pubdev(client: &reqwest::blocking::Client, query: &str) -> Vec<(String, String, String, String)> {
    use serde::Deserialize;
    #[derive(Deserialize)] struct R { packages: Vec<P> }
    #[derive(Deserialize)] struct P { package: String }
    let url = format!("https://pub.dev/api/search?q={}", query);
    client.get(&url).send().ok().and_then(|r| r.json::<R>().ok())
        .map(|r| r.packages.into_iter().take(5).map(|p| ("pub.dev".into(), p.package, "-".into(), String::new())).collect())
        .unwrap_or_default()
}
