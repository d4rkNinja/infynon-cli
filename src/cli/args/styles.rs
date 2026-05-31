use clap::builder::styling::{AnsiColor, Effects, Styles};

pub(crate) fn get_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::BrightCyan.on_default() | Effects::BOLD)
        .usage(AnsiColor::BrightGreen.on_default() | Effects::BOLD)
        .literal(AnsiColor::BrightMagenta.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::BrightBlue.on_default())
        .error(AnsiColor::Red.on_default() | Effects::BOLD)
}
