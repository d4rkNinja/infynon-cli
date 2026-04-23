pub fn init_config(repo_name: &str, owner: &str, default_user: Option<&str>) -> Result<(), String> {
    ensure_layout()?;
    let cfg = TraceConfig {
        repo_name: repo_name.to_string(),
        owner: owner.to_string(),
        default_user: default_user.and_then(normalize_user),
        default_source: None,
        sources: Vec::new(),
    };
    save_config(&cfg)
}

pub fn load_config() -> Result<TraceConfig, String> {
    match fs::read_to_string(config_path()) {
        Ok(content) => toml::from_str(&content).map_err(|e| format!("invalid trace config: {}", e)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(TraceConfig::default()),
        Err(e) => Err(e.to_string()),
    }
}

pub fn save_config(cfg: &TraceConfig) -> Result<(), String> {
    ensure_layout()?;
    let content = toml::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    fs::write(config_path(), content).map_err(|e| e.to_string())
}

pub fn add_source(source: TraceSource, make_default: bool) -> Result<(), String> {
    let mut cfg = load_config()?;
    if cfg.sources.iter().any(|s| s.id == source.id) {
        return Err(format!("source '{}' already exists", source.id));
    }
    if make_default {
        cfg.default_source = Some(source.id.clone());
    }
    cfg.sources.push(source);
    save_config(&cfg)
}

pub fn configured_user() -> Option<String> {
    load_config()
        .ok()
        .and_then(|cfg| cfg.default_user.and_then(|v| normalize_user(&v)))
}

pub fn get_source(id: Option<&str>) -> Result<TraceSource, String> {
    let cfg = load_config()?;
    let wanted = match id {
        Some(id) => id.to_string(),
        None => cfg
            .default_source
            .clone()
            .ok_or_else(|| "No default Trace source configured.".to_string())?,
    };
    cfg.sources
        .into_iter()
        .find(|s| s.id == wanted)
        .ok_or_else(|| format!("source '{}' not found", wanted))
}

pub fn remove_source(id: &str) -> Result<(), String> {
    let mut cfg = load_config()?;
    let before = cfg.sources.len();
    cfg.sources.retain(|s| s.id != id);
    if before == cfg.sources.len() {
        return Err(format!("source '{}' not found", id));
    }
    if cfg.default_source.as_deref() == Some(id) {
        cfg.default_source = cfg.sources.first().map(|s| s.id.clone());
    }
    save_config(&cfg)
}

pub fn set_default_source(id: &str) -> Result<(), String> {
    let mut cfg = load_config()?;
    if !cfg.sources.iter().any(|s| s.id == id) {
        return Err(format!("source '{}' not found", id));
    }
    cfg.default_source = Some(id.to_string());
    save_config(&cfg)
}


pub fn detect_repo_name() -> String {
    std::env::current_dir()
        .ok()
        .as_deref()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("repo")
        .to_string()
}

pub fn detect_user_name() -> Option<String> {
    for key in ["INFYNON_USER", "USER", "USERNAME"] {
        if let Ok(value) = std::env::var(key) {
            if let Some(s) = normalize_user(&value) {
                return Some(s);
            }
        }
    }
    None
}

