//! WiFi signal strength bars — ▂▄▆█ with color thresholds.

use ratatui::style::Style;
use ratatui::text::Span;

use crate::theme;

/// Returns a styled `Span` with signal bars based on dBm value.
///
/// | Bars    | dBm Range  | Color         |
/// |---------|-----------|---------------|
/// | `▂▄▆█` | >= -50    | Success Green |
/// | `▂▄▆ ` | -50 to -60| Neon Cyan     |
/// | `▂▄  ` | -60 to -70| Electric Yellow|
/// | `▂   ` | -70 to -80| Coral         |
/// | `·   ` | < -80     | Error Red     |
#[allow(dead_code)]
pub fn signal_span(dbm: Option<i32>) -> Span<'static> {
    let Some(dbm) = dbm else {
        return Span::styled("····", Style::default().fg(theme::border_unfocused()));
    };

    let (bars, color) = if dbm >= -50 {
        ("▂▄▆█", theme::success())
    } else if dbm >= -60 {
        ("▂▄▆ ", theme::accent_secondary())
    } else if dbm >= -70 {
        ("▂▄  ", theme::warning())
    } else if dbm >= -80 {
        ("▂   ", theme::accent_tertiary())
    } else {
        ("·   ", theme::error())
    };

    Span::styled(bars.to_string(), Style::default().fg(color))
}

/// Returns a styled `Span` for wired clients (no signal data).
#[allow(dead_code)]
pub fn wired_span() -> Span<'static> {
    Span::styled("····", Style::default().fg(theme::border_unfocused()))
}
