mod audit;
mod clean;
mod diff;
mod doctor;
pub mod eagle_eye;
mod fix;
mod migrate;
mod outdated;
mod search;
mod shared;
mod size;
mod why_cmd;

pub use audit::cmd_audit_deep;
pub use clean::cmd_clean;
pub use diff::cmd_diff;
pub use doctor::cmd_doctor;
pub use fix::cmd_fix_auto;
pub use migrate::cmd_migrate;
pub use outdated::cmd_outdated;
pub use search::cmd_search;
pub use size::cmd_size;
pub use why_cmd::cmd_why;

pub(crate) use shared::{
    bar, cargo_lock_deps, cargo_root_name, cargo_toml_dep_names, detect_ecosystem,
    format_bytes, format_severity_bar, http_client, load_packages, npm_declared_deps, spinner,
};

use crate::engine::{registry, scanner};
use crate::tui::logger::Logger;
use dialoguer::Select;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::sync::OnceLock;
use std::time::Duration;
