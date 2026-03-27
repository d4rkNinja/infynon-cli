use regex::RegexSet;
use crate::firewall::config::WafConfig;

pub struct WafEngine {
    enabled: bool,
    sqli_patterns: Option<RegexSet>,
    xss_patterns: Option<RegexSet>,
    path_traversal_patterns: Option<RegexSet>,
    cmd_injection_patterns: Option<RegexSet>,
    header_injection_patterns: Option<RegexSet>,
    max_url_length: usize,
    max_body_size: usize,
    allowed_methods: Vec<String>,
    blocked_extensions: Vec<String>,
    blocked_paths: Vec<String>,
    block_empty_user_agent: bool,
    blocked_user_agents_lower: Vec<String>,
}

impl WafEngine {
    pub fn new(config: &WafConfig) -> Self {
        let sqli_patterns = if config.sqli_protection {
            RegexSet::new(&[
                r"(?i)(\b(union|select|insert|update|delete|drop|alter|create|exec|execute)\b\s)",
                r"(?i)('\s*(or|and)\s*')",
                r"(?i)('\s*(or|and)\s+\d+\s*=\s*\d+)",
                r"(?i)(--\s*$|;--\s*$|/\*.*\*/)",
                r"(?i)('\s*;\s*(drop|delete|update|insert))",
                r"(?i)(\bwaitfor\b\s+\bdelay\b)",
                r"(?i)(\bsleep\s*\()",
                r"(?i)(\bbenchmark\s*\()",
                r"(?i)(0x[0-9a-f]+)",
                r"(?i)(\bchar\s*\()",
                r"(?i)(\binformation_schema\b)",
                r"(?i)(\bload_file\s*\()",
                r"(?i)(\binto\s+(out|dump)file\b)",
            ]).ok()
        } else {
            None
        };

        let xss_patterns = if config.xss_protection {
            RegexSet::new(&[
                r"(?i)(<script[^>]*>)",
                r"(?i)(javascript\s*:)",
                r"(?i)(on(load|error|click|mouseover|focus|blur|submit|change|input)\s*=)",
                r"(?i)(<iframe[^>]*>)",
                r"(?i)(<object[^>]*>)",
                r"(?i)(<embed[^>]*>)",
                r"(?i)(<img[^>]+\bonerror\b)",
                r"(?i)(document\.(cookie|write|location))",
                r"(?i)(eval\s*\()",
                r"(?i)(<svg[^>]+\bonload\b)",
                r"(?i)(expression\s*\()",
                r"(?i)(alert\s*\()",
            ]).ok()
        } else {
            None
        };

        let path_traversal_patterns = if config.path_traversal_protection {
            RegexSet::new(&[
                r"\.\./",
                r"\.\.\\",
                r"%2e%2e[/\\]",
                r"%252e%252e",
                r"\.%00",
                r"%00",
                r"/etc/passwd",
                r"/etc/shadow",
                r"\\windows\\",
                r"\\system32\\",
            ]).ok()
        } else {
            None
        };

        let cmd_injection_patterns = if config.command_injection_protection {
            RegexSet::new(&[
                r"[;&|`$]",
                r"(?i)(\b(cat|ls|dir|whoami|id|uname|wget|curl|nc|ncat|bash|sh|cmd|powershell)\b)",
                r"\$\(",
                r"`[^`]+`",
            ]).ok()
        } else {
            None
        };

        let header_injection_patterns = if config.header_injection_protection {
            RegexSet::new(&[
                r"\r\n",
                r"%0d%0a",
                r"%0D%0A",
            ]).ok()
        } else {
            None
        };

        Self {
            enabled: config.enabled,
            sqli_patterns,
            xss_patterns,
            path_traversal_patterns,
            cmd_injection_patterns,
            header_injection_patterns,
            max_url_length: config.max_url_length,
            max_body_size: config.max_body_size,
            allowed_methods: config.allowed_methods.clone(),
            blocked_extensions: config.blocked_extensions.clone(),
            blocked_paths: config.blocked_paths.clone(),
            block_empty_user_agent: config.block_empty_user_agent,
            blocked_user_agents_lower: config.blocked_user_agents.iter()
                .map(|s| s.to_lowercase())
                .collect(),
        }
    }

    /// Check a request through WAF rules.
    /// Returns None if clean, Some((rule_name, reason)) if blocked.
    pub fn check(
        &self,
        method: &str,
        path: &str,
        query: Option<&str>,
        user_agent: Option<&str>,
        content_length: Option<u64>,
        headers: &[(String, String)],
        body_preview: Option<&str>,
    ) -> Option<(String, String)> {
        if !self.enabled {
            return None;
        }

        // URL length check
        let full_url_len = path.len() + query.map(|q| q.len() + 1).unwrap_or(0);
        if full_url_len > self.max_url_length {
            return Some(("url-length".into(), format!("URL exceeds maximum length ({} > {})", full_url_len, self.max_url_length)));
        }

        // Method check
        if !self.allowed_methods.iter().any(|m| m.eq_ignore_ascii_case(method)) {
            return Some(("method-filter".into(), format!("HTTP method {} not allowed", method)));
        }

        // Body size check
        if let Some(cl) = content_length {
            if cl > self.max_body_size as u64 {
                return Some(("body-size".into(), format!("Request body exceeds maximum size ({} > {})", cl, self.max_body_size)));
            }
        }

        // User-Agent checks
        if let Some(ua) = user_agent {
            let ua_lower = ua.to_lowercase();
            for blocked in &self.blocked_user_agents_lower {
                if ua_lower.contains(blocked) {
                    return Some(("user-agent-block".into(), format!("Blocked User-Agent: {}", blocked)));
                }
            }
        } else if self.block_empty_user_agent {
            return Some(("empty-user-agent".into(), "Empty User-Agent header".into()));
        }

        // Blocked paths
        let path_lower = path.to_lowercase();
        for bp in &self.blocked_paths {
            if path_lower.starts_with(&bp.to_lowercase()) || path_lower.contains(&bp.to_lowercase()) {
                return Some(("blocked-path".into(), format!("Blocked path pattern: {}", bp)));
            }
        }

        // Blocked extensions
        for ext in &self.blocked_extensions {
            if path_lower.ends_with(ext) || path_lower.contains(&format!("{}/", ext)) {
                return Some(("blocked-extension".into(), format!("Blocked file extension: {}", ext)));
            }
        }

        // URL-decode for pattern matching
        let decoded_path = url_decode(path);
        let decoded_query = query.map(|q| url_decode(q)).unwrap_or_default();
        let combined = format!("{} {} {}", decoded_path, decoded_query, body_preview.unwrap_or(""));

        // SQL injection
        if let Some(ref patterns) = self.sqli_patterns {
            if patterns.is_match(&combined) {
                return Some(("sqli-detection".into(), "SQL injection detected".into()));
            }
            // Also check query params specifically
            if let Some(q) = query {
                let decoded = url_decode(q);
                if patterns.is_match(&decoded) {
                    return Some(("sqli-detection".into(), "SQL injection detected in query parameters".into()));
                }
            }
        }

        // XSS
        if let Some(ref patterns) = self.xss_patterns {
            if patterns.is_match(&combined) {
                return Some(("xss-detection".into(), "Cross-site scripting (XSS) detected".into()));
            }
        }

        // Path traversal
        if let Some(ref patterns) = self.path_traversal_patterns {
            if patterns.is_match(&decoded_path) {
                return Some(("path-traversal".into(), "Path traversal detected".into()));
            }
            if patterns.is_match(&decoded_query) {
                return Some(("path-traversal".into(), "Path traversal detected in query".into()));
            }
        }

        // Command injection (only in query params and body, not path)
        if let Some(ref patterns) = self.cmd_injection_patterns {
            if let Some(q) = query {
                let decoded = url_decode(q);
                if patterns.is_match(&decoded) {
                    return Some(("cmd-injection".into(), "Command injection detected in query".into()));
                }
            }
            if let Some(body) = body_preview {
                if patterns.is_match(body) {
                    return Some(("cmd-injection".into(), "Command injection detected in body".into()));
                }
            }
        }

        // Header injection
        if let Some(ref patterns) = self.header_injection_patterns {
            for (name, value) in headers {
                if patterns.is_match(value) {
                    return Some(("header-injection".into(), format!("Header injection detected in {}", name)));
                }
            }
        }

        None
    }
}

fn url_decode(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(hex_str) = std::str::from_utf8(&bytes[i + 1..i + 3]) {
                if let Ok(byte) = u8::from_str_radix(hex_str, 16) {
                    result.push(byte as char);
                    i += 3;
                    continue;
                }
            }
        }
        if bytes[i] == b'+' {
            result.push(' ');
        } else {
            result.push(bytes[i] as char);
        }
        i += 1;
    }
    result
}
