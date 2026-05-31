#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    Markdown,
    Pdf,
    Both,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FixLevel {
    Critical,
    High,
    Medium,
    Low,
    Informational,
    All,
}

impl FixLevel {
    pub fn from_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "critical" => Self::Critical,
            "high" => Self::High,
            "medium" => Self::Medium,
            "low" => Self::Low,
            "informational" => Self::Informational,
            _ => Self::All,
        }
    }

    pub fn matches(&self, severity: &str) -> bool {
        match self {
            Self::All | Self::Informational => true,
            Self::Critical => severity == "CRITICAL",
            Self::High => matches!(severity, "CRITICAL" | "HIGH"),
            Self::Medium => matches!(severity, "CRITICAL" | "HIGH" | "MEDIUM"),
            Self::Low => matches!(severity, "CRITICAL" | "HIGH" | "MEDIUM" | "LOW"),
        }
    }
}

pub struct VulnHit {
    pub package: String,
    pub cve_id: String,
    pub severity: &'static str,
    pub summary: String,
    pub fixed_version: Option<String>,
    pub upgrade_cmd: Option<String>,
    pub fix_is_clean: bool,
}
