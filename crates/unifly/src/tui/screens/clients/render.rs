mod detail;
mod table;

use super::ClientsScreen;

use ratatui::Frame;
use ratatui::layout::Rect;

impl ClientsScreen {
    pub(super) fn render_screen(&self, frame: &mut Frame, area: Rect) {
        table::render_screen(self, frame, area);
    }
}
