use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use crate::firewall::events::Verdict;

pub struct Stats {
    pub start_time: Instant,
    pub total_requests: AtomicU64,
    pub total_blocked: AtomicU64,
    pub total_flagged: AtomicU64,
    pub total_rate_limited: AtomicU64,
    pub active_connections: AtomicU64,

    // Rolling per-second counters (last 60 seconds)
    traffic_ring: Mutex<RingBuffer>,
    blocks_ring: Mutex<RingBuffer>,

    // Top trackers
    top_ips: Mutex<HashMap<String, u64>>,
    top_blocked_ips: Mutex<HashMap<String, u64>>,
    top_paths: Mutex<HashMap<String, u64>>,
    top_rules: Mutex<HashMap<String, u64>>,
    status_codes: Mutex<HashMap<u16, u64>>,
    verdict_counts: Mutex<HashMap<String, u64>>,
}

struct RingBuffer {
    data: Vec<u32>,
    current_second: u64,
}

impl RingBuffer {
    fn new(size: usize) -> Self {
        Self {
            data: vec![0; size],
            current_second: 0,
        }
    }

    fn increment(&mut self) {
        let now = current_second();
        if now != self.current_second {
            // Clear slots for elapsed seconds
            let elapsed = (now - self.current_second).min(self.data.len() as u64);
            for i in 1..=elapsed {
                let idx = ((self.current_second + i) % self.data.len() as u64) as usize;
                self.data[idx] = 0;
            }
            self.current_second = now;
        }
        let idx = (now % self.data.len() as u64) as usize;
        self.data[idx] += 1;
    }

    fn snapshot(&mut self) -> Vec<u32> {
        let now = current_second();
        if now != self.current_second {
            let elapsed = (now - self.current_second).min(self.data.len() as u64);
            for i in 1..=elapsed {
                let idx = ((self.current_second + i) % self.data.len() as u64) as usize;
                self.data[idx] = 0;
            }
            self.current_second = now;
        }
        let len = self.data.len();
        let start = ((now + 1) % len as u64) as usize;
        let mut out = Vec::with_capacity(len);
        for i in 0..len {
            out.push(self.data[(start + i) % len]);
        }
        out
    }

    fn rate_per_second(&mut self) -> f64 {
        let snap = self.snapshot();
        let sum: u32 = snap.iter().rev().take(10).sum();
        sum as f64 / 10.0
    }
}

fn current_second() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

impl Stats {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            total_requests: AtomicU64::new(0),
            total_blocked: AtomicU64::new(0),
            total_flagged: AtomicU64::new(0),
            total_rate_limited: AtomicU64::new(0),
            active_connections: AtomicU64::new(0),
            traffic_ring: Mutex::new(RingBuffer::new(60)),
            blocks_ring: Mutex::new(RingBuffer::new(60)),
            top_ips: Mutex::new(HashMap::new()),
            top_blocked_ips: Mutex::new(HashMap::new()),
            top_paths: Mutex::new(HashMap::new()),
            top_rules: Mutex::new(HashMap::new()),
            status_codes: Mutex::new(HashMap::new()),
            verdict_counts: Mutex::new(HashMap::new()),
        }
    }

    pub fn record_request(&self, ip: &str, path: &str, verdict: &Verdict, rule: Option<&str>, status: Option<u16>) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut ring) = self.traffic_ring.lock() {
            ring.increment();
        }

        match verdict {
            Verdict::Block => {
                self.total_blocked.fetch_add(1, Ordering::Relaxed);
                if let Ok(mut ring) = self.blocks_ring.lock() {
                    ring.increment();
                }
                if let Ok(mut map) = self.top_blocked_ips.lock() {
                    *map.entry(ip.to_string()).or_insert(0) += 1;
                }
            }
            Verdict::RateLimited => {
                self.total_rate_limited.fetch_add(1, Ordering::Relaxed);
                if let Ok(mut ring) = self.blocks_ring.lock() {
                    ring.increment();
                }
            }
            Verdict::Flagged => {
                self.total_flagged.fetch_add(1, Ordering::Relaxed);
            }
            Verdict::Allow => {}
        }

        if let Ok(mut map) = self.top_ips.lock() {
            *map.entry(ip.to_string()).or_insert(0) += 1;
        }
        if let Ok(mut map) = self.top_paths.lock() {
            *map.entry(path.to_string()).or_insert(0) += 1;
        }
        if let Some(r) = rule {
            if let Ok(mut map) = self.top_rules.lock() {
                *map.entry(r.to_string()).or_insert(0) += 1;
            }
        }
        if let Some(s) = status {
            if let Ok(mut map) = self.status_codes.lock() {
                *map.entry(s).or_insert(0) += 1;
            }
        }
        if let Ok(mut map) = self.verdict_counts.lock() {
            *map.entry(verdict.to_string()).or_insert(0) += 1;
        }
    }

    pub fn snapshot(&self) -> StatsSnapshot {
        let uptime = self.start_time.elapsed();
        let traffic_60s = self.traffic_ring.lock().map(|mut r| r.snapshot()).unwrap_or_default();
        let blocks_60s = self.blocks_ring.lock().map(|mut r| r.snapshot()).unwrap_or_default();
        let rps = self.traffic_ring.lock().map(|mut r| r.rate_per_second()).unwrap_or(0.0);
        let bps = self.blocks_ring.lock().map(|mut r| r.rate_per_second()).unwrap_or(0.0);

        StatsSnapshot {
            uptime_secs: uptime.as_secs(),
            total_requests: self.total_requests.load(Ordering::Relaxed),
            total_blocked: self.total_blocked.load(Ordering::Relaxed),
            total_flagged: self.total_flagged.load(Ordering::Relaxed),
            total_rate_limited: self.total_rate_limited.load(Ordering::Relaxed),
            active_connections: self.active_connections.load(Ordering::Relaxed),
            requests_per_second: rps,
            blocks_per_second: bps,
            traffic_last_60s: traffic_60s,
            blocks_last_60s: blocks_60s,
            top_blocked_ips: top_n(&self.top_blocked_ips, 10),
            top_ips: top_n(&self.top_ips, 10),
            top_paths: top_n(&self.top_paths, 10),
            top_rules: top_n(&self.top_rules, 10),
            status_codes: self.status_codes.lock().map(|m| m.clone()).unwrap_or_default(),
            verdict_counts: self.verdict_counts.lock().map(|m| m.clone()).unwrap_or_default(),
        }
    }

    pub fn conn_open(&self) {
        self.active_connections.fetch_add(1, Ordering::Relaxed);
    }

    pub fn conn_close(&self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }
}

fn top_n(map: &Mutex<HashMap<String, u64>>, n: usize) -> Vec<(String, u64)> {
    let guard = match map.lock() {
        Ok(g) => g,
        Err(_) => return Vec::new(),
    };
    let mut entries: Vec<(String, u64)> = guard.iter().map(|(k, v)| (k.clone(), *v)).collect();
    entries.sort_by(|a, b| b.1.cmp(&a.1));
    entries.truncate(n);
    entries
}

#[derive(Debug, Clone)]
pub struct StatsSnapshot {
    pub uptime_secs: u64,
    pub total_requests: u64,
    pub total_blocked: u64,
    pub total_flagged: u64,
    pub total_rate_limited: u64,
    pub active_connections: u64,
    pub requests_per_second: f64,
    pub blocks_per_second: f64,
    pub traffic_last_60s: Vec<u32>,
    pub blocks_last_60s: Vec<u32>,
    pub top_blocked_ips: Vec<(String, u64)>,
    pub top_ips: Vec<(String, u64)>,
    pub top_paths: Vec<(String, u64)>,
    pub top_rules: Vec<(String, u64)>,
    pub status_codes: HashMap<u16, u64>,
    pub verdict_counts: HashMap<String, u64>,
}

impl StatsSnapshot {
    pub fn format_uptime(&self) -> String {
        let secs = self.uptime_secs;
        let days = secs / 86400;
        let hours = (secs % 86400) / 3600;
        let mins = (secs % 3600) / 60;
        let s = secs % 60;
        if days > 0 {
            format!("{}d {}h {}m", days, hours, mins)
        } else if hours > 0 {
            format!("{}h {}m {}s", hours, mins, s)
        } else if mins > 0 {
            format!("{}m {}s", mins, s)
        } else {
            format!("{}s", s)
        }
    }
}
