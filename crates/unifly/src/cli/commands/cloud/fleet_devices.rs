use tabled::Tabled;
use unifly_api::{SiteManagerClient, site_manager_types::CloudDevice};

use crate::cli::args::{CloudDevicesArgs, GlobalOpts};
use crate::cli::error::CliError;
use crate::cli::output;

use super::api_error;

#[derive(Tabled)]
struct DeviceRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Model")]
    model: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "IP")]
    ip: String,
    #[tabled(rename = "Host")]
    host_id: String,
    #[tabled(rename = "Site")]
    site_id: String,
}

fn device_row(device: &CloudDevice, painter: &output::Painter) -> DeviceRow {
    DeviceRow {
        id: painter.id(&device.id),
        name: painter.name(&device.display_name()),
        model: painter.muted(&device.model_name()),
        status: painter.state(&device.status()),
        ip: painter.ip(&device.ip()),
        host_id: painter.id(device.host_id.as_deref().unwrap_or("")),
        site_id: painter.id(device.site_id.as_deref().unwrap_or("")),
    }
}

pub async fn handle(
    client: &SiteManagerClient,
    args: CloudDevicesArgs,
    global: &GlobalOpts,
) -> Result<(), CliError> {
    let painter = output::Painter::new(global);
    let devices = client.list_devices(&args.hosts).await.map_err(api_error)?;
    let rendered = output::render_list(
        &global.output,
        &devices,
        |device| device_row(device, &painter),
        |device| device.id.clone(),
    );
    output::print_output(&rendered, global.quiet);
    Ok(())
}
