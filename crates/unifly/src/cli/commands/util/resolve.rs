use unifly_api::{Controller, EntityId, MacAddress};

use crate::cli::error::CliError;

pub fn resolve_device_id(controller: &Controller, identifier: &str) -> Result<EntityId, CliError> {
    let snapshot = controller.devices_snapshot();
    for device in snapshot.iter() {
        if device.id.to_string() == identifier || device.mac.to_string() == identifier {
            return Ok(device.id.clone());
        }
    }

    Err(CliError::NotFound {
        resource_type: "device".into(),
        identifier: identifier.into(),
        list_command: "devices list".into(),
    })
}

pub fn resolve_device_mac(
    controller: &Controller,
    identifier: &str,
) -> Result<MacAddress, CliError> {
    let snapshot = controller.devices_snapshot();
    for device in snapshot.iter() {
        if device.id.to_string() == identifier || device.mac.to_string() == identifier {
            return Ok(device.mac.clone());
        }
    }

    identifier
        .parse::<MacAddress>()
        .map_err(|_| CliError::Validation {
            field: "device".into(),
            reason: "expected a device UUID from the snapshot or a valid MAC address".into(),
        })
}

#[allow(dead_code)]
pub fn resolve_client_id(controller: &Controller, identifier: &str) -> Result<EntityId, CliError> {
    let snapshot = controller.clients_snapshot();
    for client in snapshot.iter() {
        if client.id.to_string() == identifier || client.mac.to_string() == identifier {
            return Ok(client.id.clone());
        }
    }

    Err(CliError::NotFound {
        resource_type: "client".into(),
        identifier: identifier.into(),
        list_command: "clients list".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::resolve_device_mac;
    use unifly_api::{Controller, ControllerConfig};

    #[test]
    fn resolve_device_mac_accepts_valid_fallback_mac() {
        let controller = Controller::new(ControllerConfig::default());
        match resolve_device_mac(&controller, "AABBCCDDEEFF") {
            Ok(mac) => assert_eq!(mac.as_str(), "aa:bb:cc:dd:ee:ff"),
            Err(error) => panic!("expected valid MAC fallback, got {error:?}"),
        }
    }

    #[test]
    fn resolve_device_mac_rejects_invalid_fallback_identifier() {
        let controller = Controller::new(ControllerConfig::default());
        match resolve_device_mac(&controller, "definitely-not-a-mac") {
            Err(crate::cli::error::CliError::Validation { field, .. }) => {
                assert_eq!(field, "device");
            }
            Ok(mac) => panic!("expected validation error, got {mac:?}"),
            Err(other) => panic!("expected validation error, got {other:?}"),
        }
    }
}
