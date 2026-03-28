use serde::{Deserialize, Serialize};
use reqwest::blocking::Client;
use std::sync::OnceLock;

const BATCH_URL: &str = "https://api.osv.dev/v1/querybatch";
const VULN_URL:  &str = "https://api.osv.dev/v1/vulns";
const BATCH_CHUNK_SIZE: usize = 1000;
const DETAIL_CONCURRENCY: usize = 20;

// ── Request types ─────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct OsvPackage {
    pub name:      String,
    pub ecosystem: String,
}

#[derive(Serialize)]
pub struct OsvQuery {
    pub package: OsvPackage,
    pub version: String,
}

#[derive(Serialize)]
pub struct OsvBatchRequest {
    pub queries: Vec<OsvQuery>,
}

// ── Response types ────────────────────────────────────────────────────────────

#[derive(Deserialize, Debug, Clone)]
pub struct OsvVulnRef {
    pub id:       String,
    pub modified: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct OsvQueryResult {
    #[serde(default)]
    pub vulns: Vec<OsvVulnRef>,
}

#[derive(Deserialize, Debug)]
pub struct OsvBatchResponse {
    pub results: Vec<OsvQueryResult>,
}

// ── Full vulnerability detail ─────────────────────────────────────────────────

#[derive(Deserialize, Debug, Clone)]
pub struct OsvSeverity {
    #[serde(rename = "type")]
    pub kind:  Option<String>,
    pub score: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OsvReference {
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub url:  String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OsvEvent {
    pub introduced: Option<String>,
    pub fixed:      Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OsvRange {
    #[serde(rename = "type")]
    pub kind:   String,
    #[serde(default)]
    pub events: Vec<OsvEvent>,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct OsvAffectedPackage {
    #[serde(default)]
    pub name:      String,
    #[serde(default)]
    pub ecosystem: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OsvAffected {
    #[serde(default)]
    pub package: OsvAffectedPackage,
    #[serde(default)]
    pub ranges: Vec<OsvRange>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OsvVulnDetail {
    pub id:          String,
    pub summary:     Option<String>,
    pub details:     Option<String>,
    pub published:   Option<String>,
    pub modified:    Option<String>,
    #[serde(default)]
    pub severity:    Vec<OsvSeverity>,
    #[serde(default)]
    pub references:  Vec<OsvReference>,
    #[serde(default)]
    pub affected:    Vec<OsvAffected>,
}


/// For Go modules, extract the required major version from the module path (/vN suffix).
/// Returns `None` if no major suffix is present (v0/v1 modules).
fn go_module_major(pkg_name: &str) -> Option<u64> {
    let mut parts = pkg_name.rsplitn(2, '/');
    if let Some(last) = parts.next() {
        if last.starts_with('v') {
            let digits = &last[1..];
            if !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()) {
                return digits.parse().ok();
            }
        }
    }
    None
}

/// Check whether a version string is compatible with a Go module's major version constraint.
fn go_version_compatible(pkg_name: &str, version: &str) -> bool {
    let v = version.trim_start_matches('v');
    let ver_major: u64 = v.split('.').next()
        .and_then(|m| m.parse().ok())
        .unwrap_or(0);
    match go_module_major(pkg_name) {
        Some(m) => ver_major == m,
        None    => ver_major <= 1, // v0/v1 module — reject anything v2+
    }
}

/// Compare two version strings, handling Go pseudo-versions and standard semver.
/// Returns Ordering: Less / Equal / Greater.
fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    // Go pseudo-versions use a 14-digit timestamp as the canonical sort key
    let pseudo_ts = |v: &str| -> Option<u64> {
        let v = v.trim_start_matches('v');
        let after_first_dash = v.find('-').map(|i| &v[i + 1..])?;
        let ts_part = after_first_dash.split('-').next()?;
        if ts_part.len() >= 14 && ts_part.chars().all(|c| c.is_ascii_digit()) {
            ts_part[..14].parse::<u64>().ok()
        } else {
            None
        }
    };

    let pa = pseudo_ts(a);
    let pb = pseudo_ts(b);

    match (pa, pb) {
        (Some(ta), Some(tb)) => return ta.cmp(&tb),
        (None, Some(_))      => return std::cmp::Ordering::Greater, // regular > pseudo
        (Some(_), None)      => return std::cmp::Ordering::Less,
        (None, None)         => {}
    }

    let parse = |v: &str| -> ((u64, u64, u64), Option<String>) {
        let v = v.trim_start_matches('v');
        let mut parts = v.splitn(3, '.');
        let maj: u64 = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
        let min: u64 = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
        let patch_str = parts.next().unwrap_or("0");
        // Split patch number from pre-release label at first '-'
        let (pat, pre) = if let Some(dash) = patch_str.find('-') {
            let num: u64 = patch_str[..dash].parse().unwrap_or(0);
            let label = patch_str[dash + 1..].to_string();
            (num, Some(label))
        } else {
            (patch_str.parse().unwrap_or(0), None)
        };
        ((maj, min, pat), pre)
    };

    let (na, pre_a) = parse(a);
    let (nb, pre_b) = parse(b);

    let num_ord = na.cmp(&nb);
    if num_ord != std::cmp::Ordering::Equal {
        return num_ord;
    }

    // At equal numeric version, stable > rc > beta > alpha
    match (pre_a, pre_b) {
        (None,     None)     => std::cmp::Ordering::Equal,
        (None,     Some(_))  => std::cmp::Ordering::Greater, // stable > pre-release
        (Some(_),  None)     => std::cmp::Ordering::Less,
        (Some(pa), Some(pb)) => {
            let rank = |s: &str| -> u8 {
                let s = s.to_ascii_lowercase();
                if s.starts_with("rc")    { 3 }
                else if s.starts_with("beta") { 2 }
                else if s.starts_with("alpha") { 1 }
                else { 0 }
            };
            let ra = rank(&pa);
            let rb = rank(&pb);
            if ra != rb { ra.cmp(&rb) } else { pa.cmp(&pb) }
        }
    }
}

/// Return the highest version string from a list.
pub fn max_version(versions: &[String]) -> Option<String> {
    versions.iter()
        .max_by(|a, b| compare_versions(a, b))
        .cloned()
}

/// Return the single best fixed version for a specific package from an OSV vuln detail.
///
/// Single pass over `affected`: collects versions from exact-name-matching entries
/// (`matched`) and from all entries (`all`) simultaneously. Uses `matched` when
/// non-empty so jwt/v4 fixes don't leak into jwt/v5. Falls back to `all` for older
/// CVEs that omit the package name. Go major-version filter applied to both.
pub fn best_fixed_version(detail: &OsvVulnDetail, pkg_name: &str, ecosystem: &str) -> Option<String> {
    let is_go = ecosystem == "Go" && !pkg_name.is_empty();

    let mut matched: Vec<String> = Vec::new(); // from entries whose name == pkg_name
    let mut all:     Vec<String> = Vec::new(); // from every entry (fallback)

    for affected in &detail.affected {
        let name_matches = pkg_name.is_empty()
            || affected.package.name.is_empty()
            || affected.package.name == pkg_name;

        for range in &affected.ranges {
            if range.kind != "SEMVER" && range.kind != "ECOSYSTEM" { continue; }
            for event in &range.events {
                if let Some(ref fixed) = event.fixed {
                    if fixed.is_empty() || fixed == "0" { continue; }
                    if is_go && !go_version_compatible(pkg_name, fixed) { continue; }
                    if name_matches { matched.push(fixed.clone()); }
                    all.push(fixed.clone());
                }
            }
        }
    }

    max_version(if !matched.is_empty() { &matched } else { &all })
}

/// Extract the best fixed version without package-name filtering (legacy compatibility).
pub fn first_fixed_version(detail: &OsvVulnDetail) -> Option<String> {
    best_fixed_version(detail, "", "")
}

// ── Shared HTTP client (reused across all calls) ─────────────────────────────

static CLIENT: OnceLock<Client> = OnceLock::new();

fn client() -> &'static Client {
    CLIENT.get_or_init(|| {
        Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_default()
    })
}

// ── API calls ─────────────────────────────────────────────────────────────────

/// Send a batch of (name, ecosystem, version) queries to OSV.
/// Returns a parallel vec of vuln ID lists.
/// Filters out invalid entries (empty name/version) and chunks large batches.
pub fn batch_query(packages: &[(String, String, String)]) -> Result<Vec<Vec<OsvVulnRef>>, String> {
    let c = client();

    // Build queries, filtering out invalid entries that would cause HTTP 400
    let valid_indices: Vec<usize> = (0..packages.len())
        .filter(|&i| {
            let (name, eco, ver) = &packages[i];
            !name.trim().is_empty() && !eco.trim().is_empty() && !ver.trim().is_empty()
        })
        .collect();

    // Pre-fill results for all packages (empty for invalid ones)
    let mut all_results: Vec<Vec<OsvVulnRef>> = vec![vec![]; packages.len()];

    if valid_indices.is_empty() {
        return Ok(all_results);
    }

    // Process in chunks to avoid oversized requests
    for chunk_indices in valid_indices.chunks(BATCH_CHUNK_SIZE) {
        let queries: Vec<OsvQuery> = chunk_indices.iter().map(|&i| {
            let (name, ecosystem, version) = &packages[i];
            OsvQuery {
                package: OsvPackage { name: name.clone(), ecosystem: ecosystem.clone() },
                version: version.clone(),
            }
        }).collect();

        let body = OsvBatchRequest { queries };

        let resp = c
            .post(BATCH_URL)
            .json(&body)
            .send()
            .map_err(|e| format!("request failed: {}", e))?;

        let status = resp.status();
        if !status.is_success() {
            let body_text = resp.text().unwrap_or_default();
            return Err(format!("API returned HTTP {} — {}", status, body_text.chars().take(200).collect::<String>()));
        }

        let text = resp.text().map_err(|e| format!("failed to read response: {}", e))?;
        let batch: OsvBatchResponse = serde_json::from_str(&text)
            .map_err(|e| format!("failed to parse response: {}", e))?;

        // Map results back to original indices
        for (j, result) in batch.results.into_iter().enumerate() {
            if j < chunk_indices.len() {
                all_results[chunk_indices[j]] = result.vulns;
            }
        }
    }

    Ok(all_results)
}

/// Fetch full details for multiple vulnerability IDs in parallel.
/// Uses a thread pool with bounded concurrency.
pub fn fetch_vuln_details_batch(ids: &[String]) -> Vec<(String, Result<OsvVulnDetail, String>)> {
    use std::sync::Mutex;

    if ids.is_empty() {
        return vec![];
    }

    let results: Mutex<Vec<(String, Result<OsvVulnDetail, String>)>> = Mutex::new(Vec::with_capacity(ids.len()));
    let work: Mutex<std::slice::Iter<String>> = Mutex::new(ids.iter());

    std::thread::scope(|s| {
        let num_threads = DETAIL_CONCURRENCY.min(ids.len());
        for _ in 0..num_threads {
            let results = &results;
            let work = &work;
            s.spawn(move || {
                let c = client();
                loop {
                    let id = {
                        let mut iter = work.lock().unwrap();
                        iter.next().cloned()
                    };
                    let Some(id) = id else { break };

                    let result = fetch_single_detail(c, &id);
                    results.lock().unwrap().push((id, result));
                }
            });
        }
    });

    results.into_inner().unwrap()
}

/// Fetch full detail for a single vulnerability ID using the provided client.
fn fetch_single_detail(c: &Client, id: &str) -> Result<OsvVulnDetail, String> {
    let url = format!("{}/{}", VULN_URL, id);
    let resp = c.get(&url).send().map_err(|e| format!("request failed: {}", e))?;

    let status = resp.status();
    if !status.is_success() {
        return Err(format!("API returned HTTP {} for {}", status, id));
    }

    let text = resp.text().map_err(|e| format!("failed to read response: {}", e))?;
    let detail: OsvVulnDetail = serde_json::from_str(&text)
        .map_err(|e| format!("failed to parse vuln detail: {}", e))?;
    Ok(detail)
}

/// Fetch full detail for a single vulnerability ID (convenience wrapper).
pub fn fetch_vuln_detail(id: &str) -> Result<OsvVulnDetail, String> {
    fetch_single_detail(client(), id)
}

/// Derive a simple severity label from OSV severity array or summary keywords.
pub fn severity_label(detail: &OsvVulnDetail) -> &'static str {
    // CVSS score-based classification
    for s in &detail.severity {
        if let Some(score) = &s.score {
            // CVSS v3 vector string contains a numeric base score
            let score_str = score.to_uppercase();
            if score_str.contains("CRITICAL") { return "CRITICAL"; }
            if score_str.contains("HIGH")     { return "HIGH"; }
            if score_str.contains("MEDIUM")   { return "MEDIUM"; }
            if score_str.contains("LOW")      { return "LOW"; }
            // Try to parse numeric CVSS base score
            let parts: Vec<&str> = score.split('/').collect();
            for p in parts {
                if let Ok(n) = p.parse::<f32>() {
                    return match n {
                        s if s >= 9.0 => "CRITICAL",
                        s if s >= 7.0 => "HIGH",
                        s if s >= 4.0 => "MEDIUM",
                        _             => "LOW",
                    };
                }
            }
        }
    }
    // Keyword fallback on summary
    let summary = detail.summary.as_deref().unwrap_or("").to_uppercase();
    if summary.contains("CRITICAL") { "CRITICAL" }
    else if summary.contains("HIGH") { "HIGH" }
    else if summary.contains("MEDIUM") { "MEDIUM" }
    else if summary.contains("LOW") { "LOW" }
    else { "INFORMATIONAL" }
}
