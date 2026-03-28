use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::firewall::events::{FirewallEvent, Verdict};
use crate::firewall::server::SharedState;
use crate::firewall::stats::StatsSnapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Dashboard,
    LiveFeed,
    Blocked,
    IpInspector,
    Rules,
    Stats,
    Config,
}

impl View {
    pub fn label(&self) -> &str {
        match self {
            View::Dashboard => "Dashboard",
            View::LiveFeed => "Live Feed",
            View::Blocked => "Blocked",
            View::IpInspector => "IP Inspector",
            View::Rules => "Rules",
            View::Stats => "Stats",
            View::Config => "Config",
        }
    }

    pub fn key(&self) -> char {
        match self {
            View::Dashboard => '1',
            View::LiveFeed => '2',
            View::Blocked => '3',
            View::IpInspector => '4',
            View::Rules => '5',
            View::Stats => '6',
            View::Config => '7',
        }
    }

    pub fn all() -> &'static [View] {
        &[
            View::Dashboard,
            View::LiveFeed,
            View::Blocked,
            View::IpInspector,
            View::Rules,
            View::Stats,
            View::Config,
        ]
    }
}

pub struct App {
    pub state: Arc<SharedState>,
    pub current_view: View,
    pub should_quit: bool,
    pub paused: bool,
    pub scroll_offset: usize,
    pub selected_index: usize,
    pub feed_filter: FeedFilter,
    pub ip_search: String,
    pub ip_search_active: bool,
    pub search_input: String,
    pub search_active: bool,
    // Config editing
    pub config_selected: usize,
    pub config_editing: bool,
    pub config_edit_buf: String,
    // Notification
    pub notification: Option<(String, std::time::Instant)>,
    // Help overlay
    pub show_help: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FeedFilter {
    All,
    BlockedOnly,
    AllowedOnly,
    FlaggedOnly,
}

impl FeedFilter {
    pub fn label(&self) -> &str {
        match self {
            FeedFilter::All => "ALL",
            FeedFilter::BlockedOnly => "BLOCKED",
            FeedFilter::AllowedOnly => "ALLOWED",
            FeedFilter::FlaggedOnly => "FLAGGED",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            FeedFilter::All => FeedFilter::BlockedOnly,
            FeedFilter::BlockedOnly => FeedFilter::AllowedOnly,
            FeedFilter::AllowedOnly => FeedFilter::FlaggedOnly,
            FeedFilter::FlaggedOnly => FeedFilter::All,
        }
    }
}

impl App {
    pub fn new(state: Arc<SharedState>) -> Self {
        Self {
            state,
            current_view: View::Dashboard,
            should_quit: false,
            paused: false,
            scroll_offset: 0,
            selected_index: 0,
            feed_filter: FeedFilter::All,
            ip_search: String::new(),
            ip_search_active: false,
            search_input: String::new(),
            search_active: false,
            config_selected: 0,
            config_editing: false,
            config_edit_buf: String::new(),
            notification: None,
            show_help: false,
        }
    }

    pub fn notify(&mut self, msg: &str) {
        self.notification = Some((msg.to_string(), std::time::Instant::now()));
    }

    pub fn active_notification(&self) -> Option<&str> {
        if let Some((ref msg, when)) = self.notification {
            if when.elapsed().as_secs() < 4 {
                return Some(msg);
            }
        }
        None
    }

    pub fn stats_snapshot(&self) -> StatsSnapshot {
        self.state.stats.snapshot()
    }

    pub fn recent_events(&self) -> Vec<FirewallEvent> {
        self.state.recent_events_snapshot()
    }

    pub fn filtered_events(&self) -> Vec<FirewallEvent> {
        let events = self.recent_events();
        let filtered = match self.feed_filter {
            FeedFilter::All => events,
            FeedFilter::BlockedOnly => events.into_iter()
                .filter(|e| matches!(e.verdict, Verdict::Block | Verdict::RateLimited))
                .collect(),
            FeedFilter::AllowedOnly => events.into_iter()
                .filter(|e| matches!(e.verdict, Verdict::Allow))
                .collect(),
            FeedFilter::FlaggedOnly => events.into_iter()
                .filter(|e| matches!(e.verdict, Verdict::Flagged))
                .collect(),
        };

        // Apply search filter if active
        if !self.search_input.is_empty() {
            let query = self.search_input.to_lowercase();
            filtered.into_iter()
                .filter(|e| {
                    e.source_ip.to_lowercase().contains(&query)
                        || e.path.to_lowercase().contains(&query)
                        || e.method.to_lowercase().contains(&query)
                        || e.blocked_by_rule.as_deref().unwrap_or("").to_lowercase().contains(&query)
                })
                .collect()
        } else {
            filtered
        }
    }

    /// Count filtered events without cloning all of them.
    pub fn count_filtered_events(&self) -> usize {
        let filter = self.feed_filter;
        let search = self.search_input.to_lowercase();
        self.state.count_events(|e| {
            let matches_filter = match filter {
                FeedFilter::All => true,
                FeedFilter::BlockedOnly => matches!(e.verdict, Verdict::Block | Verdict::RateLimited),
                FeedFilter::AllowedOnly => matches!(e.verdict, Verdict::Allow),
                FeedFilter::FlaggedOnly => matches!(e.verdict, Verdict::Flagged),
            };
            if !matches_filter { return false; }
            if search.is_empty() { return true; }
            e.source_ip.to_lowercase().contains(&search)
                || e.path.to_lowercase().contains(&search)
                || e.method.to_lowercase().contains(&search)
                || e.blocked_by_rule.as_deref().unwrap_or("").to_lowercase().contains(&search)
        })
    }

    pub fn events_for_ip(&self, ip: &str) -> Vec<FirewallEvent> {
        self.recent_events().into_iter()
            .filter(|e| e.source_ip == ip)
            .collect()
    }

    pub fn is_maintenance(&self) -> bool {
        self.state.maintenance_mode.load(Ordering::Relaxed)
    }

    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;

        // Help overlay
        if self.show_help {
            self.show_help = false;
            return;
        }

        // If in search/edit mode, handle text input
        if self.search_active {
            match key.code {
                KeyCode::Esc => { self.search_active = false; }
                KeyCode::Enter => { self.search_active = false; }
                KeyCode::Backspace => { self.search_input.pop(); }
                KeyCode::Char(c) => { self.search_input.push(c); }
                _ => {}
            }
            return;
        }

        if self.ip_search_active {
            match key.code {
                KeyCode::Esc => { self.ip_search_active = false; }
                KeyCode::Enter => { self.ip_search_active = false; }
                KeyCode::Backspace => { self.ip_search.pop(); }
                KeyCode::Char(c) => { self.ip_search.push(c); }
                _ => {}
            }
            return;
        }

        if self.config_editing {
            match key.code {
                KeyCode::Esc => { self.config_editing = false; }
                KeyCode::Enter => {
                    self.apply_config_edit();
                    self.config_editing = false;
                }
                KeyCode::Backspace => { self.config_edit_buf.pop(); }
                KeyCode::Char(c) => { self.config_edit_buf.push(c); }
                _ => {}
            }
            return;
        }

        // Global keys
        match key.code {
            KeyCode::Char('q') => { self.should_quit = true; }
            KeyCode::Char('1') => { self.current_view = View::Dashboard; self.scroll_offset = 0; }
            KeyCode::Char('2') => { self.current_view = View::LiveFeed; self.scroll_offset = 0; }
            KeyCode::Char('3') => { self.current_view = View::Blocked; self.scroll_offset = 0; }
            KeyCode::Char('4') => { self.current_view = View::IpInspector; self.scroll_offset = 0; }
            KeyCode::Char('5') => { self.current_view = View::Rules; self.scroll_offset = 0; }
            KeyCode::Char('6') => { self.current_view = View::Stats; self.scroll_offset = 0; }
            KeyCode::Char('7') => { self.current_view = View::Config; self.scroll_offset = 0; }
            KeyCode::Char('?') => { self.show_help = true; }
            KeyCode::Char('/') => {
                if self.current_view == View::IpInspector {
                    self.ip_search_active = true;
                    self.ip_search.clear();
                } else {
                    self.search_active = true;
                    self.search_input.clear();
                }
            }
            KeyCode::Char('r') => {
                // Force config reload
                self.reload_config();
            }
            KeyCode::Char('m') => {
                // Toggle maintenance mode
                let current = self.state.maintenance_mode.load(Ordering::Relaxed);
                self.state.maintenance_mode.store(!current, Ordering::Relaxed);
                if !current {
                    self.notify("Maintenance mode ENABLED");
                } else {
                    self.notify("Maintenance mode DISABLED");
                }
            }
            _ => {}
        }

        // View-specific keys
        match self.current_view {
            View::LiveFeed | View::Blocked => {
                match key.code {
                    KeyCode::Char('p') => { self.paused = !self.paused; }
                    KeyCode::Char('f') => { self.feed_filter = self.feed_filter.next(); }
                    KeyCode::Up => {
                        if self.scroll_offset > 0 { self.scroll_offset -= 1; }
                    }
                    KeyCode::Down => {
                        let event_count = self.count_filtered_events();
                        if self.scroll_offset < event_count.saturating_sub(1) {
                            self.scroll_offset += 1;
                        }
                    }
                    KeyCode::Home => { self.scroll_offset = 0; }
                    _ => {}
                }
            }
            View::IpInspector => {
                match key.code {
                    KeyCode::Char('b') => {
                        if !self.ip_search.is_empty() {
                            if let Ok(pipeline) = self.state.pipeline.read() {
                                pipeline.block_ip(&self.ip_search);
                            }
                            self.notify(&format!("Blocked IP: {}", self.ip_search));
                        }
                    }
                    KeyCode::Char('u') => {
                        if !self.ip_search.is_empty() {
                            if let Ok(pipeline) = self.state.pipeline.read() {
                                pipeline.unblock_ip(&self.ip_search);
                            }
                            self.notify(&format!("Unblocked IP: {}", self.ip_search));
                        }
                    }
                    KeyCode::Up => {
                        if self.scroll_offset > 0 { self.scroll_offset -= 1; }
                    }
                    KeyCode::Down => {
                        let ip = self.ip_search.clone();
                        let event_count = self.state.count_events(|e| e.source_ip == ip);
                        if self.scroll_offset < event_count.saturating_sub(1) {
                            self.scroll_offset += 1;
                        }
                    }
                    _ => {}
                }
            }
            View::Rules => {
                match key.code {
                    KeyCode::Up => {
                        if self.selected_index > 0 { self.selected_index -= 1; }
                    }
                    KeyCode::Down => {
                        let max = self.state.pipeline.read()
                            .map(|p| p.rule_stats().len().saturating_sub(1))
                            .unwrap_or(0);
                        if self.selected_index < max { self.selected_index += 1; }
                    }
                    _ => {}
                }
            }
            View::Config => {
                match key.code {
                    KeyCode::Up => {
                        if self.config_selected > 0 { self.config_selected -= 1; }
                    }
                    KeyCode::Down => {
                        let max = self.config_fields().len().saturating_sub(1);
                        if self.config_selected < max { self.config_selected += 1; }
                    }
                    KeyCode::Enter => {
                        self.config_editing = true;
                        self.config_edit_buf = self.get_config_value(self.config_selected);
                    }
                    KeyCode::Char('s') => {
                        self.save_config();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn reload_config(&mut self) {
        let path = self.state.config_path.clone();
        match crate::firewall::config::load_firewall_config(path.as_deref()) {
            Ok(new_cfg) => {
                let maint = new_cfg.server.maintenance_mode;
                if let Ok(mut cfg) = self.state.config.write() {
                    *cfg = new_cfg;
                }
                self.state.maintenance_mode.store(maint, Ordering::Relaxed);
                self.state.rebuild_pipeline();
                self.notify("Config reloaded + pipeline rebuilt");
            }
            Err(e) => {
                self.notify(&format!("Config reload failed: {}", e));
            }
        }
    }

    fn save_config(&mut self) {
        let cfg_clone = match self.state.config.read() {
            Ok(c) => c.clone(),
            Err(_) => {
                self.notification = Some(("Failed to read config".to_string(), std::time::Instant::now()));
                return;
            }
        };
        let path = self.state.config_path.as_deref();
        match crate::firewall::config::save_firewall_config(&cfg_clone, path) {
            Ok(()) => self.notify("Config saved to file"),
            Err(e) => self.notify(&format!("Save failed: {}", e)),
        }
    }

    fn get_config_value(&self, index: usize) -> String {
        let cfg = match self.state.config.read() {
            Ok(c) => c,
            Err(_) => return String::new(),
        };
        match index {
            0 => cfg.server.listen_address.clone(),
            1 => cfg.server.listen_port.to_string(),
            2 => cfg.upstream.address.clone(),
            3 => cfg.upstream.port.to_string(),
            4 => cfg.waf.enabled.to_string(),
            5 => cfg.rate_limit.enabled.to_string(),
            6 => cfg.ip.mode.clone(),
            7 => cfg.waf.block_empty_user_agent.to_string(),
            8 => cfg.waf.max_body_size.to_string(),
            9 => cfg.waf.max_url_length.to_string(),
            10 => cfg.tui.refresh_rate_fps.to_string(),
            11 => cfg.server.maintenance_mode.to_string(),
            12 => cfg.waf.sqli_protection.to_string(),
            13 => cfg.waf.xss_protection.to_string(),
            14 => cfg.waf.path_traversal_protection.to_string(),
            15 => cfg.waf.command_injection_protection.to_string(),
            16 => cfg.waf.header_injection_protection.to_string(),
            17 => cfg.ip.auto_reputation.enabled.to_string(),
            18 => cfg.ip.auto_reputation.threshold.to_string(),
            19 => cfg.ip.auto_reputation.ban_duration_minutes.to_string(),
            _ => String::new(),
        }
    }

    fn apply_config_edit(&mut self) {
        let val = self.config_edit_buf.trim().to_string();
        if val.is_empty() {
            self.notify("Empty value — edit cancelled");
            return;
        }

        // Validate BEFORE acquiring the write lock (avoids borrow conflicts with self.notify)
        let idx = self.config_selected;
        let is_bool_field = matches!(idx, 4 | 5 | 7 | 11 | 12 | 13 | 14 | 15 | 16 | 17);
        let is_num_field = matches!(idx, 1 | 3 | 8 | 9 | 10 | 18 | 19);
        let is_mode_field = idx == 6;

        if is_bool_field {
            if val.parse::<bool>().is_err() {
                self.notify("Enter true or false");
                return;
            }
        }
        if is_num_field {
            if val.parse::<u64>().is_err() {
                self.notify("Invalid number");
                return;
            }
        }
        if is_mode_field && val != "blocklist" && val != "allowlist" && val != "disabled" {
            self.notify("Enter: blocklist, allowlist, or disabled");
            return;
        }

        // Now acquire lock and apply — all values are pre-validated
        if let Ok(mut cfg) = self.state.config.write() {
            match idx {
                0 => { cfg.server.listen_address = val; }
                1 => { cfg.server.listen_port = val.parse().unwrap(); }
                2 => { cfg.upstream.address = val; }
                3 => { cfg.upstream.port = val.parse().unwrap(); }
                4 => { cfg.waf.enabled = val.parse().unwrap(); }
                5 => { cfg.rate_limit.enabled = val.parse().unwrap(); }
                6 => { cfg.ip.mode = val; }
                7 => { cfg.waf.block_empty_user_agent = val.parse().unwrap(); }
                8 => { cfg.waf.max_body_size = val.parse().unwrap(); }
                9 => { cfg.waf.max_url_length = val.parse().unwrap(); }
                10 => { cfg.tui.refresh_rate_fps = val.parse().unwrap(); }
                11 => {
                    let maint: bool = val.parse().unwrap();
                    cfg.server.maintenance_mode = maint;
                    self.state.maintenance_mode.store(maint, Ordering::Relaxed);
                }
                12 => { cfg.waf.sqli_protection = val.parse().unwrap(); }
                13 => { cfg.waf.xss_protection = val.parse().unwrap(); }
                14 => { cfg.waf.path_traversal_protection = val.parse().unwrap(); }
                15 => { cfg.waf.command_injection_protection = val.parse().unwrap(); }
                16 => { cfg.waf.header_injection_protection = val.parse().unwrap(); }
                17 => { cfg.ip.auto_reputation.enabled = val.parse().unwrap(); }
                18 => { cfg.ip.auto_reputation.threshold = val.parse().unwrap(); }
                19 => { cfg.ip.auto_reputation.ban_duration_minutes = val.parse().unwrap(); }
                _ => {}
            }
        }
        // Rebuild pipeline so changes take effect immediately
        self.state.rebuild_pipeline();
        self.notify("Config updated (save with 's' to persist)");
    }

    pub fn config_fields(&self) -> Vec<(&str, String)> {
        let cfg = match self.state.config.read() {
            Ok(c) => c,
            Err(_) => return vec![],
        };
        vec![
            ("Listen Address", cfg.server.listen_address.clone()),
            ("Listen Port", cfg.server.listen_port.to_string()),
            ("Upstream Address", cfg.upstream.address.clone()),
            ("Upstream Port", cfg.upstream.port.to_string()),
            ("WAF Enabled", cfg.waf.enabled.to_string()),
            ("Rate Limit Enabled", cfg.rate_limit.enabled.to_string()),
            ("IP Filter Mode", cfg.ip.mode.clone()),
            ("Block Empty UA", cfg.waf.block_empty_user_agent.to_string()),
            ("Max Body Size", format_bytes(cfg.waf.max_body_size as u64)),
            ("Max URL Length", cfg.waf.max_url_length.to_string()),
            ("TUI Refresh FPS", cfg.tui.refresh_rate_fps.to_string()),
            ("Maintenance Mode", cfg.server.maintenance_mode.to_string()),
            ("SQLi Protection", cfg.waf.sqli_protection.to_string()),
            ("XSS Protection", cfg.waf.xss_protection.to_string()),
            ("Path Traversal", cfg.waf.path_traversal_protection.to_string()),
            ("Cmd Injection", cfg.waf.command_injection_protection.to_string()),
            ("Header Injection", cfg.waf.header_injection_protection.to_string()),
            ("Auto Reputation", cfg.ip.auto_reputation.enabled.to_string()),
            ("Rep. Threshold", cfg.ip.auto_reputation.threshold.to_string()),
            ("Ban Duration (min)", cfg.ip.auto_reputation.ban_duration_minutes.to_string()),
        ]
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_073_741_824 { format!("{:.1} GB", bytes as f64 / 1_073_741_824.0) }
    else if bytes >= 1_048_576 { format!("{:.1} MB", bytes as f64 / 1_048_576.0) }
    else if bytes >= 1024 { format!("{:.1} KB", bytes as f64 / 1024.0) }
    else { format!("{} B", bytes) }
}
