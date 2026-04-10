//! Shared block styling for HyperChart widgets.
//!
//! Every hyperchart widget renders inside a rounded-border block with a
//! semantic title. Centralising the builder keeps focus treatment, border
//! type, and title styling consistent across every chart in the TUI.

use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Borders};

use crate::tui::theme;

/// Build the standard rounded-border block used by every hyperchart widget.
pub fn standard(title: Line<'_>, focused: bool) -> Block<'_> {
    let border_style = if focused {
        theme::border_focused()
    } else {
        theme::border_default()
    };

    Block::default()
        .title(title)
        .title_style(theme::title_style())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style)
}
