use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table};

use crate::tui::action::FirewallSubTab;
use crate::tui::theme;
use crate::tui::widgets::sub_tabs;

use super::FirewallScreen;

impl FirewallScreen {
    fn render_policies(&self, frame: &mut Frame, area: Rect) {
        let header = Row::new(vec![
            Cell::from("#").style(theme::table_header()),
            Cell::from("Enabled").style(theme::table_header()),
            Cell::from("Name").style(theme::table_header()),
            Cell::from("Action").style(theme::table_header()),
            Cell::from("Protocol").style(theme::table_header()),
            Cell::from("Source").style(theme::table_header()),
            Cell::from("Destination").style(theme::table_header()),
        ]);

        let selected_idx = self.policy_table.selected().unwrap_or(0);
        let rows: Vec<Row> = self
            .policies
            .iter()
            .enumerate()
            .map(|(index, policy)| {
                let is_selected = index == selected_idx;
                let prefix = if is_selected { "▸" } else { " " };

                let display_index = policy
                    .index
                    .map_or_else(|| (index + 1).to_string(), |value| value.to_string());
                let enabled = if policy.enabled { "✓" } else { "✗" };
                let action_str = format!("{:?}", policy.action);
                let action_color = match policy.action {
                    unifly_api::model::FirewallAction::Allow => theme::success(),
                    unifly_api::model::FirewallAction::Block => theme::error(),
                    unifly_api::model::FirewallAction::Reject => theme::accent_tertiary(),
                };
                let protocol = policy.protocol_summary.as_deref().unwrap_or("Any");
                let src = policy.source_summary.as_deref().unwrap_or("─");
                let dst = policy.destination_summary.as_deref().unwrap_or("─");

                let row_style = if is_selected {
                    theme::table_selected()
                } else {
                    theme::table_row()
                };

                Row::new(vec![
                    Cell::from(format!("{prefix}{display_index}")),
                    Cell::from(enabled.to_string()).style(Style::default().fg(if policy.enabled {
                        theme::success()
                    } else {
                        theme::border_unfocused()
                    })),
                    Cell::from(policy.name.clone()).style(
                        Style::default().fg(theme::accent_secondary()).add_modifier(
                            if is_selected {
                                Modifier::BOLD
                            } else {
                                Modifier::empty()
                            },
                        ),
                    ),
                    Cell::from(action_str).style(Style::default().fg(action_color)),
                    Cell::from(protocol.to_string()),
                    Cell::from(src.to_string()),
                    Cell::from(dst.to_string()),
                ])
                .style(row_style)
            })
            .collect();

        let widths = [
            Constraint::Length(4),
            Constraint::Length(7),
            Constraint::Min(16),
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Length(14),
            Constraint::Length(14),
        ];

        let table = Table::new(rows, widths)
            .header(header)
            .row_highlight_style(theme::table_selected());

        let mut state = self.policy_table;
        frame.render_stateful_widget(table, area, &mut state);
    }

    fn render_zones(&self, frame: &mut Frame, area: Rect) {
        let header = Row::new(vec![
            Cell::from("Name").style(theme::table_header()),
            Cell::from("Networks").style(theme::table_header()),
        ]);

        let selected_idx = self.zone_table.selected().unwrap_or(0);
        let rows: Vec<Row> = self
            .zones
            .iter()
            .enumerate()
            .map(|(index, zone)| {
                let is_selected = index == selected_idx;
                let prefix = if is_selected { "▸" } else { " " };
                let network_count = zone.network_ids.len();

                let row_style = if is_selected {
                    theme::table_selected()
                } else {
                    theme::table_row()
                };

                Row::new(vec![
                    Cell::from(format!("{prefix}{}", zone.name)).style(
                        Style::default().fg(theme::accent_secondary()).add_modifier(
                            if is_selected {
                                Modifier::BOLD
                            } else {
                                Modifier::empty()
                            },
                        ),
                    ),
                    Cell::from(format!("{network_count} networks")),
                ])
                .style(row_style)
            })
            .collect();

        let widths = [Constraint::Min(20), Constraint::Length(14)];

        let table = Table::new(rows, widths)
            .header(header)
            .row_highlight_style(theme::table_selected());

        let mut state = self.zone_table;
        frame.render_stateful_widget(table, area, &mut state);
    }

    fn render_acl_rules(&self, frame: &mut Frame, area: Rect) {
        let header = Row::new(vec![
            Cell::from("Name").style(theme::table_header()),
            Cell::from("Enabled").style(theme::table_header()),
            Cell::from("Type").style(theme::table_header()),
            Cell::from("Action").style(theme::table_header()),
            Cell::from("Source").style(theme::table_header()),
            Cell::from("Destination").style(theme::table_header()),
        ]);

        let selected_idx = self.acl_table.selected().unwrap_or(0);
        let rows: Vec<Row> = self
            .acl_rules
            .iter()
            .enumerate()
            .map(|(index, rule)| {
                let is_selected = index == selected_idx;
                let prefix = if is_selected { "▸" } else { " " };
                let enabled = if rule.enabled { "✓" } else { "✗" };
                let rule_type = format!("{:?}", rule.rule_type);
                let action_str = format!("{:?}", rule.action);
                let action_color = match rule.action {
                    unifly_api::model::AclAction::Allow => theme::success(),
                    unifly_api::model::AclAction::Block => theme::error(),
                };
                let src = rule.source_summary.as_deref().unwrap_or("─");
                let dst = rule.destination_summary.as_deref().unwrap_or("─");

                let row_style = if is_selected {
                    theme::table_selected()
                } else {
                    theme::table_row()
                };

                Row::new(vec![
                    Cell::from(format!("{prefix}{}", rule.name)).style(
                        Style::default().fg(theme::accent_secondary()).add_modifier(
                            if is_selected {
                                Modifier::BOLD
                            } else {
                                Modifier::empty()
                            },
                        ),
                    ),
                    Cell::from(enabled.to_string()),
                    Cell::from(rule_type),
                    Cell::from(action_str).style(Style::default().fg(action_color)),
                    Cell::from(src.to_string()),
                    Cell::from(dst.to_string()),
                ])
                .style(row_style)
            })
            .collect();

        let widths = [
            Constraint::Min(16),
            Constraint::Length(7),
            Constraint::Length(6),
            Constraint::Length(8),
            Constraint::Length(14),
            Constraint::Length(14),
        ];

        let table = Table::new(rows, widths)
            .header(header)
            .row_highlight_style(theme::table_selected());

        let mut state = self.acl_table;
        frame.render_stateful_widget(table, area, &mut state);
    }

    pub(super) fn render_screen(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" Firewall ")
            .title_style(theme::title_style())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(if self.focused {
                theme::border_focused()
            } else {
                theme::border_default()
            });

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(inner);

        let tab_labels = &["Policies", "Zones", "ACL Rules"];
        let tab_line = sub_tabs::render_sub_tabs(tab_labels, self.sub_tab_index());
        frame.render_widget(Paragraph::new(tab_line), layout[0]);

        match self.sub_tab {
            FirewallSubTab::Policies => self.render_policies(frame, layout[1]),
            FirewallSubTab::Zones => self.render_zones(frame, layout[1]),
            FirewallSubTab::AclRules => self.render_acl_rules(frame, layout[1]),
        }

        frame.render_widget(Paragraph::new(self.hint_line()), layout[2]);
    }

    fn hint_line(&self) -> Line<'static> {
        match self.sub_tab {
            FirewallSubTab::Policies => Line::from(vec![
                Span::styled("  j/k ", theme::key_hint_key()),
                Span::styled("navigate  ", theme::key_hint()),
                Span::styled("K/J ", theme::key_hint_key()),
                Span::styled("reorder  ", theme::key_hint()),
                Span::styled("h/l ", theme::key_hint_key()),
                Span::styled("sub-tab  ", theme::key_hint()),
                Span::styled("Enter ", theme::key_hint_key()),
                Span::styled("detail", theme::key_hint()),
            ]),
            _ => Line::from(vec![
                Span::styled("  j/k ", theme::key_hint_key()),
                Span::styled("navigate  ", theme::key_hint()),
                Span::styled("h/l ", theme::key_hint_key()),
                Span::styled("sub-tab  ", theme::key_hint()),
                Span::styled("Enter ", theme::key_hint_key()),
                Span::styled("detail", theme::key_hint()),
            ]),
        }
    }
}
