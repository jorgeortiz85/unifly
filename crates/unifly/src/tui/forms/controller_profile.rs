#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuthMode {
    ApiKey,
    Legacy,
    Hybrid,
}

impl AuthMode {
    pub(crate) const ALL: [Self; 3] = [Self::ApiKey, Self::Legacy, Self::Hybrid];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::ApiKey => "API Key (Integration API)",
            Self::Legacy => "Username / Password (Legacy API)",
            Self::Hybrid => "Hybrid (API Key + Credentials)",
        }
    }

    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::ApiKey => "Recommended for most setups",
            Self::Legacy => "For stats, events, and admin operations",
            Self::Hybrid => "Full access to both API surfaces",
        }
    }

    pub(crate) fn config_value(self) -> &'static str {
        match self {
            Self::ApiKey => "integration",
            Self::Legacy => "legacy",
            Self::Hybrid => "hybrid",
        }
    }

    pub(crate) fn from_config(value: &str) -> Self {
        match value {
            "legacy" => Self::Legacy,
            "hybrid" => Self::Hybrid,
            _ => Self::ApiKey,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ControllerProfileDraft {
    pub(crate) url: String,
    pub(crate) auth_mode: AuthMode,
    pub(crate) api_key: String,
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) site: String,
    pub(crate) insecure: bool,
}

impl Default for ControllerProfileDraft {
    fn default() -> Self {
        Self {
            url: "https://192.168.1.1".into(),
            auth_mode: AuthMode::ApiKey,
            api_key: String::new(),
            username: String::new(),
            password: String::new(),
            site: "default".into(),
            insecure: true,
        }
    }
}

impl ControllerProfileDraft {
    pub(crate) fn from_profile(profile: &crate::config::Profile) -> Self {
        Self {
            url: profile.controller.clone(),
            auth_mode: AuthMode::from_config(&profile.auth_mode),
            api_key: profile.api_key.clone().unwrap_or_default(),
            username: profile.username.clone().unwrap_or_default(),
            password: profile.password.clone().unwrap_or_default(),
            site: profile.site.clone(),
            insecure: profile.insecure.unwrap_or(false),
        }
    }

    pub(crate) fn validate_url(&self) -> std::result::Result<(), String> {
        let trimmed = self.url.trim();
        if trimmed.is_empty() {
            return Err("URL cannot be empty".into());
        }
        if trimmed.parse::<url::Url>().is_err() {
            return Err("Invalid URL format".into());
        }
        Ok(())
    }

    pub(crate) fn validate_credentials(&self) -> std::result::Result<(), String> {
        match self.auth_mode {
            AuthMode::ApiKey => {
                if self.api_key.trim().is_empty() {
                    return Err("API key cannot be empty".into());
                }
            }
            AuthMode::Legacy => {
                if self.username.trim().is_empty() {
                    return Err("Username cannot be empty".into());
                }
                if self.password.is_empty() {
                    return Err("Password cannot be empty".into());
                }
            }
            AuthMode::Hybrid => {
                if self.api_key.trim().is_empty() {
                    return Err("API key cannot be empty".into());
                }
                if self.username.trim().is_empty() {
                    return Err("Username cannot be empty".into());
                }
                if self.password.is_empty() {
                    return Err("Password cannot be empty".into());
                }
            }
        }

        Ok(())
    }

    pub(crate) fn validate_complete(&self) -> std::result::Result<(), String> {
        self.validate_url()?;
        self.validate_credentials()?;

        if self.site.trim().is_empty() {
            return Err("Site name cannot be empty".into());
        }

        Ok(())
    }

    pub(crate) fn to_profile(&self) -> crate::config::Profile {
        crate::config::Profile {
            controller: self.url.trim().to_string(),
            site: self.site.trim().to_string(),
            auth_mode: self.auth_mode.config_value().to_string(),
            api_key: match self.auth_mode {
                AuthMode::ApiKey | AuthMode::Hybrid => Some(self.api_key.trim().to_string()),
                AuthMode::Legacy => None,
            },
            api_key_env: None,
            username: match self.auth_mode {
                AuthMode::Legacy | AuthMode::Hybrid => Some(self.username.trim().to_string()),
                AuthMode::ApiKey => None,
            },
            password: match self.auth_mode {
                AuthMode::Legacy | AuthMode::Hybrid => Some(self.password.clone()),
                AuthMode::ApiKey => None,
            },
            totp_env: None,
            ca_cert: None,
            insecure: Some(self.insecure),
            timeout: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AuthMode, ControllerProfileDraft};

    fn sample_draft() -> ControllerProfileDraft {
        ControllerProfileDraft {
            url: "https://console.example.com".into(),
            auth_mode: AuthMode::Hybrid,
            api_key: "api-key".into(),
            username: "bliss".into(),
            password: "hunter2".into(),
            site: "default".into(),
            insecure: false,
        }
    }

    #[test]
    fn validate_complete_rejects_invalid_url() {
        let mut draft = sample_draft();
        draft.url = "not a url".into();

        assert_eq!(draft.validate_complete(), Err("Invalid URL format".into()));
    }

    #[test]
    fn validate_credentials_requires_password_for_hybrid() {
        let mut draft = sample_draft();
        draft.password.clear();

        assert_eq!(
            draft.validate_credentials(),
            Err("Password cannot be empty".into())
        );
    }

    #[test]
    fn to_profile_omits_non_selected_auth_fields() {
        let mut draft = sample_draft();
        draft.auth_mode = AuthMode::ApiKey;

        let profile = draft.to_profile();

        assert_eq!(profile.api_key.as_deref(), Some("api-key"));
        assert_eq!(profile.username, None);
        assert_eq!(profile.password, None);
    }

    #[test]
    fn from_profile_restores_draft_fields() {
        let profile = crate::config::Profile {
            controller: "https://console.example.com".into(),
            site: "default".into(),
            auth_mode: "legacy".into(),
            api_key: None,
            api_key_env: None,
            username: Some("bliss".into()),
            password: Some("hunter2".into()),
            totp_env: None,
            ca_cert: None,
            insecure: Some(false),
            timeout: None,
        };

        let draft = ControllerProfileDraft::from_profile(&profile);

        assert_eq!(draft.url, "https://console.example.com");
        assert_eq!(draft.auth_mode, AuthMode::Legacy);
        assert_eq!(draft.username, "bliss");
        assert_eq!(draft.password, "hunter2");
        assert!(!draft.insecure);
    }
}
