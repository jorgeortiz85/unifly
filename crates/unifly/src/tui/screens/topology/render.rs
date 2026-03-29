use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::canvas::{Canvas, Context, Rectangle};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

use unifly_api::DeviceType;

use super::TopologyScreen;
use crate::tui::theme;

impl TopologyScreen {
    #[allow(
        clippy::too_many_lines,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::as_conversions
    )]
    pub(super) fn render_screen(&self, frame: &mut Frame, area: Rect) {
        let zoom_pct = (self.zoom * 100.0) as u32;
        let title = format!(
            " Topology  ·  Zoom: {zoom_pct}%  Pan: {:.0},{:.0} ",
            self.pan_x, self.pan_y
        );
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

        let content_area = Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: inner.height.saturating_sub(1),
        };
        let hints_area = Rect {
            x: inner.x,
            y: inner.y + inner.height.saturating_sub(1),
            width: inner.width,
            height: 1,
        };

        let nodes = self.build_nodes();

        let width = f64::from(content_area.width.max(1));
        let height = f64::from(content_area.height.max(1));
        let aspect = (height * 2.0) / width;

        let x_span = 120.0 / self.zoom;
        let y_span = x_span * aspect;
        let x_center = 50.0 + self.pan_x;
        let y_center = 50.0 + self.pan_y;

        let x_min = x_center - x_span / 2.0;
        let x_max = x_center + x_span / 2.0;
        let y_min = y_center - y_span / 2.0;
        let y_max = y_center + y_span / 2.0;

        let canvas = Canvas::default()
            .x_bounds([x_min, x_max])
            .y_bounds([y_min, y_max])
            .paint(|ctx: &mut Context<'_>| {
                for node in &nodes {
                    let border_color = match node.device_type {
                        DeviceType::Gateway => theme::accent_tertiary(),
                        DeviceType::Switch => theme::accent_secondary(),
                        DeviceType::AccessPoint => theme::accent_primary(),
                        _ => theme::text_secondary(),
                    };

                    let color = if node.state.is_online() {
                        border_color
                    } else {
                        theme::error()
                    };

                    ctx.draw(&Rectangle {
                        x: node.x,
                        y: node.y,
                        width: node.width,
                        height: node.height,
                        color,
                    });

                    let short_label: String = node.label.chars().take(10).collect();
                    ctx.print(
                        node.x + 1.0,
                        node.y + node.height - 1.5,
                        Span::styled(short_label, Style::default().fg(color)),
                    );

                    if !node.ip.is_empty() {
                        ctx.print(
                            node.x + 1.0,
                            node.y + 0.5,
                            Span::styled(
                                node.ip.clone(),
                                Style::default().fg(theme::text_secondary()),
                            ),
                        );
                    }
                }

                let gateway_nodes: Vec<_> = nodes
                    .iter()
                    .filter(|node| node.device_type == DeviceType::Gateway)
                    .collect();
                let switch_nodes: Vec<_> = nodes
                    .iter()
                    .filter(|node| node.device_type == DeviceType::Switch)
                    .collect();
                let ap_nodes: Vec<_> = nodes
                    .iter()
                    .filter(|node| node.device_type == DeviceType::AccessPoint)
                    .collect();

                for gateway in &gateway_nodes {
                    let gw_cx = gateway.x + gateway.width / 2.0;
                    let gw_bottom = gateway.y;
                    for switch in &switch_nodes {
                        let sw_cx = switch.x + switch.width / 2.0;
                        let sw_top = switch.y + switch.height;
                        ctx.draw(&ratatui::widgets::canvas::Line {
                            x1: gw_cx,
                            y1: gw_bottom,
                            x2: sw_cx,
                            y2: sw_top,
                            color: theme::accent_secondary(),
                        });
                    }
                }

                for switch in &switch_nodes {
                    let sw_cx = switch.x + switch.width / 2.0;
                    let sw_bottom = switch.y;
                    for ap in &ap_nodes {
                        let ap_cx = ap.x + ap.width / 2.0;
                        let ap_top = ap.y + ap.height;
                        ctx.draw(&ratatui::widgets::canvas::Line {
                            x1: sw_cx,
                            y1: sw_bottom,
                            x2: ap_cx,
                            y2: ap_top,
                            color: theme::accent_primary(),
                        });
                    }
                }
            });

        frame.render_widget(canvas, content_area);

        let hints = Line::from(vec![
            Span::styled("  ←→↑↓ ", theme::key_hint_key()),
            Span::styled("pan  ", theme::key_hint()),
            Span::styled("+/- ", theme::key_hint_key()),
            Span::styled("zoom  ", theme::key_hint()),
            Span::styled("f ", theme::key_hint_key()),
            Span::styled("fit  ", theme::key_hint()),
            Span::styled("r ", theme::key_hint_key()),
            Span::styled("reset", theme::key_hint()),
        ]);
        frame.render_widget(Paragraph::new(hints), hints_area);
    }
}
