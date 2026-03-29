use super::*;

impl Controller {
    // ── Connection lifecycle ─────────────────────────────────────

    /// Connect to the controller.
    ///
    /// Detects the platform, authenticates, performs an initial data
    /// refresh, and spawns background tasks (periodic refresh, command
    /// processor).
    #[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
    pub async fn connect(&self) -> Result<(), CoreError> {
        let _ = self
            .inner
            .connection_state
            .send(ConnectionState::Connecting);

        // Fresh child token for this connection (supports reconnect).
        let child = self.inner.cancel.child_token();
        *self.inner.cancel_child.lock().await = child.clone();

        let config = &self.inner.config;
        let transport = build_transport(config);

        match &config.auth {
            AuthCredentials::ApiKey(api_key) => {
                // Detect platform so we use the right URL prefix
                let platform = LegacyClient::detect_platform(&config.url).await?;
                debug!(?platform, "detected controller platform");

                // Integration API client (preferred)
                let integration = IntegrationClient::from_api_key(
                    config.url.as_str(),
                    api_key,
                    &transport,
                    platform,
                )?;

                // Resolve site UUID from Integration API
                let site_id = resolve_site_id(&integration, &config.site).await?;
                debug!(site_id = %site_id, "resolved Integration API site UUID");

                *self.inner.integration_client.lock().await = Some(Arc::new(integration));
                *self.inner.site_id.lock().await = Some(site_id);
            }
            AuthCredentials::Credentials { username, password } => {
                // Legacy-only auth
                let platform = LegacyClient::detect_platform(&config.url).await?;
                debug!(?platform, "detected controller platform");

                let client = LegacyClient::new(
                    config.url.clone(),
                    config.site.clone(),
                    platform,
                    &transport,
                )?;
                client.login(username, password).await?;
                debug!("session authentication successful");

                *self.inner.legacy_client.lock().await = Some(Arc::new(client));
            }
            AuthCredentials::Hybrid {
                api_key,
                username,
                password,
            } => {
                // Hybrid: both Integration API (API key) and Legacy API (session auth)
                let platform = LegacyClient::detect_platform(&config.url).await?;
                debug!(?platform, "detected controller platform (hybrid)");

                // Integration API client
                let integration = IntegrationClient::from_api_key(
                    config.url.as_str(),
                    api_key,
                    &transport,
                    platform,
                )?;

                let site_id = resolve_site_id(&integration, &config.site).await?;
                debug!(site_id = %site_id, "resolved Integration API site UUID");

                *self.inner.integration_client.lock().await = Some(Arc::new(integration));
                *self.inner.site_id.lock().await = Some(site_id);

                // Legacy API client — attempt login but degrade gracefully
                // if it fails. The Integration API is the primary surface;
                // Legacy adds events, stats, and admin ops.
                match LegacyClient::new(
                    config.url.clone(),
                    config.site.clone(),
                    platform,
                    &transport,
                ) {
                    Ok(client) => match client.login(username, password).await {
                        Ok(()) => {
                            debug!("legacy session authentication successful (hybrid)");
                            *self.inner.legacy_client.lock().await = Some(Arc::new(client));
                        }
                        Err(e) => {
                            let msg = format!(
                                "Legacy login failed: {e} — events, health stats, and client traffic will be unavailable"
                            );
                            warn!("{msg}");
                            self.inner.warnings.lock().await.push(msg);
                        }
                    },
                    Err(e) => {
                        let msg = format!("Legacy client setup failed: {e}");
                        warn!("{msg}");
                        self.inner.warnings.lock().await.push(msg);
                    }
                }
            }
            AuthCredentials::Cloud { api_key, host_id } => {
                let integration = IntegrationClient::from_api_key(
                    config.url.as_str(),
                    api_key,
                    &transport,
                    crate::ControllerPlatform::Cloud,
                )?;

                let site_id = if let Ok(uuid) = uuid::Uuid::parse_str(&config.site) {
                    uuid
                } else if let Ok(uuid) = uuid::Uuid::parse_str(host_id) {
                    uuid
                } else {
                    resolve_site_id(&integration, &config.site).await?
                };
                debug!(site_id = %site_id, "resolved cloud Integration API site UUID");

                *self.inner.integration_client.lock().await = Some(Arc::new(integration));
                *self.inner.site_id.lock().await = Some(site_id);

                let msg =
                    "Cloud auth mode active: Legacy API and WebSocket features are unavailable"
                        .to_string();
                self.inner.warnings.lock().await.push(msg);
            }
        }

        // Initial data load
        self.full_refresh().await?;

        // Spawn background tasks
        let mut handles = self.inner.task_handles.lock().await;

        if let Some(rx) = self.inner.command_rx.lock().await.take() {
            let ctrl = self.clone();
            handles.push(tokio::spawn(command_processor_task(ctrl, rx)));
        }

        let interval_secs = config.refresh_interval_secs;
        if interval_secs > 0 {
            let ctrl = self.clone();
            let cancel = child.clone();
            handles.push(tokio::spawn(refresh::refresh_task(
                ctrl,
                interval_secs,
                cancel,
            )));
        }

        if config.websocket_enabled {
            self.spawn_websocket(&child, &mut handles).await;
        }

        let _ = self.inner.connection_state.send(ConnectionState::Connected);
        info!("connected to controller");
        Ok(())
    }

    /// Spawn the WebSocket event stream and a bridge task that converts
    /// raw [`UnifiEvent`]s into domain [`Event`]s and broadcasts them.
    ///
    /// Non-fatal on failure — the TUI falls back to polling.
    async fn spawn_websocket(&self, cancel: &CancellationToken, handles: &mut Vec<JoinHandle<()>>) {
        let Some(legacy) = self.inner.legacy_client.lock().await.clone() else {
            debug!("no legacy client — WebSocket unavailable");
            return;
        };

        let platform = legacy.platform();
        let Some(ws_path_template) = platform.websocket_path() else {
            debug!("platform does not support WebSocket");
            return;
        };

        let ws_path = ws_path_template.replace("{site}", &self.inner.config.site);
        let base_url = &self.inner.config.url;
        let scheme = if base_url.scheme() == "https" {
            "wss"
        } else {
            "ws"
        };
        let host = base_url.host_str().unwrap_or("localhost");
        let ws_url_str = match base_url.port() {
            Some(port) => format!("{scheme}://{host}:{port}{ws_path}"),
            None => format!("{scheme}://{host}{ws_path}"),
        };
        let ws_url = match url::Url::parse(&ws_url_str) {
            Ok(url) => url,
            Err(error) => {
                warn!(error = %error, url = %ws_url_str, "invalid WebSocket URL");
                return;
            }
        };

        let cookie = legacy.cookie_header();

        if cookie.is_none() {
            warn!("no session cookie — WebSocket requires legacy auth (skipping)");
            return;
        }

        let ws_tls = tls_to_transport(&self.inner.config.tls);
        let ws_cancel = cancel.child_token();
        let handle = match WebSocketHandle::connect(
            ws_url,
            ReconnectConfig::default(),
            ws_cancel.clone(),
            cookie,
            ws_tls,
        ) {
            Ok(handle) => handle,
            Err(error) => {
                warn!(error = %error, "WebSocket connection failed (non-fatal)");
                return;
            }
        };

        // Bridge task: WS events → domain Events → broadcast channel.
        // Also extracts real-time device stats from `device:sync` messages
        // to feed the dashboard chart without waiting for full_refresh().
        let mut ws_rx = handle.subscribe();
        let event_tx = self.inner.event_tx.clone();
        let store = Arc::clone(&self.inner.store);
        let bridge_cancel = ws_cancel;

        handles.push(tokio::spawn(async move {
            loop {
                tokio::select! {
                    biased;
                    () = bridge_cancel.cancelled() => break,
                    result = ws_rx.recv() => {
                        match result {
                            Ok(ws_event) => {
                                store.mark_ws_event(chrono::Utc::now());

                                if ws_event.key == "device:sync" || ws_event.key == "device:update" {
                                    apply_device_sync(&store, &ws_event.extra);
                                }

                                if ws_event.key.starts_with("EVT_") {
                                    let event = crate::model::event::Event::from((*ws_event).clone());
                                    let _ = event_tx.send(Arc::new(event));
                                }
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                                warn!(skipped, "WS bridge: receiver lagged");
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                        }
                    }
                }
            }
        }));

        *self.inner.ws_handle.lock().await = Some(handle);
        info!("WebSocket event stream spawned (handshake in progress)");
    }

    /// Disconnect from the controller.
    ///
    /// Cancels background tasks, logs out if session-based, and resets
    /// the connection state to [`Disconnected`](ConnectionState::Disconnected).
    pub async fn disconnect(&self) {
        self.inner.cancel_child.lock().await.cancel();

        if let Some(handle) = self.inner.ws_handle.lock().await.take() {
            handle.shutdown();
        }

        let mut handles = self.inner.task_handles.lock().await;
        for handle in handles.drain(..) {
            let _ = handle.await;
        }

        let legacy = self.inner.legacy_client.lock().await.clone();

        if matches!(
            self.inner.config.auth,
            AuthCredentials::Credentials { .. } | AuthCredentials::Hybrid { .. }
        ) && let Some(client) = legacy
            && let Err(error) = client.logout().await
        {
            warn!(error = %error, "logout failed (non-fatal)");
        }

        *self.inner.legacy_client.lock().await = None;
        *self.inner.integration_client.lock().await = None;
        *self.inner.site_id.lock().await = None;

        {
            let (tx, rx) = mpsc::channel(COMMAND_CHANNEL_SIZE);
            *self.inner.command_tx.lock().await = tx;
            *self.inner.command_rx.lock().await = Some(rx);
        }

        let _ = self
            .inner
            .connection_state
            .send(ConnectionState::Disconnected);
        debug!("disconnected");
    }
}
