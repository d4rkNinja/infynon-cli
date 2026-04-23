use chrono::{DateTime, Utc};
use serde_json::Value;

use super::matching::{levenshtein, normalize_pkg_name};

pub(super) fn signal_has(signal: &str, needle: &str) -> bool {
    signal.split(',').any(|value| value.trim() == needle)
}

pub(super) fn push_qualifier(qualifiers: &mut Vec<String>, value: &str) {
    if !qualifiers.iter().any(|existing| existing == value) {
        qualifiers.push(value.to_string());
    }
}

pub(super) fn has_non_empty(value: Option<&str>) -> bool {
    value.map(|item| !item.trim().is_empty()).unwrap_or(false)
}

pub(super) fn has_json_value(value: Option<&Value>) -> bool {
    match value {
        Some(Value::String(text)) => !text.trim().is_empty(),
        Some(Value::Array(items)) => !items.is_empty(),
        Some(Value::Object(map)) => {
            map.get("url")
                .and_then(Value::as_str)
                .map(|value| !value.trim().is_empty())
                .unwrap_or(!map.is_empty())
        }
        Some(Value::Bool(flag)) => *flag,
        Some(Value::Number(_)) => true,
        _ => false,
    }
}

pub(super) fn license_present(value: Option<&str>) -> bool {
    value
        .map(|item| {
            let normalized = item.trim().to_ascii_lowercase();
            !normalized.is_empty()
                && normalized != "unknown"
                && normalized != "none"
                && normalized != "unlicensed"
        })
        .unwrap_or(false)
}

pub(super) fn licenses_present(values: &[String]) -> bool {
    values.iter().any(|value| license_present(Some(value)))
}

pub(super) fn days_since_rfc3339(value: &str) -> Option<i64> {
    let parsed = DateTime::parse_from_rfc3339(value).ok()?;
    Some(Utc::now().signed_duration_since(parsed.with_timezone(&Utc)).num_days())
}

pub(super) fn add_release_age_qualifiers(qualifiers: &mut Vec<String>, released_at: Option<&str>) {
    let Some(days) = released_at.and_then(days_since_rfc3339) else {
        return;
    };

    if days <= 30 {
        push_qualifier(qualifiers, "new");
    } else if days <= 180 {
        push_qualifier(qualifiers, "fresh");
    } else if days >= 730 {
        push_qualifier(qualifiers, "stale");
    }
}

pub(super) fn is_prerelease(version: &str) -> bool {
    let normalized = version.trim().to_ascii_lowercase();
    if normalized.is_empty() || normalized == "-" {
        return false;
    }

    ["alpha", "beta", "rc", "dev", "preview", "snapshot", "canary", "next"]
        .iter()
        .any(|needle| normalized.contains(needle))
}

pub(super) fn add_version_qualifiers(qualifiers: &mut Vec<String>, version: &str) {
    let trimmed = version.trim();
    if trimmed.is_empty() || trimmed == "-" {
        push_qualifier(qualifiers, "no-version");
    } else if is_prerelease(trimmed) {
        push_qualifier(qualifiers, "unstable");
    } else {
        push_qualifier(qualifiers, "stable");
    }
}
