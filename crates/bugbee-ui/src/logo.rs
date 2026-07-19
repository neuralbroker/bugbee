//! Wordmark banners.

use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::theme::{PRIMARY, PRIMARY_BRIGHT, TEXT_MUTED};

/// OpenCode-style multi-line wordmark for home.
pub fn bugbee_logo_lines() -> Vec<Line<'static>> {
    let accent = Style::default().fg(PRIMARY).add_modifier(Modifier::BOLD);
    let soft = Style::default().fg(PRIMARY_BRIGHT);
    let muted = Style::default().fg(TEXT_MUTED);

    vec![
        Line::from(vec![Span::styled("█▀▀▄ █  █ █▀▀▀ █▀▀▄ █▀▀▀ █▀▀▀", accent)]),
        Line::from(vec![Span::styled("█▀▀▄ █  █ █ ▀█ █▀▀▄ █▀▀  █▀▀ ", soft)]),
        Line::from(vec![Span::styled("▀▀▀  ▀▀▀▀ ▀▀▀▀ ▀▀▀  ▀▀▀▀ ▀▀▀▀", accent)]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  security agent  ·  opencode-class ide",
            muted,
        )]),
    ]
}

pub fn small_brand() -> Line<'static> {
    Line::from(vec![Span::styled(
        " BUGBEE ",
        Style::default()
            .fg(ratatui::style::Color::Black)
            .bg(PRIMARY)
            .add_modifier(Modifier::BOLD),
    )])
}
