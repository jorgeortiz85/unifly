//! Country code command handler.

use tabled::Tabled;
use unifly_api::{Controller, Country};

use crate::cli::args::GlobalOpts;
use crate::cli::error::CliError;
use crate::cli::output;

// ── Table row ───────────────────────────────────────────────────────

#[derive(Tabled)]
struct CountryRow {
    #[tabled(rename = "Code")]
    code: String,
    #[tabled(rename = "Name")]
    name: String,
}

fn country_row(c: &Country, p: &output::Painter) -> CountryRow {
    CountryRow {
        code: p.muted(&c.code),
        name: p.name(&c.name),
    }
}

// ── Handler ─────────────────────────────────────────────────────────

pub async fn handle(controller: &Controller, global: &GlobalOpts) -> Result<(), CliError> {
    let p = output::Painter::new(global);
    let countries = controller.list_countries().await?;
    let out = output::render_list(
        &global.output,
        &countries,
        |c| country_row(c, &p),
        |c| c.code.clone(),
    );
    output::print_output(&out, global.quiet);
    Ok(())
}
