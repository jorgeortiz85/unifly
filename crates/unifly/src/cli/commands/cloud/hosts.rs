use tabled::Tabled;
use unifly_api::{SiteManagerClient, site_manager_types::Host};

use crate::cli::args::{CloudHostsArgs, CloudHostsCommand, GlobalOpts};
use crate::cli::error::CliError;
use crate::cli::output;

use super::api_error;

#[derive(Tabled)]
struct HostRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Model")]
    model: String,
    #[tabled(rename = "Firmware")]
    firmware: String,
    #[tabled(rename = "Owner")]
    owner: String,
}

fn host_row(host: &Host, painter: &output::Painter) -> HostRow {
    HostRow {
        id: painter.id(&host.id),
        name: painter.name(&host.display_name()),
        status: painter.state(&host.status()),
        model: painter.muted(&host.model_name()),
        firmware: painter.muted(&host.firmware()),
        owner: painter.enabled(host.is_owner_host()),
    }
}

pub async fn handle(
    client: &SiteManagerClient,
    args: CloudHostsArgs,
    global: &GlobalOpts,
) -> Result<(), CliError> {
    let painter = output::Painter::new(global);

    match args.command {
        None => {
            let hosts = client.list_hosts().await.map_err(api_error)?;
            let rendered = output::render_list(
                &global.output,
                &hosts,
                |host| host_row(host, &painter),
                |host| host.id.clone(),
            );
            output::print_output(&rendered, global.quiet);
            Ok(())
        }
        Some(CloudHostsCommand::Get { id }) => {
            let host = client.get_host(&id).await.map_err(api_error)?;
            let rendered =
                output::render_single(&global.output, &host, output::render_json_pretty, |host| {
                    host.id.clone()
                });
            output::print_output(&rendered, global.quiet);
            Ok(())
        }
    }
}
