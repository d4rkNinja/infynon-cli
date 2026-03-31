use ratatui::style::{Color, Modifier, Style};

// ── WEAVE Warm Color Palette (ANSI-compatible, macOS-safe) ───────────────────
//
// A warm, cozy theme with amber/orange accents. Uses standard ANSI colors
// plus 256-palette indexed colors for subtle variations.
// Works on: macOS Terminal.app, iTerm2, Alacritty, Kitty, Windows Terminal

// ── Accent colors (warm tones) ─────────────────────────────────────────────────

pub const CYAN: Color = Color::Indexed(180);       // Warm cyan (soft teal)
pub const RED: Color = Color::Indexed(167);        // Warm red (terracotta)
pub const GREEN: Color = Color::Indexed(142);      // Warm green (olive/sage)
pub const YELLOW: Color = Color::Indexed(214);     // Warm yellow (amber)
pub const ORANGE: Color = Color::Indexed(208);     // Bright orange (primary accent)
pub const PURPLE: Color = Color::Indexed(139);     // Warm purple (dusty)
pub const TEAL: Color = Color::Indexed(108);       // Soft teal
pub const PINK: Color = Color::Indexed(175);       // Warm pink (dusty rose)

// ── Text hierarchy (warm grays) ───────────────────────────────────────────────

pub const WHITE: Color = Color::Indexed(229);      // Warm white (cream)
pub const TEXT: Color = Color::Indexed(223);       // Primary text (light cream)
pub const TEXT_DIM: Color = Color::Indexed(180);   // Secondary text (warm gray)
pub const DIM: Color = Color::Indexed(138);        // Dim text (warm brown-gray)
pub const DIMMER: Color = Color::Indexed(101);     // Very dim (dark warm gray)

// ── Backgrounds (warm darks) ──────────────────────────────────────────────────

pub const BG: Color = Color::Indexed(234);         // Main bg (warm black)
pub const BG_SURFACE: Color = Color::Indexed(235); // Surface bg (slightly lighter)
pub const BG_HIGHLIGHT: Color = Color::Indexed(238); // Highlight bg
pub const BG_SELECTED: Color = Color::Indexed(96);  // Selection bg (visible warm blue)
pub const BG_NODE_SELECTED: Color = Color::Indexed(130); // Node selection (warm red-brown)

// ── Borders ───────────────────────────────────────────────────────────────────

pub const BORDER: Color = Color::Indexed(101);     // Warm gray border
pub const BORDER_ACTIVE: Color = Color::Indexed(208); // Orange active border

// ── Verdict colors ────────────────────────────────────────────────────────────

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

// ── Common styles ─────────────────────────────────────────────────────────────

pub fn title_style() -> Style {
    Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)
}

pub fn header_style() -> Style {
    Style::default().fg(WHITE).add_modifier(Modifier::BOLD)
}

pub fn selected_style() -> Style {
    Style::default().bg(BG_HIGHLIGHT).fg(ORANGE).add_modifier(Modifier::BOLD)
}

pub fn normal_style() -> Style {
    Style::default().fg(TEXT)
}

/// Text dim style for secondary content
pub fn text_dim_style() -> Style {
    Style::default().fg(TEXT_DIM)
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

pub fn method_color(method: &str) -> Color {
    match method {
        "GET"    => GREEN,
        "POST"   => YELLOW,
        "PUT"    => CYAN,
        "PATCH"  => TEAL,
        "DELETE" => RED,
        "HEAD"   => DIM,
        _        => DIM,
    }
}

pub fn method_style(method: &str) -> Style {
    Style::default().fg(method_color(method))
}

/// Returns (icon, color) for pass/fail status.
pub fn pass_fail_icon(passed: bool) -> (&'static str, Color) {
    if passed { ("\u{2714}", GREEN) } else { ("\u{2718}", RED) }
}

/// Returns color for HTTP status code.
pub fn status_code_color(code: Option<u16>) -> Color {
    match code {
        Some(s) if s < 300 => GREEN,
        Some(s) if s < 400 => YELLOW,
        Some(_) => RED,
        None => RED,
    }
}

// ── Sidebar styles ────────────────────────────────────────────────────────────

pub fn sidebar_active_style() -> Style {
    Style::default().bg(BG_SELECTED).fg(WHITE).add_modifier(Modifier::BOLD)
}

pub fn sidebar_inactive_style() -> Style {
    Style::default().fg(TEXT_DIM)
}

pub fn sidebar_accent() -> Style {
    Style::default().fg(BORDER_ACTIVE)
}

pub fn sidebar_icon_active() -> Style {
    Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)
}

pub fn sidebar_icon_inactive() -> Style {
    Style::default().fg(DIM)
}

// ── Runner sub-tab styles ─────────────────────────────────────────────────────

pub fn subtab_active() -> Style {
    Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)
}

pub fn subtab_inactive() -> Style {
    Style::default().fg(DIM)
}

// ── Shared utilities ──────────────────────────────────────────────────────────

/// Gradient from GREEN (0.0) → YELLOW (0.5) → RED (1.0).
pub fn gradient_color(ratio: f64) -> Color {
    let ratio = ratio.clamp(0.0, 1.0);
    if ratio < 0.5 {
        YELLOW
    } else {
        RED
    }
}

/// Badge style: colored background with dark text.
pub fn badge_style(color: Color) -> Style {
    Style::default().fg(BG).bg(color).add_modifier(Modifier::BOLD)
}

/// Build a centered section header string: `─── Title ───` padded to `width`.
pub fn section_line(title: &str, width: usize) -> String {
    let inner = width.saturating_sub(2);
    let label = if title.is_empty() {
        String::new()
    } else {
        format!(" {} ", title)
    };
    let label_len = label.chars().count();
    let remaining = inner.saturating_sub(label_len);
    let left = remaining / 2;
    let right = remaining - left;
    let mut s = String::with_capacity(width);
    s.push(' ');
    for _ in 0..left { s.push('\u{2500}'); }
    s.push_str(&label);
    for _ in 0..right { s.push('\u{2500}'); }
    s
}

/// Build a left-aligned section header: `── Title ──────` padded to `width`.
pub fn section_line_left(title: &str, width: usize) -> String {
    let label = format!("\u{2500}\u{2500} {} ", title);
    let label_len = label.chars().count();
    let remaining = width.saturating_sub(label_len + 2);
    let mut s = String::with_capacity(width);
    s.push_str("  ");
    s.push_str(&label);
    for _ in 0..remaining { s.push('\u{2500}'); }
    s
}

/// Progress bar string: `████░░░░░░░ 45%` with given width.
pub fn progress_bar(pct: u8, width: usize) -> String {
    let pct = pct.min(100) as usize;
    let bar_w = width.saturating_sub(6).max(4);
    let filled = if pct == 0 { 0 } else { (pct * bar_w) / 100 };
    let empty = bar_w.saturating_sub(filled);
    let fill_char = if pct == 100 { '\u{2588}' } else { '\u{2593}' };
    let fill_str = fill_char.to_string().repeat(filled);
    let trail_str = "\u{2591}".repeat(empty);
    let pad = if pct < 10 { "  " } else if pct < 100 { " " } else { "" };
    format!("{}{} {}{}%", fill_str, trail_str, pad, pct)
}

/// Pretty-print JSON string, returning original on parse error.
pub fn pretty_json(raw: &str) -> String {
    serde_json::from_str::<serde_json::Value>(raw)
        .map(|v| serde_json::to_string_pretty(&v).unwrap_or_else(|_| raw.to_string()))
        .unwrap_or_else(|_| raw.to_string())
}
