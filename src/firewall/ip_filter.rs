use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use std::path::Path;
use std::sync::Mutex;
use std::time::Instant;
use ipnet::IpNet;

use crate::firewall::config::{IpConfig, load_ip_list};

pub struct IpFilter {
    mode: String, // "blocklist" | "allowlist" | "disabled"
    blocklist: HashSet<String>,
    allowlist: HashSet<String>,
    cidr_blocklist: Vec<IpNet>,
    cidr_allowlist: Vec<IpNet>,
    // Auto-reputation: ip -> (block_count, first_block_time)
    reputation: Mutex<HashMap<String, (u32, Instant)>>,
    reputation_threshold: u32,
    reputation_window_secs: u64,
    reputation_ban_secs: u64,
    reputation_enabled: bool,
    // Dynamic bans: ip -> ban_expires_at
    dynamic_bans: Mutex<HashMap<String, Instant>>,
}

impl IpFilter {
    pub fn new(config: &IpConfig) -> Self {
        let mut blocklist = HashSet::new();
        let mut allowlist = HashSet::new();
        let mut cidr_blocklist = Vec::new();
        let mut cidr_allowlist = Vec::new();

        // Load inline lists
        for entry in &config.blocklist {
            if entry.contains('/') {
                if let Ok(net) = entry.parse::<IpNet>() {
                    cidr_blocklist.push(net);
                }
            } else {
                blocklist.insert(entry.clone());
            }
        }
        for entry in &config.allowlist {
            if entry.contains('/') {
                if let Ok(net) = entry.parse::<IpNet>() {
                    cidr_allowlist.push(net);
                }
            } else {
                allowlist.insert(entry.clone());
            }
        }

        // Load from files
        if let Some(ref path) = config.blocklist_file {
            for entry in load_ip_list(Path::new(path)) {
                if entry.contains('/') {
                    if let Ok(net) = entry.parse::<IpNet>() {
                        cidr_blocklist.push(net);
                    }
                } else {
                    blocklist.insert(entry);
                }
            }
        }
        if let Some(ref path) = config.allowlist_file {
            for entry in load_ip_list(Path::new(path)) {
                if entry.contains('/') {
                    if let Ok(net) = entry.parse::<IpNet>() {
                        cidr_allowlist.push(net);
                    }
                } else {
                    allowlist.insert(entry);
                }
            }
        }

        Self {
            mode: config.mode.clone(),
            blocklist,
            allowlist,
            cidr_blocklist,
            cidr_allowlist,
            reputation: Mutex::new(HashMap::new()),
            reputation_threshold: config.auto_reputation.threshold,
            reputation_window_secs: config.auto_reputation.window_minutes as u64 * 60,
            reputation_ban_secs: config.auto_reputation.ban_duration_minutes as u64 * 60,
            reputation_enabled: config.auto_reputation.enabled,
            dynamic_bans: Mutex::new(HashMap::new()),
        }
    }

    /// Returns None if allowed, Some(reason) if blocked
    pub fn check(&self, ip_str: &str) -> Option<String> {
        if self.mode == "disabled" {
            return None;
        }

        // Check dynamic bans first
        if let Ok(bans) = self.dynamic_bans.lock() {
            if let Some(expires) = bans.get(ip_str) {
                if Instant::now() < *expires {
                    return Some(format!("IP auto-banned (reputation): {}", ip_str));
                }
            }
        }

        let ip: Option<IpAddr> = ip_str.parse().ok();

        if self.mode == "allowlist" {
            // In allowlist mode, everything NOT in the allowlist is blocked
            if self.allowlist.contains(ip_str) {
                return None;
            }
            if let Some(addr) = ip {
                for net in &self.cidr_allowlist {
                    if net.contains(&addr) {
                        return None;
                    }
                }
            }
            return Some(format!("IP not in allowlist: {}", ip_str));
        }

        // Blocklist mode
        if self.blocklist.contains(ip_str) {
            return Some(format!("IP in blocklist: {}", ip_str));
        }
        if let Some(addr) = ip {
            for net in &self.cidr_blocklist {
                if net.contains(&addr) {
                    return Some(format!("IP in CIDR blocklist: {} ({})", ip_str, net));
                }
            }
        }

        None
    }

    /// Record a block for auto-reputation tracking
    pub fn record_block(&self, ip_str: &str) {
        if !self.reputation_enabled {
            return;
        }
        let now = Instant::now();
        if let Ok(mut rep) = self.reputation.lock() {
            let entry = rep.entry(ip_str.to_string()).or_insert((0, now));
            // Reset if window expired
            if now.duration_since(entry.1).as_secs() > self.reputation_window_secs {
                *entry = (1, now);
            } else {
                entry.0 += 1;
            }
            if entry.0 >= self.reputation_threshold {
                // Auto-ban
                if let Ok(mut bans) = self.dynamic_bans.lock() {
                    let ban_until = now + std::time::Duration::from_secs(self.reputation_ban_secs);
                    bans.insert(ip_str.to_string(), ban_until);
                }
                // Reset count
                *entry = (0, now);
            }
        }
    }

    /// Add an IP to the blocklist at runtime
    pub fn add_block(&self, ip_str: &str) {
        if let Ok(mut bans) = self.dynamic_bans.lock() {
            // Permanent ban = very far in the future
            let ban_until = Instant::now() + std::time::Duration::from_secs(365 * 24 * 3600);
            bans.insert(ip_str.to_string(), ban_until);
        }
    }

    /// Remove an IP from runtime blocklist
    pub fn remove_block(&self, ip_str: &str) {
        if let Ok(mut bans) = self.dynamic_bans.lock() {
            bans.remove(ip_str);
        }
    }

    /// Clean up expired entries
    pub fn cleanup(&self) {
        let now = Instant::now();
        if let Ok(mut bans) = self.dynamic_bans.lock() {
            bans.retain(|_, expires| *expires > now);
        }
        if let Ok(mut rep) = self.reputation.lock() {
            rep.retain(|_, (_, first)| now.duration_since(*first).as_secs() < self.reputation_window_secs);
        }
    }
}
