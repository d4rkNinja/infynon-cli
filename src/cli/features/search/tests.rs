use super::backends::escape_go_module_path;
use super::matching::best_follow_up_hint;
use super::matching::normalize_pkg_name;
use super::resolve::resolve_search_ecosystems;
use super::signals::signal_has;
use super::signals_extra::{build_signal, compute_match_score};
use super::types::SearchHit;

#[test]
fn aliases_resolve_to_supported_search_ecosystems() {
    assert_eq!(
        resolve_search_ecosystems(Some("cargo")).unwrap(),
        vec!["crates.io"]
    );
    assert_eq!(
        resolve_search_ecosystems(Some("pip")).unwrap(),
        vec!["PyPI"]
    );
    assert_eq!(
        resolve_search_ecosystems(Some("nuget")).unwrap(),
        vec!["NuGet"]
    );
    assert_eq!(resolve_search_ecosystems(Some("mix")).unwrap(), vec!["Hex"]);
    assert_eq!(resolve_search_ecosystems(Some("go")).unwrap(), vec!["Go"]);
}

#[test]
fn all_package_manager_aliases_map_to_search_backends() {
    let cases = [
        ("npm", "npm"),
        ("yarn", "npm"),
        ("pnpm", "npm"),
        ("bun", "npm"),
        ("pip", "PyPI"),
        ("pip3", "PyPI"),
        ("pypi", "PyPI"),
        ("uv", "PyPI"),
        ("poetry", "PyPI"),
        ("cargo", "crates.io"),
        ("crates.io", "crates.io"),
        ("go", "Go"),
        ("golang", "Go"),
        ("gem", "RubyGems"),
        ("rubygems", "RubyGems"),
        ("composer", "Packagist"),
        ("packagist", "Packagist"),
        ("nuget", "NuGet"),
        ("dotnet", "NuGet"),
        ("hex", "Hex"),
        ("mix", "Hex"),
        ("pub", "pub.dev"),
        ("dart", "pub.dev"),
        ("pub.dev", "pub.dev"),
    ];
    for (input, expected) in cases {
        assert_eq!(
            resolve_search_ecosystems(Some(input)).unwrap(),
            vec![expected]
        );
    }
}

#[test]
fn unsupported_ecosystems_fail_loudly() {
    assert!(resolve_search_ecosystems(Some("postgres")).is_err());
}

#[test]
fn exact_matches_rank_above_close_matches() {
    let query_norm = normalize_pkg_name("express");
    assert!(
        compute_match_score("express", &query_norm, "express", "exact")
            > compute_match_score("express", &query_norm, "expres", "close")
    );
}

#[test]
fn risky_install_script_signal_is_penalized() {
    let query_norm = normalize_pkg_name("express");
    let safe = compute_match_score(
        "express",
        &query_norm,
        "express",
        "exact, popular, licensed",
    );
    let risky = compute_match_score(
        "express",
        &query_norm,
        "express-tools",
        "prefix, popular, licensed, install-script-risk",
    );
    assert!(safe > risky);
}

#[test]
fn close_match_hint_detects_probable_typos() {
    let results = vec![
        SearchHit {
            eco: "npm".into(),
            pkg: "expres".into(),
            ver: "0.0.5".into(),
            desc: String::new(),
            signal: "exact, unstable".into(),
            score: 200,
        },
        SearchHit {
            eco: "npm".into(),
            pkg: "express".into(),
            ver: "5.0.0".into(),
            desc: String::new(),
            signal: build_signal(
                "expres",
                "express",
                &["popular".to_string(), "trusted".to_string()],
            ),
            score: 100,
        },
        SearchHit {
            eco: "npm".into(),
            pkg: "something-else".into(),
            ver: "1.0.0".into(),
            desc: String::new(),
            signal: "match".into(),
            score: 10,
        },
    ];
    assert_eq!(
        best_follow_up_hint("expres", &results).unwrap().pkg,
        "express"
    );
}

#[test]
fn short_exact_queries_do_not_force_a_typo_hint() {
    let results = vec![
        SearchHit {
            eco: "Go".into(),
            pkg: "github.com/gin-gonic/gin".into(),
            ver: "v1.10.0".into(),
            desc: String::new(),
            signal: "popular, licensed".into(),
            score: 100,
        },
        SearchHit {
            eco: "RubyGems".into(),
            pkg: "ginst".into(),
            ver: "1.0.0".into(),
            desc: String::new(),
            signal: "close".into(),
            score: 10,
        },
    ];
    assert!(best_follow_up_hint("gin", &results).is_none());
}

#[test]
fn signal_matching_is_token_aware() {
    assert!(signal_has("exact, stable", "stable"));
    assert!(!signal_has("exact, unstable", "stable"));
}

#[test]
fn go_module_path_escaping_handles_uppercase_segments() {
    assert_eq!(
        escape_go_module_path("github.com/Azure/azure-sdk-for-go"),
        "github.com/!azure/azure-sdk-for-go"
    );
}
