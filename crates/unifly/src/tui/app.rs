//! Application core — event loop, screen management, action dispatch.

mod commands;
mod dispatch;
mod lifecycle;
mod navigation;
mod render;
mod stats;

use std::collections::HashMap;
use std::time::{Duration, Instant};

use color_eyre::eyre::Result;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::info;

use unifly_api::Controller;

use crate::tui::action::{Action, ConfirmAction, Notification, StatsPeriod};
use crate::tui::component::Component;
use crate::tui::event::{Event, EventReader};
use crate::tui::screen::ScreenId;
use crate::tui::screens::create_screens;
use crate::tui::terminal::Tui;

/// Connection status as seen by the TUI.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ConnectionStatus {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
}

/// Top-level application state and event loop.
pub struct App {
    /// Current active screen.
    active_screen: ScreenId,
    /// Previous screen for GoBack.
    previous_screen: Option<ScreenId>,
    /// All screen components, keyed by ScreenId.
    screens: HashMap<ScreenId, Box<dyn Component>>,
    /// Whether the app should keep running.
    running: bool,
    /// Connection status indicator.
    connection_status: ConnectionStatus,
    /// Help overlay visibility.
    help_visible: bool,
    /// Search overlay visibility.
    search_active: bool,
    /// Current search query.
    search_query: String,
    /// Terminal size for responsive layout.
    terminal_size: (u16, u16),
    /// Action sender — components can dispatch actions through this.
    action_tx: mpsc::UnboundedSender<Action>,
    /// Action receiver — main loop drains this.
    action_rx: mpsc::UnboundedReceiver<Action>,
    /// Optional controller for live data.
    controller: Option<Controller>,
    /// Cancellation token for the data bridge task.
    data_cancel: CancellationToken,
    /// Pending confirmation dialog (blocks other input while active).
    pending_confirm: Option<ConfirmAction>,
    /// Active notification toast with display timestamp.
    notification: Option<(Notification, Instant)>,
    /// Generation counter for stats requests — prevents stale responses from
    /// overwriting fresh data when the user rapidly switches periods.
    stats_generation: std::sync::Arc<std::sync::atomic::AtomicU64>,
    /// Timestamp of the last stats fetch — drives auto-refresh.
    last_stats_fetch: Option<std::time::Instant>,
    /// Currently selected stats period — preserved for auto-refresh.
    stats_period: StatsPeriod,
}

impl App {
    /// Create a new App with all screens. Optionally accepts a [`Controller`]
    /// for live data — if `None`, the TUI shows the onboarding wizard.
    pub fn new(controller: Option<Controller>) -> Self {
        let (action_tx, action_rx) = mpsc::unbounded_channel();

        let mut screens: HashMap<ScreenId, Box<dyn Component>> =
            create_screens().into_iter().collect();

        // If no controller, show the onboarding wizard instead of the dashboard
        let active_screen = if controller.is_none() {
            screens.insert(
                ScreenId::Setup,
                Box::new(crate::tui::screens::onboarding::OnboardingScreen::new()),
            );
            ScreenId::Setup
        } else {
            ScreenId::Dashboard
        };

        Self {
            active_screen,
            previous_screen: None,
            screens,
            running: true,
            connection_status: ConnectionStatus::default(),
            help_visible: false,
            search_active: false,
            search_query: String::new(),
            terminal_size: (0, 0),
            action_tx,
            action_rx,
            controller,
            data_cancel: CancellationToken::new(),
            pending_confirm: None,
            notification: None,
            stats_generation: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
            last_stats_fetch: None,
            stats_period: StatsPeriod::default(),
        }
    }

    /// Initialize all screen components with the action sender.
    fn init_screens(&mut self) -> Result<()> {
        for screen in self.screens.values_mut() {
            screen.init(self.action_tx.clone())?;
        }

        if let Some(screen) = self.screens.get_mut(&self.active_screen) {
            screen.set_focused(true);
        }

        Ok(())
    }

    /// Run the main event loop. This is the heart of the TUI.
    pub async fn run(&mut self) -> Result<()> {
        let mut tui = Tui::new()?;
        tui.enter()?;
        self.terminal_size = tui.size().unwrap_or((80, 24));
        self.init_screens()?;

        if let Some(controller) = self.controller.clone() {
            let cancel = self.data_cancel.clone();
            let tx = self.action_tx.clone();
            tokio::spawn(async move {
                crate::tui::data_bridge::spawn_data_bridge(controller, tx, cancel).await;
            });
        }

        let mut events = EventReader::new(Duration::from_millis(250), Duration::from_millis(33));

        info!("TUI event loop started");

        while self.running {
            let Some(event) = events.next().await else {
                break;
            };

            match event {
                Event::Key(key) => {
                    if let Some(action) = self.handle_key_event(key)? {
                        self.action_tx.send(action)?;
                    }
                }
                Event::Mouse(mouse) => {
                    if let Some(action) = self.handle_mouse_event(mouse)? {
                        self.action_tx.send(action)?;
                    }
                }
                Event::Resize(w, h) => {
                    self.action_tx.send(Action::Resize(w, h))?;
                }
                Event::Tick => {
                    self.action_tx.send(Action::Tick)?;
                }
                Event::Render => {
                    self.action_tx.send(Action::Render)?;
                }
            }

            while let Ok(action) = self.action_rx.try_recv() {
                self.process_action(&action)?;

                if let Action::Render = action {
                    tui.draw(|frame| self.render(frame))?;
                }
            }
        }

        self.data_cancel.cancel();
        events.stop();
        info!("TUI event loop ended");
        Ok(())
    }
}
