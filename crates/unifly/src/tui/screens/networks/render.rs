use std::sync::Arc;

use super::NetworksScreen;
use super::state::NetworkEditState;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Table};

use unifly_api::Network;

use crate::tui::theme;

impl NetworksScreen {
    pub(super) fn render_screen(&self, frame: &mut Frame, area: Rect) {
        let count = self.networks.len();
        let title = format!(" Networks ({count}) ");
        let block = Block::default()
            .title(title)
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

        let (table_area, detail_area) = if self.detail_open {
            let chunks = Layout::vertical([Constraint::Percentage(45), Constraint::Percentage(55)])
                .split(inner);
            (chunks[0], Some(chunks[1]))
        } else {
            (inner, None)
        };

        let layout =
            Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(table_area);

        let header = Row::new(vec![
            Cell::from("Name").style(theme::table_header()),
            Cell::from("VLAN").style(theme::table_header()),
            Cell::from("Gateway").style(theme::table_header()),
            Cell::from("Subnet").style(theme::table_header()),
            Cell::from("DHCP").style(theme::table_header()),
            Cell::from("Type").style(theme::table_header()),
            Cell::from("IPv6").style(theme::table_header()),
        ]);

        let selected_idx = self.selected_index();
        let rows: Vec<Row> = self
            .networks
            .iter()
            .enumerate()
            .map(|(index, network)| self.render_table_row(index, selected_idx, network))
            .collect();

        let widths = [
            Constraint::Min(14),
            Constraint::Length(6),
            Constraint::Length(16),
            Constraint::Length(18),
            Constraint::Length(5),
            Constraint::Length(10),
            Constraint::Length(10),
        ];

        let table = Table::new(rows, widths)
            .header(header)
            .row_highlight_style(theme::table_selected());

        let mut state = self.table_state;
        frame.render_stateful_widget(table, layout[0], &mut state);

        let hints = Line::from(vec![
            Span::styled("  j/k ", theme::key_hint_key()),
            Span::styled("navigate  ", theme::key_hint()),
            Span::styled("Enter ", theme::key_hint_key()),
            Span::styled("expand  ", theme::key_hint()),
            Span::styled("e ", theme::key_hint_key()),
            Span::styled("edit  ", theme::key_hint()),
            Span::styled("Esc ", theme::key_hint_key()),
            Span::styled("collapse", theme::key_hint()),
        ]);
        frame.render_widget(Paragraph::new(hints), layout[1]);

        if let Some(detail_area) = detail_area
            && let Some(network) = self.networks.get(selected_idx)
        {
            self.render_detail(frame, detail_area, network);
        }

        if let Some(edit) = self.edit_state.as_ref() {
            self.render_edit_overlay(frame, area, edit);
        }
    }

    fn render_table_row(
        &self,
        index: usize,
        selected_idx: usize,
        network: &Arc<Network>,
    ) -> Row<'_> {
        let is_selected = index == selected_idx;
        let prefix = if is_selected { "▸" } else { " " };

        let vlan = network
            .vlan_id
            .map_or_else(|| "—".into(), |value| value.to_string());
        let gateway = network
            .gateway_ip
            .map_or_else(|| "—".into(), |ip| ip.to_string());
        let subnet = network.subnet.as_deref().unwrap_or("—");
        let dhcp = network
            .dhcp
            .as_ref()
            .map_or("—", |dhcp| if dhcp.enabled { "On" } else { "Off" });
        let management = network
            .management
            .as_ref()
            .map_or_else(|| "—".into(), |value| format!("{value:?}"));
        let ipv6 = if network.ipv6_enabled {
            network
                .ipv6_mode
                .as_ref()
                .map_or_else(|| "On".into(), ToString::to_string)
        } else {
            "Off".into()
        };

        let row_style = if is_selected {
            theme::table_selected()
        } else {
            theme::table_row()
        };

        Row::new(vec![
            Cell::from(format!("{prefix}{}", network.name)).style(
                Style::default()
                    .fg(theme::accent_secondary())
                    .add_modifier(if is_selected {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
            ),
            Cell::from(vlan),
            Cell::from(gateway).style(Style::default().fg(theme::accent_tertiary())),
            Cell::from(subnet.to_string()).style(Style::default().fg(theme::accent_tertiary())),
            Cell::from(dhcp),
            Cell::from(management),
            Cell::from(ipv6),
        ])
        .style(row_style)
    }

    #[allow(clippy::unused_self, clippy::too_many_lines)]
    fn render_detail(&self, frame: &mut Frame, area: Rect, network: &Network) {
        let block = Block::default()
            .title(format!(" {} — Detail ", network.name))
            .title_style(theme::title_style())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(theme::border_focused());

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.height < 3 || inner.width < 20 {
            return;
        }

        let label = Style::default().fg(theme::text_secondary());
        let value = Style::default().fg(theme::accent_secondary());
        let enabled_style = Style::default().fg(theme::success());
        let disabled_style = Style::default().fg(theme::border_unfocused());

        let gateway_str = network
            .gateway_ip
            .map_or_else(|| "—".into(), |ip| ip.to_string());
        let subnet_str = network.subnet.as_deref().unwrap_or("—");
        let vlan_str = network
            .vlan_id
            .map_or_else(|| "—".into(), |value| value.to_string());
        let management_str = network
            .management
            .as_ref()
            .map_or_else(|| "—".into(), |value| format!("{value:?}"));

        let (dhcp_status, dhcp_style) = network.dhcp.as_ref().map_or(("—", label), |dhcp| {
            if dhcp.enabled {
                ("Enabled", enabled_style)
            } else {
                ("Disabled", disabled_style)
            }
        });

        let dhcp_range = network
            .dhcp
            .as_ref()
            .filter(|dhcp| dhcp.enabled)
            .map(|dhcp| {
                let start = dhcp
                    .range_start
                    .map_or_else(|| "?".into(), |ip| ip.to_string());
                let stop = dhcp
                    .range_stop
                    .map_or_else(|| "?".into(), |ip| ip.to_string());
                format!("{start} — {stop}")
            });

        let lease_str = network
            .dhcp
            .as_ref()
            .and_then(|dhcp| dhcp.lease_time_secs)
            .map(super::state::format_lease_time);

        let dns_str = network
            .dhcp
            .as_ref()
            .filter(|dhcp| !dhcp.dns_servers.is_empty())
            .map(|dhcp| {
                dhcp.dns_servers
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            });

        let bool_span = |value: bool, enabled: &str, disabled: &str| -> Span<'static> {
            if value {
                Span::styled(enabled.to_string(), enabled_style)
            } else {
                Span::styled(disabled.to_string(), disabled_style)
            }
        };

        let ipv6_str = if network.ipv6_enabled {
            network
                .ipv6_mode
                .as_ref()
                .map_or_else(|| "Enabled".into(), ToString::to_string)
        } else {
            "Disabled".into()
        };
        let ipv6_style = if network.ipv6_enabled {
            enabled_style
        } else {
            disabled_style
        };

        let mut lines = vec![
            Line::from(Span::styled(
                " Network Config",
                Style::default()
                    .fg(theme::accent_primary())
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                " ─────────────────────────────────────────",
                Style::default().fg(theme::border_unfocused()),
            )),
            Line::from(vec![
                Span::styled("  Gateway IP    ", label),
                Span::styled(gateway_str, value),
                Span::styled("       VLAN          ", label),
                Span::styled(vlan_str, value),
            ]),
            Line::from(vec![
                Span::styled("  Subnet        ", label),
                Span::styled(subnet_str.to_string(), value),
                Span::styled("       Management    ", label),
                Span::styled(management_str, value),
            ]),
        ];

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " DHCP Server",
            Style::default()
                .fg(theme::accent_primary())
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            " ─────────────────────────────────────────",
            Style::default().fg(theme::border_unfocused()),
        )));
        lines.push(Line::from(vec![
            Span::styled("  DHCP          ", label),
            Span::styled(dhcp_status.to_string(), dhcp_style),
            if let Some(ref lease) = lease_str {
                Span::styled(format!("       Lease Time    {lease}"), label)
            } else {
                Span::raw("")
            },
        ]));

        if let Some(ref range) = dhcp_range {
            lines.push(Line::from(vec![
                Span::styled("  Range         ", label),
                Span::styled(range.clone(), value),
            ]));
        }

        if let Some(ref dns) = dns_str {
            lines.push(Line::from(vec![
                Span::styled("  DNS           ", label),
                Span::styled(dns.clone(), value),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " Features",
            Style::default()
                .fg(theme::accent_primary())
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            " ─────────────────────────────────────────",
            Style::default().fg(theme::border_unfocused()),
        )));
        lines.push(Line::from(vec![
            Span::styled("  Internet      ", label),
            bool_span(network.internet_access_enabled, "Enabled", "Disabled"),
            Span::styled("       Isolation     ", label),
            bool_span(network.isolation_enabled, "Enabled", "Disabled"),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  mDNS Fwd      ", label),
            bool_span(network.mdns_forwarding_enabled, "Enabled", "Disabled"),
            Span::styled("       IPv6          ", label),
            Span::styled(ipv6_str, ipv6_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Cellular BU   ", label),
            bool_span(network.cellular_backup_enabled, "Enabled", "Disabled"),
        ]));

        frame.render_widget(Paragraph::new(lines), inner);
    }

    #[allow(clippy::unused_self)]
    fn render_edit_overlay(&self, frame: &mut Frame, area: Rect, edit: &NetworkEditState) {
        let overlay_w = 44u16.min(area.width.saturating_sub(4));
        #[allow(clippy::cast_possible_truncation, clippy::as_conversions)]
        let overlay_h =
            (NetworkEditState::FIELD_COUNT as u16 + 6).min(area.height.saturating_sub(2));
        let x = area.x + (area.width.saturating_sub(overlay_w)) / 2;
        let y = area.y + (area.height.saturating_sub(overlay_h)) / 2;
        let overlay_area = Rect::new(x, y, overlay_w, overlay_h);

        frame.render_widget(Clear, overlay_area);

        let block = Block::default()
            .title(" Edit Network ")
            .title_style(
                Style::default()
                    .fg(theme::warning())
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(theme::accent_primary()));

        let inner = block.inner(overlay_area);
        frame.render_widget(block, overlay_area);

        let label = Style::default().fg(theme::text_secondary());
        let value_style = Style::default().fg(theme::accent_secondary());
        let focused_label = Style::default()
            .fg(theme::warning())
            .add_modifier(Modifier::BOLD);
        let enabled_style = Style::default().fg(theme::success());
        let disabled_style = Style::default().fg(theme::border_unfocused());

        let mut lines = Vec::new();

        for index in 0..NetworkEditState::FIELD_COUNT {
            let is_focused = index == edit.field_idx;
            let label_style = if is_focused { focused_label } else { label };
            let marker = if is_focused { "▸ " } else { "  " };
            let field_label = NetworkEditState::field_label(index);
            let field_value = edit.field_value(index);

            let field_style = if NetworkEditState::is_bool_field(index) {
                if matches!(field_value.as_str(), "Enabled") {
                    enabled_style
                } else {
                    disabled_style
                }
            } else {
                value_style
            };

            let cursor = if is_focused && !NetworkEditState::is_bool_field(index) {
                "▎"
            } else {
                ""
            };

            lines.push(Line::from(vec![
                Span::styled(marker, label_style),
                Span::styled(format!("{field_label:<14}"), label_style),
                Span::styled(field_value, field_style),
                Span::styled(cursor, Style::default().fg(theme::warning())),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(" Tab", theme::key_hint_key()),
            Span::styled(" next  ", theme::key_hint()),
            Span::styled("Space", theme::key_hint_key()),
            Span::styled(" toggle  ", theme::key_hint()),
            Span::styled("Enter", theme::key_hint_key()),
            Span::styled(" save  ", theme::key_hint()),
            Span::styled("Esc", theme::key_hint_key()),
            Span::styled(" cancel", theme::key_hint()),
        ]));

        frame.render_widget(Paragraph::new(lines), inner);
    }
}
