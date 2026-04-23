use super::html::{build_eagle_eye_html, severity_color};
use super::prompt::parse_csv_list;
use super::secret::{password_status, resolve_smtp_password};
use super::types::{risk_levels_for_choice, EagleEyeConfig, ScanFinding};

#[test]
fn risk_levels_choice_maps_expected_values() {
    assert_eq!(risk_levels_for_choice("1"), vec!["CRITICAL"]);
    assert_eq!(risk_levels_for_choice("2"), vec!["CRITICAL", "HIGH"]);
    assert_eq!(
        risk_levels_for_choice("5"),
        vec!["CRITICAL", "HIGH", "MEDIUM", "LOW", "INFORMATIONAL"]
    );
}

#[test]
fn parse_csv_list_trims_and_drops_empty_values() {
    assert_eq!(
        parse_csv_list(" one@example.com, , two@example.com "),
        vec!["one@example.com", "two@example.com"]
    );
}

#[test]
fn severity_color_covers_known_levels() {
    assert_eq!(severity_color("CRITICAL"), "#ff4444");
    assert_eq!(severity_color("HIGH"), "#ff6644");
    assert_eq!(severity_color("MEDIUM"), "#ffc832");
    assert_eq!(severity_color("LOW"), "#44cc44");
}

#[test]
fn html_includes_project_and_ecosystem_details() {
    let config = EagleEyeConfig {
        scan_paths: vec!["D:/demo".into()],
        risk_levels: vec!["CRITICAL".into(), "HIGH".into()],
        ..EagleEyeConfig::default()
    };
    let findings = vec![ScanFinding {
        project_path: "D:/demo".into(),
        package: "serde".into(),
        version: "1.0.0".into(),
        ecosystem: "crates.io".into(),
        cve_id: "CVE-2026-0001".into(),
        severity: "HIGH".into(),
        summary: "Example issue".into(),
        fixed_version: "1.0.1".into(),
    }];

    let html = build_eagle_eye_html(&findings, &config);
    assert!(html.contains("D:/demo"));
    assert!(html.contains("crates.io"));
    assert!(html.contains("CVE-2026-0001"));
    assert!(html.contains("Fix: 1.0.1"));
}

#[test]
fn env_password_is_preferred_over_legacy_password() {
    let key = "INFYNON_TEST_SMTP_PASSWORD";
    std::env::set_var(key, "env-secret");
    let config = super::types::SmtpConfig {
        password_env: key.into(),
        legacy_password: "legacy-secret".into(),
        ..Default::default()
    };
    assert_eq!(
        resolve_smtp_password(&config).as_deref(),
        Some("env-secret")
    );
    assert_eq!(password_status(&config), format!("env:{}", key));
    std::env::remove_var(key);
}

#[test]
fn legacy_password_still_resolves_when_no_env_is_set() {
    let config = super::types::SmtpConfig {
        legacy_password: "legacy-secret".into(),
        ..Default::default()
    };
    assert_eq!(
        resolve_smtp_password(&config).as_deref(),
        Some("legacy-secret")
    );
    assert_eq!(password_status(&config), "legacy config");
}
