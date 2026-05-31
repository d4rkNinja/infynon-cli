use super::*;
use crate::engine::osv;

pub fn cmd_diff(package: &str, v1: &str, v2: &str, ecosystem: Option<&str>) {
    println!();
    Logger::title("INFYNON Package Diff", "blue");

    let eco = match ecosystem {
        Some(e) => e,
        None => detect_ecosystem(),
    };

    println!();
    println!(
        "  {}  {} {} → {}  ({})",
        "◆".truecolor(0, 210, 255),
        package.bold().bright_white(),
        v1.truecolor(255, 100, 100),
        v2.truecolor(100, 255, 100),
        eco.truecolor(120, 120, 140)
    );
    println!("  {}", "─".repeat(66).truecolor(40, 40, 60));

    let client = http_client();

    match eco {
        "npm" => diff_npm(client, package, v1, v2),
        "PyPI" | "pip" => diff_pypi(client, package, v1, v2),
        "crates.io" | "cargo" => diff_crates(client, package, v1, v2),
        _ => {
            Logger::info("Detailed metadata diff not available for this ecosystem.");
        }
    }

    // Vulnerability comparison for both versions
    println!();
    Logger::step("Comparing vulnerabilities...");

    let osv_eco = match eco {
        "pip" => "PyPI",
        "cargo" => "crates.io",
        o => o,
    };
    let tuples = vec![
        (package.to_string(), osv_eco.to_string(), v1.to_string()),
        (package.to_string(), osv_eco.to_string(), v2.to_string()),
    ];
    if let Ok(results) = osv::batch_query(&tuples) {
        let v1_n = results.first().map(|r| r.len()).unwrap_or(0);
        let v2_n = results.get(1).map(|r| r.len()).unwrap_or(0);
        println!();
        let icon = |n: usize| {
            if n > 0 {
                "⚠".bright_yellow().to_string()
            } else {
                "✔".bright_green().to_string()
            }
        };
        let count = |n: usize| {
            if n > 0 {
                n.to_string().bright_red().bold().to_string()
            } else {
                "0".bright_green().to_string()
            }
        };
        println!("  {}  v{}: {} CVE(s)", icon(v1_n), v1, count(v1_n));
        println!("  {}  v{}: {} CVE(s)", icon(v2_n), v2, count(v2_n));
        if v2_n < v1_n {
            println!(
                "\n  {}  {} v{} has fewer vulnerabilities",
                "→".bright_green(),
                "Upgrade to".truecolor(100, 200, 100),
                v2
            );
        }
    }
    println!();
}

fn diff_npm(client: &reqwest::blocking::Client, package: &str, v1: &str, v2: &str) {
    use serde::Deserialize;
    #[derive(Deserialize)]
    struct V {
        dependencies: Option<HashMap<String, String>>,
        scripts: Option<HashMap<String, String>>,
        dist: Option<D>,
    }
    #[derive(Deserialize)]
    struct D {
        #[serde(rename = "unpackedSize")]
        unpacked_size: Option<u64>,
    }

    let u1 = format!(
        "https://registry.npmjs.org/{}/{}",
        registry::urlenc(package),
        v1
    );
    let u2 = format!(
        "https://registry.npmjs.org/{}/{}",
        registry::urlenc(package),
        v2
    );
    let d1: Option<V> = client.get(&u1).send().ok().and_then(|r| r.json().ok());
    let d2: Option<V> = client.get(&u2).send().ok().and_then(|r| r.json().ok());
    let (d1, d2) = match (d1, d2) {
        (Some(a), Some(b)) => (a, b),
        _ => {
            Logger::error("Could not fetch version metadata from npm.");
            return;
        }
    };

    // Size
    if let (Some(s1), Some(s2)) = (
        d1.dist.as_ref().and_then(|d| d.unpacked_size),
        d2.dist.as_ref().and_then(|d| d.unpacked_size),
    ) {
        let diff = s2 as i64 - s1 as i64;
        println!();
        println!(
            "  {}  {}",
            "Size".bold().truecolor(255, 170, 50),
            "─".repeat(40).truecolor(40, 40, 60)
        );
        println!(
            "     v{}: {}  ·  v{}: {}  ({}{})",
            v1,
            format_bytes(s1),
            v2,
            format_bytes(s2),
            if diff >= 0 { "+" } else { "" },
            format_bytes(diff.unsigned_abs())
        );
    }

    print_deps_diff(
        "Dependencies",
        &d1.dependencies.unwrap_or_default(),
        &d2.dependencies.unwrap_or_default(),
    );

    let s1 = d1.scripts.unwrap_or_default();
    let s2 = d2.scripts.unwrap_or_default();
    if s1 != s2 {
        println!();
        println!(
            "  {}  {}",
            "Scripts".bold().truecolor(255, 170, 50),
            "─".repeat(38).truecolor(40, 40, 60)
        );
        for k in s2.keys() {
            if !s1.contains_key(k) {
                println!("     {} {} (new)", "+".bright_green(), k.bold());
            }
        }
        for k in s1.keys() {
            if !s2.contains_key(k) {
                println!("     {} {} (removed)", "-".bright_red(), k.bold());
            }
        }
        for (k, v2) in &s2 {
            if s1.get(k).is_some_and(|v1| v1 != v2) {
                println!("     {} {} (changed)", "~".bright_yellow(), k.bold());
            }
        }
    }
}

fn diff_pypi(client: &reqwest::blocking::Client, package: &str, v1: &str, v2: &str) {
    use serde::Deserialize;
    #[derive(Deserialize)]
    struct R {
        info: I,
    }
    #[derive(Deserialize)]
    struct I {
        requires_dist: Option<Vec<String>>,
    }

    let u1 = format!("https://pypi.org/pypi/{}/{}/json", package, v1);
    let u2 = format!("https://pypi.org/pypi/{}/{}/json", package, v2);
    let d1: Option<R> = client.get(&u1).send().ok().and_then(|r| r.json().ok());
    let d2: Option<R> = client.get(&u2).send().ok().and_then(|r| r.json().ok());
    let (d1, d2) = match (d1, d2) {
        (Some(a), Some(b)) => (a, b),
        _ => {
            Logger::error("Could not fetch metadata from PyPI.");
            return;
        }
    };

    let to_map = |v: Vec<String>| -> HashMap<String, String> {
        v.into_iter()
            .map(|d| {
                let name = d.split_whitespace().next().unwrap_or(&d).to_string();
                (name, d)
            })
            .collect()
    };
    print_deps_diff(
        "Dependencies",
        &to_map(d1.info.requires_dist.unwrap_or_default()),
        &to_map(d2.info.requires_dist.unwrap_or_default()),
    );
}

fn diff_crates(client: &reqwest::blocking::Client, package: &str, v1: &str, v2: &str) {
    use serde::Deserialize;
    #[derive(Deserialize)]
    struct R {
        version: VI,
    }
    #[derive(Deserialize)]
    struct VI {
        crate_size: Option<u64>,
    }
    #[derive(Deserialize)]
    struct DR {
        dependencies: Vec<Dep>,
    }
    #[derive(Deserialize)]
    struct Dep {
        crate_id: String,
        req: String,
        kind: String,
    }

    let u1 = format!("https://crates.io/api/v1/crates/{}/{}", package, v1);
    let u2 = format!("https://crates.io/api/v1/crates/{}/{}", package, v2);
    let d1: Option<R> = client.get(&u1).send().ok().and_then(|r| r.json().ok());
    let d2: Option<R> = client.get(&u2).send().ok().and_then(|r| r.json().ok());

    if let (Some(r1), Some(r2)) = (d1, d2) {
        if let (Some(s1), Some(s2)) = (r1.version.crate_size, r2.version.crate_size) {
            let diff = s2 as i64 - s1 as i64;
            println!();
            println!(
                "  {}  {}",
                "Crate Size".bold().truecolor(255, 170, 50),
                "─".repeat(35).truecolor(40, 40, 60)
            );
            println!(
                "     v{}: {}  ·  v{}: {}  ({}{})",
                v1,
                format_bytes(s1),
                v2,
                format_bytes(s2),
                if diff >= 0 { "+" } else { "" },
                format_bytes(diff.unsigned_abs())
            );
        }
    }

    let du1 = format!(
        "https://crates.io/api/v1/crates/{}/{}/dependencies",
        package, v1
    );
    let du2 = format!(
        "https://crates.io/api/v1/crates/{}/{}/dependencies",
        package, v2
    );
    let dd1: Option<DR> = client.get(&du1).send().ok().and_then(|r| r.json().ok());
    let dd2: Option<DR> = client.get(&du2).send().ok().and_then(|r| r.json().ok());
    if let (Some(d1), Some(d2)) = (dd1, dd2) {
        let m = |deps: Vec<Dep>| -> HashMap<String, String> {
            deps.into_iter()
                .filter(|d| d.kind == "normal")
                .map(|d| (d.crate_id, d.req))
                .collect()
        };
        print_deps_diff("Dependencies", &m(d1.dependencies), &m(d2.dependencies));
    }
}

fn print_deps_diff(label: &str, old: &HashMap<String, String>, new: &HashMap<String, String>) {
    let added: Vec<&String> = new.keys().filter(|k| !old.contains_key(*k)).collect();
    let removed: Vec<&String> = old.keys().filter(|k| !new.contains_key(*k)).collect();
    let changed: Vec<&String> = new
        .keys()
        .filter(|k| old.get(*k).is_some_and(|v| v != &new[*k]))
        .collect();
    if added.is_empty() && removed.is_empty() && changed.is_empty() {
        return;
    }

    println!();
    println!(
        "  {}  {}  (+{} -{} ~{})",
        label.bold().truecolor(255, 170, 50),
        "─".repeat(25).truecolor(40, 40, 60),
        added.len().to_string().bright_green(),
        removed.len().to_string().bright_red(),
        changed.len().to_string().bright_yellow(),
    );
    for n in &added {
        println!(
            "     {} {} {}",
            "+".bright_green(),
            n.bold(),
            new[*n].truecolor(100, 255, 100)
        );
    }
    for n in &removed {
        println!(
            "     {} {} {}",
            "-".bright_red(),
            n.bold(),
            old[*n].truecolor(255, 100, 100)
        );
    }
    for n in &changed {
        println!(
            "     {} {} {} → {}",
            "~".bright_yellow(),
            n.bold(),
            old[*n].truecolor(180, 100, 100),
            new[*n].truecolor(100, 255, 100)
        );
    }
}
