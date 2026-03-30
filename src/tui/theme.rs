use ratatui::style::{Color, Modifier, Style};

// ── INFYNON Color Palette ───────────────────────────────────────────────────

pub const CYAN: Color = Color::Rgb(0, 210, 255);
pub const RED: Color = Color::Rgb(255, 68, 68);
pub const GREEN: Color = Color::Rgb(0, 255, 160);
pub const YELLOW: Color = Color::Rgb(255, 200, 50);
pub const ORANGE: Color = Color::Rgb(255, 140, 50);
pub const PURPLE: Color = Color::Rgb(180, 80, 255);
pub const DIM: Color = Color::Rgb(100, 100, 120);
pub const DIMMER: Color = Color::Rgb(60, 60, 80);
pub const TEXT: Color = Color::Rgb(220, 220, 230);
pub const TEXT_DIM: Color = Color::Rgb(160, 160, 180);
pub const BG: Color = Color::Rgb(10, 10, 20);
pub const BG_HIGHLIGHT: Color = Color::Rgb(25, 25, 45);
pub const BORDER: Color = Color::Rgb(80, 80, 120);
pub const WHITE: Color = Color::Rgb(240, 240, 250);

// ── Verdict colors ──────────────────────────────────────────────────────────

pub fn verdict_color(verdict: &str) -> Color {
    match verdict {
        "ALLOW" => GREEN,
        "BLOCK" => RED,
        "RATE_LIMITED" => ORANGE,
        "FLAG" => YELLOW,
        _ => DIM,
    }
}

pub fn verdict_style(verdict: &str) -> Style {
    Style::default().fg(verdict_color(verdict)).add_modifier(Modifier::BOLD)
}

// ── Common styles ───────────────────────────────────────────────────────────

pub fn title_style() -> Style {
    Style::default().fg(CYAN).add_modifier(Modifier::BOLD)
}

pub fn header_style() -> Style {
    Style::default().fg(WHITE).add_modifier(Modifier::BOLD)
}

pub fn selected_style() -> Style {
    Style::default().bg(BG_HIGHLIGHT).fg(CYAN).add_modifier(Modifier::BOLD)
}

pub fn normal_style() -> Style {
    Style::default().fg(TEXT)
}

pub fn dim_style() -> Style {
    Style::default().fg(DIM)
}

pub fn border_style() -> Style {
    Style::default().fg(BORDER)
}

pub fn status_running() -> Style {
    Style::default().fg(GREEN).add_modifier(Modifier::BOLD)
}

pub fn stat_value() -> Style {
    Style::default().fg(WHITE).add_modifier(Modifier::BOLD)
}

pub fn stat_label() -> Style {
    Style::default().fg(TEXT_DIM)
}
