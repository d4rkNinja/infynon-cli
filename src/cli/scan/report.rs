use crate::engine::reporter;
use owo_colors::OwoColorize;

pub fn severity_colored(sev: &str) -> String {
    match sev {
        "CRITICAL" => sev.bright_red().bold().to_string(),
        "HIGH" => sev.red().bold().to_string(),
        "MEDIUM" => sev.yellow().bold().to_string(),
        "LOW" => sev.bright_green().to_string(),
        _ => sev.truecolor(140, 140, 160).to_string(),
    }
}

pub(super) fn severity_badge(sev: &str) -> String {
    match sev {
        "CRITICAL" => format!(" {} ", sev)
            .bold()
            .on_bright_red()
            .white()
            .to_string(),
        "HIGH" => format!(" {} ", sev).bold().on_red().white().to_string(),
        "MEDIUM" => format!(" {} ", sev).bold().on_yellow().black().to_string(),
        "LOW" => format!(" {} ", sev)
            .bold()
            .on_bright_green()
            .black()
            .to_string(),
        _ => format!(" {} ", sev).truecolor(120, 120, 140).to_string(),
    }
}

pub(super) fn print_report_table(findings: &[reporter::ScanFinding]) {
    use tabled::settings::{object::Rows, Color, Padding, Style};
    use tabled::{Table, Tabled};
    #[derive(Tabled)]
    struct Row {
        #[tabled(rename = " Risk ")]
        sev: String,
        #[tabled(rename = " Package ")]
        pkg: String,
        #[tabled(rename = " Version ")]
        ver: String,
        #[tabled(rename = " CVE / ID ")]
        cve: String,
        #[tabled(rename = " Remediation ")]
        fix: String,
    }
    let rows: Vec<Row> = findings
        .iter()
        .map(|f| Row {
            sev: f.severity.to_string(),
            pkg: f.package.name.chars().take(25).collect(),
            ver: f.package.version.chars().take(12).collect(),
            cve: f.vuln.id.clone(),
            fix: f
                .fixed_version
                .clone()
                .unwrap_or_else(|| {
                    f.suggested_version
                        .clone()
                        .map(|v| format!("~{} (latest)", v))
                        .unwrap_or_else(|| "No fix".into())
                })
                .chars()
                .take(18)
                .collect(),
        })
        .collect();
    let mut table = Table::new(rows);
    table
        .with(Style::modern())
        .with(Padding::new(1, 1, 0, 0))
        .modify(Rows::first(), Color::BOLD | Color::FG_BRIGHT_CYAN);
    println!("  {}\n{}", "Vulnerability Report:".bold().white(), table);
    for finding in findings {
        print_remediation_detail(finding);
    }
    let (crit, high, med, low, info) = reporter::severity_counts(findings);
    println!(
        "\n  {}  {}  {}  {}  {}\n",
        format!("CRITICAL: {}", crit).bold().bright_red(),
        format!("HIGH: {}", high).bold().red(),
        format!("MEDIUM: {}", med).bold().yellow(),
        format!("LOW: {}", low).bold().bright_green(),
        format!("INFO: {}", info).truecolor(140, 140, 160)
    );
    println!("  {}  Upgrade to the 'Remediation' column value to fix each finding.\n     {}  ~ prefix means: no known fix in vulnerability DB — latest stable version suggested.\n", "ℹ".bright_cyan(), "ℹ".bright_cyan());
}

fn print_remediation_detail(finding: &reporter::ScanFinding) {
    let summary = finding
        .vuln
        .summary
        .as_deref()
        .unwrap_or("CVE in this version range");
    let short: String = summary.chars().take(80).collect();
    if let Some(ref fixed) = finding.fixed_version {
        let cmd = crate::cli::scan::upgrade_cmd(&finding.package, fixed);
        println!(
            "       {} {} {}  {} → {}  {}",
            "→".truecolor(80, 80, 100),
            finding.vuln.id.truecolor(100, 160, 255),
            finding.package.name.bold(),
            "fix:".bright_green(),
            cmd.bright_green().bold(),
            short.truecolor(140, 140, 160)
        );
    } else if let Some(ref suggested) = finding.suggested_version {
        let cmd = crate::cli::scan::upgrade_cmd(&finding.package, suggested);
        println!(
            "       {} {} {}  {} {} → {}  {}",
            "→".truecolor(80, 80, 100),
            finding.vuln.id.truecolor(100, 160, 255),
            finding.package.name.bold(),
            "no DB fix".truecolor(180, 120, 50),
            "try latest:".truecolor(200, 180, 80),
            cmd.truecolor(200, 200, 100).bold(),
            short.truecolor(140, 140, 160)
        );
    } else {
        println!(
            "       {} {} {}  {}  {}",
            "→".truecolor(80, 80, 100),
            finding.vuln.id.truecolor(100, 160, 255),
            finding.package.name.bold(),
            "no fix available".truecolor(180, 80, 50),
            short.truecolor(140, 140, 160)
        );
    }
}
