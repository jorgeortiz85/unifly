use std::collections::HashMap;

use dialoguer::{Input, Select};

use crate::cli::error::CliError;
use crate::config::{Config, Defaults, Profile};

use super::support::save_config;

fn prompt_err(error: impl std::fmt::Display) -> CliError {
    CliError::Validation {
        field: "interactive".into(),
        reason: format!("prompt failed: {error}"),
    }
}

fn prompt_credentials() -> Result<(String, String), CliError> {
    let user: String = Input::new()
        .with_prompt("Username")
        .interact_text()
        .map_err(prompt_err)?;

    let pass = rpassword::prompt_password("Password: ").map_err(prompt_err)?;

    if user.is_empty() || pass.is_empty() {
        return Err(CliError::Validation {
            field: "credentials".into(),
            reason: "username and password cannot be empty".into(),
        });
    }

    Ok((user, pass))
}

fn prompt_keyring_storage(
    secret: &str,
    keyring_key: &str,
    prompt: &str,
    label: &str,
) -> Result<Option<String>, CliError> {
    let choices = &[
        "Store in system keyring (recommended)",
        "Save to config file (plaintext)",
    ];
    let selection = Select::new()
        .with_prompt(prompt)
        .items(choices)
        .default(0)
        .interact()
        .map_err(prompt_err)?;

    if selection == 0 {
        let entry =
            keyring::Entry::new("unifly", keyring_key).map_err(|error| CliError::Validation {
                field: "keyring".into(),
                reason: format!("failed to access keyring: {error}"),
            })?;
        entry
            .set_password(secret)
            .map_err(|error| CliError::Validation {
                field: "keyring".into(),
                reason: format!("failed to store {label} in keyring: {error}"),
            })?;
        eprintln!("   ✓ {label} stored in system keyring");
        Ok(None)
    } else {
        Ok(Some(secret.to_owned()))
    }
}

fn store_secret(key: &str, label: &str) -> Result<(), CliError> {
    let secret = rpassword::prompt_password(label).map_err(prompt_err)?;
    if secret.is_empty() {
        return Err(CliError::Validation {
            field: "secret".into(),
            reason: "value cannot be empty".into(),
        });
    }
    let entry = keyring::Entry::new("unifly", key).map_err(|error| CliError::Validation {
        field: "keyring".into(),
        reason: format!("failed to access keyring: {error}"),
    })?;
    entry
        .set_password(&secret)
        .map_err(|error| CliError::Validation {
            field: "keyring".into(),
            reason: format!("failed to store secret in keyring: {error}"),
        })?;
    Ok(())
}

#[allow(clippy::too_many_lines)]
pub(super) fn run_init() -> Result<(), CliError> {
    let config_path = crate::config::config_path();
    eprintln!("✨ UniFi CLI — configuration wizard");
    eprintln!("   Config path: {}\n", config_path.display());

    let profile_name: String = Input::new()
        .with_prompt("Profile name")
        .default("default".into())
        .interact_text()
        .map_err(prompt_err)?;

    let controller: String = Input::new()
        .with_prompt("Controller URL")
        .default("https://192.168.1.1".into())
        .interact_text()
        .map_err(prompt_err)?;

    let auth_choices = &[
        "API Key (recommended)",
        "Username/Password",
        "Hybrid (API key + credentials for full access)",
    ];
    let auth_selection = Select::new()
        .with_prompt("Authentication method")
        .items(auth_choices)
        .default(0)
        .interact()
        .map_err(prompt_err)?;

    let (auth_mode, api_key, username, password) = match auth_selection {
        0 => {
            let key = rpassword::prompt_password("API key: ").map_err(prompt_err)?;

            if key.is_empty() {
                return Err(CliError::Validation {
                    field: "api_key".into(),
                    reason: "API key cannot be empty".into(),
                });
            }

            let api_key_field = prompt_keyring_storage(
                &key,
                &format!("{profile_name}/api-key"),
                "Where to store the API key?",
                "API key",
            )?;

            ("integration".to_string(), api_key_field, None, None)
        }
        1 => {
            let (user, pass) = prompt_credentials()?;

            let password_field = prompt_keyring_storage(
                &pass,
                &format!("{profile_name}/password"),
                "Where to store the password?",
                "Password",
            )?;

            ("legacy".to_string(), None, Some(user), password_field)
        }
        _ => {
            eprintln!("\n   Hybrid mode uses an API key for the Integration API");
            eprintln!("   and username/password for the Legacy API (stats, events, alarms).\n");

            let key = rpassword::prompt_password("API key: ").map_err(prompt_err)?;

            if key.is_empty() {
                return Err(CliError::Validation {
                    field: "api_key".into(),
                    reason: "API key cannot be empty".into(),
                });
            }

            let api_key_field = prompt_keyring_storage(
                &key,
                &format!("{profile_name}/api-key"),
                "Where to store the API key?",
                "API key",
            )?;

            let (user, pass) = prompt_credentials()?;

            let password_field = prompt_keyring_storage(
                &pass,
                &format!("{profile_name}/password"),
                "Where to store the password?",
                "Password",
            )?;

            (
                "hybrid".to_string(),
                api_key_field,
                Some(user),
                password_field,
            )
        }
    };

    let site: String = Input::new()
        .with_prompt("Site name")
        .default("default".into())
        .interact_text()
        .map_err(prompt_err)?;

    let profile = Profile {
        controller,
        site,
        auth_mode,
        api_key,
        api_key_env: None,
        username,
        password,
        totp_env: None,
        ca_cert: None,
        insecure: None,
        timeout: None,
    };

    let mut profiles = HashMap::new();
    profiles.insert(profile_name.clone(), profile);

    let cfg = Config {
        default_profile: Some(profile_name.clone()),
        defaults: Defaults::default(),
        profiles,
    };

    save_config(&cfg)?;

    eprintln!("\n✓ Configuration written to {}", config_path.display());
    eprintln!("  Active profile: {profile_name}");
    eprintln!("\n  Test it: unifly system info --insecure");

    Ok(())
}

pub(super) fn store_profile_secrets(profile_name: &str, auth_mode: &str) -> Result<(), CliError> {
    match auth_mode {
        "hybrid" => {
            store_secret(&format!("{profile_name}/api-key"), "API key: ")?;
            store_secret(&format!("{profile_name}/password"), "Password: ")?;
        }
        "integration" => {
            store_secret(&format!("{profile_name}/api-key"), "API key: ")?;
        }
        _ => {
            store_secret(&format!("{profile_name}/password"), "Password: ")?;
        }
    }

    eprintln!("✓ Secret(s) stored in system keyring for profile '{profile_name}'");
    Ok(())
}
