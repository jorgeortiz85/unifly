use clap::{Args, ValueEnum};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ApiMethod {
    Get,
    Post,
}

#[derive(Debug, Args)]
pub struct ApiArgs {
    /// API path (appended to the controller's base URL + proxy prefix).
    ///
    /// Legacy style:  api/s/{site}/stat/sitedpi
    /// V2 style:      v2/api/site/{site}/traffic-flow-latest-statistics
    /// Integration:   integration/v1/dpi/applications
    pub path: String,

    /// HTTP method (GET or POST)
    #[arg(long, short, value_enum, default_value_t = ApiMethod::Get)]
    pub method: ApiMethod,

    /// JSON request body (for POST/PUT)
    #[arg(long, short)]
    pub data: Option<String>,
}
