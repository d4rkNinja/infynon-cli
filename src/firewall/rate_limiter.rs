use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use crate::firewall::config::RateLimitConfig;

struct SlidingWindow {
    timestamps: Vec<Instant>,
    max_requests: u32,
    window_secs: u64,
}

impl SlidingWindow {
    fn new(max_requests: u32, window_secs: u32) -> Self {
        Self {
            timestamps: Vec::new(),
            max_requests,
            window_secs: window_secs as u64,
        }
    }

    /// Returns true if the request should be allowed, false if rate limited.
    /// Also returns seconds until the window resets.
    fn check_and_record(&mut self) -> (bool, u64) {
        let now = Instant::now();
        let cutoff = now - std::time::Duration::from_secs(self.window_secs);
        self.timestamps.retain(|t| *t > cutoff);

        if self.timestamps.len() >= self.max_requests as usize {
            let oldest = self.timestamps.first().copied().unwrap_or(now);
            let retry_after = self.window_secs.saturating_sub(now.duration_since(oldest).as_secs());
            return (false, retry_after.max(1));
        }

        self.timestamps.push(now);
        (true, 0)
    }
}

pub struct RateLimiter {
    enabled: bool,
    // Per-IP windows
    per_ip: Mutex<HashMap<String, SlidingWindow>>,
    per_ip_max: u32,
    per_ip_window: u32,
    // Per-path windows (ip:path -> window)
    per_path: Mutex<HashMap<String, SlidingWindow>>,
    path_limits: HashMap<String, (u32, u32)>, // path -> (max_requests, window_secs)
    // Global counter
    global: Mutex<SlidingWindow>,
    global_rps: u32,
}

impl RateLimiter {
    pub fn new(config: &RateLimitConfig) -> Self {
        let (per_ip_max, per_ip_window) = config.per_ip
            .as_ref()
            .map(|c| (c.requests, c.window_seconds))
            .unwrap_or((100, 60));

        let global_rps = config.global
            .as_ref()
            .map(|c| c.requests_per_second)
            .unwrap_or(1000);

        let mut path_limits = HashMap::new();
        for (path, cfg) in &config.per_path {
            path_limits.insert(path.clone(), (cfg.requests, cfg.window_seconds));
        }

        Self {
            enabled: config.enabled,
            per_ip: Mutex::new(HashMap::new()),
            per_ip_max,
            per_ip_window,
            per_path: Mutex::new(HashMap::new()),
            path_limits,
            global: Mutex::new(SlidingWindow::new(global_rps, 1)),
            global_rps,
        }
    }

    /// Check rate limits. Returns None if allowed, Some((reason, retry_after)) if limited.
    pub fn check(&self, ip: &str, path: &str) -> Option<(String, u64)> {
        if !self.enabled {
            return None;
        }

        // Global rate check
        if let Ok(mut global) = self.global.lock() {
            let (allowed, retry_after) = global.check_and_record();
            if !allowed {
                return Some((
                    format!("Global rate limit exceeded ({}/s)", self.global_rps),
                    retry_after,
                ));
            }
        }

        // Per-IP rate check
        if let Ok(mut per_ip) = self.per_ip.lock() {
            let window = per_ip.entry(ip.to_string())
                .or_insert_with(|| SlidingWindow::new(self.per_ip_max, self.per_ip_window));
            let (allowed, retry_after) = window.check_and_record();
            if !allowed {
                return Some((
                    format!("Per-IP rate limit exceeded ({}/{} for {})",
                            self.per_ip_max, self.per_ip_window, ip),
                    retry_after,
                ));
            }
        }

        // Per-path rate check
        for (pattern, (max_req, window_secs)) in &self.path_limits {
            if path.starts_with(pattern) || path == pattern {
                let key = format!("{}:{}", ip, pattern);
                if let Ok(mut per_path) = self.per_path.lock() {
                    let window = per_path.entry(key)
                        .or_insert_with(|| SlidingWindow::new(*max_req, *window_secs));
                    let (allowed, retry_after) = window.check_and_record();
                    if !allowed {
                        return Some((
                            format!("Rate limit exceeded for {} ({}/{}s)",
                                    pattern, max_req, window_secs),
                            retry_after,
                        ));
                    }
                }
            }
        }

        None
    }

    /// Periodic cleanup of old entries to prevent memory leaks
    pub fn cleanup(&self) {
        let now = Instant::now();
        let cutoff_duration = std::time::Duration::from_secs(300); // 5 min stale threshold

        if let Ok(mut per_ip) = self.per_ip.lock() {
            per_ip.retain(|_, w| {
                w.timestamps.last()
                    .map(|t| now.duration_since(*t) < cutoff_duration)
                    .unwrap_or(false)
            });
        }
        if let Ok(mut per_path) = self.per_path.lock() {
            per_path.retain(|_, w| {
                w.timestamps.last()
                    .map(|t| now.duration_since(*t) < cutoff_duration)
                    .unwrap_or(false)
            });
        }
    }
}
