use super::types::SmtpConfig;

pub(super) fn resolve_smtp_password(config: &SmtpConfig) -> Option<String> {
    if !config.password_env.trim().is_empty() {
        if let Ok(value) = std::env::var(&config.password_env) {
            if !value.trim().is_empty() {
                return Some(value);
            }
        }
    }
    if !config.legacy_password.trim().is_empty() {
        return Some(config.legacy_password.clone());
    }
    None
}

pub(super) fn password_status(config: &SmtpConfig) -> String {
    if !config.password_env.trim().is_empty() {
        if std::env::var(&config.password_env)
            .ok()
            .filter(|value| !value.trim().is_empty())
            .is_some()
        {
            format!("env:{}", config.password_env)
        } else {
            format!("missing env:{}", config.password_env)
        }
    } else if !config.legacy_password.trim().is_empty() {
        "legacy config".to_string()
    } else {
        "not configured".to_string()
    }
}
