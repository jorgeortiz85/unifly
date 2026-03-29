mod detail;
mod edit;
mod table;

use ratatui::Frame;
use ratatui::layout::Rect;

use super::NetworksScreen;

impl NetworksScreen {
    pub(super) fn render_screen(&self, frame: &mut Frame, area: Rect) {
        table::render_screen(self, frame, area);
    }
}
