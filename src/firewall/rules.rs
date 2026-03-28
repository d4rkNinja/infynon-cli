use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use regex::Regex;

use crate::firewall::config::{RuleConfig, RuleActionType, load_ip_list};

pub struct CompiledRule {
    pub name: String,
    pub description: String,
    pub priority: u32,
    pub enabled: bool,
    pub conditions: Vec<CompiledCondition>,
    pub action: RuleAction,
    pub hit_count: AtomicU64,
}

pub enum CompiledCondition {
    IpMatch(HashSet<String>),
    PathPrefix(String),
    PathExact(String),
    PathRegex(Regex),
    MethodIs(Vec<String>),
    HeaderExact { name: String, value: String },
    UserAgentRegex(Regex),
    BodyContains(String),
    ContentTypeIs(Vec<String>),
    RequestSizeGt(u64),
}

#[derive(Debug, Clone)]
pub enum RuleAction {
    Block { status: u16, message: String },
    Allow,
    Flag { tag: String },
    RateLimit { requests: u32, window_seconds: u32 },
}

pub struct RequestContext<'a> {
    pub ip: &'a str,
    pub method: &'a str,
    pub path: &'a str,
    pub headers: &'a [(String, String)],
    pub user_agent: Option<&'a str>,
    pub content_type: Option<&'a str>,
    pub content_length: Option<u64>,
    pub body_preview: Option<&'a str>,
}

impl CompiledRule {
    pub fn from_config(config: &RuleConfig) -> Option<Self> {
        let mut conditions = Vec::new();

        // IP conditions
        let mut ips = HashSet::new();
        for ip in &config.condition.ip {
            ips.insert(ip.clone());
        }
        if let Some(ref file) = config.condition.ip_file {
            for ip in load_ip_list(Path::new(file)) {
                ips.insert(ip);
            }
        }
        if !ips.is_empty() {
            conditions.push(CompiledCondition::IpMatch(ips));
        }

        // Path conditions
        if let Some(ref prefix) = config.condition.path_prefix {
            conditions.push(CompiledCondition::PathPrefix(prefix.clone()));
        }
        if let Some(ref exact) = config.condition.path_exact {
            conditions.push(CompiledCondition::PathExact(exact.clone()));
        }
        if let Some(ref pattern) = config.condition.path_regex {
            if let Ok(re) = Regex::new(pattern) {
                conditions.push(CompiledCondition::PathRegex(re));
            }
        }

        // Method
        if !config.condition.method.is_empty() {
            conditions.push(CompiledCondition::MethodIs(config.condition.method.clone()));
        }

        // Header
        if let Some(ref headers) = config.condition.header {
            for (name, value) in headers {
                conditions.push(CompiledCondition::HeaderExact {
                    name: name.clone(),
                    value: value.clone(),
                });
            }
        }

        // User-Agent regex
        if let Some(ref pattern) = config.condition.user_agent_regex {
            if let Ok(re) = Regex::new(pattern) {
                conditions.push(CompiledCondition::UserAgentRegex(re));
            }
        }

        // Body contains
        if let Some(ref body) = config.condition.body_contains {
            conditions.push(CompiledCondition::BodyContains(body.clone()));
        }

        // Content type
        if !config.condition.content_type.is_empty() {
            conditions.push(CompiledCondition::ContentTypeIs(config.condition.content_type.clone()));
        }

        // Request size
        if let Some(size) = config.condition.request_size_gt {
            conditions.push(CompiledCondition::RequestSizeGt(size));
        }

        // Action
        let action = match config.action.action_type {
            RuleActionType::Allow => RuleAction::Allow,
            RuleActionType::Flag => RuleAction::Flag {
                tag: config.action.tag.clone().unwrap_or_else(|| config.name.clone()),
            },
            RuleActionType::RateLimit => RuleAction::RateLimit {
                requests: config.action.requests.unwrap_or(10),
                window_seconds: config.action.window_seconds.unwrap_or(60),
            },
            RuleActionType::Block => RuleAction::Block {
                status: config.action.status,
                message: config.action.message.clone(),
            },
        };

        Some(Self {
            name: config.name.clone(),
            description: config.description.clone(),
            priority: config.priority,
            enabled: config.enabled,
            conditions,
            action,
            hit_count: AtomicU64::new(0),
        })
    }

    /// Check if all conditions match the request context
    pub fn matches(&self, ctx: &RequestContext) -> bool {
        if !self.enabled || self.conditions.is_empty() {
            return false;
        }

        for condition in &self.conditions {
            let matched = match condition {
                CompiledCondition::IpMatch(ips) => ips.contains(ctx.ip),
                CompiledCondition::PathPrefix(prefix) => ctx.path.starts_with(prefix),
                CompiledCondition::PathExact(exact) => ctx.path == exact,
                CompiledCondition::PathRegex(re) => re.is_match(ctx.path),
                CompiledCondition::MethodIs(methods) => {
                    methods.iter().any(|m| m.eq_ignore_ascii_case(ctx.method))
                }
                CompiledCondition::HeaderExact { name, value } => {
                    ctx.headers.iter().any(|(h, v)| h.eq_ignore_ascii_case(name) && v == value)
                }
                CompiledCondition::UserAgentRegex(re) => {
                    ctx.user_agent.map(|ua| re.is_match(ua)).unwrap_or(false)
                }
                CompiledCondition::BodyContains(needle) => {
                    ctx.body_preview.map(|b| b.contains(needle)).unwrap_or(false)
                }
                CompiledCondition::ContentTypeIs(types) => {
                    ctx.content_type.map(|ct| types.iter().any(|t| ct.starts_with(t))).unwrap_or(false)
                }
                CompiledCondition::RequestSizeGt(size) => {
                    ctx.content_length.map(|cl| cl > *size).unwrap_or(false)
                }
            };

            if !matched {
                return false; // AND logic — all must match
            }
        }

        self.hit_count.fetch_add(1, Ordering::Relaxed);
        true
    }

    pub fn hits(&self) -> u64 {
        self.hit_count.load(Ordering::Relaxed)
    }
}

/// Compile all rules from config, sorted by priority
pub fn compile_rules(configs: &[RuleConfig]) -> Vec<CompiledRule> {
    let mut rules: Vec<CompiledRule> = configs
        .iter()
        .filter_map(|c| CompiledRule::from_config(c))
        .collect();
    rules.sort_by_key(|r| r.priority);
    rules
}
