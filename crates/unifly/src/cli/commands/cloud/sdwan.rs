use tabled::Tabled;
use unifly_api::{
    SiteManagerClient,
    site_manager_types::{SdWanConfig, SdWanStatus},
};

use crate::cli::args::{CloudSdwanArgs, CloudSdwanCommand, GlobalOpts};
use crate::cli::error::CliError;
use crate::cli::output;

use super::api_error;

#[derive(Tabled)]
struct SdWanRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Hubs")]
    hubs: String,
    #[tabled(rename = "Sites")]
    sites: String,
}

fn config_row(config: &SdWanConfig, painter: &output::Painter) -> SdWanRow {
    SdWanRow {
        id: painter.id(&config.id),
        name: painter.name(&config.display_name()),
        status: painter.state(&config.status_text()),
        hubs: painter.number(&config.hub_count()),
        sites: painter.number(&config.site_count()),
    }
}

fn status_detail(status: &SdWanStatus) -> String {
    let mut lines = vec![format!("Status:   {}", status.status_text())];
    if !status.progress_text().is_empty() {
        lines.push(format!("Progress: {}", status.progress_text()));
    }
    if !status.error_count().is_empty() {
        lines.push(format!("Errors:   {}", status.error_count()));
    }
    lines.join("\n")
}

pub async fn handle(
    client: &SiteManagerClient,
    args: CloudSdwanArgs,
    global: &GlobalOpts,
) -> Result<(), CliError> {
    let painter = output::Painter::new(global);

    match args.command {
        None => {
            let configs = client.list_sdwan_configs().await.map_err(api_error)?;
            let rendered = output::render_list(
                &global.output,
                &configs,
                |config| config_row(config, &painter),
                |config| config.id.clone(),
            );
            output::print_output(&rendered, global.quiet);
            Ok(())
        }
        Some(CloudSdwanCommand::Get { id }) => {
            let config = client.get_sdwan_config(&id).await.map_err(api_error)?;
            let rendered = output::render_single(
                &global.output,
                &config,
                output::render_json_pretty,
                |config| config.id.clone(),
            );
            output::print_output(&rendered, global.quiet);
            Ok(())
        }
        Some(CloudSdwanCommand::Status { id }) => {
            let status = client.get_sdwan_status(&id).await.map_err(api_error)?;
            let rendered =
                output::render_single(&global.output, &status, status_detail, |status| {
                    status.id.clone().unwrap_or_else(|| "sdwan-status".into())
                });
            output::print_output(&rendered, global.quiet);
            Ok(())
        }
    }
}
