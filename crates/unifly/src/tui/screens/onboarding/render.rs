mod chrome;
mod steps;

use ratatui::Frame;
use ratatui::layout::Rect;

use super::OnboardingScreen;

impl OnboardingScreen {
    pub(super) fn render_screen(&self, frame: &mut Frame, area: Rect) {
        chrome::render_screen(self, frame, area);
    }
}
