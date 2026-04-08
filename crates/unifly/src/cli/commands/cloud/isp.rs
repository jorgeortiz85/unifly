use tabled::Tabled;
use unifly_api::{
    SiteManagerClient,
    site_manager_types::{FleetPage, IspMetric, IspMetricInterval},
};

use crate::cli::args::{CloudIspArgs, CloudIspCommand, GlobalOpts};
use crate::cli::error::CliError;
use crate::cli::output;

use super::api_error;

#[derive(Tabled)]
struct IspRow {
    #[tabled(rename = "Site")]
    site_id: String,
    #[tabled(rename = "Timestamp")]
    timestamp: String,
    #[tabled(rename = "Latency ms")]
    latency_ms: String,
    #[tabled(rename = "Down Mbps")]
    download_mbps: String,
    #[tabled(rename = "Up Mbps")]
    upload_mbps: String,
    #[tabled(rename = "Status")]
    status: String,
}

fn interval_from_args(args: &CloudIspArgs) -> Result<IspMetricInterval, CliError> {
    IspMetricInterval::parse(&args.interval).ok_or_else(|| CliError::Validation {
        field: "type".into(),
        reason: "must be '5m' or '1h'".into(),
    })
}

fn metric_row(metric: &IspMetric, painter: &output::Painter) -> IspRow {
    IspRow {
        site_id: painter.id(metric.site_id.as_deref().unwrap_or("")),
        timestamp: painter.muted(&metric.timestamp_text()),
        latency_ms: painter.number(&metric.latency_text()),
        download_mbps: painter.number(&metric.download_text()),
        upload_mbps: painter.number(&metric.upload_text()),
        status: painter.state(&metric.status_text()),
    }
}

fn warn_partial(page: &FleetPage<IspMetric>, global: &GlobalOpts) {
    if page.status.as_deref() == Some("partialSuccess") && !global.quiet {
        eprintln!("warning: cloud ISP metrics returned partialSuccess");
    }
}

pub async fn handle(
    client: &SiteManagerClient,
    args: CloudIspArgs,
    global: &GlobalOpts,
) -> Result<(), CliError> {
    let painter = output::Painter::new(global);
    let interval = interval_from_args(&args)?;

    let page = match args.command {
        None => client.get_isp_metrics(interval).await.map_err(api_error)?,
        Some(CloudIspCommand::Query { sites }) => client
            .query_isp_metrics(interval, &sites)
            .await
            .map_err(api_error)?,
    };

    warn_partial(&page, global);

    let rendered = output::render_list(
        &global.output,
        &page.data,
        |metric| metric_row(metric, &painter),
        |metric| {
            metric
                .site_id
                .clone()
                .or_else(|| metric.timestamp.clone())
                .unwrap_or_default()
        },
    );
    output::print_output(&rendered, global.quiet);
    Ok(())
}
