//! Device status indicator — ●/○/◐/◉ with color mapping.

use ratatui::style::Style;
use ratatui::text::Span;
use unifly_core::DeviceState;

use crate::theme;

/// Returns a styled `Span` with the appropriate status dot and color.
pub fn status_span(state: DeviceState) -> Span<'static> {
    let (symbol, color) = match state {
        DeviceState::Online => ("●", theme::success()),
        DeviceState::Offline | DeviceState::ConnectionInterrupted | DeviceState::Isolated => {
            ("○", theme::error())
        }
        DeviceState::PendingAdoption => ("◉", theme::accent_primary()),
        DeviceState::Updating
        | DeviceState::GettingReady
        | DeviceState::Adopting
        | DeviceState::Deleting => ("◐", theme::warning()),
        _ => ("?", theme::text_secondary()),
    };
    Span::styled(symbol.to_string(), Style::default().fg(color))
}

/// Returns the status dot character without styling (for raw output).
pub fn status_char(state: DeviceState) -> &'static str {
    match state {
        DeviceState::Online => "●",
        DeviceState::Offline | DeviceState::ConnectionInterrupted | DeviceState::Isolated => "○",
        DeviceState::PendingAdoption => "◉",
        DeviceState::Updating
        | DeviceState::GettingReady
        | DeviceState::Adopting
        | DeviceState::Deleting => "◐",
        _ => "?",
    }
}
