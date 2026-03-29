use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};

use super::App;
use crate::tui::action::Action;
use crate::tui::screen::ScreenId;

impl App {
    /// Map a key event to an action. Global keys are handled here;
    /// screen-specific keys are delegated to the active screen component.
    pub(super) fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        if let Some(action) = self.handle_special_screen_key_event(key)? {
            return Ok(Some(action));
        }

        if self.active_screen == ScreenId::Setup || self.active_screen == ScreenId::Settings {
            return Ok(None);
        }

        if self.pending_confirm.is_some() {
            return Ok(match key.code {
                KeyCode::Char('y' | 'Y') => Some(Action::ConfirmYes),
                KeyCode::Char('n' | 'N') | KeyCode::Esc => Some(Action::ConfirmNo),
                _ => None,
            });
        }

        if self.search_active {
            return self.handle_search_key_event(key);
        }

        if self.help_visible {
            return Ok(match key.code {
                KeyCode::Esc | KeyCode::Char('?') => Some(Action::ToggleHelp),
                _ => None,
            });
        }

        if let Some(action) = self.handle_global_key_event(key) {
            return Ok(Some(action));
        }

        self.forward_key_to_active_screen(key)
    }

    /// Handle mouse events (delegate to active screen).
    pub(super) fn handle_mouse_event(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
        if let Some(screen) = self.screens.get_mut(&self.active_screen) {
            return screen.handle_mouse_event(mouse);
        }
        Ok(None)
    }

    fn handle_special_screen_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        if self.active_screen == ScreenId::Setup || self.active_screen == ScreenId::Settings {
            if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('c') {
                return Ok(Some(Action::Quit));
            }

            if let Some(screen) = self.screens.get_mut(&self.active_screen) {
                return screen.handle_key_event(key);
            }
        }

        Ok(None)
    }

    fn handle_search_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        Ok(match key.code {
            KeyCode::Esc => {
                self.search_query.clear();
                Some(Action::CloseSearch)
            }
            KeyCode::Enter => Some(Action::SearchSubmit),
            KeyCode::Backspace => {
                self.search_query.pop();
                Some(Action::SearchInput(self.search_query.clone()))
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                Some(Action::SearchInput(self.search_query.clone()))
            }
            _ => None,
        })
    }

    fn handle_global_key_event(&self, key: KeyEvent) -> Option<Action> {
        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('c'))
            | (KeyModifiers::NONE, KeyCode::Char('q')) => Some(Action::Quit),
            (KeyModifiers::NONE, KeyCode::Char('?')) => Some(Action::ToggleHelp),
            (KeyModifiers::NONE, KeyCode::Char('/')) => Some(Action::OpenSearch),
            (KeyModifiers::NONE, KeyCode::Char(',')) => Some(Action::OpenSettings),
            (KeyModifiers::NONE, KeyCode::Char(c @ '1'..='8')) => {
                #[allow(clippy::cast_possible_truncation, clippy::as_conversions)]
                let n = c.to_digit(10).unwrap_or(0) as u8;
                ScreenId::from_number(n).map(Action::SwitchScreen)
            }
            (KeyModifiers::NONE, KeyCode::Tab) => {
                Some(Action::SwitchScreen(self.active_screen.next()))
            }
            (KeyModifiers::SHIFT, KeyCode::BackTab) => {
                Some(Action::SwitchScreen(self.active_screen.prev()))
            }
            (KeyModifiers::NONE, KeyCode::Esc) => Some(Action::GoBack),
            _ => None,
        }
    }

    fn forward_key_to_active_screen(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        if let Some(screen) = self.screens.get_mut(&self.active_screen) {
            return screen.handle_key_event(key);
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::*;

    #[test]
    fn tab_navigation_cycles_between_primary_screens() {
        let mut app = App::new(None);
        app.active_screen = ScreenId::Dashboard;

        let action = app
            .handle_key_event(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE))
            .expect("key handling should succeed");
        assert!(matches!(
            action,
            Some(Action::SwitchScreen(ScreenId::Devices))
        ));

        let action = app
            .handle_key_event(KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT))
            .expect("key handling should succeed");
        assert!(matches!(
            action,
            Some(Action::SwitchScreen(ScreenId::Stats))
        ));
    }

    #[test]
    fn search_input_updates_query_and_can_close() {
        let mut app = App::new(None);
        app.active_screen = ScreenId::Dashboard;
        app.search_active = true;

        let action = app
            .handle_key_event(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::NONE))
            .expect("search key handling should succeed");
        assert!(matches!(action, Some(Action::SearchInput(ref q)) if q == "u"));
        assert_eq!(app.search_query, "u");

        let action = app
            .handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE))
            .expect("search close should succeed");
        assert!(matches!(action, Some(Action::CloseSearch)));
        assert!(app.search_query.is_empty());
    }
}
