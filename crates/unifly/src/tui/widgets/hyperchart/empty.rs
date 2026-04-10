//! Consistent empty-state messaging for HyperChart widgets.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Paragraph, Widget};

use crate::tui::theme;

/// Render a muted empty-state message inside the given area.
pub fn render(inner: Rect, buf: &mut Buffer, message: &str) {
    let text = format!("  {message}");
    Paragraph::new(text)
        .style(Style::default().fg(theme::border_unfocused()))
        .render(inner, buf);
}
