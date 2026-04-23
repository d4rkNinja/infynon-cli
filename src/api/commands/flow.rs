use std::fs::File;
use std::io::{self, BufWriter, Write};

use owo_colors::OwoColorize;
use printpdf::{BuiltinFont, Mm, PdfDocument, PdfDocumentReference, PdfLayerIndex, PdfPageIndex};

use crate::api::ai;
use crate::api::executor::{execute_flow, FlowExecuteOptions};
use crate::api::storage;
use crate::api::types::{Edge, Flow};
use crate::api::variables;
use crate::tui::logger::Logger;

include!("flow/create.rs");
include!("flow/inspect.rs");
include!("flow/run.rs");
include!("flow/manage.rs");

fn name_to_id(name: &str) -> String {
    name.to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
}

fn prompt(message: &str) -> String {
    print!("{}", message);
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    input.trim().to_string()
}