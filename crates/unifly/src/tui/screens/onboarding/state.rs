use super::{AuthMode, CredentialField, OnboardingScreen, WizardStep};

impl WizardStep {
    pub(super) fn index(self) -> usize {
        match self {
            Self::Welcome => 0,
            Self::Url => 1,
            Self::AuthMode => 2,
            Self::Credentials => 3,
            Self::Site => 4,
            Self::Testing => 5,
            Self::Done => 6,
        }
    }
}

impl AuthMode {
    pub(super) const ALL: [AuthMode; 3] = [Self::ApiKey, Self::Legacy, Self::Hybrid];

    pub(super) fn label(self) -> &'static str {
        match self {
            Self::ApiKey => "API Key (Integration API)",
            Self::Legacy => "Username / Password (Legacy API)",
            Self::Hybrid => "Hybrid (API Key + Credentials)",
        }
    }

    pub(super) fn description(self) -> &'static str {
        match self {
            Self::ApiKey => "Recommended for most setups",
            Self::Legacy => "For stats, events, and admin operations",
            Self::Hybrid => "Full access to both API surfaces",
        }
    }

    pub(super) fn config_value(self) -> &'static str {
        match self {
            Self::ApiKey => "integration",
            Self::Legacy => "legacy",
            Self::Hybrid => "hybrid",
        }
    }
}

impl OnboardingScreen {
    pub fn new() -> Self {
        Self {
            focused: false,
            action_tx: None,
            step: WizardStep::Welcome,
            url_input: "https://192.168.1.1".into(),
            auth_mode: AuthMode::ApiKey,
            auth_mode_index: 0,
            api_key_input: String::new(),
            username_input: String::new(),
            password_input: String::new(),
            site_input: "default".into(),
            cred_field: CredentialField::ApiKey,
            show_password: false,
            testing: false,
            test_error: None,
            error: None,
            throbber_state: throbber_widgets_tui::ThrobberState::default(),
        }
    }

    pub(super) fn advance(&mut self) {
        self.error = None;
        match self.step {
            WizardStep::Welcome => self.step = WizardStep::Url,
            WizardStep::Url => {
                let trimmed = self.url_input.trim();
                if trimmed.is_empty() {
                    self.error = Some("URL cannot be empty".into());
                    return;
                }
                if trimmed.parse::<url::Url>().is_err() {
                    self.error = Some("Invalid URL format".into());
                    return;
                }
                self.step = WizardStep::AuthMode;
            }
            WizardStep::AuthMode => {
                self.auth_mode = AuthMode::ALL[self.auth_mode_index];
                self.cred_field = match self.auth_mode {
                    AuthMode::ApiKey | AuthMode::Hybrid => CredentialField::ApiKey,
                    AuthMode::Legacy => CredentialField::Username,
                };
                self.step = WizardStep::Credentials;
            }
            WizardStep::Credentials => {
                if let Err(message) = self.validate_credentials() {
                    self.error = Some(message);
                    return;
                }
                self.step = WizardStep::Site;
            }
            WizardStep::Site => {
                if self.site_input.trim().is_empty() {
                    self.error = Some("Site name cannot be empty".into());
                    return;
                }
                self.step = WizardStep::Testing;
                self.start_connection_test();
            }
            WizardStep::Testing => {}
            WizardStep::Done => {
                self.send_completion();
            }
        }
    }

    pub(super) fn go_back(&mut self) {
        self.error = None;
        self.test_error = None;
        match self.step {
            WizardStep::Welcome => {}
            WizardStep::Url => self.step = WizardStep::Welcome,
            WizardStep::AuthMode => self.step = WizardStep::Url,
            WizardStep::Credentials => self.step = WizardStep::AuthMode,
            WizardStep::Site => self.step = WizardStep::Credentials,
            WizardStep::Testing => {
                self.testing = false;
                self.step = WizardStep::Site;
            }
            WizardStep::Done => self.step = WizardStep::Site,
        }
    }

    pub(super) fn validate_credentials(&self) -> std::result::Result<(), String> {
        match self.auth_mode {
            AuthMode::ApiKey => {
                if self.api_key_input.trim().is_empty() {
                    return Err("API key cannot be empty".into());
                }
            }
            AuthMode::Legacy => {
                if self.username_input.trim().is_empty() {
                    return Err("Username cannot be empty".into());
                }
                if self.password_input.is_empty() {
                    return Err("Password cannot be empty".into());
                }
            }
            AuthMode::Hybrid => {
                if self.api_key_input.trim().is_empty() {
                    return Err("API key cannot be empty".into());
                }
                if self.username_input.trim().is_empty() {
                    return Err("Username cannot be empty".into());
                }
                if self.password_input.is_empty() {
                    return Err("Password cannot be empty".into());
                }
            }
        }
        Ok(())
    }

    fn build_profile(&self) -> crate::config::Profile {
        crate::config::Profile {
            controller: self.url_input.trim().to_string(),
            site: self.site_input.trim().to_string(),
            auth_mode: self.auth_mode.config_value().to_string(),
            api_key: match self.auth_mode {
                AuthMode::ApiKey | AuthMode::Hybrid => Some(self.api_key_input.trim().to_string()),
                AuthMode::Legacy => None,
            },
            api_key_env: None,
            username: match self.auth_mode {
                AuthMode::Legacy | AuthMode::Hybrid => Some(self.username_input.trim().to_string()),
                AuthMode::ApiKey => None,
            },
            password: match self.auth_mode {
                AuthMode::Legacy | AuthMode::Hybrid => Some(self.password_input.clone()),
                AuthMode::ApiKey => None,
            },
            ca_cert: None,
            insecure: Some(true),
            timeout: None,
        }
    }

    fn start_connection_test(&mut self) {
        self.testing = true;
        self.test_error = None;

        let profile = self.build_profile();
        let profile_name = "default".to_string();

        let Some(tx) = self.action_tx.clone() else {
            return;
        };

        tokio::spawn(async move {
            let result = match crate::config::profile_to_controller_config(&profile, &profile_name)
            {
                Ok(config) => {
                    let controller = unifly_api::Controller::new(config);
                    match controller.connect().await {
                        Ok(()) => {
                            controller.disconnect().await;
                            let cfg = crate::config::Config {
                                default_profile: Some(profile_name),
                                defaults: crate::config::Defaults::default(),
                                profiles: {
                                    let mut profiles = std::collections::HashMap::new();
                                    profiles.insert("default".to_string(), profile);
                                    profiles
                                },
                            };
                            if let Err(error) = crate::config::save_config(&cfg) {
                                Err(format!("Connected, but failed to save config: {error}"))
                            } else {
                                Ok(())
                            }
                        }
                        Err(error) => Err(format!("{error}")),
                    }
                }
                Err(error) => Err(format!("{error}")),
            };

            let _ = tx.send(crate::tui::action::Action::OnboardingTestResult(result));
        });
    }

    pub(super) fn send_completion(&self) {
        let profile = self.build_profile();
        let profile_name = "default";

        let Some(tx) = self.action_tx.clone() else {
            return;
        };

        match crate::config::profile_to_controller_config(&profile, profile_name) {
            Ok(config) => {
                let _ = tx.send(crate::tui::action::Action::OnboardingComplete {
                    profile_name: profile_name.to_string(),
                    config: Box::new(config),
                });
            }
            Err(error) => {
                let _ = tx.send(crate::tui::action::Action::Notify(
                    crate::tui::action::Notification::error(format!("{error}")),
                ));
            }
        }
    }

    pub(super) fn active_input_mut(&mut self) -> Option<&mut String> {
        match self.step {
            WizardStep::Url => Some(&mut self.url_input),
            WizardStep::Site => Some(&mut self.site_input),
            WizardStep::Credentials => match self.cred_field {
                CredentialField::ApiKey => Some(&mut self.api_key_input),
                CredentialField::Username => Some(&mut self.username_input),
                CredentialField::Password => Some(&mut self.password_input),
            },
            _ => None,
        }
    }

    pub(super) fn next_cred_field(&mut self) {
        self.cred_field = match (self.auth_mode, self.cred_field) {
            (AuthMode::Legacy | AuthMode::Hybrid, CredentialField::Username) => {
                CredentialField::Password
            }
            (AuthMode::Hybrid, CredentialField::Password) | (AuthMode::ApiKey, _) => {
                CredentialField::ApiKey
            }
            (AuthMode::Legacy, _) | (AuthMode::Hybrid, CredentialField::ApiKey) => {
                CredentialField::Username
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::{AuthMode, CredentialField, OnboardingScreen, WizardStep};

    #[test]
    fn advance_rejects_invalid_url() {
        let mut screen = OnboardingScreen::new();
        screen.step = WizardStep::Url;
        screen.url_input = "not a url".into();

        screen.advance();

        assert_eq!(screen.step, WizardStep::Url);
        assert_eq!(screen.error.as_deref(), Some("Invalid URL format"));
    }

    #[test]
    fn auth_mode_step_sets_legacy_credential_field() {
        let mut screen = OnboardingScreen::new();
        screen.step = WizardStep::AuthMode;
        screen.auth_mode_index = 1;

        screen.advance();

        assert_eq!(screen.step, WizardStep::Credentials);
        assert_eq!(screen.auth_mode, AuthMode::Legacy);
        assert_eq!(screen.cred_field, CredentialField::Username);
    }

    #[test]
    fn validate_credentials_requires_password_for_hybrid() {
        let mut screen = OnboardingScreen::new();
        screen.auth_mode = AuthMode::Hybrid;
        screen.api_key_input = "api-key".into();
        screen.username_input = "bliss".into();
        screen.password_input.clear();

        assert_eq!(
            screen.validate_credentials(),
            Err("Password cannot be empty".into())
        );
    }
}
