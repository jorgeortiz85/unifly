use tabled::Tabled;
use unifly_api::{SiteManagerClient, site_manager_types::FleetSite};

use crate::cli::args::{CloudSitesArgs, GlobalOpts};
use crate::cli::error::CliError;
use crate::cli::output;

use super::api_error;

#[derive(Tabled)]
struct SiteRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Host")]
    host_id: String,
    #[tabled(rename = "Devices")]
    devices: String,
    #[tabled(rename = "Clients")]
    clients: String,
}

fn site_row(site: &FleetSite, painter: &output::Painter) -> SiteRow {
    SiteRow {
        id: painter.id(&site.id),
        name: painter.name(&site.display_name()),
        host_id: painter.id(site.host_id.as_deref().unwrap_or("")),
        devices: painter.number(&site.device_count()),
        clients: painter.number(&site.client_count()),
    }
}

pub async fn handle(
    client: &SiteManagerClient,
    _args: CloudSitesArgs,
    global: &GlobalOpts,
) -> Result<(), CliError> {
    let painter = output::Painter::new(global);
    let sites = client.list_sites().await.map_err(api_error)?;
    let rendered = output::render_list(
        &global.output,
        &sites,
        |site| site_row(site, &painter),
        |site| site.id.clone(),
    );
    output::print_output(&rendered, global.quiet);
    Ok(())
}
