use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

use unifly_api::Network;

use crate::tui::theme;

pub(super) fn render_detail(frame: &mut Frame, area: Rect, network: &Network) {
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
        .map(super::super::state::format_lease_time);

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
