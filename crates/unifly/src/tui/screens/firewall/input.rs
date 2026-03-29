use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::tui::action::{Action, Direction, FirewallSubTab};

use super::FirewallScreen;

impl FirewallScreen {
    pub(super) fn handle_key_input(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.move_selection(1);
                Ok(None)
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.move_selection(-1);
                Ok(None)
            }
            KeyCode::Char('g') => {
                self.select(0);
                Ok(Some(Action::ScrollToTop))
            }
            KeyCode::Char('G') => {
                let len = self.active_len();
                if len > 0 {
                    self.select(len - 1);
                }
                Ok(Some(Action::ScrollToBottom))
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.move_selection(10);
                Ok(Some(Action::PageDown))
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.move_selection(-10);
                Ok(Some(Action::PageUp))
            }
            KeyCode::Char('l') => {
                self.sub_tab = match self.sub_tab {
                    FirewallSubTab::Policies => FirewallSubTab::Zones,
                    FirewallSubTab::Zones => FirewallSubTab::AclRules,
                    FirewallSubTab::AclRules => FirewallSubTab::Policies,
                };
                Ok(Some(Action::FirewallSubTab(self.sub_tab)))
            }
            KeyCode::Char('h') => {
                self.sub_tab = match self.sub_tab {
                    FirewallSubTab::Policies => FirewallSubTab::AclRules,
                    FirewallSubTab::Zones => FirewallSubTab::Policies,
                    FirewallSubTab::AclRules => FirewallSubTab::Zones,
                };
                Ok(Some(Action::FirewallSubTab(self.sub_tab)))
            }
            KeyCode::Char('K') if self.sub_tab == FirewallSubTab::Policies => {
                let index = self.selected_index();
                Ok(Some(Action::ReorderPolicy(index, Direction::Up)))
            }
            KeyCode::Char('J') if self.sub_tab == FirewallSubTab::Policies => {
                let index = self.selected_index();
                Ok(Some(Action::ReorderPolicy(index, Direction::Down)))
            }
            _ => Ok(None),
        }
    }
}
