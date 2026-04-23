use std::collections::{HashMap, HashSet};
use std::io::{self, Write};

use owo_colors::OwoColorize;
use serde_json::Value;

use crate::api::ai;
use crate::api::executor;
use crate::api::storage;
use crate::api::types::{Assertion, Edge, Extraction, Node, OnFail, PromptInput};
use crate::api::variables;
use crate::tui::logger::Logger;

include!("node/create.rs");
include!("node/inspect.rs");
include!("node/run.rs");
include!("node/manage.rs");

fn check_index(idx: usize, len: usize) -> bool {
    if idx >= len {
        Logger::error(&format!("Index {} out of range (0..{})", idx, len.saturating_sub(1)));
        return false;
    }
    true
}

fn prompt(message: &str) -> String {
    print!("{}", message);
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    input.trim().to_string()
}

pub fn print_step_result_pub(step: &crate::api::types::StepResult) {
    print_step_result(step);
}