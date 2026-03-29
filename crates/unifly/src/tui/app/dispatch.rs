use std::time::Duration;

use color_eyre::eyre::Result;

use unifly_api::Command;

use super::{App, ConnectionStatus};
use crate::tui::action::{Action, ConfirmAction};
use crate::tui::screen::ScreenId;

impl App {
    /// Process a single action — update app state and propagate to components.
    #[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
    pub(super) fn process_action(&mut self, action: &Action) -> Result<()> {
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
                    && last.elapsed() > Duration::from_secs(60)
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
            | Action::StatsUpdated(_)
            | Action::NetworkEditResult(_) => {
                self.forward_to_all_screens(action)?;
            }
            Action::RequestRestart(id) => {
                let name = self.resolve_device_name(id);
                self.action_tx
                    .send(Action::ShowConfirm(ConfirmAction::RestartDevice {
                        id: id.clone(),
                        name,
                    }))?;
            }
            Action::RequestUnadopt(id) => {
                let name = self.resolve_device_name(id);
                self.action_tx
                    .send(Action::ShowConfirm(ConfirmAction::UnadoptDevice {
                        id: id.clone(),
                        name,
                    }))?;
            }
            Action::RequestLocate(id) => {
                if let Some(mac) = self.resolve_device_mac(id) {
                    self.execute_command(
                        Command::LocateDevice {
                            mac: mac.clone(),
                            enable: true,
                        },
                        format!("Locating {mac}"),
                    );
                }
            }
            Action::RequestBlockClient(id) => {
                let name = self.resolve_client_name(id);
                self.action_tx
                    .send(Action::ShowConfirm(ConfirmAction::BlockClient {
                        id: id.clone(),
                        name,
                    }))?;
            }
            Action::RequestUnblockClient(id) => {
                let name = self.resolve_client_name(id);
                self.action_tx
                    .send(Action::ShowConfirm(ConfirmAction::UnblockClient {
                        id: id.clone(),
                        name,
                    }))?;
            }
            Action::RequestForgetClient(id) => {
                let name = self.resolve_client_name(id);
                self.action_tx
                    .send(Action::ShowConfirm(ConfirmAction::ForgetClient {
                        id: id.clone(),
                        name,
                    }))?;
            }
            Action::RequestKickClient(id) => {
                if let Some(mac) = self.resolve_client_mac(id) {
                    let name = self.resolve_client_name(id);
                    self.execute_command(Command::KickClient { mac }, format!("Kicked {name}"));
                }
            }
            Action::ShowConfirm(confirm) => {
                self.pending_confirm = Some(confirm.clone());
            }
            Action::ConfirmYes => {
                if let Some(confirm) = self.pending_confirm.take() {
                    self.execute_confirm(confirm);
                }
            }
            Action::ConfirmNo => {
                self.pending_confirm = None;
            }
            Action::NetworkSave(id, update) => {
                self.execute_command(
                    Command::UpdateNetwork {
                        id: id.clone(),
                        update: *update.clone(),
                    },
                    "Updated network".into(),
                );
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
            other => {
                self.forward_to_screen(self.active_screen, other)?;
            }
        }

        Ok(())
    }
}
