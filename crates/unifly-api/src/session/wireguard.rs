use serde_json::Value;
use tracing::debug;

use super::SessionClient;
use crate::error::Error;

impl SessionClient {
    /// List WireGuard peers for a specific remote-access server via the v2 API.
    ///
    /// `GET /v2/api/site/{site}/wireguard/{server_id}/users?networkId={server_id}`
    pub async fn list_wireguard_peers(&self, server_id: &str) -> Result<Vec<Value>, Error> {
        let mut url = self.site_url_v2(&format!("wireguard/{server_id}/users"));
        url.query_pairs_mut().append_pair("networkId", server_id);
        debug!(server_id, "listing WireGuard peers (v2)");
        let val = self.get_raw(url).await?;
        Ok(val.as_array().cloned().unwrap_or_default())
    }

    /// List WireGuard peers across all servers via the v2 API.
    ///
    /// `GET /v2/api/site/{site}/wireguard/users`
    pub async fn list_all_wireguard_peers(&self) -> Result<Vec<Value>, Error> {
        let url = self.site_url_v2("wireguard/users");
        debug!("listing all WireGuard peers (v2)");
        let val = self.get_raw(url).await?;
        Ok(val.as_array().cloned().unwrap_or_default())
    }

    /// List existing subnets already used by WireGuard peers.
    ///
    /// `GET /v2/api/site/{site}/wireguard/users/existing-subnets`
    pub async fn get_wireguard_peer_existing_subnets(&self) -> Result<Value, Error> {
        let url = self.site_url_v2("wireguard/users/existing-subnets");
        debug!("listing WireGuard peer existing subnets (v2)");
        self.get_raw(url).await
    }

    /// Create one or more WireGuard peers via the v2 API.
    ///
    /// `POST /v2/api/site/{site}/wireguard/{server_id}/users/batch`
    pub async fn create_wireguard_peers(
        &self,
        server_id: &str,
        body: &Value,
    ) -> Result<Value, Error> {
        let path = format!(
            "v2/api/site/{}/wireguard/{server_id}/users/batch",
            self.site()
        );
        debug!(server_id, "creating WireGuard peers (v2)");
        self.raw_post(&path, body).await
    }

    /// Update one or more WireGuard peers via the v2 API.
    ///
    /// `PUT /v2/api/site/{site}/wireguard/{server_id}/users/batch`
    pub async fn update_wireguard_peers(
        &self,
        server_id: &str,
        body: &Value,
    ) -> Result<Value, Error> {
        let path = format!(
            "v2/api/site/{}/wireguard/{server_id}/users/batch",
            self.site()
        );
        debug!(server_id, "updating WireGuard peers (v2)");
        self.raw_put(&path, body).await
    }

    /// Delete one or more WireGuard peers via the v2 API.
    ///
    /// `POST /v2/api/site/{site}/wireguard/{server_id}/users/batch_delete`
    pub async fn delete_wireguard_peers(
        &self,
        server_id: &str,
        body: &Value,
    ) -> Result<Value, Error> {
        let path = format!(
            "v2/api/site/{}/wireguard/{server_id}/users/batch_delete",
            self.site()
        );
        debug!(server_id, "deleting WireGuard peers (v2)");
        self.raw_post(&path, body).await
    }
}
