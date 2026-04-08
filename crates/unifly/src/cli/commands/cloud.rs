pub mod fleet_devices;
pub mod fleet_sites;
pub mod hosts;
pub mod isp;
pub mod sdwan;

use std::time::Duration;

use secrecy::SecretString;
use unifly_api::site_manager_types::Host;
use unifly_api::{CoreError, SiteManagerClient, TlsMode, TransportConfig};

use crate::cli::args::{CloudArgs, CloudCommand, GlobalOpts};
use crate::cli::error::CliError;
use crate::config::{self, Profile};

pub async fn handle(args: CloudArgs, global: &GlobalOpts) -> Result<(), CliError> {
    let client = build_site_manager_client(global)?;

    match args.command {
        CloudCommand::Hosts(args) => hosts::handle(&client, args, global).await,
        CloudCommand::Sites(args) => fleet_sites::handle(&client, args, global).await,
        CloudCommand::Devices(args) => fleet_devices::handle(&client, args, global).await,
        CloudCommand::Isp(args) => isp::handle(&client, args, global).await,
        CloudCommand::Sdwan(args) => sdwan::handle(&client, args, global).await,
    }
}

pub(crate) fn build_site_manager_client(
    global: &GlobalOpts,
) -> Result<SiteManagerClient, CliError> {
    let cfg = config::resolve::load_config_or_default();
    let profile_name = config::resolve::active_profile_name(global, &cfg);
    let profile = cfg.profiles.get(&profile_name);

    let api_key = resolve_cloud_api_key(profile, &profile_name, global)?;
    let controller = resolve_site_manager_url(profile, global);
    let transport = TransportConfig {
        tls: TlsMode::System,
        timeout: Duration::from_secs(global.timeout),
        cookie_jar: None,
    };

    SiteManagerClient::from_api_key(&controller, &api_key, &transport).map_err(api_error)
}

pub async fn auto_resolve_host_id(global: &GlobalOpts) -> Result<String, CliError> {
    let client = build_site_manager_client(global)?;
    let hosts = client.list_hosts().await.map_err(api_error)?;
    choose_host_id(&hosts)
}

pub(crate) fn api_error(error: unifly_api::Error) -> CliError {
    CliError::from(CoreError::from(error))
}

fn resolve_cloud_api_key(
    profile: Option<&Profile>,
    profile_name: &str,
    global: &GlobalOpts,
) -> Result<SecretString, CliError> {
    if let Some(api_key) = &global.api_key {
        return Ok(SecretString::from(api_key.clone()));
    }

    let profile = profile.ok_or_else(|| CliError::NoCredentials {
        profile: profile_name.to_owned(),
    })?;

    config::resolve_api_key(profile, profile_name).map_err(CliError::from)
}

fn resolve_site_manager_url(profile: Option<&Profile>, global: &GlobalOpts) -> String {
    global
        .controller
        .clone()
        .or_else(|| {
            profile.and_then(|profile| {
                if profile.auth_mode == "cloud" && !profile.controller.trim().is_empty() {
                    Some(profile.controller.clone())
                } else {
                    None
                }
            })
        })
        .unwrap_or_else(|| config::DEFAULT_CLOUD_CONTROLLER_URL.into())
}

fn choose_host_id(hosts: &[Host]) -> Result<String, CliError> {
    match hosts {
        [] => Err(CliError::Validation {
            field: "host_id".into(),
            reason: "no cloud consoles are accessible with the current API key".into(),
        }),
        [host] => Ok(host.id.clone()),
        _ => {
            let owner_hosts: Vec<&Host> =
                hosts.iter().filter(|host| host.is_owner_host()).collect();
            if owner_hosts.len() == 1 {
                return Ok(owner_hosts[0].id.clone());
            }

            let available = hosts
                .iter()
                .map(|host| format!("{} ({})", host.display_name(), host.id))
                .collect::<Vec<_>>()
                .join(", ");

            Err(CliError::Validation {
                field: "host_id".into(),
                reason: format!(
                    "multiple cloud consoles are available; set host_id in your profile or pass --host-id. Available hosts: {available}"
                ),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use unifly_api::site_manager_types::Host;

    use super::choose_host_id;
    use crate::cli::error::CliError;

    fn host(id: &str, name: &str, is_owner: bool) -> Host {
        Host {
            id: id.into(),
            name: Some(name.into()),
            model: None,
            firmware_version: None,
            mac_address: None,
            reported_state: None,
            user_data: None,
            is_owner: Some(is_owner),
            extra: std::collections::BTreeMap::default(),
        }
    }

    #[test]
    fn choose_host_prefers_single_owner_console() {
        let hosts = vec![host("host-1", "Home", true), host("host-2", "Lab", false)];
        let selected = choose_host_id(&hosts).expect("owner host should be selected");
        assert_eq!(selected, "host-1");
    }

    #[test]
    fn choose_host_errors_when_multiple_non_owner_hosts_exist() {
        let hosts = vec![host("host-1", "Home", false), host("host-2", "Lab", false)];
        let error = choose_host_id(&hosts).expect_err("selection should be ambiguous");
        assert!(matches!(error, CliError::Validation { field, .. } if field == "host_id"));
    }
}
