use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

use unifly_api::Device;

use crate::tui::action::DeviceDetailTab;
use crate::tui::theme;
use crate::tui::widgets::{bytes_fmt, status_indicator, sub_tabs};

use crate::tui::screens::devices::DevicesScreen;

pub(super) fn render_detail(
    screen: &DevicesScreen,
    frame: &mut Frame,
    area: Rect,
    device: &Device,
) {
    let name = device.name.as_deref().unwrap_or("Unknown");
    let model = device.model.as_deref().unwrap_or("─");
    let ip = device
        .ip
        .map_or_else(|| "─".into(), |device_ip| device_ip.to_string());
    let mac = device.mac.to_string();

    let title = format!(" {name}  ·  {model}  ·  {ip}  ·  {mac} ");
    let block = Block::default()
        .title(title)
        .title_style(theme::title_style())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::border_focused());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let tabs_layout = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .split(inner);

    render_detail_tabs(screen, frame, tabs_layout[0]);

    match screen.detail_tab {
        DeviceDetailTab::Overview => render_overview_tab(frame, tabs_layout[1], device),
        DeviceDetailTab::Performance => render_performance_tab(frame, tabs_layout[1], device),
        DeviceDetailTab::Radios => render_radios_tab(frame, tabs_layout[1], device),
        DeviceDetailTab::Clients => render_clients_tab(frame, tabs_layout[1], device),
        DeviceDetailTab::Ports => render_ports_tab(frame, tabs_layout[1], device),
    }

    render_detail_hints(frame, tabs_layout[2]);
}

fn render_detail_tabs(screen: &DevicesScreen, frame: &mut Frame, area: Rect) {
    let tab_labels = &["Overview", "Performance", "Radios", "Clients", "Ports"];
    let active_idx = match screen.detail_tab {
        DeviceDetailTab::Overview => 0,
        DeviceDetailTab::Performance => 1,
        DeviceDetailTab::Radios => 2,
        DeviceDetailTab::Clients => 3,
        DeviceDetailTab::Ports => 4,
    };
    let tab_line = sub_tabs::render_sub_tabs(tab_labels, active_idx);
    frame.render_widget(Paragraph::new(vec![Line::from(""), tab_line]), area);
}

fn render_detail_hints(frame: &mut Frame, area: Rect) {
    let hints = Line::from(vec![
        Span::styled("  h/l ", theme::key_hint_key()),
        Span::styled("switch tabs  ", theme::key_hint()),
        Span::styled("R ", theme::key_hint_key()),
        Span::styled("restart  ", theme::key_hint()),
        Span::styled("L ", theme::key_hint_key()),
        Span::styled("locate  ", theme::key_hint()),
        Span::styled("Esc ", theme::key_hint_key()),
        Span::styled("back", theme::key_hint()),
    ]);
    frame.render_widget(Paragraph::new(hints), area);
}

fn render_overview_tab(frame: &mut Frame, area: Rect, device: &Device) {
    let state_span = status_indicator::status_span(device.state);
    let state_label = format!("{:?}", device.state);
    let firmware = device.firmware_version.as_deref().unwrap_or("─");
    let fw_status = if device.firmware_updatable {
        "update available"
    } else {
        "up to date"
    };
    let uptime = device
        .stats
        .uptime_secs
        .map_or_else(|| "─".into(), bytes_fmt::fmt_uptime);
    let adopted = device.adopted_at.map_or_else(
        || "─".into(),
        |dt| dt.format("%Y-%m-%d %H:%M UTC").to_string(),
    );

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  State          ",
                Style::default().fg(theme::text_secondary()),
            ),
            state_span,
            Span::styled(
                format!(" {state_label}"),
                Style::default().fg(theme::text_secondary()),
            ),
            Span::styled(
                "       Adopted     ",
                Style::default().fg(theme::text_secondary()),
            ),
            Span::styled(adopted, Style::default().fg(theme::accent_secondary())),
        ]),
        Line::from(vec![
            Span::styled(
                "  Firmware       ",
                Style::default().fg(theme::text_secondary()),
            ),
            Span::styled(
                format!("{firmware} ({fw_status})"),
                Style::default().fg(theme::accent_secondary()),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  Uptime         ",
                Style::default().fg(theme::text_secondary()),
            ),
            Span::styled(uptime, Style::default().fg(theme::accent_secondary())),
        ]),
        Line::from(vec![
            Span::styled(
                "  MAC            ",
                Style::default().fg(theme::text_secondary()),
            ),
            Span::styled(
                device.mac.to_string(),
                Style::default().fg(theme::accent_tertiary()),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  Type           ",
                Style::default().fg(theme::text_secondary()),
            ),
            Span::styled(
                format!("{:?}", device.device_type),
                Style::default().fg(theme::text_secondary()),
            ),
        ]),
    ];

    frame.render_widget(Paragraph::new(lines), area);
}

fn render_performance_tab(frame: &mut Frame, area: Rect, device: &Device) {
    let cpu = device
        .stats
        .cpu_utilization_pct
        .map_or_else(|| "─".into(), |value| format!("{value:.1}%"));
    let mem = device
        .stats
        .memory_utilization_pct
        .map_or_else(|| "─".into(), |value| format!("{value:.1}%"));
    let load = device
        .stats
        .load_average_1m
        .map_or_else(|| "─".into(), |value| format!("{value:.2}"));

    let cpu_color = device
        .stats
        .cpu_utilization_pct
        .map_or(theme::text_secondary(), |value| {
            if value < 50.0 {
                theme::success()
            } else if value < 75.0 {
                theme::accent_secondary()
            } else if value < 90.0 {
                theme::warning()
            } else {
                theme::error()
            }
        });

    let mem_color = device
        .stats
        .memory_utilization_pct
        .map_or(theme::text_secondary(), |value| {
            if value < 50.0 {
                theme::success()
            } else if value < 75.0 {
                theme::accent_secondary()
            } else if value < 90.0 {
                theme::warning()
            } else {
                theme::error()
            }
        });

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  CPU     ", Style::default().fg(theme::text_secondary())),
            Span::styled(cpu, Style::default().fg(cpu_color)),
        ]),
        Line::from(vec![
            Span::styled("  Memory  ", Style::default().fg(theme::text_secondary())),
            Span::styled(mem, Style::default().fg(mem_color)),
        ]),
        Line::from(vec![
            Span::styled("  Load    ", Style::default().fg(theme::text_secondary())),
            Span::styled(load, Style::default().fg(theme::text_secondary())),
        ]),
    ];

    frame.render_widget(Paragraph::new(lines), area);
}

fn render_radios_tab(frame: &mut Frame, area: Rect, device: &Device) {
    let mut lines = vec![Line::from("")];

    if device.radios.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No radio data available",
            Style::default().fg(theme::border_unfocused()),
        )));
    } else {
        for radio in &device.radios {
            let freq = format!("{:.1} GHz", radio.frequency_ghz);
            let channel = radio
                .channel
                .map_or_else(|| "─".into(), |value| format!("ch {value}"));
            let width = radio
                .channel_width_mhz
                .map_or_else(|| "─".into(), |value| format!("{value} MHz"));

            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {freq:<10}"),
                    Style::default()
                        .fg(theme::accent_secondary())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{channel:<8} {width}"),
                    Style::default().fg(theme::text_secondary()),
                ),
            ]));
        }
    }

    frame.render_widget(Paragraph::new(lines), area);
}

fn render_clients_tab(frame: &mut Frame, area: Rect, device: &Device) {
    let text = format!("  Connected clients: {}", device.client_count.unwrap_or(0));
    frame.render_widget(Paragraph::new(text).style(theme::table_row()), area);
}

fn render_ports_tab(frame: &mut Frame, area: Rect, device: &Device) {
    let mut lines = vec![Line::from("")];

    if device.ports.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No port data available",
            Style::default().fg(theme::border_unfocused()),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "  Port  State   Speed      PoE",
            theme::table_header(),
        )));

        for port in &device.ports {
            let idx_str = port.index.to_string();
            let name = port.name.as_deref().unwrap_or(&idx_str);
            let state_color = match port.state {
                unifly_api::model::PortState::Up => theme::success(),
                unifly_api::model::PortState::Down => theme::error(),
                unifly_api::model::PortState::Unknown => theme::text_secondary(),
            };
            let state_str = format!("{:?}", port.state);
            let speed = port.speed_mbps.map_or_else(
                || "─".into(),
                |value| {
                    if value >= 1000 {
                        format!("{}G", value / 1000)
                    } else {
                        format!("{value}M")
                    }
                },
            );
            let poe = port
                .poe
                .as_ref()
                .map_or("─", |poe| if poe.enabled { "✓" } else { "✗" });

            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {name:<6}"),
                    Style::default().fg(theme::accent_secondary()),
                ),
                Span::styled(format!("{state_str:<8}"), Style::default().fg(state_color)),
                Span::styled(
                    format!("{speed:<11}"),
                    Style::default().fg(theme::text_secondary()),
                ),
                Span::styled(poe, Style::default().fg(theme::text_secondary())),
            ]));
        }
    }

    frame.render_widget(Paragraph::new(lines), area);
}
