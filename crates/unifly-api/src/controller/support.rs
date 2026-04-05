use std::net::Ipv6Addr;
use std::sync::Arc;

use crate::config::{ControllerConfig, TlsVerification};
use crate::core_error::CoreError;
use crate::model::{EntityId, HealthSummary, MacAddress};
use crate::store::DataStore;
use crate::transport::{TlsMode, TransportConfig};
use crate::{IntegrationClient, SessionClient};

use super::Controller;

fn parse_ipv6_text(raw: &str) -> Option<Ipv6Addr> {
    let candidate = raw.trim().split('/').next().unwrap_or(raw).trim();
    candidate.parse::<Ipv6Addr>().ok()
}

fn pick_ipv6_from_value(value: &serde_json::Value) -> Option<String> {
    let mut first_link_local: Option<String> = None;

    let iter: Box<dyn Iterator<Item = &serde_json::Value> + '_> = match value {
        serde_json::Value::Array(items) => Box::new(items.iter()),
        _ => Box::new(std::iter::once(value)),
    };

    for item in iter {
        if let Some(ipv6) = item.as_str().and_then(parse_ipv6_text) {
            let ip_text = ipv6.to_string();
            if !ipv6.is_unicast_link_local() {
                return Some(ip_text);
            }
            if first_link_local.is_none() {
                first_link_local = Some(ip_text);
            }
        }
    }

    first_link_local
}

pub(super) fn parse_session_device_wan_ipv6(
    extra: &serde_json::Map<String, serde_json::Value>,
) -> Option<String> {
    if let Some(value) = extra
        .get("wan1")
        .and_then(|wan| wan.get("ipv6"))
        .and_then(pick_ipv6_from_value)
    {
        return Some(value);
    }

    extra.get("ipv6").and_then(pick_ipv6_from_value)
}

pub(super) fn convert_health_summaries(raw: Vec<serde_json::Value>) -> Vec<HealthSummary> {
    raw.into_iter()
        .map(|value| HealthSummary {
            subsystem: value
                .get("subsystem")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown")
                .to_owned(),
            status: value
                .get("status")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown")
                .to_owned(),
            #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
            num_adopted: value
                .get("num_adopted")
                .and_then(serde_json::Value::as_u64)
                .map(|value| value as u32),
            #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
            num_sta: value
                .get("num_sta")
                .and_then(serde_json::Value::as_u64)
                .map(|value| value as u32),
            tx_bytes_r: value.get("tx_bytes-r").and_then(serde_json::Value::as_u64),
            rx_bytes_r: value.get("rx_bytes-r").and_then(serde_json::Value::as_u64),
            latency: value.get("latency").and_then(serde_json::Value::as_f64),
            wan_ip: value
                .get("wan_ip")
                .and_then(|value| value.as_str())
                .map(String::from),
            gateways: value
                .get("gateways")
                .and_then(|value| value.as_array())
                .map(|values| {
                    values
                        .iter()
                        .filter_map(|value| value.as_str().map(String::from))
                        .collect()
                }),
            extra: value,
        })
        .collect()
}

/// Build a [`TransportConfig`] from the controller configuration.
pub(super) fn build_transport(config: &ControllerConfig) -> TransportConfig {
    TransportConfig {
        tls: tls_to_transport(&config.tls),
        timeout: config.timeout,
        cookie_jar: None, // SessionClient::new adds one automatically
    }
}

pub(super) fn tls_to_transport(tls: &TlsVerification) -> TlsMode {
    match tls {
        TlsVerification::SystemDefaults => TlsMode::System,
        TlsVerification::CustomCa(path) => TlsMode::CustomCa(path.clone()),
        TlsVerification::DangerAcceptInvalid => TlsMode::DangerAcceptInvalid,
    }
}

/// Resolve the Integration API site UUID from a site name or UUID string.
///
/// If `site_name` is already a valid UUID, returns it directly.
/// Otherwise lists all sites and finds the one matching by `internal_reference`.
pub(super) async fn resolve_site_id(
    client: &IntegrationClient,
    site_name: &str,
) -> Result<uuid::Uuid, CoreError> {
    // Fast path: if the input is already a UUID, use it directly.
    if let Ok(uuid) = uuid::Uuid::parse_str(site_name) {
        return Ok(uuid);
    }

    let sites = client
        .paginate_all(50, |off, lim| client.list_sites(off, lim))
        .await?;

    sites
        .into_iter()
        .find(|site| site.internal_reference == site_name)
        .map(|site| site.id)
        .ok_or_else(|| CoreError::SiteNotFound {
            name: site_name.to_owned(),
        })
}

/// Extract a `Uuid` from an `EntityId`, or return an error.
pub(super) fn require_uuid(id: &EntityId) -> Result<uuid::Uuid, CoreError> {
    id.as_uuid().copied().ok_or_else(|| CoreError::Unsupported {
        operation: "Integration API operation on legacy ID".into(),
        required: "UUID-based entity ID".into(),
    })
}

pub(super) fn require_session(
    session: Option<&Arc<SessionClient>>,
) -> Result<&SessionClient, CoreError> {
    session
        .map(Arc::as_ref)
        .ok_or_else(|| CoreError::Unsupported {
            operation: "Session API operation".into(),
            required: "Session API credentials (session or hybrid auth mode)".into(),
        })
}

pub(super) fn require_integration<'a>(
    integration: Option<&'a Arc<IntegrationClient>>,
    site_id: Option<uuid::Uuid>,
    operation: &str,
) -> Result<(&'a IntegrationClient, uuid::Uuid), CoreError> {
    let client = integration
        .map(Arc::as_ref)
        .ok_or_else(|| unsupported(operation))?;
    let sid = site_id.ok_or_else(|| unsupported(operation))?;
    Ok((client, sid))
}

pub(super) async fn integration_client_context(
    controller: &Controller,
    operation: &str,
) -> Result<Arc<IntegrationClient>, CoreError> {
    controller
        .inner
        .integration_client
        .lock()
        .await
        .as_ref()
        .cloned()
        .ok_or_else(|| unsupported(operation))
}

pub(super) async fn integration_site_context(
    controller: &Controller,
    operation: &str,
) -> Result<(Arc<IntegrationClient>, uuid::Uuid), CoreError> {
    let client = integration_client_context(controller, operation).await?;
    let site_id = controller
        .inner
        .site_id
        .lock()
        .await
        .ok_or_else(|| unsupported(operation))?;
    Ok((client, site_id))
}

pub(super) fn unsupported(operation: &str) -> CoreError {
    CoreError::Unsupported {
        operation: operation.into(),
        required: "Integration API".into(),
    }
}

/// Resolve an [`EntityId`] to a device MAC via the DataStore.
pub(super) fn device_mac(store: &DataStore, id: &EntityId) -> Result<MacAddress, CoreError> {
    store
        .device_by_id(id)
        .map(|device| device.mac.clone())
        .ok_or_else(|| CoreError::DeviceNotFound {
            identifier: id.to_string(),
        })
}

/// Resolve an [`EntityId`] to a client MAC via the DataStore.
pub(super) fn client_mac(store: &DataStore, id: &EntityId) -> Result<MacAddress, CoreError> {
    store
        .client_by_id(id)
        .map(|client| client.mac.clone())
        .ok_or_else(|| CoreError::ClientNotFound {
            identifier: id.to_string(),
        })
}
