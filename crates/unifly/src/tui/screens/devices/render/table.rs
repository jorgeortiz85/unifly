use std::sync::Arc;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table};

use unifly_api::{Device, DeviceState};

use crate::tui::theme;
use crate::tui::widgets::{bytes_fmt, status_indicator};

use super::detail::render_detail;
use crate::tui::screens::devices::DevicesScreen;

pub(super) fn render_screen(screen: &DevicesScreen, frame: &mut Frame, area: Rect) {
    let filtered = screen.filtered_devices();
    let selected_index = screen.selected_row_index(&filtered);
    let total = screen.devices.len();
    let shown = filtered.len();
    let title = if screen.search_query.is_empty() {
        format!(" Devices ({total}) ")
    } else {
        format!(" Devices ({shown}/{total}) ")
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

    let header_layout = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .split(table_area);

    render_filter_bar(screen, frame, header_layout[0], shown);
    render_device_table(screen, frame, header_layout[1], &filtered, selected_index);
    render_table_hints(frame, header_layout[2]);

    if let Some(detail_area) = detail_area
        && let Some(device) = screen.detail_device()
    {
        render_detail(screen, frame, detail_area, device);
    }
}

fn render_filter_bar(screen: &DevicesScreen, frame: &mut Frame, area: Rect, shown: usize) {
    let filter_text = if screen.search_query.is_empty() {
        Span::styled("[all]", Style::default().fg(theme::accent_secondary()))
    } else {
        Span::styled(
            format!("[\"{}\" ]", screen.search_query),
            Style::default().fg(theme::warning()),
        )
    };
    let filter_line = Line::from(vec![
        Span::styled(" Filter: ", Style::default().fg(theme::text_secondary())),
        filter_text,
        Span::styled("  Sort: ", Style::default().fg(theme::text_secondary())),
        Span::styled("[name ↑]", Style::default().fg(theme::accent_secondary())),
        Span::styled(
            format!("  {:>width$}", format!("{shown} devices"), width = 20),
            Style::default().fg(theme::text_secondary()),
        ),
    ]);
    frame.render_widget(Paragraph::new(filter_line), area);
}

fn render_device_table(
    screen: &DevicesScreen,
    frame: &mut Frame,
    area: Rect,
    filtered: &[&Arc<Device>],
    selected_index: Option<usize>,
) {
    let header = Row::new(vec![
        Cell::from("Status").style(theme::table_header()),
        Cell::from("Name").style(theme::table_header()),
        Cell::from("Model").style(theme::table_header()),
        Cell::from("IP").style(theme::table_header()),
        Cell::from("CPU").style(theme::table_header()),
        Cell::from("Mem").style(theme::table_header()),
        Cell::from("TX/RX").style(theme::table_header()),
        Cell::from("Uptime").style(theme::table_header()),
    ]);

    let rows: Vec<Row> = filtered
        .iter()
        .enumerate()
        .map(|(index, device)| render_table_row(index, selected_index, device))
        .collect();

    let widths = [
        Constraint::Length(3),
        Constraint::Min(14),
        Constraint::Length(12),
        Constraint::Length(15),
        Constraint::Length(7),
        Constraint::Length(7),
        Constraint::Length(11),
        Constraint::Length(8),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .row_highlight_style(theme::table_selected());

    let mut state = screen.table_state;
    frame.render_stateful_widget(table, area, &mut state);
}

fn render_table_row(
    index: usize,
    selected_index: Option<usize>,
    device: &Arc<Device>,
) -> Row<'static> {
    let is_selected = Some(index) == selected_index;
    let prefix = if is_selected { "▸" } else { " " };
    let status = status_indicator::status_char(device.state);
    let name = device.name.as_deref().unwrap_or("Unknown");
    let model = device.model.as_deref().unwrap_or("─");
    let ip = device.ip.map_or_else(|| "─".into(), |ip| ip.to_string());
    let cpu = device
        .stats
        .cpu_utilization_pct
        .map_or_else(|| "·····".into(), |value| format!("{value:.0}%"));
    let mem = device
        .stats
        .memory_utilization_pct
        .map_or_else(|| "·····".into(), |value| format!("{value:.0}%"));
    let traffic = device.stats.uplink_bandwidth.as_ref().map_or_else(
        || "···/···".into(),
        |bandwidth| bytes_fmt::fmt_tx_rx(bandwidth.tx_bytes_per_sec, bandwidth.rx_bytes_per_sec),
    );
    let uptime = device
        .stats
        .uptime_secs
        .map_or_else(|| "···".into(), bytes_fmt::fmt_uptime);

    let status_color = match device.state {
        DeviceState::Online => theme::success(),
        DeviceState::Offline | DeviceState::ConnectionInterrupted | DeviceState::Isolated => {
            theme::error()
        }
        DeviceState::PendingAdoption => theme::accent_primary(),
        _ => theme::warning(),
    };

    let row_style = if is_selected {
        theme::table_selected()
    } else {
        theme::table_row()
    };

    Row::new(vec![
        Cell::from(format!("{prefix}{status}")).style(Style::default().fg(status_color)),
        Cell::from(name.to_string()).style(
            Style::default()
                .fg(theme::accent_secondary())
                .add_modifier(if is_selected {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
        ),
        Cell::from(model.to_string()),
        Cell::from(ip).style(Style::default().fg(theme::accent_tertiary())),
        Cell::from(cpu),
        Cell::from(mem),
        Cell::from(traffic),
        Cell::from(uptime),
    ])
    .style(row_style)
}

fn render_table_hints(frame: &mut Frame, area: Rect) {
    let hints = Line::from(vec![
        Span::styled("  j/k ", theme::key_hint_key()),
        Span::styled("navigate  ", theme::key_hint()),
        Span::styled("Enter ", theme::key_hint_key()),
        Span::styled("detail  ", theme::key_hint()),
        Span::styled("R ", theme::key_hint_key()),
        Span::styled("restart  ", theme::key_hint()),
        Span::styled("L ", theme::key_hint_key()),
        Span::styled("locate", theme::key_hint()),
    ]);
    frame.render_widget(Paragraph::new(hints), area);
}
