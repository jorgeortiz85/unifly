use std::sync::Arc;

use super::ClientsScreen;

use ratatui::widgets::TableState;
use unifly_api::{Client, ClientType, EntityId};

use crate::tui::action::ClientTypeFilter;

impl ClientsScreen {
    pub fn new() -> Self {
        Self {
            focused: false,
            action_tx: None,
            clients: Arc::new(Vec::new()),
            table_state: TableState::default(),
            filter: ClientTypeFilter::All,
            search_query: String::new(),
            detail_open: false,
            detail_client_id: None,
        }
    }

    pub(super) fn filtered_clients(&self) -> Vec<&Arc<Client>> {
        let query = self.search_query.to_lowercase();
        self.clients
            .iter()
            .filter(|client| match self.filter {
                ClientTypeFilter::All => true,
                ClientTypeFilter::Wireless => client.client_type == ClientType::Wireless,
                ClientTypeFilter::Wired => client.client_type == ClientType::Wired,
                ClientTypeFilter::Vpn => client.client_type == ClientType::Vpn,
                ClientTypeFilter::Guest => client.is_guest,
            })
            .filter(|client| {
                if query.is_empty() {
                    return true;
                }

                client
                    .name
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(&query)
                    || client
                        .hostname
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&query)
                    || client
                        .ip
                        .map(|ip| ip.to_string())
                        .unwrap_or_default()
                        .contains(&query)
                    || client.mac.to_string().contains(&query)
            })
            .collect()
    }

    pub(super) fn selected_index(&self) -> usize {
        self.table_state.selected().unwrap_or(0)
    }

    pub(super) fn detail_client_index(&self, filtered: &[&Arc<Client>]) -> Option<usize> {
        let id = self.detail_client_id.as_ref()?;
        filtered.iter().position(|client| client.id == *id)
    }

    pub(super) fn detail_client<'a>(
        &self,
        filtered: &'a [&Arc<Client>],
    ) -> Option<&'a Arc<Client>> {
        let idx = self.detail_client_index(filtered)?;
        filtered.get(idx).copied()
    }

    pub(super) fn select(&mut self, idx: usize) {
        let filtered = self.filtered_clients();
        let clamped = if filtered.is_empty() {
            0
        } else {
            idx.min(filtered.len() - 1)
        };
        self.table_state.select(Some(clamped));
    }

    #[allow(clippy::cast_sign_loss, clippy::as_conversions)]
    pub(super) fn move_selection(&mut self, delta: isize) {
        let filtered = self.filtered_clients();
        if filtered.is_empty() {
            return;
        }

        #[allow(clippy::cast_possible_wrap)]
        let current = self.selected_index() as isize;
        #[allow(clippy::cast_possible_wrap)]
        let next = (current + delta).clamp(0, filtered.len() as isize - 1);
        self.select(next as usize);
    }

    pub(super) fn cycle_filter(&mut self) {
        self.filter = match self.filter {
            ClientTypeFilter::All => ClientTypeFilter::Wireless,
            ClientTypeFilter::Wireless => ClientTypeFilter::Wired,
            ClientTypeFilter::Wired => ClientTypeFilter::Vpn,
            ClientTypeFilter::Vpn => ClientTypeFilter::Guest,
            ClientTypeFilter::Guest => ClientTypeFilter::All,
        };
        self.table_state.select(Some(0));
    }

    pub(super) fn reconcile_selection_after_view_change(&mut self, reset_to_top: bool) {
        let (filtered_len, detail_idx) = {
            let filtered = self.filtered_clients();
            let detail_idx = if self.detail_open {
                self.detail_client_index(&filtered)
            } else {
                None
            };
            (filtered.len(), detail_idx)
        };

        if self.detail_open {
            if let Some(index) = detail_idx {
                self.table_state.select(Some(index));
                return;
            }

            self.detail_open = false;
            self.detail_client_id = None;
        }

        if reset_to_top || filtered_len == 0 {
            self.table_state.select(Some(0));
        } else {
            let idx = self.selected_index().min(filtered_len - 1);
            self.table_state.select(Some(idx));
        }
    }

    pub(super) fn filter_index(&self) -> usize {
        match self.filter {
            ClientTypeFilter::All => 0,
            ClientTypeFilter::Wireless => 1,
            ClientTypeFilter::Wired => 2,
            ClientTypeFilter::Vpn => 3,
            ClientTypeFilter::Guest => 4,
        }
    }

    pub(super) fn selected_client_id(&self) -> Option<EntityId> {
        self.filtered_clients()
            .get(self.selected_index())
            .map(|client| client.id.clone())
    }

    pub(super) fn detail_action_client_id(&self) -> Option<EntityId> {
        let filtered = self.filtered_clients();
        self.detail_client(&filtered)
            .map(|client| client.id.clone())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::ClientsScreen;

    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use serde_json::json;
    use unifly_api::Client;
    use unifly_api::integration_types::ClientResponse;
    use uuid::Uuid;

    use crate::tui::action::Action;

    fn test_client(id: Uuid, name: &str, mac: &str) -> Arc<Client> {
        Arc::new(
            ClientResponse {
                id,
                name: name.to_owned(),
                client_type: "WIRELESS".to_owned(),
                ip_address: Some("192.168.1.10".to_owned()),
                connected_at: None,
                mac_address: Some(mac.to_owned()),
                access: json!({ "type": "DEFAULT" }),
            }
            .into(),
        )
    }

    #[test]
    fn detail_actions_follow_client_identity_after_refresh() {
        let alpha_id = Uuid::from_u128(1);
        let bravo_id = Uuid::from_u128(2);
        let alpha = test_client(alpha_id, "alpha", "aa:bb:cc:dd:ee:01");
        let bravo = test_client(bravo_id, "bravo", "aa:bb:cc:dd:ee:02");

        let mut screen = ClientsScreen::new();
        let clients = Arc::new(vec![Arc::clone(&alpha), Arc::clone(&bravo)]);
        screen.clients = Arc::clone(&clients);
        screen.table_state.select(Some(1));
        assert!(matches!(
            screen
                .handle_key_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
            Some(Action::OpenClientDetail(id)) if id == bravo.id
        ));

        let refreshed = Arc::new(vec![Arc::clone(&bravo), Arc::clone(&alpha)]);
        screen.clients = Arc::clone(&refreshed);
        screen.apply_action(&Action::ClientsUpdated(refreshed));

        assert_eq!(screen.detail_client_id.as_ref(), Some(&bravo.id));
        assert_eq!(screen.table_state.selected(), Some(0));

        assert!(matches!(
            screen
                .handle_key_input(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE)),
            Some(Action::RequestKickClient(id)) if id == bravo.id
        ));
    }

    #[test]
    fn detail_closes_when_target_falls_out_of_filter() {
        let alpha = test_client(Uuid::from_u128(1), "alpha", "aa:bb:cc:dd:ee:01");
        let bravo = test_client(Uuid::from_u128(2), "bravo", "aa:bb:cc:dd:ee:02");

        let mut screen = ClientsScreen::new();
        screen.clients = Arc::new(vec![Arc::clone(&alpha), Arc::clone(&bravo)]);
        screen.table_state.select(Some(1));
        let _ = screen.handle_key_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        screen.apply_action(&Action::SearchInput("alpha".to_owned()));

        assert!(!screen.detail_open);
        assert!(screen.detail_client_id.is_none());
        assert_eq!(screen.table_state.selected(), Some(0));
    }
}
