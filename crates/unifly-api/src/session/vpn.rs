use tracing::debug;

use crate::error::Error;
use crate::model::IpsecSa;
use crate::session::client::SessionClient;

impl SessionClient {
    pub async fn list_ipsec_sa(&self) -> Result<Vec<IpsecSa>, Error> {
        let url = self.site_url("stat/ipsec-sa");
        debug!("listing ipsec security associations");
        match self.get(url).await {
            Ok(sas) => Ok(sas),
            Err(error) if error.is_not_found() => Ok(Vec::new()),
            Err(error) => Err(error),
        }
    }
}
