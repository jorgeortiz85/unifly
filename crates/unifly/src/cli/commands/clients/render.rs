use std::sync::Arc;

use serde::Serialize;
use tabled::Tabled;
use unifly_api::Client;
use unifly_api::legacy_models::LegacyUserEntry;

use crate::cli::output::Painter;

#[derive(Tabled)]
pub(super) struct ClientRow {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "IP")]
    ip: String,
    #[tabled(rename = "Type")]
    ctype: String,
    #[tabled(rename = "Uplink")]
    uplink: String,
}

pub(super) fn client_row(client: &Arc<Client>, painter: &Painter) -> ClientRow {
    let name = client
        .name
        .clone()
        .or_else(|| client.hostname.clone())
        .unwrap_or_default();
    ClientRow {
        name: painter.name(&name),
        ip: painter.ip(&client.ip.map(|ip| ip.to_string()).unwrap_or_default()),
        ctype: painter.muted(&format!("{:?}", client.client_type)),
        uplink: painter.mac(
            &client
                .uplink_device_mac
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default(),
        ),
    }
}

pub(super) fn detail(client: &Arc<Client>) -> String {
    let mut lines = vec![
        format!("ID:        {}", client.id),
        format!("Name:      {}", client.name.as_deref().unwrap_or("-")),
        format!("Hostname:  {}", client.hostname.as_deref().unwrap_or("-")),
        format!("MAC:       {}", client.mac),
        format!(
            "IP:        {}",
            client.ip.map_or_else(|| "-".into(), |ip| ip.to_string())
        ),
        format!("Type:      {:?}", client.client_type),
        format!("Guest:     {}", client.is_guest),
        format!("Blocked:   {}", client.blocked),
    ];
    if client.use_fixedip {
        lines.push(format!(
            "Fixed IP:  {}",
            client.fixed_ip.map_or("-".into(), |ip| ip.to_string())
        ));
    }
    if let Some(wireless) = &client.wireless {
        lines.push(format!(
            "SSID:      {}",
            wireless.ssid.as_deref().unwrap_or("-")
        ));
        if let Some(signal) = wireless.signal_dbm {
            lines.push(format!("Signal:    {signal} dBm"));
        }
    }
    if let Some(os_name) = &client.os_name {
        lines.push(format!("OS:        {os_name}"));
    }
    lines.join("\n")
}

// ── DHCP Reservations ────────────────────────────────────────────

/// Clean serializable representation of a DHCP reservation.
#[derive(Debug, Serialize)]
pub(super) struct Reservation {
    pub mac: String,
    pub name: Option<String>,
    pub fixed_ip: Option<String>,
    pub network_id: Option<String>,
}

impl From<&LegacyUserEntry> for Reservation {
    fn from(u: &LegacyUserEntry) -> Self {
        Self {
            mac: u.mac.clone(),
            name: u.name.clone(),
            fixed_ip: u.fixed_ip.clone(),
            network_id: u.network_id.clone(),
        }
    }
}

#[derive(Tabled)]
pub(super) struct ReservationRow {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "MAC")]
    mac: String,
    #[tabled(rename = "Fixed IP")]
    fixed_ip: String,
    #[tabled(rename = "Network")]
    network: String,
}

pub(super) fn reservation_row(reservation: &Reservation, painter: &Painter) -> ReservationRow {
    ReservationRow {
        name: painter.name(reservation.name.as_deref().unwrap_or("-")),
        mac: painter.mac(&reservation.mac),
        fixed_ip: painter.ip(reservation.fixed_ip.as_deref().unwrap_or("-")),
        network: painter.id(reservation.network_id.as_deref().unwrap_or("-")),
    }
}
