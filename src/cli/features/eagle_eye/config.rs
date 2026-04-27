use super::types::EagleEyeConfig;

pub(super) fn config_path() -> std::path::PathBuf {
    config_dir().join("eagle-eye.toml")
}

pub(super) fn config_dir() -> std::path::PathBuf {
    crate::utils::home_infynon_dir()
}

pub(super) fn load_config() -> EagleEyeConfig {
    let path = config_path();
    if !path.exists() {
        return EagleEyeConfig::default();
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => toml::from_str(&content).unwrap_or_default(),
        Err(_) => EagleEyeConfig::default(),
    }
}

pub(super) fn save_config(config: &EagleEyeConfig) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| format!("Failed to create config dir: {}", err))?;
    }
    let content = toml::to_string_pretty(config)
        .map_err(|err| format!("Failed to serialize config: {}", err))?;
    std::fs::write(&path, content).map_err(|err| format!("Failed to write config: {}", err))
}
