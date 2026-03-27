use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ── Top-level config ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallConfig {
    #[serde(default = "default_server")]
    pub server: ServerConfig,
    #[serde(default = "default_upstream")]
    pub upstream: UpstreamConfig,
    #[serde(default)]
    pub tls: TlsConfig,
    #[serde(default)]
    pub ip: IpConfig,
    #[serde(default)]
    pub rate_limit: RateLimitConfig,
    #[serde(default)]
    pub waf: WafConfig,
    #[serde(default)]
    pub rules: Vec<RuleConfig>,
    #[serde(default = "default_logging")]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub tui: TuiConfig,
    #[serde(default)]
    pub responses: ResponsesConfig,
}

impl Default for FirewallConfig {
    fn default() -> Self {
        Self {
            server: default_server(),
            upstream: default_upstream(),
            tls: TlsConfig::default(),
            ip: IpConfig::default(),
            rate_limit: RateLimitConfig::default(),
            waf: WafConfig::default(),
            rules: Vec::new(),
            logging: default_logging(),
            tui: TuiConfig::default(),
            responses: ResponsesConfig::default(),
        }
    }
}

// ── Server ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_listen_address")]
    pub listen_address: String,
    #[serde(default = "default_listen_port")]
    pub listen_port: u16,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    #[serde(default = "default_request_timeout")]
    pub request_timeout_seconds: u64,
    #[serde(default = "default_worker_threads")]
    pub worker_threads: usize,
}

fn default_listen_address() -> String { "0.0.0.0".to_string() }
fn default_listen_port() -> u16 { 8080 }
fn default_max_connections() -> u32 { 10000 }
fn default_request_timeout() -> u64 { 30 }
fn default_worker_threads() -> usize { 0 }
fn default_server() -> ServerConfig {
    ServerConfig {
        listen_address: default_listen_address(),
        listen_port: default_listen_port(),
        max_connections: default_max_connections(),
        request_timeout_seconds: default_request_timeout(),
        worker_threads: default_worker_threads(),
    }
}

// ── Upstream ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamConfig {
    #[serde(default = "default_upstream_address")]
    pub address: String,
    #[serde(default = "default_upstream_port")]
    pub port: u16,
    #[serde(default = "default_health_check_path")]
    pub health_check_path: String,
    #[serde(default = "default_health_interval")]
    pub health_check_interval_seconds: u64,
    #[serde(default = "default_forward_timeout")]
    pub forward_timeout_seconds: u64,
}

fn default_upstream_address() -> String { "127.0.0.1".to_string() }
fn default_upstream_port() -> u16 { 3000 }
fn default_health_check_path() -> String { "/health".to_string() }
fn default_health_interval() -> u64 { 10 }
fn default_forward_timeout() -> u64 { 30 }
fn default_upstream() -> UpstreamConfig {
    UpstreamConfig {
        address: default_upstream_address(),
        port: default_upstream_port(),
        health_check_path: default_health_check_path(),
        health_check_interval_seconds: default_health_interval(),
        forward_timeout_seconds: default_forward_timeout(),
    }
}

// ── TLS ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TlsConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub cert_path: Option<String>,
    #[serde(default)]
    pub key_path: Option<String>,
}

// ── IP Filtering ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpConfig {
    #[serde(default = "default_ip_mode")]
    pub mode: String, // "blocklist" | "allowlist" | "disabled"
    #[serde(default)]
    pub blocklist_file: Option<String>,
    #[serde(default)]
    pub allowlist_file: Option<String>,
    #[serde(default)]
    pub blocklist: Vec<String>,
    #[serde(default)]
    pub allowlist: Vec<String>,
    #[serde(default)]
    pub auto_reputation: AutoReputationConfig,
}

fn default_ip_mode() -> String { "blocklist".to_string() }

impl Default for IpConfig {
    fn default() -> Self {
        Self {
            mode: default_ip_mode(),
            blocklist_file: None,
            allowlist_file: None,
            blocklist: Vec::new(),
            allowlist: Vec::new(),
            auto_reputation: AutoReputationConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoReputationConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_reputation_threshold")]
    pub threshold: u32,
    #[serde(default = "default_reputation_window")]
    pub window_minutes: u32,
    #[serde(default = "default_ban_duration")]
    pub ban_duration_minutes: u32,
}

fn default_true() -> bool { true }
fn default_reputation_threshold() -> u32 { 50 }
fn default_reputation_window() -> u32 { 60 }
fn default_ban_duration() -> u32 { 1440 }

impl Default for AutoReputationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            threshold: default_reputation_threshold(),
            window_minutes: default_reputation_window(),
            ban_duration_minutes: default_ban_duration(),
        }
    }
}

// ── Rate Limiting ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub global: Option<GlobalRateConfig>,
    #[serde(default)]
    pub per_ip: Option<PerIpRateConfig>,
    #[serde(default)]
    pub per_path: HashMap<String, PathRateConfig>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            global: Some(GlobalRateConfig { requests_per_second: 1000 }),
            per_ip: Some(PerIpRateConfig { requests: 100, window_seconds: 60 }),
            per_path: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalRateConfig {
    #[serde(default = "default_global_rps")]
    pub requests_per_second: u32,
}

fn default_global_rps() -> u32 { 1000 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerIpRateConfig {
    #[serde(default = "default_per_ip_requests")]
    pub requests: u32,
    #[serde(default = "default_per_ip_window")]
    pub window_seconds: u32,
}

fn default_per_ip_requests() -> u32 { 100 }
fn default_per_ip_window() -> u32 { 60 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathRateConfig {
    pub requests: u32,
    pub window_seconds: u32,
}

// ── WAF ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WafConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub sqli_protection: bool,
    #[serde(default = "default_true")]
    pub xss_protection: bool,
    #[serde(default = "default_true")]
    pub path_traversal_protection: bool,
    #[serde(default = "default_true")]
    pub command_injection_protection: bool,
    #[serde(default = "default_true")]
    pub header_injection_protection: bool,
    #[serde(default = "default_max_url")]
    pub max_url_length: usize,
    #[serde(default = "default_max_header")]
    pub max_header_size: usize,
    #[serde(default = "default_max_body")]
    pub max_body_size: usize,
    #[serde(default = "default_allowed_methods")]
    pub allowed_methods: Vec<String>,
    #[serde(default = "default_blocked_extensions")]
    pub blocked_extensions: Vec<String>,
    #[serde(default = "default_blocked_paths")]
    pub blocked_paths: Vec<String>,
    #[serde(default = "default_true")]
    pub block_empty_user_agent: bool,
    #[serde(default = "default_blocked_user_agents")]
    pub blocked_user_agents: Vec<String>,
}

fn default_max_url() -> usize { 4096 }
fn default_max_header() -> usize { 8192 }
fn default_max_body() -> usize { 10_485_760 }
fn default_allowed_methods() -> Vec<String> {
    vec!["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS", "HEAD"]
        .into_iter().map(String::from).collect()
}
fn default_blocked_extensions() -> Vec<String> {
    vec![".env", ".git", ".sql", ".bak", ".log", ".ini", ".cfg"]
        .into_iter().map(String::from).collect()
}
fn default_blocked_paths() -> Vec<String> {
    vec![
        "/wp-admin", "/wp-login.php", "/.git/", "/.env",
        "/phpinfo.php", "/server-status", "/actuator",
    ].into_iter().map(String::from).collect()
}
fn default_blocked_user_agents() -> Vec<String> {
    vec!["sqlmap", "nikto", "nmap", "dirbuster", "gobuster", "masscan", "ZmEu"]
        .into_iter().map(String::from).collect()
}

impl Default for WafConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sqli_protection: true,
            xss_protection: true,
            path_traversal_protection: true,
            command_injection_protection: true,
            header_injection_protection: true,
            max_url_length: default_max_url(),
            max_header_size: default_max_header(),
            max_body_size: default_max_body(),
            allowed_methods: default_allowed_methods(),
            blocked_extensions: default_blocked_extensions(),
            blocked_paths: default_blocked_paths(),
            block_empty_user_agent: true,
            blocked_user_agents: default_blocked_user_agents(),
        }
    }
}

// ── Custom Rules ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConfig {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_priority")]
    pub priority: u32,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub condition: RuleConditionConfig,
    #[serde(default)]
    pub action: RuleActionConfig,
}

fn default_priority() -> u32 { 100 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuleConditionConfig {
    #[serde(default)]
    pub ip: Vec<String>,
    #[serde(default)]
    pub ip_file: Option<String>,
    #[serde(default)]
    pub path_prefix: Option<String>,
    #[serde(default)]
    pub path_regex: Option<String>,
    #[serde(default)]
    pub path_exact: Option<String>,
    #[serde(default)]
    pub method: Vec<String>,
    #[serde(default)]
    pub header: Option<HashMap<String, String>>,
    #[serde(default)]
    pub user_agent_regex: Option<String>,
    #[serde(default)]
    pub body_contains: Option<String>,
    #[serde(default)]
    pub query_param: Option<HashMap<String, String>>,
    #[serde(default)]
    pub content_type: Vec<String>,
    #[serde(default)]
    pub request_size_gt: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleActionConfig {
    #[serde(default = "default_action_type", rename = "type")]
    pub action_type: String, // "block" | "allow" | "flag" | "rate_limit"
    #[serde(default = "default_block_status")]
    pub status: u16,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub tag: Option<String>,
    // for rate_limit action
    #[serde(default)]
    pub requests: Option<u32>,
    #[serde(default)]
    pub window_seconds: Option<u32>,
}

fn default_action_type() -> String { "block".to_string() }
fn default_block_status() -> u16 { 403 }

impl Default for RuleActionConfig {
    fn default() -> Self {
        Self {
            action_type: default_action_type(),
            status: default_block_status(),
            message: "Access denied".to_string(),
            tag: None,
            requests: None,
            window_seconds: None,
        }
    }
}

// ── Logging ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_access_log")]
    pub access_log: String,
    #[serde(default = "default_blocked_log")]
    pub blocked_log: String,
    #[serde(default = "default_alert_log")]
    pub alert_log: String,
    #[serde(default = "default_error_log")]
    pub error_log: String,
    #[serde(default = "default_max_file_size")]
    pub max_file_size_mb: u32,
    #[serde(default = "default_max_files")]
    pub max_files: u32,
    #[serde(default = "default_true")]
    pub log_request_headers: bool,
    #[serde(default)]
    pub log_request_body: bool,
}

fn default_access_log() -> String { "logs/access.jsonl".to_string() }
fn default_blocked_log() -> String { "logs/blocked.jsonl".to_string() }
fn default_alert_log() -> String { "logs/alerts.jsonl".to_string() }
fn default_error_log() -> String { "logs/error.log".to_string() }
fn default_max_file_size() -> u32 { 100 }
fn default_max_files() -> u32 { 10 }
fn default_logging() -> LoggingConfig {
    LoggingConfig {
        access_log: default_access_log(),
        blocked_log: default_blocked_log(),
        alert_log: default_alert_log(),
        error_log: default_error_log(),
        max_file_size_mb: default_max_file_size(),
        max_files: default_max_files(),
        log_request_headers: true,
        log_request_body: false,
    }
}

// ── TUI ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuiConfig {
    #[serde(default = "default_refresh_rate")]
    pub refresh_rate_fps: u32,
    #[serde(default = "default_view")]
    pub default_view: String,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_max_events")]
    pub max_events_in_memory: usize,
}

fn default_refresh_rate() -> u32 { 10 }
fn default_view() -> String { "dashboard".to_string() }
fn default_theme() -> String { "dark".to_string() }
fn default_max_events() -> usize { 10000 }

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            refresh_rate_fps: default_refresh_rate(),
            default_view: default_view(),
            theme: default_theme(),
            max_events_in_memory: default_max_events(),
        }
    }
}

// ── Responses ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResponsesConfig {
    #[serde(default)]
    pub block_page_html: Option<String>,
    #[serde(default)]
    pub block_page_json: Option<String>,
    #[serde(default)]
    pub ratelimit_page_html: Option<String>,
}

// ── Load / Save ─────────────────────────────────────────────────────────────

pub fn config_dir() -> PathBuf {
    #[cfg(windows)]
    let home = std::env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string());
    #[cfg(not(windows))]
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".infynon")
}

pub fn default_config_path() -> PathBuf {
    // Check local infynon.toml first, then ~/.infynon/infynon.toml
    let local = PathBuf::from("infynon.toml");
    if local.exists() {
        return local;
    }
    config_dir().join("infynon.toml")
}

pub fn load_firewall_config(path: Option<&str>) -> Result<FirewallConfig, String> {
    let config_path = match path {
        Some(p) => PathBuf::from(p),
        None => default_config_path(),
    };

    if !config_path.exists() {
        return Ok(FirewallConfig::default());
    }

    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config {}: {}", config_path.display(), e))?;

    toml::from_str(&content)
        .map_err(|e| format!("Failed to parse config {}: {}", config_path.display(), e))
}

pub fn save_firewall_config(config: &FirewallConfig, path: Option<&str>) -> Result<(), String> {
    let config_path = match path {
        Some(p) => PathBuf::from(p),
        None => default_config_path(),
    };

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let content = toml::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    std::fs::write(&config_path, content)
        .map_err(|e| format!("Failed to write config to {}: {}", config_path.display(), e))
}

pub fn init_config(listen_port: u16, upstream_addr: &str, upstream_port: u16) -> FirewallConfig {
    let mut config = FirewallConfig::default();
    config.server.listen_port = listen_port;
    config.upstream.address = upstream_addr.to_string();
    config.upstream.port = upstream_port;
    config
}

pub fn load_ip_list(path: &Path) -> Vec<String> {
    match std::fs::read_to_string(path) {
        Ok(content) => content
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(String::from)
            .collect(),
        Err(_) => Vec::new(),
    }
}
