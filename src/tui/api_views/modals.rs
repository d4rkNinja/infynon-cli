use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::tui::api_app::ApiApp;
use crate::tui::theme::*;

use super::{blank_line, section_header, truncate};

include!("modals/json.rs");
include!("modals/prompt.rs");
include!("modals/body.rs");
include!("modals/detail.rs");
