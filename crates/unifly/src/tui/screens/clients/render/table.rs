use super::super::ClientsScreen;
use super::detail::render_detail;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table};

use unifly_api::{Client, ClientType};

use crate::tui::theme;
use crate::tui::widgets::{bytes_fmt, sub_tabs};

pub(super) fn render_screen(screen: &ClientsScreen, frame: &mut Frame, area: Rect) {
    let filtered = screen.filtered_clients();
    let total = screen.clients.len();
    let shown = filtered.len();
    let title = if screen.search_query.is_empty() {
        format!(" Clients ({shown}/{total}) ")
    } else {
        format!(" Clients ({shown}/{total}) [\"{}\" ] ", screen.search_query)
    };

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
            Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).split(inner);
        (chunks[0], Some(chunks[1]))
    } else {
        (inner, None)
    };

    let layout = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .split(table_area);

    let filter_labels = &["All", "Wireless", "Wired", "VPN", "Guest"];
    let filter_line = sub_tabs::render_sub_tabs(filter_labels, screen.filter_index());
    frame.render_widget(Paragraph::new(filter_line), layout[0]);

    let header = Row::new(vec![
        Cell::from("Type").style(theme::table_header()),
        Cell::from("Name").style(theme::table_header()),
        Cell::from("IP").style(theme::table_header()),
        Cell::from("MAC").style(theme::table_header()),
        Cell::from("Signal").style(theme::table_header()),
        Cell::from("TX/RX").style(theme::table_header()),
        Cell::from("Duration").style(theme::table_header()),
    ]);

    let selected_idx = if screen.detail_open {
        screen
            .detail_client_index(&filtered)
            .unwrap_or_else(|| screen.selected_index())
    } else {
        screen.selected_index()
    };

    let rows: Vec<Row> = filtered
        .iter()
        .enumerate()
        .map(|(index, client)| render_table_row(screen, index, selected_idx, client))
        .collect();

    let widths = [
        Constraint::Length(3),
        Constraint::Min(14),
        Constraint::Length(15),
        Constraint::Length(17),
        Constraint::Length(6),
        Constraint::Length(11),
        Constraint::Length(8),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .row_highlight_style(theme::table_selected());

    let mut state = screen.table_state.clone();
    frame.render_stateful_widget(table, layout[1], &mut state);

    let hints = Line::from(vec![
        Span::styled("  j/k ", theme::key_hint_key()),
        Span::styled("navigate  ", theme::key_hint()),
        Span::styled("Enter ", theme::key_hint_key()),
        Span::styled("detail  ", theme::key_hint()),
        Span::styled("Tab ", theme::key_hint_key()),
        Span::styled("filter  ", theme::key_hint()),
        Span::styled("b ", theme::key_hint_key()),
        Span::styled("block  ", theme::key_hint()),
        Span::styled("x ", theme::key_hint_key()),
        Span::styled("kick", theme::key_hint()),
    ]);
    frame.render_widget(Paragraph::new(hints), layout[2]);

    if let Some(detail_area) = detail_area
        && let Some(client) = screen.detail_client(&filtered)
    {
        render_detail(frame, detail_area, client);
    }
}

fn render_table_row(
    screen: &ClientsScreen,
    index: usize,
    selected_idx: usize,
    client: &Client,
) -> Row<'static> {
    let is_selected = index == selected_idx;
    let prefix = if is_selected { "▸" } else { " " };

    let type_char = match client.client_type {
        ClientType::Wireless => "W",
        ClientType::Wired => "E",
        ClientType::Vpn => "V",
        ClientType::Teleport => "T",
        _ => "?",
    };
    let type_str = if client.is_guest {
        format!("{prefix}G")
    } else {
        format!("{prefix}{type_char}")
    };

    let name = client
        .name
        .as_deref()
        .or(client.hostname.as_deref())
        .unwrap_or("unknown");
    let ip = client
        .ip
        .map_or_else(|| "─".into(), |client_ip| client_ip.to_string());
    let mac = client.mac.to_string();
    let signal = client
        .wireless
        .as_ref()
        .and_then(|wireless| wireless.signal_dbm)
        .map_or("····", |dbm| {
            if dbm >= -50 {
                "▂▄▆█"
            } else if dbm >= -60 {
                "▂▄▆ "
            } else if dbm >= -70 {
                "▂▄  "
            } else if dbm >= -80 {
                "▂   "
            } else {
                "·   "
            }
        });
    let traffic = bytes_fmt::fmt_tx_rx(client.tx_bytes.unwrap_or(0), client.rx_bytes.unwrap_or(0));
    let duration = client.connected_at.map_or_else(
        || "─".into(),
        |ts| {
            let dur = chrono::Utc::now().signed_duration_since(ts);
            #[allow(clippy::cast_sign_loss)]
            let secs = dur.num_seconds().max(0) as u64;
            bytes_fmt::fmt_uptime(secs)
        },
    );

    let type_color = if client.is_guest {
        theme::warning()
    } else {
        match client.client_type {
            ClientType::Wireless => theme::accent_secondary(),
            ClientType::Vpn => theme::accent_primary(),
            ClientType::Teleport => theme::accent_tertiary(),
            _ => theme::text_secondary(),
        }
    };

    let signal_color = client
        .wireless
        .as_ref()
        .and_then(|wireless| wireless.signal_dbm)
        .map_or(theme::border_unfocused(), |dbm| {
            if dbm >= -50 {
                theme::success()
            } else if dbm >= -60 {
                theme::accent_secondary()
            } else if dbm >= -70 {
                theme::warning()
            } else if dbm >= -80 {
                theme::accent_tertiary()
            } else {
                theme::error()
            }
        });

    let row_style = if is_selected {
        theme::table_selected()
    } else {
        theme::table_row()
    };

    Row::new(vec![
        Cell::from(type_str).style(Style::default().fg(type_color)),
        Cell::from(name.to_string()).style(
            Style::default()
                .fg(theme::accent_secondary())
                .add_modifier(if is_selected {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
        ),
        Cell::from(ip).style(Style::default().fg(theme::accent_tertiary())),
        Cell::from(mac),
        Cell::from(signal.to_string()).style(Style::default().fg(signal_color)),
        Cell::from(traffic),
        Cell::from(duration),
    ])
    .style(if screen.detail_open && is_selected {
        theme::table_selected()
    } else {
        row_style
    })
}
