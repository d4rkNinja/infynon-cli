use super::signals::signal_has;
use super::types::SearchHit;

pub(super) fn normalize_pkg_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

pub(super) fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }

    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr = vec![0; b.len() + 1];

    for (i, ca) in a.iter().enumerate() {
        curr[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            curr[j + 1] = (curr[j] + 1).min(prev[j + 1] + 1).min(prev[j] + cost);
        }
        prev.clone_from(&curr);
    }

    prev[b.len()]
}

pub(super) fn best_follow_up_hint<'a>(
    query: &str,
    results: &'a [SearchHit],
) -> Option<&'a SearchHit> {
    if query.chars().count() < 4 {
        return None;
    }

    let query_norm = normalize_pkg_name(query);
    let exact = results.iter().find(|hit| {
        hit.pkg.eq_ignore_ascii_case(query) || normalize_pkg_name(&hit.pkg) == query_norm
    });

    let candidate = results
        .iter()
        .find(|hit| {
            !hit.pkg.eq_ignore_ascii_case(query)
                && levenshtein(&query_norm, &normalize_pkg_name(&hit.pkg)) <= 2
                && (signal_has(&hit.signal, "popular")
                    || signal_has(&hit.signal, "trusted")
                    || signal_has(&hit.signal, "verified")
                    || signal_has(&hit.signal, "licensed"))
        })
        .or_else(|| {
            results.iter().find(|hit| {
                !hit.pkg.eq_ignore_ascii_case(query)
                    && levenshtein(&query_norm, &normalize_pkg_name(&hit.pkg)) <= 2
            })
        });

    match (exact, candidate) {
        (Some(current), Some(suggestion))
            if signal_has(&current.signal, "unstable")
                && (signal_has(&suggestion.signal, "popular")
                    || signal_has(&suggestion.signal, "trusted")
                    || signal_has(&suggestion.signal, "verified")
                    || signal_has(&suggestion.signal, "licensed")) =>
        {
            Some(suggestion)
        }
        (None, Some(suggestion)) => Some(suggestion),
        _ => None,
    }
}
