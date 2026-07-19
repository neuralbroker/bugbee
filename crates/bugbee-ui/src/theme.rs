//! Bugbee default theme palette (dark).

use ratatui::style::{Color, Modifier, Style};

/// Primary warm accent (OpenCode darkStep9).
pub const PRIMARY: Color = Color::Rgb(0xfa, 0xb2, 0x83);
pub const PRIMARY_BRIGHT: Color = Color::Rgb(0xff, 0xc0, 0x9f);
pub const SECONDARY: Color = Color::Rgb(0x5c, 0x9c, 0xf5);
pub const ACCENT: Color = Color::Rgb(0x9d, 0x7c, 0xd8);
pub const ERROR: Color = Color::Rgb(0xe0, 0x6c, 0x75);
pub const WARNING: Color = Color::Rgb(0xf5, 0xa7, 0x42);
pub const SUCCESS: Color = Color::Rgb(0x7f, 0xd8, 0x8f);
pub const INFO: Color = Color::Rgb(0x56, 0xb6, 0xc2);
pub const TEXT: Color = Color::Rgb(0xee, 0xee, 0xee);
pub const TEXT_MUTED: Color = Color::Rgb(0x80, 0x80, 0x80);
pub const BG: Color = Color::Rgb(0x0a, 0x0a, 0x0a);
pub const BG_PANEL: Color = Color::Rgb(0x14, 0x14, 0x14);
pub const BG_ELEMENT: Color = Color::Rgb(0x1e, 0x1e, 0x1e);
pub const BORDER: Color = Color::Rgb(0x32, 0x32, 0x32);
pub const BORDER_ACTIVE: Color = Color::Rgb(0xfa, 0xb2, 0x83);

pub fn text() -> Style {
    Style::default().fg(TEXT)
}

pub fn muted() -> Style {
    Style::default().fg(TEXT_MUTED)
}

pub fn primary() -> Style {
    Style::default().fg(PRIMARY)
}

pub fn secondary() -> Style {
    Style::default().fg(SECONDARY)
}

pub fn primary_bold() -> Style {
    Style::default().fg(PRIMARY).add_modifier(Modifier::BOLD)
}

pub fn success() -> Style {
    Style::default().fg(SUCCESS)
}

pub fn error() -> Style {
    Style::default().fg(ERROR)
}

pub fn warning() -> Style {
    Style::default().fg(WARNING)
}

pub fn info() -> Style {
    Style::default().fg(INFO)
}

pub fn border() -> Style {
    Style::default().fg(BORDER)
}

pub fn border_active() -> Style {
    Style::default().fg(BORDER_ACTIVE)
}

pub fn selected() -> Style {
    Style::default()
        .bg(BG_ELEMENT)
        .fg(PRIMARY)
        .add_modifier(Modifier::BOLD)
}

pub fn user_msg() -> Style {
    Style::default().fg(SECONDARY)
}

pub fn assistant_msg() -> Style {
    Style::default().fg(TEXT)
}

pub fn tool_msg() -> Style {
    Style::default().fg(ACCENT)
}

pub fn system_msg() -> Style {
    Style::default().fg(TEXT_MUTED)
}
