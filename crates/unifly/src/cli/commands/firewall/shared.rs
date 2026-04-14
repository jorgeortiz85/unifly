use unifly_api::model::{FirewallAction as ModelFirewallAction, FirewallGroupType};
use unifly_api::{
    Controller, CreateFirewallPolicyRequest, EntityId, TrafficFilterSpec,
    UpdateFirewallPolicyRequest,
};

use crate::cli::args::FirewallAction;
use crate::cli::error::CliError;

pub(super) fn map_fw_action(action: &FirewallAction) -> ModelFirewallAction {
    match action {
        FirewallAction::Allow => ModelFirewallAction::Allow,
        FirewallAction::Block => ModelFirewallAction::Block,
        FirewallAction::Reject => ModelFirewallAction::Reject,
    }
}

pub(super) fn build_filter_spec(
    field_prefix: &str,
    networks: Option<Vec<String>>,
    ips: Option<Vec<String>>,
    ports: Option<Vec<String>>,
) -> Result<Option<TrafficFilterSpec>, CliError> {
    // network + ip is invalid; port can combine with either
    if networks.is_some() && ips.is_some() {
        return Err(CliError::Validation {
            field: format!("{field_prefix}-filter"),
            reason: format!("cannot combine --{field_prefix}-network and --{field_prefix}-ip"),
        });
    }

    Ok(if let Some(network_ids) = networks {
        Some(TrafficFilterSpec::Network {
            network_ids,
            match_opposite: false,
            ports,
        })
    } else if let Some(addresses) = ips {
        Some(TrafficFilterSpec::IpAddress {
            addresses,
            match_opposite: false,
            ports,
        })
    } else {
        ports.map(|ports| TrafficFilterSpec::Port {
            ports,
            match_opposite: false,
        })
    })
}

pub(super) fn parse_reorder_zone_pair(
    source_zone: Option<&str>,
    dest_zone: Option<&str>,
) -> Result<(EntityId, EntityId), CliError> {
    match (source_zone, dest_zone) {
        (Some(source_zone), Some(dest_zone)) => {
            Ok((EntityId::from(source_zone), EntityId::from(dest_zone)))
        }
        _ => Err(CliError::Validation {
            field: "zone-pair".into(),
            reason: "firewall policy reordering requires both --source-zone and --dest-zone".into(),
        }),
    }
}

/// Resolve group name shorthands (`src_port_group`, `dst_port_group`,
/// `src_address_group`, `dst_address_group`) into `TrafficFilterSpec` filters
/// on a `CreateFirewallPolicyRequest`.
///
/// Must be called *before* `resolve_filters()`.
pub(super) fn resolve_group_refs_create(
    controller: &Controller,
    req: &mut CreateFirewallPolicyRequest,
) -> Result<(), CliError> {
    let groups = controller.firewall_groups_snapshot();

    if let Some(name) = req.src_port_group.take() {
        check_no_conflict(
            "src",
            req.source_filter.is_some(),
            req.src_port.as_ref(),
            req.src_ip.as_ref(),
        )?;
        req.source_filter = Some(resolve_port_group(&name, &groups)?);
    }
    if let Some(name) = req.src_address_group.take() {
        check_no_conflict(
            "src",
            req.source_filter.is_some(),
            req.src_port.as_ref(),
            req.src_ip.as_ref(),
        )?;
        req.source_filter = Some(resolve_address_group(&name, &groups)?);
    }
    if let Some(name) = req.dst_port_group.take() {
        check_no_conflict(
            "dst",
            req.destination_filter.is_some(),
            req.dst_port.as_ref(),
            req.dst_ip.as_ref(),
        )?;
        req.destination_filter = Some(resolve_port_group(&name, &groups)?);
    }
    if let Some(name) = req.dst_address_group.take() {
        check_no_conflict(
            "dst",
            req.destination_filter.is_some(),
            req.dst_port.as_ref(),
            req.dst_ip.as_ref(),
        )?;
        req.destination_filter = Some(resolve_address_group(&name, &groups)?);
    }
    Ok(())
}

/// Same as [`resolve_group_refs_create`] but for update requests.
pub(super) fn resolve_group_refs_update(
    controller: &Controller,
    req: &mut UpdateFirewallPolicyRequest,
) -> Result<(), CliError> {
    let groups = controller.firewall_groups_snapshot();

    if let Some(name) = req.src_port_group.take() {
        check_no_conflict(
            "src",
            req.source_filter.is_some(),
            req.src_port.as_ref(),
            req.src_ip.as_ref(),
        )?;
        req.source_filter = Some(resolve_port_group(&name, &groups)?);
    }
    if let Some(name) = req.src_address_group.take() {
        check_no_conflict(
            "src",
            req.source_filter.is_some(),
            req.src_port.as_ref(),
            req.src_ip.as_ref(),
        )?;
        req.source_filter = Some(resolve_address_group(&name, &groups)?);
    }
    if let Some(name) = req.dst_port_group.take() {
        check_no_conflict(
            "dst",
            req.destination_filter.is_some(),
            req.dst_port.as_ref(),
            req.dst_ip.as_ref(),
        )?;
        req.destination_filter = Some(resolve_port_group(&name, &groups)?);
    }
    if let Some(name) = req.dst_address_group.take() {
        check_no_conflict(
            "dst",
            req.destination_filter.is_some(),
            req.dst_port.as_ref(),
            req.dst_ip.as_ref(),
        )?;
        req.destination_filter = Some(resolve_address_group(&name, &groups)?);
    }
    Ok(())
}

fn check_no_conflict(
    side: &str,
    has_filter: bool,
    ports: Option<&Vec<String>>,
    ips: Option<&Vec<String>>,
) -> Result<(), CliError> {
    if has_filter {
        return Err(CliError::Validation {
            field: format!("{side}_group"),
            reason: format!("cannot combine {side}_*_group with {side} filter"),
        });
    }
    if ports.is_some() || ips.is_some() {
        return Err(CliError::Validation {
            field: format!("{side}_group"),
            reason: format!(
                "cannot combine {side}_*_group with {side}_port or {side}_ip shorthands"
            ),
        });
    }
    Ok(())
}

fn resolve_port_group(
    name: &str,
    groups: &[std::sync::Arc<unifly_api::model::FirewallGroup>],
) -> Result<TrafficFilterSpec, CliError> {
    let group = groups
        .iter()
        .find(|g| g.name == name)
        .ok_or_else(|| CliError::Validation {
            field: "port_group".into(),
            reason: format!("firewall group \"{name}\" not found"),
        })?;
    if group.group_type != FirewallGroupType::PortGroup {
        return Err(CliError::Validation {
            field: "port_group".into(),
            reason: format!(
                "firewall group \"{name}\" is a {}, not a port-group",
                group.group_type
            ),
        });
    }
    let list_id = group
        .external_id
        .as_ref()
        .ok_or_else(|| CliError::Validation {
            field: "port_group".into(),
            reason: format!("firewall group \"{name}\" has no external_id"),
        })?;
    Ok(TrafficFilterSpec::PortMatchingList {
        list_id: list_id.clone(),
        match_opposite: false,
    })
}

fn resolve_address_group(
    name: &str,
    groups: &[std::sync::Arc<unifly_api::model::FirewallGroup>],
) -> Result<TrafficFilterSpec, CliError> {
    let group = groups
        .iter()
        .find(|g| g.name == name)
        .ok_or_else(|| CliError::Validation {
            field: "address_group".into(),
            reason: format!("firewall group \"{name}\" not found"),
        })?;
    if group.group_type != FirewallGroupType::AddressGroup
        && group.group_type != FirewallGroupType::Ipv6AddressGroup
    {
        return Err(CliError::Validation {
            field: "address_group".into(),
            reason: format!(
                "firewall group \"{name}\" is a {}, not an address-group",
                group.group_type
            ),
        });
    }
    let list_id = group
        .external_id
        .as_ref()
        .ok_or_else(|| CliError::Validation {
            field: "address_group".into(),
            reason: format!("firewall group \"{name}\" has no external_id"),
        })?;
    Ok(TrafficFilterSpec::IpMatchingList {
        list_id: list_id.clone(),
        match_opposite: false,
    })
}

#[cfg(test)]
mod tests {
    use super::{build_filter_spec, parse_reorder_zone_pair};
    use crate::cli::error::CliError;
    use unifly_api::{EntityId, TrafficFilterSpec};

    #[test]
    fn build_filter_spec_accepts_single_filter_family() {
        let spec = build_filter_spec("src", Some(vec!["lan".into()]), None, None);
        assert!(matches!(spec, Ok(Some(TrafficFilterSpec::Network { .. }))));
    }

    #[test]
    fn build_filter_spec_rejects_multiple_filter_families() {
        let err = build_filter_spec(
            "src",
            Some(vec!["lan".into()]),
            Some(vec!["10.0.0.1".into()]),
            None,
        );

        match err {
            Err(CliError::Validation { field, .. }) => assert_eq!(field, "src-filter"),
            Ok(_) => panic!("expected validation error, got success"),
            Err(other) => panic!("expected validation error, got {other:?}"),
        }
    }

    #[test]
    fn build_filter_spec_combines_ip_and_port() {
        let spec = build_filter_spec(
            "dst",
            None,
            Some(vec!["10.0.40.10".into()]),
            Some(vec!["80".into()]),
        )
        .expect("ip + port should succeed");

        match spec {
            Some(TrafficFilterSpec::IpAddress {
                addresses, ports, ..
            }) => {
                assert_eq!(addresses, vec!["10.0.40.10"]);
                assert_eq!(ports, Some(vec!["80".into()]));
            }
            other => panic!("expected IpAddress with ports, got {other:?}"),
        }
    }

    #[test]
    fn parse_reorder_zone_pair_requires_both_zones() {
        let err = parse_reorder_zone_pair(Some("src"), None)
            .expect_err("missing destination zone should fail");
        match err {
            CliError::Validation { field, .. } => assert_eq!(field, "zone-pair"),
            other => panic!("expected validation error, got {other:?}"),
        }
    }

    #[test]
    fn parse_reorder_zone_pair_returns_entity_ids() {
        let zone_pair = parse_reorder_zone_pair(
            Some("550e8400-e29b-41d4-a716-446655440000"),
            Some("550e8400-e29b-41d4-a716-446655440001"),
        )
        .expect("zone pair should parse");
        assert_eq!(
            zone_pair,
            (
                EntityId::from("550e8400-e29b-41d4-a716-446655440000"),
                EntityId::from("550e8400-e29b-41d4-a716-446655440001"),
            )
        );
    }
}
