use super::{BANDWIDTH_TICK_COUNT, CLIENT_TICK_COUNT, MIN_BANDWIDTH_SCALE, StatsScreen};

use crate::tui::action::{Action, StatsPeriod};
use crate::tui::widgets::hyperchart::axis;

impl StatsScreen {
    pub fn new() -> Self {
        Self {
            focused: false,
            period: StatsPeriod::default(),
            bandwidth_tx: Vec::new(),
            bandwidth_rx: Vec::new(),
            bandwidth_y_max: 0.0,
            client_counts: Vec::new(),
            client_y_max: 0.0,
            dpi_apps: Vec::new(),
            dpi_categories: Vec::new(),
        }
    }

    pub(super) fn period_index(&self) -> usize {
        match self.period {
            StatsPeriod::OneHour => 0,
            StatsPeriod::TwentyFourHours => 1,
            StatsPeriod::SevenDays => 2,
            StatsPeriod::ThirtyDays => 3,
        }
    }

    pub(super) fn apply_action(&mut self, action: &Action) {
        match action {
            Action::SetStatsPeriod(period) => {
                self.period = *period;
                self.bandwidth_y_max = 0.0;
                self.client_y_max = 0.0;
            }
            Action::StatsUpdated(data) => {
                self.bandwidth_tx.clone_from(&data.bandwidth_tx);
                self.bandwidth_rx.clone_from(&data.bandwidth_rx);
                self.client_counts.clone_from(&data.client_counts);
                self.dpi_apps.clone_from(&data.dpi_apps);
                self.dpi_categories.clone_from(&data.dpi_categories);

                let bandwidth_max = self
                    .bandwidth_tx
                    .iter()
                    .chain(self.bandwidth_rx.iter())
                    .map(|&(_, value)| value)
                    .fold(0.0_f64, f64::max);
                self.bandwidth_y_max = axis::stable_upper_bound(
                    self.bandwidth_y_max,
                    bandwidth_max,
                    BANDWIDTH_TICK_COUNT,
                    MIN_BANDWIDTH_SCALE,
                );

                let client_max = self
                    .client_counts
                    .iter()
                    .map(|&(_, value)| value)
                    .fold(0.0_f64, f64::max);
                self.client_y_max =
                    axis::stable_upper_bound(self.client_y_max, client_max, CLIENT_TICK_COUNT, 1.0);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    fn sample_stats_data(bandwidth: f64, clients: f64) -> crate::tui::action::StatsData {
        crate::tui::action::StatsData {
            bandwidth_tx: vec![(1.0, bandwidth)],
            bandwidth_rx: vec![(1.0, bandwidth / 2.0)],
            client_counts: vec![(1.0, clients)],
            dpi_apps: vec![("Video".into(), 1024)],
            dpi_categories: vec![("Streaming".into(), 2048)],
        }
    }

    #[test]
    fn set_stats_period_resets_axis_bounds() {
        let mut screen = StatsScreen::new();
        screen.bandwidth_y_max = 42_000.0;
        screen.client_y_max = 99.0;

        screen.apply_action(&Action::SetStatsPeriod(StatsPeriod::SevenDays));

        assert_eq!(screen.period, StatsPeriod::SevenDays);
        assert_eq!(screen.bandwidth_y_max, 0.0);
        assert_eq!(screen.client_y_max, 0.0);
    }

    #[test]
    fn stats_updates_use_stable_axis_bounds() {
        let mut screen = StatsScreen::new();

        screen.apply_action(&Action::StatsUpdated(sample_stats_data(120_000.0, 18.0)));
        let first_bandwidth_max = screen.bandwidth_y_max;
        let first_client_max = screen.client_y_max;

        assert_eq!(
            first_bandwidth_max,
            axis::stable_upper_bound(0.0, 120_000.0, BANDWIDTH_TICK_COUNT, MIN_BANDWIDTH_SCALE)
        );
        assert_eq!(
            first_client_max,
            axis::stable_upper_bound(0.0, 18.0, CLIENT_TICK_COUNT, 1.0)
        );

        screen.apply_action(&Action::StatsUpdated(sample_stats_data(40_000.0, 8.0)));

        assert_eq!(
            screen.bandwidth_y_max,
            axis::stable_upper_bound(
                first_bandwidth_max,
                40_000.0,
                BANDWIDTH_TICK_COUNT,
                MIN_BANDWIDTH_SCALE
            )
        );
        assert_eq!(
            screen.client_y_max,
            axis::stable_upper_bound(first_client_max, 8.0, CLIENT_TICK_COUNT, 1.0)
        );
    }
}
