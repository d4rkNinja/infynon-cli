use super::matching::{levenshtein, normalize_pkg_name};
use super::signals::signal_has;

pub(super) fn version_base_score(version: &str) -> i32 {
    if version.trim().is_empty() || version == "-" {
        -10
    } else if super::signals::is_prerelease(version) {
        -25
    } else {
        10
    }
}

pub(super) fn compute_match_score(query: &str, query_norm: &str, candidate: &str, signal: &str) -> i32 {
    let candidate_norm = normalize_pkg_name(candidate);
    let distance = levenshtein(query_norm, &candidate_norm) as i32;
    let mut score = if candidate.eq_ignore_ascii_case(query) {
        1_000
    } else if candidate_norm == query_norm {
        950
    } else if candidate_norm.starts_with(query_norm) {
        700
    } else if candidate_norm.contains(query_norm) {
        550
    } else {
        300 - distance * 25
    };

    for (token, boost) in [
        ("verified", 50),
        ("trusted", 40),
        ("popular", 30),
        ("licensed", 18),
        ("maintained", 16),
        ("owners", 14),
        ("mfa", 12),
        ("repo", 10),
        ("docs", 8),
        ("fresh", 12),
        ("stable", 10),
        ("exact-only", 6),
    ] {
        if signal_has(signal, token) {
            score += boost;
        }
    }

    for (token, penalty) in [
        ("install-script-risk", 90),
        ("yanked", 80),
        ("unlisted", 60),
        ("stale", 25),
        ("new", 20),
        ("unstable", 30),
        ("no-version", 10),
    ] {
        if signal_has(signal, token) {
            score -= penalty;
        }
    }

    score
}

pub(super) fn build_signal(query: &str, package: &str, qualifiers: &[String]) -> String {
    let mut parts: Vec<String> = Vec::new();
    let query_norm = normalize_pkg_name(query);
    let package_norm = normalize_pkg_name(package);
    let distance = levenshtein(&query_norm, &package_norm);

    if package.eq_ignore_ascii_case(query) || package_norm == query_norm {
        super::signals::push_qualifier(&mut parts, "exact");
    } else if distance <= 2 {
        super::signals::push_qualifier(&mut parts, "close");
    } else if package_norm.starts_with(&query_norm) {
        super::signals::push_qualifier(&mut parts, "prefix");
    }

    for value in qualifiers {
        super::signals::push_qualifier(&mut parts, value);
    }

    if parts.is_empty() {
        "match".to_string()
    } else {
        parts.join(", ")
    }
}
