use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Verdict {
    Allow,
    Block,
    RateLimited,
    Flagged,
}

impl std::fmt::Display for Verdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Verdict::Allow => write!(f, "ALLOW"),
            Verdict::Block => write!(f, "BLOCK"),
            Verdict::RateLimited => write!(f, "RATE_LIMITED"),
            Verdict::Flagged => write!(f, "FLAG"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub source_ip: String,
    pub source_port: u16,
    pub method: String,
    pub path: String,
    pub query: Option<String>,
    pub host: String,
    pub user_agent: Option<String>,
    pub content_type: Option<String>,
    pub content_length: Option<u64>,
    pub verdict: Verdict,
    pub blocked_by_stage: Option<String>,
    pub blocked_by_rule: Option<String>,
    pub blocked_reason: Option<String>,
    pub upstream_status: Option<u16>,
    pub upstream_latency_ms: Option<f64>,
    pub total_latency_ms: f64,
}

impl FirewallEvent {
    pub fn new(source_ip: String, source_port: u16) -> Self {
        let now = Utc::now();
        let id = format!("{}-{:04x}", now.format("%Y%m%d%H%M%S%3f"), rand_u16());
        Self {
            id,
            timestamp: now,
            source_ip,
            source_port,
            method: String::new(),
            path: String::new(),
            query: None,
            host: String::new(),
            user_agent: None,
            content_type: None,
            content_length: None,
            verdict: Verdict::Allow,
            blocked_by_stage: None,
            blocked_by_rule: None,
            blocked_reason: None,
            upstream_status: None,
            upstream_latency_ms: None,
            total_latency_ms: 0.0,
        }
    }

    pub fn block(&mut self, stage: &str, rule: &str, reason: &str) {
        self.verdict = Verdict::Block;
        self.blocked_by_stage = Some(stage.to_string());
        self.blocked_by_rule = Some(rule.to_string());
        self.blocked_reason = Some(reason.to_string());
    }

    pub fn rate_limit(&mut self, reason: &str) {
        self.verdict = Verdict::RateLimited;
        self.blocked_by_stage = Some("rate_limiter".to_string());
        self.blocked_reason = Some(reason.to_string());
    }

    pub fn flag(&mut self, tag: &str) {
        self.verdict = Verdict::Flagged;
        self.blocked_reason = Some(tag.to_string());
    }
}

fn rand_u16() -> u16 {
    use std::time::SystemTime;
    let d = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    ((d.subsec_nanos() ^ (d.as_secs() as u32)) & 0xFFFF) as u16
}
