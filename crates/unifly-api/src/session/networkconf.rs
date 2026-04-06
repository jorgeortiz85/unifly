// Legacy networkconf operations.
//
// Site-to-site VPN, VPN server, and VPN client records are stored in the
// classic `rest/networkconf` collection behind the legacy API.

use serde_json::Value;
use tracing::debug;

use super::SessionClient;
use crate::error::Error;

impl SessionClient {
    /// List raw networkconf records for the current site.
    pub async fn list_network_conf(&self) -> Result<Vec<Value>, Error> {
        let url = self.site_url("rest/networkconf");
        debug!("listing networkconf records");
        self.get(url).await
    }

    /// Create a networkconf record for the current site.
    pub async fn create_network_conf(&self, body: &Value) -> Result<Vec<Value>, Error> {
        let url = self.site_url("rest/networkconf");
        debug!("creating networkconf record");
        self.post(url, body).await
    }

    /// Update a networkconf record for the current site.
    pub async fn update_network_conf(
        &self,
        record_id: &str,
        body: &Value,
    ) -> Result<Vec<Value>, Error> {
        let url = self.site_url(&format!("rest/networkconf/{record_id}"));
        debug!(record_id, "updating networkconf record");
        self.put(url, body).await
    }

    /// Delete a networkconf record for the current site.
    pub async fn delete_network_conf(&self, record_id: &str) -> Result<Vec<Value>, Error> {
        let url = self.site_url(&format!("rest/networkconf/{record_id}"));
        debug!(record_id, "deleting networkconf record");
        self.delete(url).await
    }
}
