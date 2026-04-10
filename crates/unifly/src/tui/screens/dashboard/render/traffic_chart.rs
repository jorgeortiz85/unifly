use ratatui::style::Style;
use ratatui::text::{Line, Span};

use crate::tui::theme;
use crate::tui::widgets::bytes_fmt;
use crate::tui::widgets::hyperchart::{Domain, HyperChart, Renderer, Series};

use super::super::DashboardScreen;
use super::super::{
    BANDWIDTH_GUTTER_WIDTH, BANDWIDTH_LABEL_WIDTH, BANDWIDTH_TICK_COUNT, LIVE_CHART_WINDOW_SAMPLES,
    MIN_BANDWIDTH_SCALE,
};

impl DashboardScreen {
    /// Hero panel: WAN traffic chart with Octant-rendered area fill and line
    /// overlay, plus a live TX/RX/peak legend in the title bar.
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::as_conversions
    )]
    pub(super) fn render_traffic_chart(
        &self,
        frame: &mut ratatui::Frame,
        area: ratatui::layout::Rect,
    ) {
        let (current_tx, current_rx) = self
            .current_bandwidth()
            .or_else(|| {
                Some((
                    self.bandwidth_tx.last().map(|&(_, value)| value as u64)?,
                    self.bandwidth_rx.last().map(|&(_, value)| value as u64)?,
                ))
            })
            .unwrap_or((0, 0));

        let title = Line::from(vec![
            Span::styled(" WAN Traffic ", theme::title_style()),
            Span::styled("── ", Style::default().fg(theme::border_unfocused())),
            Span::styled(
                format!("TX {} ↑", bytes_fmt::fmt_rate(current_tx)),
                Style::default().fg(theme::accent_secondary()),
            ),
            Span::styled("  ", Style::default()),
            Span::styled(
                format!("RX {} ↓", bytes_fmt::fmt_rate(current_rx)),
                Style::default().fg(theme::accent_tertiary()),
            ),
            Span::styled(
                format!(
                    "  Peak {} ",
                    bytes_fmt::fmt_rate(self.peak_rx.max(self.peak_tx))
                ),
                Style::default().fg(theme::border_unfocused()),
            ),
        ]);

        let window_span = LIVE_CHART_WINDOW_SAMPLES.saturating_sub(1) as f64;
        let x_max = self.sample_counter.max(0.0);
        let x_min = x_max - window_span;
        let y_max = self.chart_y_max.max(MIN_BANDWIDTH_SCALE);

        let series = [
            Series {
                name: "RX",
                data: &self.bandwidth_rx,
                line_color: theme::accent_tertiary(),
                fill_color: Some(theme::rx_fill()),
            },
            Series {
                name: "TX",
                data: &self.bandwidth_tx,
                line_color: theme::accent_secondary(),
                fill_color: Some(theme::tx_fill()),
            },
        ];

        let chart = HyperChart::new(title, &series, (x_min, x_max), y_max)
            .domain(Domain::Rate)
            .tick_count(BANDWIDTH_TICK_COUNT)
            .label_width(BANDWIDTH_LABEL_WIDTH)
            .renderer(Renderer::Canvas {
                gutter_width: BANDWIDTH_GUTTER_WIDTH,
            })
            .empty_message("Waiting for data…");

        frame.render_widget(chart, area);
    }
}
