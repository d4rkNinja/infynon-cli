use crate::firewall::config::FirewallConfig;
use crate::firewall::events::{FirewallEvent, Verdict};
use crate::firewall::ip_filter::IpFilter;
use crate::firewall::rate_limiter::RateLimiter;
use crate::firewall::waf::WafEngine;
use crate::firewall::rules::{self, CompiledRule, RuleAction, RequestContext};

/// The request evaluation pipeline: IP → Rate Limit → WAF → Custom Rules
pub struct Pipeline {
    pub ip_filter: IpFilter,
    pub rate_limiter: RateLimiter,
    pub waf: WafEngine,
    pub custom_rules: Vec<CompiledRule>,
}

impl Pipeline {
    pub fn new(config: &FirewallConfig) -> Self {
        Self {
            ip_filter: IpFilter::new(&config.ip),
            rate_limiter: RateLimiter::new(&config.rate_limit),
            waf: WafEngine::new(&config.waf),
            custom_rules: rules::compile_rules(&config.rules),
        }
    }

    /// Evaluate a request through all pipeline stages.
    /// Mutates the event with verdict details. Returns true if request should be forwarded.
    pub fn evaluate(
        &self,
        event: &mut FirewallEvent,
        headers: &[(String, String)],
        body_preview: Option<&str>,
    ) -> bool {
        // Clone values to avoid holding immutable borrows across mutable event calls
        let ip = event.source_ip.clone();
        let method = event.method.clone();
        let path = event.path.clone();
        let query_owned = event.query.clone();
        let query = query_owned.as_deref();
        let ua_owned = event.user_agent.clone();
        let user_agent = ua_owned.as_deref();
        let ct_owned = event.content_type.clone();
        let content_length = event.content_length;

        // ── Stage 1: IP Filter ──────────────────────────────────────────────
        if let Some(reason) = self.ip_filter.check(&ip) {
            event.block("ip_filter", "ip-blocklist", &reason);
            return false;
        }

        // ── Stage 2: Rate Limiter ───────────────────────────────────────────
        if let Some((reason, _retry_after)) = self.rate_limiter.check(&ip, &path) {
            event.rate_limit(&reason);
            return false;
        }

        // ── Stage 3: WAF Engine ─────────────────────────────────────────────
        if let Some((rule_name, reason)) = self.waf.check(
            &method,
            &path,
            query,
            user_agent,
            content_length,
            headers,
            body_preview,
        ) {
            event.block("waf", &rule_name, &reason);
            self.ip_filter.record_block(&ip);
            return false;
        }

        // ── Stage 4: Custom Rules ───────────────────────────────────────────
        let ctx = RequestContext {
            ip: &ip,
            method: &method,
            path: &path,
            headers,
            user_agent,
            content_type: ct_owned.as_deref(),
            content_length,
            body_preview,
        };

        for rule in &self.custom_rules {
            if rule.matches(&ctx) {
                match &rule.action {
                    RuleAction::Allow => {
                        return true;
                    }
                    RuleAction::Block { status: _, message } => {
                        event.block("custom_rules", &rule.name, message);
                        self.ip_filter.record_block(&ip);
                        return false;
                    }
                    RuleAction::Flag { tag } => {
                        event.flag(tag);
                        return true;
                    }
                    RuleAction::RateLimit { .. } => {
                        event.flag(&format!("rate-limit:{}", rule.name));
                        return true;
                    }
                }
            }
        }

        // Default: ALLOW
        true
    }

    /// Periodic cleanup of internal state
    pub fn cleanup(&self) {
        self.ip_filter.cleanup();
        self.rate_limiter.cleanup();
    }

    /// Dynamically block an IP
    pub fn block_ip(&self, ip: &str) {
        self.ip_filter.add_block(ip);
    }

    /// Dynamically unblock an IP
    pub fn unblock_ip(&self, ip: &str) {
        self.ip_filter.remove_block(ip);
    }

    /// Get rule stats for TUI display
    pub fn rule_stats(&self) -> Vec<(String, String, bool, u64, u32)> {
        self.custom_rules.iter()
            .map(|r| (r.name.clone(), r.description.clone(), r.enabled, r.hits(), r.priority))
            .collect()
    }
}
