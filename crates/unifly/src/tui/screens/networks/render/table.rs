use std::sync::Arc;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table};

use unifly_api::Network;

use crate::tui::screens::networks::NetworksScreen;
use crate::tui::theme;

use super::detail::render_detail;
use super::edit::render_edit_overlay;

pub(super) fn render_screen(screen: &NetworksScreen, frame: &mut Frame, area: Rect) {
    let count = screen.networks.len();
    let title = format!(" Networks ({count}) ");
    let block = Block::default()
        .title(title)
        .title_style(theme::title_style())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(if screen.focused {
            theme::border_focused()
        } else {
            theme::border_default()
        });

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let (table_area, detail_area) = if screen.detail_open {
        let chunks =
            Layout::vertical([Constraint::Percentage(45), Constraint::Percentage(55)]).split(inner);
        (chunks[0], Some(chunks[1]))
    } else {
        (inner, None)
    };

    let layout = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(table_area);

    let header = Row::new(vec![
        Cell::from("Name").style(theme::table_header()),
        Cell::from("VLAN").style(theme::table_header()),
        Cell::from("Gateway").style(theme::table_header()),
        Cell::from("Subnet").style(theme::table_header()),
        Cell::from("DHCP").style(theme::table_header()),
        Cell::from("Type").style(theme::table_header()),
        Cell::from("IPv6").style(theme::table_header()),
    ]);

    let selected_idx = screen.selected_index();
    let rows: Vec<Row> = screen
        .networks
        .iter()
        .enumerate()
        .map(|(index, network)| render_table_row(index, selected_idx, network))
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

    let mut state = screen.table_state;
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

    if let Some(detail_area) = detail_area {
        if let Some(network) = screen.networks.get(selected_idx) {
            render_detail(frame, detail_area, network);
        }
    }

    if let Some(edit) = screen.edit_state.as_ref() {
        render_edit_overlay(frame, area, edit);
    }
}

fn render_table_row(index: usize, selected_idx: usize, network: &Arc<Network>) -> Row<'static> {
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
