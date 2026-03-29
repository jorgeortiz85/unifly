use super::{SettingsField, SettingsScreen, SettingsState};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use opaline::ThemeSelectorAction;

use crate::tui::action::Action;

impl SettingsScreen {
    pub(super) fn handle_key_input(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        if self.handle_theme_selector_key(key) {
            return Ok(None);
        }

        if self.state == SettingsState::Testing {
            if key.code == KeyCode::Esc {
                self.state = SettingsState::Editing;
                self.test_error = None;
            }
            return Ok(None);
        }

        self.test_error = None;

        let action = match self.active_field {
            SettingsField::AuthMode => self.handle_auth_mode_key(key),
            SettingsField::Insecure => self.handle_insecure_key(key),
            SettingsField::Theme => self.handle_theme_key(key),
            _ => self.handle_text_field_key(key),
        };

        Ok(action)
    }

    pub(super) fn handle_mouse_input(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
        if self.state != SettingsState::Editing || self.theme_selector.borrow().is_some() {
            return Ok(None);
        }

        if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
            let area = self.last_area.get();
            if area.width == 0 {
                return Ok(None);
            }

            let panel = super::render::panel_rect(area);
            let content_y = panel.y + 2;
            let fields_x = panel.x + 2;
            let fields_w = panel.width.saturating_sub(4);

            let mut y = content_y;
            for (field, height) in self.field_layout() {
                if mouse.row >= y && mouse.row < y + height {
                    self.active_field = field;

                    match field {
                        SettingsField::Insecure => {
                            self.insecure = !self.insecure;
                        }
                        SettingsField::Theme => {
                            self.open_theme_selector();
                        }
                        SettingsField::AuthMode => {
                            let mid_x = fields_x + fields_w / 2;
                            if mouse.column < mid_x {
                                self.cycle_auth_mode_previous();
                            } else {
                                self.cycle_auth_mode_next();
                            }
                        }
                        _ => {}
                    }

                    break;
                }
                y += height;
            }
        }

        Ok(None)
    }

    pub(super) fn apply_action(&mut self, action: &Action) {
        match action {
            Action::SettingsTestResult(result) => match result {
                Ok(()) => self.send_apply(),
                Err(message) => {
                    self.state = SettingsState::Editing;
                    self.test_error = Some(message.clone());
                }
            },
            Action::Tick => {
                if self.state == SettingsState::Testing {
                    self.throbber_state.calc_next();
                }
            }
            _ => {}
        }
    }

    fn handle_theme_selector_key(&mut self, key: KeyEvent) -> bool {
        let mut selector = self.theme_selector.borrow_mut();
        let Some(ref mut theme_selector) = *selector else {
            return false;
        };

        match theme_selector.handle_key(key) {
            ThemeSelectorAction::Select(theme_id) => {
                SettingsScreen::save_theme_preference(&theme_id);
                *selector = None;
            }
            ThemeSelectorAction::Cancel => {
                *selector = None;
            }
            _ => {}
        }

        true
    }

    fn handle_auth_mode_key(&mut self, key: KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Up | KeyCode::Left | KeyCode::Char('k') | KeyCode::Char('h') => {
                self.cycle_auth_mode_previous();
                None
            }
            KeyCode::Down | KeyCode::Right | KeyCode::Char('j') | KeyCode::Char('l') => {
                self.cycle_auth_mode_next();
                None
            }
            KeyCode::Tab => {
                self.focus_next();
                None
            }
            KeyCode::BackTab => {
                self.focus_prev();
                None
            }
            KeyCode::Enter => {
                self.submit_connection_test();
                None
            }
            KeyCode::Esc => Some(Action::CloseSettings),
            _ => None,
        }
    }

    fn handle_insecure_key(&mut self, key: KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Char(' ') => {
                self.insecure = !self.insecure;
                None
            }
            KeyCode::Tab => {
                self.focus_next();
                None
            }
            KeyCode::BackTab => {
                self.focus_prev();
                None
            }
            KeyCode::Enter => {
                self.submit_connection_test();
                None
            }
            KeyCode::Esc => Some(Action::CloseSettings),
            _ => None,
        }
    }

    fn handle_theme_key(&mut self, key: KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Enter => {
                self.open_theme_selector();
                None
            }
            KeyCode::Tab => {
                self.focus_next();
                None
            }
            KeyCode::BackTab => {
                self.focus_prev();
                None
            }
            KeyCode::Esc => Some(Action::CloseSettings),
            _ => None,
        }
    }

    fn handle_text_field_key(&mut self, key: KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Tab => {
                self.focus_next();
                None
            }
            KeyCode::BackTab => {
                self.focus_prev();
                None
            }
            KeyCode::Enter => {
                self.submit_connection_test();
                None
            }
            KeyCode::Esc => Some(Action::CloseSettings),
            KeyCode::Backspace => {
                if let Some(input) = self.active_input_mut() {
                    input.pop();
                }
                None
            }
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) && c == 'u' {
                    self.show_password = !self.show_password;
                } else if let Some(input) = self.active_input_mut() {
                    input.push(c);
                }
                None
            }
            _ => None,
        }
    }

    fn open_theme_selector(&mut self) {
        *self.theme_selector.borrow_mut() = Some(
            opaline::ThemeSelectorState::with_current_selected()
                .with_derive(crate::tui::theme::derive_tokens),
        );
    }
}
