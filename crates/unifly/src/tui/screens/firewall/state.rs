use std::sync::Arc;

use ratatui::widgets::TableState;

use crate::tui::action::{Action, Direction, FirewallSubTab};

use super::FirewallScreen;

impl FirewallScreen {
    pub fn new() -> Self {
        Self {
            focused: false,
            action_tx: None,
            sub_tab: FirewallSubTab::default(),
            policies: Arc::new(Vec::new()),
            zones: Arc::new(Vec::new()),
            acl_rules: Arc::new(Vec::new()),
            policy_table: TableState::default(),
            zone_table: TableState::default(),
            acl_table: TableState::default(),
        }
    }

    pub(super) fn active_table_state(&mut self) -> &mut TableState {
        match self.sub_tab {
            FirewallSubTab::Policies => &mut self.policy_table,
            FirewallSubTab::Zones => &mut self.zone_table,
            FirewallSubTab::AclRules => &mut self.acl_table,
        }
    }

    pub(super) fn active_len(&self) -> usize {
        match self.sub_tab {
            FirewallSubTab::Policies => self.policies.len(),
            FirewallSubTab::Zones => self.zones.len(),
            FirewallSubTab::AclRules => self.acl_rules.len(),
        }
    }

    pub(super) fn selected_index(&self) -> usize {
        match self.sub_tab {
            FirewallSubTab::Policies => self.policy_table.selected().unwrap_or(0),
            FirewallSubTab::Zones => self.zone_table.selected().unwrap_or(0),
            FirewallSubTab::AclRules => self.acl_table.selected().unwrap_or(0),
        }
    }

    pub(super) fn select(&mut self, index: usize) {
        let len = self.active_len();
        let clamped = if len == 0 { 0 } else { index.min(len - 1) };
        self.active_table_state().select(Some(clamped));
    }

    #[allow(clippy::cast_sign_loss, clippy::as_conversions)]
    pub(super) fn move_selection(&mut self, delta: isize) {
        let len = self.active_len();
        if len == 0 {
            return;
        }

        #[allow(clippy::cast_possible_wrap)]
        let current = self.selected_index() as isize;
        #[allow(clippy::cast_possible_wrap)]
        let next = (current + delta).clamp(0, len as isize - 1);
        self.select(next as usize);
    }

    pub(super) fn sub_tab_index(&self) -> usize {
        match self.sub_tab {
            FirewallSubTab::Policies => 0,
            FirewallSubTab::Zones => 1,
            FirewallSubTab::AclRules => 2,
        }
    }

    pub(super) fn apply_action(&mut self, action: &Action) {
        match action {
            Action::FirewallPoliciesUpdated(policies) => {
                self.policies = Arc::clone(policies);
            }
            Action::FirewallZonesUpdated(zones) => {
                self.zones = Arc::clone(zones);
            }
            Action::AclRulesUpdated(rules) => {
                self.acl_rules = Arc::clone(rules);
            }
            Action::FirewallSubTab(tab) => {
                self.sub_tab = *tab;
            }
            Action::ReorderPolicy(index, direction) => {
                let len = self.policies.len();
                if len < 2 {
                    return;
                }

                let target = match direction {
                    Direction::Up if *index > 0 => index - 1,
                    Direction::Down if *index + 1 < len => index + 1,
                    _ => return,
                };

                let policies = Arc::make_mut(&mut self.policies);
                policies.swap(*index, target);
                self.select(target);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use unifly_api::model::{AclRule, FirewallPolicy, FirewallZone};

    fn policy(name: &str) -> Arc<FirewallPolicy> {
        Arc::new(
            serde_json::from_value(serde_json::json!({
                "id": name,
                "name": name,
                "description": null,
                "enabled": true,
                "index": 1,
                "action": "Allow",
                "ip_version": "Both",
                "source": { "zone_id": null, "filter": null },
                "destination": { "zone_id": null, "filter": null },
                "source_summary": "src",
                "destination_summary": "dst",
                "protocol_summary": "Any",
                "schedule": null,
                "ipsec_mode": null,
                "connection_states": [],
                "logging_enabled": false,
                "origin": null
            }))
            .unwrap(),
        )
    }

    fn zone(name: &str) -> Arc<FirewallZone> {
        Arc::new(
            serde_json::from_value(serde_json::json!({
                "id": name,
                "name": name,
                "network_ids": [],
                "origin": null
            }))
            .unwrap(),
        )
    }

    fn acl_rule(name: &str) -> Arc<AclRule> {
        Arc::new(
            serde_json::from_value(serde_json::json!({
                "id": name,
                "name": name,
                "enabled": true,
                "rule_type": "Ipv4",
                "action": "Allow",
                "source_summary": "src",
                "destination_summary": "dst",
                "origin": null
            }))
            .unwrap(),
        )
    }

    #[test]
    fn select_clamps_to_active_tab_length() {
        let mut screen = FirewallScreen::new();
        screen.zones = Arc::new(vec![zone("lan"), zone("iot")]);
        screen.sub_tab = FirewallSubTab::Zones;

        screen.select(99);

        assert_eq!(screen.zone_table.selected(), Some(1));
        assert_eq!(screen.policy_table.selected(), None);
    }

    #[test]
    fn reorder_policy_swaps_items_and_updates_selection() {
        let mut screen = FirewallScreen::new();
        screen.policies = Arc::new(vec![policy("one"), policy("two"), policy("three")]);
        screen.policy_table.select(Some(1));

        screen.apply_action(&Action::ReorderPolicy(1, Direction::Down));

        let names: Vec<_> = screen
            .policies
            .iter()
            .map(|policy| policy.name.as_str())
            .collect();
        assert_eq!(names, vec!["one", "three", "two"]);
        assert_eq!(screen.policy_table.selected(), Some(2));
    }

    #[test]
    fn selected_index_tracks_active_tab() {
        let mut screen = FirewallScreen::new();
        screen.policy_table.select(Some(2));
        screen.zone_table.select(Some(1));
        screen.acl_rules = Arc::new(vec![
            acl_rule("one"),
            acl_rule("two"),
            acl_rule("three"),
            acl_rule("four"),
        ]);
        screen.acl_table.select(Some(3));

        screen.sub_tab = FirewallSubTab::Policies;
        assert_eq!(screen.selected_index(), 2);

        screen.sub_tab = FirewallSubTab::Zones;
        assert_eq!(screen.selected_index(), 1);

        screen.sub_tab = FirewallSubTab::AclRules;
        assert_eq!(screen.selected_index(), 3);
    }
}
