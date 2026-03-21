//! Shared chart helpers for stable axes and dense area fills.

use ratatui::style::Style;
use ratatui::text::Span;

use crate::tui::widgets::bytes_fmt;

const SCALE_HEADROOM: f64 = 1.10;
const SCALE_GROW_THRESHOLD: f64 = 0.92;
const SCALE_SHRINK_THRESHOLD: f64 = 0.55;

/// Linearly interpolate data points to create dense fill data for area charts.
#[allow(clippy::cast_precision_loss, clippy::as_conversions)]
pub fn interpolate_fill(data: &[(f64, f64)], target_density: usize) -> Vec<(f64, f64)> {
    if data.len() < 2 {
        return data.to_vec();
    }

    let x_min = data.first().map_or(0.0, |&(x, _)| x);
    let x_max = data.last().map_or(1.0, |&(x, _)| x);
    let x_range = (x_max - x_min).max(1.0);
    let step = x_range / target_density.max(1) as f64;

    let mut result = Vec::with_capacity(target_density + 1);
    let mut data_idx = 0;
    let mut x = x_min;

    while x <= x_max + step * 0.5 {
        while data_idx + 1 < data.len() && data[data_idx + 1].0 < x {
            data_idx += 1;
        }

        let y = if data_idx + 1 < data.len() {
            let (x0, y0) = data[data_idx];
            let (x1, y1) = data[data_idx + 1];
            let dx = x1 - x0;
            if dx.abs() < f64::EPSILON {
                y0
            } else {
                y0 + (y1 - y0) * ((x - x0) / dx)
            }
        } else {
            data[data.len() - 1].1
        };

        result.push((x, y));
        x += step;
    }

    result
}

/// Compute a rounded upper bound for a linear axis.
#[allow(clippy::cast_precision_loss, clippy::as_conversions)]
pub fn nice_upper_bound(observed_max: f64, tick_count: usize, minimum: f64) -> f64 {
    let divisions = tick_count.saturating_sub(1).max(1);
    let target = (observed_max * SCALE_HEADROOM).max(minimum);
    let raw_step = (target / divisions as f64).max(1.0);
    let magnitude = 10_f64.powf(raw_step.log10().floor());
    let normalized = raw_step / magnitude;
    let nice_step = if normalized <= 1.0 {
        1.0
    } else if normalized <= 2.0 {
        2.0
    } else if normalized <= 2.5 {
        2.5
    } else if normalized <= 5.0 {
        5.0
    } else {
        10.0
    };

    nice_step * magnitude * divisions as f64
}

/// Keep an axis stable until the observed data materially leaves the current range.
pub fn stable_upper_bound(
    current_max: f64,
    observed_max: f64,
    tick_count: usize,
    minimum: f64,
) -> f64 {
    let desired = nice_upper_bound(observed_max, tick_count, minimum);

    if current_max <= 0.0 {
        return desired;
    }

    if observed_max >= current_max * SCALE_GROW_THRESHOLD {
        return desired.max(current_max);
    }

    if observed_max <= current_max * SCALE_SHRINK_THRESHOLD {
        return desired;
    }

    current_max
}

fn build_axis_labels<F>(
    max_value: f64,
    tick_count: usize,
    width: usize,
    style: Style,
    mut format_label: F,
) -> Vec<Span<'static>>
where
    F: FnMut(f64) -> String,
{
    let divisions = tick_count.saturating_sub(1).max(1);

    (0..=divisions)
        .map(|idx| {
            #[allow(clippy::cast_precision_loss, clippy::as_conversions)]
            let value = max_value * idx as f64 / divisions as f64;
            let label = format!("{:>width$}", format_label(value), width = width);
            Span::styled(label, style)
        })
        .collect()
}

/// Generate fixed-width rate labels so chart plot areas do not resize frame to frame.
pub fn rate_axis_labels(
    max_value: f64,
    tick_count: usize,
    width: usize,
    style: Style,
) -> Vec<Span<'static>> {
    build_axis_labels(
        max_value,
        tick_count,
        width,
        style,
        bytes_fmt::fmt_rate_axis,
    )
}

/// Generate fixed-width integer labels for count-based charts.
pub fn count_axis_labels(
    max_value: f64,
    tick_count: usize,
    width: usize,
    style: Style,
) -> Vec<Span<'static>> {
    build_axis_labels(max_value, tick_count, width, style, |value| {
        format!("{value:.0}")
    })
}

#[cfg(test)]
mod tests {
    use ratatui::style::Style;

    use super::{nice_upper_bound, rate_axis_labels, stable_upper_bound};

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < f64::EPSILON,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn nice_upper_bound_rounds_up_to_human_steps() {
        assert_close(nice_upper_bound(1_010_000.0, 4, 10_000.0), 1_500_000.0);
        assert_close(nice_upper_bound(90_000.0, 4, 10_000.0), 150_000.0);
    }

    #[test]
    fn stable_upper_bound_holds_until_data_really_moves() {
        let current = 1_500_000.0;
        assert_close(
            stable_upper_bound(current, 1_000_000.0, 4, 10_000.0),
            current,
        );
        assert_close(
            stable_upper_bound(current, 2_000_000.0, 4, 10_000.0),
            3_000_000.0,
        );
        assert_close(
            stable_upper_bound(current, 400_000.0, 4, 10_000.0),
            600_000.0,
        );
    }

    #[test]
    fn rate_labels_are_fixed_width() {
        let labels = rate_axis_labels(2_500_000.0, 4, 6, Style::default());
        let widths: Vec<_> = labels.iter().map(|label| label.content.len()).collect();
        assert_eq!(widths, vec![6, 6, 6, 6]);
    }
}
