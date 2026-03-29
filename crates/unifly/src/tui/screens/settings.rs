//! Settings screen - edit controller config from within the TUI.
//!
//! Opened with `,`, not in the tab bar. Esc cancels without saving.
//! On successful connection test, saves config and emits `SettingsApply`
//! so the app can reconnect with the new configuration.

mod input;
mod render;
mod state;

use std::cell::{Cell, RefCell};

use color_eyre::eyre::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::Frame;
use ratatui::layout::Rect;
use tokio::sync::mpsc::UnboundedSender;

use crate::tui::action::Action;
use crate::tui::component::Component;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SettingsState {
    Editing,
    Testing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AuthMode {
    ApiKey,
    Legacy,
    Hybrid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SettingsField {
    Url,
    AuthMode,
    ApiKey,
    Username,
    Password,
    Site,
    Insecure,
    Theme,
}

pub struct SettingsScreen {
    focused: bool,
    action_tx: Option<UnboundedSender<Action>>,
    state: SettingsState,
    active_field: SettingsField,
    url_input: String,
    auth_mode: AuthMode,
    auth_mode_index: usize,
    api_key_input: String,
    username_input: String,
    password_input: String,
    site_input: String,
    insecure: bool,
    show_password: bool,
    profile_name: String,
    test_error: Option<String>,
    throbber_state: throbber_widgets_tui::ThrobberState,
    last_area: Cell<Rect>,
    theme_selector: RefCell<Option<opaline::ThemeSelectorState>>,
}

impl Default for SettingsScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for SettingsScreen {
    fn init(&mut self, action_tx: UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(action_tx);
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        self.handle_key_input(key)
    }

    fn handle_mouse_event(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
        self.handle_mouse_input(mouse)
    }

    fn update(&mut self, action: &Action) -> Result<Option<Action>> {
        self.apply_action(action);
        Ok(None)
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        self.render_screen(frame, area);
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn focused(&self) -> bool {
        self.focused
    }

    fn id(&self) -> &'static str {
        "settings"
    }
}
