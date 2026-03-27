use serde::{Deserialize, Serialize};
use reqwest::blocking::Client;

const BATCH_URL: &str = "https://api.osv.dev/v1/querybatch";
const VULN_URL:  &str = "https://api.osv.dev/v1/vulns";

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

#[derive(Deserialize, Debug, Clone)]
pub struct OsvAffected {
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

/// Extract the first 'fixed' version from OSV affected ranges.
pub fn first_fixed_version(detail: &OsvVulnDetail) -> Option<String> {
    for affected in &detail.affected {
        for range in &affected.ranges {
            if range.kind == "SEMVER" || range.kind == "ECOSYSTEM" {
                for event in &range.events {
                    if let Some(ref fixed) = event.fixed {
                        if !fixed.is_empty() && fixed != "0" {
                            return Some(fixed.clone());
                        }
                    }
                }
            }
        }
    }
    None
}

// ── Shared HTTP client ────────────────────────────────────────────────────────

fn client() -> Client {
    Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap_or_default()
}

// ── API calls ─────────────────────────────────────────────────────────────────

/// Send a batch of (name, ecosystem, version) queries to OSV.
/// Returns a parallel vec of vuln ID lists.
pub fn batch_query(packages: &[(String, String, String)]) -> Result<Vec<Vec<OsvVulnRef>>, String> {
    let client = client();

    let queries: Vec<OsvQuery> = packages
        .iter()
        .map(|(name, ecosystem, version)| OsvQuery {
            package: OsvPackage { name: name.clone(), ecosystem: ecosystem.clone() },
            version: version.clone(),
        })
        .collect();

    let body = OsvBatchRequest { queries };

    let resp = client
        .post(BATCH_URL)
        .json(&body)
        .send()
        .map_err(|e| format!("request failed: {}", e))?;

    let status = resp.status();
    if !status.is_success() {
        return Err(format!("OSV API returned HTTP {}", status));
    }

    let text = resp.text().map_err(|e| format!("failed to read response: {}", e))?;
    let batch: OsvBatchResponse = serde_json::from_str(&text)
        .map_err(|e| format!("failed to parse OSV response: {}", e))?;
    Ok(batch.results.into_iter().map(|r| r.vulns).collect())
}

/// Fetch full detail for a single vulnerability ID.
pub fn fetch_vuln_detail(id: &str) -> Result<OsvVulnDetail, String> {
    let client = client();

    let url = format!("{}/{}", VULN_URL, id);
    let resp = client.get(&url).send().map_err(|e| format!("request failed: {}", e))?;

    let status = resp.status();
    if !status.is_success() {
        return Err(format!("OSV API returned HTTP {} for {}", status, id));
    }

    let text = resp.text().map_err(|e| format!("failed to read response: {}", e))?;
    let detail: OsvVulnDetail = serde_json::from_str(&text)
        .map_err(|e| format!("failed to parse vuln detail: {}", e))?;
    Ok(detail)
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
