fn execute_source(action: SourceAction) -> i32 {
    match action {
        SourceAction::AddRedis {
            id,
            url,
            namespace,
            notes,
            user,
            default,
        } => cmd_source_add_redis(
            &id,
            &url,
            &namespace,
            notes.as_deref(),
            user.as_deref(),
            default,
        ),
        SourceAction::AddSql {
            id,
            engine,
            url,
            database,
            username,
            password_env,
            notes,
            user,
            default,
        } => cmd_source_add_sql(
            &id,
            &engine,
            &url,
            database.as_deref(),
            username.as_deref(),
            password_env.as_deref(),
            notes.as_deref(),
            user.as_deref(),
            default,
        ),
        SourceAction::List => cmd_source_list(),
        SourceAction::Remove { id } => cmd_source_remove(&id),
        SourceAction::Default { id } => cmd_source_default(&id),
    }
}

fn cmd_init(repo: Option<&str>, owner: Option<&str>, user: Option<&str>) {
    let repo_name = repo
        .map(|s| s.to_string())
        .unwrap_or_else(storage::detect_repo_name);
    let owner_name = owner.unwrap_or("team");
    let detected_user = storage::detect_user_name();
    let default_user = user.or(detected_user.as_deref());
    match storage::init_config(&repo_name, owner_name, default_user) {
        Ok(()) => {
            if let Ok(source) = storage::get_source(None) {
                if let Err(e) = crate::trace::backend::validate_and_prepare(&source) {
                    return Logger::error(&format!(
                        "Trace was initialized, but the default local source could not be prepared: {}",
                        e
                    ));
                }
            }
            Logger::success(&format!("Initialized Trace for '{}'", repo_name));
            Logger::detail("Owner:", owner_name);
            if let Some(user) = default_user {
                Logger::detail("Default user:", user);
            }
            Logger::detail("Default source:", "local-sqlite");
            Logger::detail("SQLite DB:", ".infynon/trace/trace.db");
            Logger::detail("Path:", &storage::trace_dir().display().to_string());
        }
        Err(e) => Logger::error(&e),
    }
}

fn cmd_source_add_redis(
    id: &str,
    url: &str,
    namespace: &str,
    notes: Option<&str>,
    user: Option<&str>,
    make_default: bool,
) -> i32 {
    let source = TraceSource {
        id: id.to_string(),
        kind: SourceKind::Redis,
        url: url.to_string(),
        enabled: true,
        owner_user: user
            .map(|value| value.to_string())
            .or_else(storage::configured_user),
        database: None,
        namespace: Some(namespace.to_string()),
        username: None,
        password_env: None,
        notes: notes.map(|s| s.to_string()),
    };
    if let Err(e) = crate::trace::backend::validate_and_prepare(&source) {
        Logger::error(&format!("Redis validation failed: {}", e));
        return EXIT_TRACE_STORAGE_ERROR;
    }
    match storage::add_source(source, make_default) {
        Ok(()) => {
            Logger::success(&format!("Added Redis source '{}'", id));
            Logger::raw_dim(
                "Benefit: low-latency lookups, live presence, and fast overlap detection.",
            );
            0
        }
        Err(e) => {
            Logger::error(&e);
            EXIT_TRACE_STORAGE_ERROR
        }
    }
}

fn cmd_source_add_sql(
    id: &str,
    engine: &str,
    url: &str,
    database: Option<&str>,
    username: Option<&str>,
    password_env: Option<&str>,
    notes: Option<&str>,
    user: Option<&str>,
    make_default: bool,
) -> i32 {
    let kind = match engine.to_ascii_lowercase().as_str() {
        "postgres" | "postgresql" => SourceKind::Postgres,
        "mysql" => SourceKind::Mysql,
        "sqlite" => SourceKind::Sqlite,
        other => {
            Logger::error(&format!(
                "Unsupported SQL engine '{}'. Use postgres | mysql | sqlite.",
                other
            ));
            return EXIT_TRACE_INVALID_INPUT;
        }
    };
    let source = TraceSource {
        id: id.to_string(),
        kind,
        url: url.to_string(),
        enabled: true,
        owner_user: user
            .map(|value| value.to_string())
            .or_else(storage::configured_user),
        database: database.map(|s| s.to_string()),
        namespace: None,
        username: username.map(|s| s.to_string()),
        password_env: password_env.map(|s| s.to_string()),
        notes: notes.map(|s| s.to_string()),
    };
    if let Err(e) = crate::trace::backend::validate_and_prepare(&source) {
        Logger::error(&format!("SQL validation failed: {}", e));
        return EXIT_TRACE_STORAGE_ERROR;
    }
    match storage::add_source(source, make_default) {
        Ok(()) => {
            Logger::success(&format!("Added {} source '{}'", kind.as_str(), id));
            Logger::raw_dim("Benefit: durable structured storage, better filtering, reporting, and canonical memory.");
            0
        }
        Err(e) => {
            Logger::error(&e);
            EXIT_TRACE_STORAGE_ERROR
        }
    }
}

fn cmd_source_list() -> i32 {
    match storage::load_config() {
        Ok(cfg) => {
            if cfg.sources.is_empty() {
                Logger::info("No Trace backends configured.");
                return 0;
            }
            println!(
                "  {:<18} {:<10} {:<8} {:<16} {}",
                "ID", "KIND", "DEFAULT", "USER", "URL"
            );
            println!("  {}", "-".repeat(80));
            for source in cfg.sources {
                let is_default = cfg.default_source.as_deref() == Some(source.id.as_str());
                println!(
                    "  {:<18} {:<10} {:<8} {:<16} {}",
                    source.id,
                    source.kind.as_str(),
                    if is_default { "yes" } else { "no" },
                    source.owner_user.clone().unwrap_or_else(|| "-".to_string()),
                    source.url
                );
            }
            0
        }
        Err(e) => {
            Logger::error(&e);
            EXIT_TRACE_STORAGE_ERROR
        }
    }
}

fn cmd_source_remove(id: &str) -> i32 {
    match storage::remove_source(id) {
        Ok(()) => {
            Logger::success(&format!("Removed source '{}'", id));
            0
        }
        Err(e) => {
            Logger::error(&e);
            EXIT_TRACE_STORAGE_ERROR
        }
    }
}

fn cmd_source_default(id: &str) -> i32 {
    match storage::set_default_source(id) {
        Ok(()) => {
            Logger::success(&format!("Default source set to '{}'", id));
            0
        }
        Err(e) => {
            Logger::error(&e);
            EXIT_TRACE_STORAGE_ERROR
        }
    }
}
