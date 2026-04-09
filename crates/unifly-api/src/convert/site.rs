use crate::integration_types;
use crate::model::common::DataSource;
use crate::model::entity_id::EntityId;
use crate::model::site::Site;
use crate::session::models::SessionSite;

impl From<SessionSite> for Site {
    fn from(s: SessionSite) -> Self {
        let display_name = s
            .desc
            .filter(|d| !d.is_empty())
            .unwrap_or_else(|| s.name.clone());

        Site {
            id: EntityId::from(s.id),
            internal_name: s.name,
            name: display_name,
            device_count: None,
            client_count: None,
            source: DataSource::SessionApi,
        }
    }
}

impl From<integration_types::SiteResponse> for Site {
    fn from(s: integration_types::SiteResponse) -> Self {
        Site {
            id: EntityId::Uuid(s.id),
            internal_name: s.internal_reference,
            name: s.name,
            device_count: None,
            client_count: None,
            source: DataSource::IntegrationApi,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_site_uses_desc_as_display_name() {
        let site = SessionSite {
            id: "abc123".into(),
            name: "default".into(),
            desc: Some("Main Office".into()),
            role: None,
            extra: serde_json::Map::new(),
        };
        let converted: Site = site.into();
        assert_eq!(converted.internal_name, "default");
        assert_eq!(converted.name, "Main Office");
    }

    #[test]
    fn legacy_site_falls_back_to_name_when_desc_empty() {
        let site = SessionSite {
            id: "abc123".into(),
            name: "branch-1".into(),
            desc: Some(String::new()),
            role: None,
            extra: serde_json::Map::new(),
        };
        let converted: Site = site.into();
        assert_eq!(converted.name, "branch-1");
    }
}
