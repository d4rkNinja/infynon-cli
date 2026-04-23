use super::*;

pub fn cmd_outdated(pkg_file: Option<&str>) {
    use tabled::settings::{object::Rows, Color, Padding, Style};
    use tabled::{Table, Tabled};

    println!();
    Logger::title("INFYNON Outdated Check", "blue");
    Logger::step("Checking for outdated dependencies...");

    let packages = load_packages(pkg_file);
    if packages.is_empty() {
        Logger::error("No packages found in lock files.");
        return;
    }

    // Deduplicate
    let mut seen: HashSet<String> = HashSet::new();
    let unique: Vec<&scanner::LockedPackage> = packages
        .iter()
        .filter(|p| seen.insert(format!("{}:{}", p.ecosystem, p.name)))
        .collect();

    Logger::success(&format!("Found {} unique packages", unique.len()));

    let pb = bar(unique.len() as u64);
    let mut outdated_count = 0usize;
    let mut up_to_date = 0usize;
    let mut unknown = 0usize;

    #[derive(Tabled)]
    struct Row {
        #[tabled(rename = " Ecosystem ")]
        eco: String,
        #[tabled(rename = " Package ")]
        pkg: String,
        #[tabled(rename = " Current ")]
        current: String,
        #[tabled(rename = " Latest ")]
        latest: String,
        #[tabled(rename = " Update ")]
        update: String,
    }
    let mut rows: Vec<Row> = Vec::new();

    for pkg in &unique {
        pb.set_message(format!("{} ({})", pkg.name, pkg.ecosystem));
        pb.inc(1);
        match registry::fetch_latest_version(&pkg.name, &pkg.ecosystem) {
            Some(latest) if latest != pkg.version => {
                outdated_count += 1;
                rows.push(Row {
                    eco: pkg.ecosystem.clone(),
                    pkg: pkg.name.chars().take(35).collect(),
                    current: pkg.version.clone(),
                    latest: latest.clone(),
                    update: classify_update(&pkg.version, &latest),
                });
            }
            Some(_) => {
                up_to_date += 1;
            }
            None => {
                unknown += 1;
            }
        }
    }
    pb.finish_and_clear();

    if rows.is_empty() {
        println!();
        Logger::success("All dependencies are up to date!");
        println!();
        return;
    }

    // Sort: major first
    rows.sort_by_key(|r| match r.update.as_str() {
        "MAJOR" => 0,
        "MINOR" => 1,
        _ => 2,
    });

    let mut table = Table::new(rows);
    table
        .with(Style::modern())
        .with(Padding::new(1, 1, 0, 0))
        .modify(Rows::first(), Color::BOLD | Color::FG_BRIGHT_CYAN);

    println!();
    println!("  {}\n", "Outdated Dependencies:".bold().white());
    println!("{}", table);
    println!(
        "\n  {}  {} outdated  ·  {} up-to-date  ·  {} unknown\n",
        "◆".truecolor(0, 210, 255),
        outdated_count.to_string().bold().bright_yellow(),
        up_to_date.to_string().bold().bright_green(),
        unknown.to_string().truecolor(120, 120, 140),
    );
}

fn classify_update(current: &str, latest: &str) -> String {
    let parse = |v: &str| -> (u32, u32, u32) {
        let parts: Vec<u32> = v
            .trim_start_matches('v')
            .split('.')
            .filter_map(|p| p.split('-').next().and_then(|n| n.parse().ok()))
            .collect();
        (
            *parts.first().unwrap_or(&0),
            *parts.get(1).unwrap_or(&0),
            *parts.get(2).unwrap_or(&0),
        )
    };
    let (cm, cmi, _) = parse(current);
    let (lm, lmi, _) = parse(latest);
    if lm > cm {
        "MAJOR".to_string()
    } else if lmi > cmi {
        "MINOR".to_string()
    } else {
        "PATCH".to_string()
    }
}
