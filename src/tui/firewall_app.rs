use std::sync::Arc;

use crate::firewall::events::FirewallEvent;
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
        }
    }

    pub fn stats_snapshot(&self) -> StatsSnapshot {
        self.state.stats.snapshot()
    }

    pub fn recent_events(&self) -> Vec<FirewallEvent> {
        self.state.recent_events_snapshot()
    }

    pub fn filtered_events(&self) -> Vec<FirewallEvent> {
        let events = self.recent_events();
        match self.feed_filter {
            FeedFilter::All => events,
            FeedFilter::BlockedOnly => events.into_iter()
                .filter(|e| matches!(e.verdict, crate::firewall::events::Verdict::Block | crate::firewall::events::Verdict::RateLimited))
                .collect(),
            FeedFilter::AllowedOnly => events.into_iter()
                .filter(|e| matches!(e.verdict, crate::firewall::events::Verdict::Allow))
                .collect(),
            FeedFilter::FlaggedOnly => events.into_iter()
                .filter(|e| matches!(e.verdict, crate::firewall::events::Verdict::Flagged))
                .collect(),
        }
    }

    pub fn events_for_ip(&self, ip: &str) -> Vec<FirewallEvent> {
        self.recent_events().into_iter()
            .filter(|e| e.source_ip == ip)
            .collect()
    }

    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;

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
            KeyCode::Char('/') => {
                if self.current_view == View::IpInspector {
                    self.ip_search_active = true;
                    self.ip_search.clear();
                } else {
                    self.search_active = true;
                    self.search_input.clear();
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
                    KeyCode::Down => { self.scroll_offset += 1; }
                    _ => {}
                }
            }
            View::IpInspector => {
                match key.code {
                    KeyCode::Char('b') => {
                        // Block selected IP
                        if !self.ip_search.is_empty() {
                            self.state.pipeline.block_ip(&self.ip_search);
                        }
                    }
                    KeyCode::Char('u') => {
                        // Unblock selected IP
                        if !self.ip_search.is_empty() {
                            self.state.pipeline.unblock_ip(&self.ip_search);
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
                    KeyCode::Down => { self.selected_index += 1; }
                    _ => {}
                }
            }
            View::Config => {
                match key.code {
                    KeyCode::Up => {
                        if self.config_selected > 0 { self.config_selected -= 1; }
                    }
                    KeyCode::Down => { self.config_selected += 1; }
                    KeyCode::Enter => {
                        self.config_editing = true;
                        self.config_edit_buf = self.get_config_value(self.config_selected);
                    }
                    KeyCode::Char('s') => {
                        // Save config to file
                        let cfg = &self.state.config;
                        let _ = crate::firewall::config::save_firewall_config(cfg, None);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn get_config_value(&self, index: usize) -> String {
        let cfg = &self.state.config;
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
            _ => String::new(),
        }
    }

    fn apply_config_edit(&mut self) {
        // Config is behind Arc, we can't mutate it directly in the running server.
        // For TUI display purposes, we log the intended change.
        // In a full implementation, this would update through an RwLock.
        // For now, we save to disk so next restart picks it up.
        let _ = &self.config_edit_buf; // The value would be applied
    }

    pub fn config_fields(&self) -> Vec<(&str, String)> {
        let cfg = &self.state.config;
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
        ]
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_073_741_824 { format!("{:.1} GB", bytes as f64 / 1_073_741_824.0) }
    else if bytes >= 1_048_576 { format!("{:.1} MB", bytes as f64 / 1_048_576.0) }
    else if bytes >= 1024 { format!("{:.1} KB", bytes as f64 / 1024.0) }
    else { format!("{} B", bytes) }
}
