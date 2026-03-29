use crate::cli::args::{ConfigArgs, ConfigCommand, GlobalOpts};
use crate::cli::error::CliError;
use crate::cli::output;
use crate::config::resolve;

use super::interactive::{run_init, store_profile_secrets};
use super::support::{
    empty_profile, format_config_redacted, profile_not_found, resolve_set_target, save_config,
};

#[allow(clippy::too_many_lines)]
pub(super) fn handle(args: ConfigArgs, global: &GlobalOpts) -> Result<(), CliError> {
    match args.command {
        ConfigCommand::Init => run_init(),

        ConfigCommand::Show => {
            let cfg = crate::config::load_config_or_default();
            let out = output::render_single(&global.output, &cfg, format_config_redacted, |_| {
                "config".into()
            });
            output::print_output(&out, global.quiet);
            Ok(())
        }

        ConfigCommand::Set { key, value } => {
            let mut cfg = crate::config::load_config_or_default();
            let active_profile = resolve::active_profile_name(global, &cfg);
            let (profile_name, field) = resolve_set_target(&key, &active_profile)?;

            let profile = cfg
                .profiles
                .entry(profile_name.clone())
                .or_insert_with(empty_profile);

            match field.as_str() {
                "controller" => profile.controller = value,
                "site" => profile.site = value,
                "auth_mode" | "auth-mode" => {
                    if !matches!(value.as_str(), "integration" | "legacy" | "hybrid") {
                        return Err(CliError::Validation {
                            field: "auth_mode".into(),
                            reason: "must be 'integration', 'legacy', or 'hybrid'".into(),
                        });
                    }
                    profile.auth_mode = value;
                }
                "api_key" | "api-key" => profile.api_key = Some(value),
                "api_key_env" | "api-key-env" => profile.api_key_env = Some(value),
                "username" => profile.username = Some(value),
                "insecure" => {
                    profile.insecure = Some(value.parse().map_err(|_| CliError::Validation {
                        field: "insecure".into(),
                        reason: "must be 'true' or 'false'".into(),
                    })?);
                }
                "timeout" => {
                    profile.timeout = Some(value.parse().map_err(|_| CliError::Validation {
                        field: "timeout".into(),
                        reason: "must be a number (seconds)".into(),
                    })?);
                }
                "ca_cert" | "ca-cert" => profile.ca_cert = Some(value.into()),
                other => {
                    return Err(CliError::Validation {
                        field: other.into(),
                        reason: format!(
                            "unknown config key '{other}'. Valid keys: controller, site, \
                             auth_mode, api_key, api_key_env, username, insecure, timeout, ca_cert"
                        ),
                    });
                }
            }

            save_config(&cfg)?;
            eprintln!("✓ Set {field} on profile '{profile_name}'");
            Ok(())
        }

        ConfigCommand::Profiles => {
            let cfg = crate::config::load_config_or_default();
            let default = cfg.default_profile.as_deref().unwrap_or("default");

            if cfg.profiles.is_empty() {
                eprintln!("No profiles configured. Run: unifly config init");
                return Ok(());
            }

            let mut names: Vec<_> = cfg.profiles.keys().collect();
            names.sort();
            for name in names {
                let marker = if name == default { " *" } else { "" };
                println!("{name}{marker}");
            }
            Ok(())
        }

        ConfigCommand::Use { name } => {
            let mut cfg = crate::config::load_config_or_default();

            if !cfg.profiles.contains_key(&name) {
                return Err(profile_not_found(&cfg, name));
            }

            cfg.default_profile = Some(name.clone());
            save_config(&cfg)?;
            eprintln!("✓ Default profile set to '{name}'");
            Ok(())
        }

        ConfigCommand::SetPassword {
            profile,
            profile_flag,
        } => {
            let cfg = crate::config::load_config_or_default();
            let profile_name = profile
                .or(profile_flag)
                .unwrap_or_else(|| resolve::active_profile_name(global, &cfg));

            let prof = cfg
                .profiles
                .get(&profile_name)
                .ok_or_else(|| profile_not_found(&cfg, profile_name.clone()))?;

            store_profile_secrets(&profile_name, &prof.auth_mode)
        }
    }
}
