use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
    #[serde(default)]
    pub host: String,
    #[serde(default = "default_smtp_port")]
    pub port: u16,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password_env: String,
    #[serde(default, alias = "password", skip_serializing)]
    pub legacy_password: String,
    #[serde(default = "default_tls")]
    pub tls: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EagleEyeConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub smtp: SmtpConfig,
    #[serde(default)]
    pub scan_paths: Vec<String>,
    #[serde(default = "default_interval")]
    pub interval_hours: u32,
    #[serde(default = "default_risk_levels")]
    pub risk_levels: Vec<String>,
    #[serde(default)]
    pub recipients: Vec<String>,
    #[serde(default)]
    pub from: String,
}

#[derive(Debug, Clone)]
pub struct ScanFinding {
    pub project_path: String,
    pub package: String,
    pub version: String,
    pub ecosystem: String,
    pub cve_id: String,
    pub severity: String,
    pub summary: String,
    pub fixed_version: String,
}

impl Default for SmtpConfig {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: default_smtp_port(),
            username: String::new(),
            password_env: String::new(),
            legacy_password: String::new(),
            tls: default_tls(),
        }
    }
}

impl Default for EagleEyeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            smtp: SmtpConfig::default(),
            scan_paths: Vec::new(),
            interval_hours: default_interval(),
            risk_levels: default_risk_levels(),
            recipients: Vec::new(),
            from: String::new(),
        }
    }
}

fn default_smtp_port() -> u16 {
    587
}

fn default_tls() -> bool {
    true
}

fn default_interval() -> u32 {
    24
}

fn default_risk_levels() -> Vec<String> {
    vec!["CRITICAL".into(), "HIGH".into()]
}

pub(super) fn risk_levels_for_choice(choice: &str) -> Vec<String> {
    match choice.trim() {
        "1" => vec!["CRITICAL".into()],
        "3" => vec!["CRITICAL".into(), "HIGH".into(), "MEDIUM".into()],
        "4" => vec![
            "CRITICAL".into(),
            "HIGH".into(),
            "MEDIUM".into(),
            "LOW".into(),
        ],
        "5" => vec![
            "CRITICAL".into(),
            "HIGH".into(),
            "MEDIUM".into(),
            "LOW".into(),
            "INFORMATIONAL".into(),
        ],
        _ => default_risk_levels(),
    }
}
