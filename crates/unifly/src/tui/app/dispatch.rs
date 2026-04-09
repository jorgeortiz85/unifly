use std::time::Duration;

use color_eyre::eyre::Result;

use super::{App, ConnectionStatus};
use crate::tui::action::Action;
use crate::tui::screen::ScreenId;

impl App {
    /// Process a single action — update app state and propagate to components.
    #[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
    pub(super) fn process_action(&mut self, action: &Action) -> Result<()> {
        if self.handle_command_action(action)? {
            return Ok(());
        }

        match action {
            Action::Quit => {
                self.running = false;
            }
            Action::Resize(w, h) => {
                self.terminal_size = (*w, *h);
            }
            Action::SwitchScreen(target) => self.switch_screen(*target)?,
            Action::GoBack => {
                if let Some(prev) = self.previous_screen.take() {
                    self.action_tx.send(Action::SwitchScreen(prev))?;
                }
            }
            Action::ToggleHelp => {
                self.help_visible = !self.help_visible;
            }
            Action::ToggleAbout => {
                self.about_visible = !self.about_visible;
            }
            Action::OpenSearch => {
                self.search_active = true;
                self.search_query.clear();
            }
            Action::CloseSearch => {
                self.search_active = false;
                self.search_query.clear();
            }
            Action::Connected => {
                self.connection_status = ConnectionStatus::Connected;
            }
            Action::Disconnected(_) => {
                self.connection_status = ConnectionStatus::Disconnected;
            }
            Action::Reconnecting => {
                self.connection_status = ConnectionStatus::Reconnecting;
            }
            Action::Render => {}
            Action::Tick => {
                self.forward_to_all_screens(action)?;

                if let Some((_, created)) = &self.notification
                    && created.elapsed() > Duration::from_secs(3)
                {
                    self.notification = None;
                }

                if self.active_screen == ScreenId::Stats
                    && let Some(last) = self.last_stats_fetch
                    && last.elapsed() > Duration::from_mins(1)
                {
                    let _ = self.action_tx.send(Action::RequestStats(self.stats_period));
                }
            }
            Action::DevicesUpdated(_)
            | Action::ClientsUpdated(_)
            | Action::NetworksUpdated(_)
            | Action::FirewallPoliciesUpdated(_)
            | Action::FirewallZonesUpdated(_)
            | Action::AclRulesUpdated(_)
            | Action::WifiBroadcastsUpdated(_)
            | Action::EventReceived(_)
            | Action::HealthUpdated(_)
            | Action::SiteUpdated(_)
            | Action::WifiNeighborsUpdated(_)
            | Action::WifiChannelsUpdated(_)
            | Action::WifiClientDetailLoaded { .. }
            | Action::WifiRoamHistoryLoaded { .. }
            | Action::StatsUpdated(_)
            | Action::NetworkEditResult(_) => {
                self.forward_to_all_screens(action)?;
            }
            Action::RequestWifiNeighbors(within_secs) => {
                self.fetch_wifi_neighbors(*within_secs);
            }
            Action::RequestWifiChannels => {
                self.fetch_wifi_channels();
            }
            Action::RequestWifiClientDetail(ip) => {
                self.fetch_wifi_client_detail(ip);
            }
            Action::RequestWifiRoamHistory { mac, limit } => {
                self.fetch_wifi_roam_history(mac, *limit);
            }
            Action::RequestStats(period) => {
                self.stats_period = *period;
                self.last_stats_fetch = Some(std::time::Instant::now());
                self.fetch_stats(*period);
            }
            Action::OnboardingComplete { config, .. } => {
                self.handle_onboarding_complete(config)?;
            }
            Action::OnboardingTestResult(_) => {
                self.forward_to_screen(ScreenId::Setup, action)?;
            }
            Action::OpenSettings => {
                self.open_settings()?;
            }
            Action::CloseSettings => {
                self.close_settings();
            }
            Action::SettingsTestResult(_) => {
                self.forward_to_screen(ScreenId::Settings, action)?;
            }
            Action::SettingsApply { config, .. } => {
                self.apply_settings(config)?;
            }
            Action::Notify(notification) => {
                self.show_notification(notification.clone());
            }
            Action::DismissNotification => {
                self.notification = None;
            }
            Action::OpenDonate => {
                open_url("https://github.com/sponsors/hyperb1iss");
            }
            Action::SetShowDonate(show) => {
                self.show_donate = *show;
            }
            other => {
                self.forward_to_screen(self.active_screen, other)?;
            }
        }

        Ok(())
    }
}

/// Open a URL in the user's default browser.
pub(super) fn open_url(url: &str) {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/c", "start", url])
            .spawn();
    }
}
