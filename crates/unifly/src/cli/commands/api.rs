//! Raw API passthrough command.

use unifly_api::Controller;

use crate::cli::args::{ApiArgs, ApiMethod, GlobalOpts, OutputFormat};
use crate::cli::error::CliError;
use crate::cli::output;

pub async fn handle(
    controller: &Controller,
    args: ApiArgs,
    global: &GlobalOpts,
) -> Result<(), CliError> {
    let body: Option<serde_json::Value> = args
        .data
        .as_deref()
        .map(serde_json::from_str)
        .transpose()
        .map_err(|e| CliError::Validation {
            field: "data".into(),
            reason: format!("invalid JSON: {e}"),
        })?;

    let result = match args.method {
        ApiMethod::Get => controller.raw_get(&args.path).await?,
        ApiMethod::Post => {
            let payload = body.unwrap_or(serde_json::json!({}));
            controller.raw_post(&args.path, &payload).await?
        }
        ApiMethod::Put => {
            let payload = body.unwrap_or(serde_json::json!({}));
            controller.raw_put(&args.path, &payload).await?
        }
        ApiMethod::Patch => {
            let payload = body.unwrap_or(serde_json::json!({}));
            controller.raw_patch(&args.path, &payload).await?
        }
        ApiMethod::Delete => {
            controller.raw_delete(&args.path).await?;
            serde_json::json!({ "ok": true })
        }
    };

    let out = match &global.output {
        OutputFormat::JsonCompact => output::render_json_compact(&result),
        OutputFormat::Yaml => output::render_yaml(&result),
        _ => output::render_json_pretty(&result),
    };
    output::print_output(&out, global.quiet);
    Ok(())
}
