//! Time-series chart widget with two rendering back-ends.
//!
//! [`HyperChart`] accepts one or more [`Series`], a [`Domain`] for y-axis
//! label formatting, explicit x-axis bounds, a y-axis upper bound, and a
//! [`Renderer`] choice:
//!
//! - [`Renderer::Canvas`] uses Ratatui's `Canvas` with `Marker::Octant` for
//!   smooth sub-cell resolution, and renders manual y-axis labels in a gutter.
//!   This is the hero look used by the dashboard traffic chart.
//! - [`Renderer::Tiled`] uses Ratatui's `Chart` widget with built-in `Axis`
//!   support — denser, simpler, and better suited for grid-tile panels like
//!   the stats screen bandwidth and client count panels.
//!
//! Both renderers share axis math, empty-state handling, block styling,
//! theme-driven colors, and fill interpolation. Visual tweaks land once.

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::symbols::Marker;
use ratatui::text::{Line, Span};
use ratatui::widgets::canvas::{Canvas, Line as CanvasLine};
use ratatui::widgets::{Axis, Chart, Dataset, GraphType, Paragraph, Widget};

use super::{axis, block, empty};
use crate::tui::theme;

/// One time-series dataset rendered by [`HyperChart`].
#[derive(Debug, Clone, Copy)]
pub struct Series<'a> {
    pub name: &'a str,
    pub data: &'a [(f64, f64)],
    pub line_color: Color,
    pub fill_color: Option<Color>,
}

/// Y-axis label formatting domain.
#[derive(Debug, Clone, Copy)]
pub enum Domain {
    /// Bytes-per-second rate (labels look like `"1.2G"`, `"500K"`).
    Rate,
    /// Integer counts (labels look like `"42"`).
    Count,
}

/// Rendering back-end for [`HyperChart`].
#[derive(Debug, Clone, Copy)]
pub enum Renderer {
    /// Octant-marker Canvas with a manual y-axis gutter. Hero look with
    /// higher sub-cell resolution for filled area charts.
    Canvas {
        /// Width reserved for y-axis labels on the left of the plot area.
        gutter_width: u16,
    },
    /// Ratatui `Chart` widget with built-in axes. Dense grid-tile look.
    Tiled,
}

/// Unified time-series chart widget.
pub struct HyperChart<'a> {
    title: Line<'a>,
    series: &'a [Series<'a>],
    domain: Domain,
    x_bounds: (f64, f64),
    y_max: f64,
    renderer: Renderer,
    tick_count: usize,
    label_width: usize,
    empty_message: &'a str,
    focused: bool,
}

impl<'a> HyperChart<'a> {
    /// Construct a new `HyperChart` with sensible defaults (Tiled renderer,
    /// Rate domain, 4 ticks, 6-char label width).
    pub fn new(
        title: Line<'a>,
        series: &'a [Series<'a>],
        x_bounds: (f64, f64),
        y_max: f64,
    ) -> Self {
        Self {
            title,
            series,
            domain: Domain::Rate,
            x_bounds,
            y_max,
            renderer: Renderer::Tiled,
            tick_count: 4,
            label_width: 6,
            empty_message: "No data",
            focused: false,
        }
    }

    #[must_use]
    pub fn domain(mut self, domain: Domain) -> Self {
        self.domain = domain;
        self
    }

    #[must_use]
    pub fn renderer(mut self, renderer: Renderer) -> Self {
        self.renderer = renderer;
        self
    }

    #[must_use]
    pub fn tick_count(mut self, tick_count: usize) -> Self {
        self.tick_count = tick_count;
        self
    }

    #[must_use]
    pub fn label_width(mut self, label_width: usize) -> Self {
        self.label_width = label_width;
        self
    }

    #[must_use]
    pub fn empty_message(mut self, message: &'a str) -> Self {
        self.empty_message = message;
        self
    }

    #[must_use]
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    fn is_empty(&self) -> bool {
        self.series.iter().all(|series| series.data.is_empty())
    }

    fn resolved_x_bounds(&self) -> (f64, f64) {
        let (min, max) = self.x_bounds;
        if (max - min).abs() < f64::EPSILON {
            (min - 0.5, max + 0.5)
        } else {
            (min, max)
        }
    }

    fn build_y_labels(&self) -> Vec<Span<'static>> {
        let axis_style = Style::default().fg(theme::border_unfocused());
        match self.domain {
            Domain::Rate => {
                axis::rate_axis_labels(self.y_max, self.tick_count, self.label_width, axis_style)
            }
            Domain::Count => {
                axis::count_axis_labels(self.y_max, self.tick_count, self.label_width, axis_style)
            }
        }
    }
}

impl Widget for HyperChart<'_> {
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::as_conversions,
        clippy::too_many_lines
    )]
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = block::standard(self.title.clone(), self.focused);
        let inner = block.inner(area);
        block.render(area, buf);

        if self.is_empty() {
            empty::render(inner, buf, self.empty_message);
            return;
        }

        let (x_min, x_max) = self.resolved_x_bounds();
        let axis_style = Style::default().fg(theme::border_unfocused());

        match self.renderer {
            Renderer::Tiled => {
                let fill_density = (usize::from(area.width.saturating_sub(8)) * 3).max(120);

                let fill_buffers: Vec<Vec<(f64, f64)>> = self
                    .series
                    .iter()
                    .map(|series| {
                        if series.fill_color.is_some() {
                            axis::interpolate_fill(series.data, fill_density)
                        } else {
                            Vec::new()
                        }
                    })
                    .collect();

                let mut datasets: Vec<Dataset> = Vec::new();
                for (series, fill_buf) in self.series.iter().zip(fill_buffers.iter()) {
                    if let Some(color) = series.fill_color {
                        datasets.push(
                            Dataset::default()
                                .marker(Marker::HalfBlock)
                                .graph_type(GraphType::Bar)
                                .style(Style::default().fg(color))
                                .data(fill_buf),
                        );
                    }
                }

                for series in self.series {
                    datasets.push(
                        Dataset::default()
                            .name(series.name)
                            .marker(Marker::Braille)
                            .graph_type(GraphType::Line)
                            .style(Style::default().fg(series.line_color))
                            .data(series.data),
                    );
                }

                let y_labels = self.build_y_labels();
                let chart = Chart::new(datasets)
                    .x_axis(Axis::default().bounds([x_min, x_max]).style(axis_style))
                    .y_axis(
                        Axis::default()
                            .bounds([0.0, self.y_max])
                            .labels(y_labels)
                            .style(axis_style),
                    );

                chart.render(inner, buf);
            }
            Renderer::Canvas { gutter_width } => {
                let layout =
                    Layout::horizontal([Constraint::Length(gutter_width), Constraint::Min(1)])
                        .split(inner);
                let gutter_area = layout[0];
                let plot_area = layout[1];

                let y_labels = self.build_y_labels();
                let label_steps = self.tick_count.saturating_sub(1).max(1);
                for (idx, label) in y_labels.iter().rev().enumerate() {
                    let y_offset = {
                        let rows = plot_area.height.saturating_sub(1);
                        (u32::from(rows) * idx as u32 / label_steps as u32) as u16
                    };
                    let label_area = Rect {
                        x: gutter_area.x,
                        y: plot_area.y + y_offset,
                        width: gutter_area.width,
                        height: 1,
                    };
                    Paragraph::new(Line::from(label.clone())).render(label_area, buf);
                }

                let plot_density = (usize::from(plot_area.width.max(1)) * 4).max(160);
                let fill_paths: Vec<Vec<(f64, f64)>> = self
                    .series
                    .iter()
                    .map(|series| {
                        if series.fill_color.is_some() {
                            axis::interpolate_fill(series.data, plot_density)
                        } else {
                            Vec::new()
                        }
                    })
                    .collect();

                let canvas = Canvas::default()
                    .background_color(theme::bg_base())
                    .marker(Marker::Octant)
                    .x_bounds([x_min, x_max])
                    .y_bounds([0.0, self.y_max])
                    .paint(|ctx| {
                        ctx.draw(&CanvasLine {
                            x1: x_min,
                            y1: 0.0,
                            x2: x_max,
                            y2: 0.0,
                            color: theme::border_unfocused(),
                        });

                        for (series, fill_path) in self.series.iter().zip(fill_paths.iter()) {
                            let Some(fill_color) = series.fill_color else {
                                continue;
                            };
                            for &(x, y) in fill_path {
                                ctx.draw(&CanvasLine {
                                    x1: x,
                                    y1: 0.0,
                                    x2: x,
                                    y2: y,
                                    color: fill_color,
                                });
                            }
                        }

                        ctx.layer();

                        for (series, fill_path) in self.series.iter().zip(fill_paths.iter()) {
                            if fill_path.is_empty() {
                                continue;
                            }
                            for pair in fill_path.windows(2) {
                                let [(x1, y1), (x2, y2)] = pair else {
                                    continue;
                                };
                                ctx.draw(&CanvasLine {
                                    x1: *x1,
                                    y1: *y1,
                                    x2: *x2,
                                    y2: *y2,
                                    color: series.line_color,
                                });
                            }
                        }
                    });

                canvas.render(plot_area, buf);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::style::Color;

    fn render_chart(widget: HyperChart, width: u16, height: u16) -> Buffer {
        let area = Rect::new(0, 0, width, height);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);
        buf
    }

    #[test]
    fn empty_series_renders_empty_message() {
        let series: &[Series] = &[];
        let widget = HyperChart::new(Line::from(" Bandwidth "), series, (0.0, 10.0), 1_000.0)
            .empty_message("No bandwidth data yet");
        let buf = render_chart(widget, 60, 12);

        let rendered: String = (0..buf.area().height)
            .map(|y| {
                (0..buf.area().width)
                    .filter_map(|x| buf.cell((x, y)).map(|c| c.symbol().to_string()))
                    .collect::<String>()
            })
            .collect();
        assert!(rendered.contains("No bandwidth data yet"));
    }

    #[test]
    fn tiled_renderer_draws_single_series_without_panic() {
        let data: Vec<(f64, f64)> = (0..20)
            .map(|i| (f64::from(i), f64::from(i * 100)))
            .collect();
        let series = [Series {
            name: "TX",
            data: &data,
            line_color: Color::Cyan,
            fill_color: Some(Color::Blue),
        }];
        let widget = HyperChart::new(Line::from(" Bandwidth "), &series, (0.0, 20.0), 2_000.0);
        let _ = render_chart(widget, 60, 12);
    }

    #[test]
    fn canvas_renderer_draws_dual_series_without_panic() {
        let tx: Vec<(f64, f64)> = (0..30)
            .map(|i| (f64::from(i), f64::from(i) * 123.0))
            .collect();
        let rx: Vec<(f64, f64)> = (0..30)
            .map(|i| (f64::from(i), f64::from(i) * 456.0))
            .collect();
        let series = [
            Series {
                name: "TX",
                data: &tx,
                line_color: Color::Cyan,
                fill_color: Some(Color::Blue),
            },
            Series {
                name: "RX",
                data: &rx,
                line_color: Color::Magenta,
                fill_color: Some(Color::Red),
            },
        ];
        let widget = HyperChart::new(Line::from(" WAN Traffic "), &series, (0.0, 30.0), 20_000.0)
            .renderer(Renderer::Canvas { gutter_width: 7 });
        let _ = render_chart(widget, 80, 16);
    }
}
