use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct CloudArgs {
    #[command(subcommand)]
    pub command: CloudCommand,
}

#[derive(Debug, Subcommand)]
pub enum CloudCommand {
    /// List cloud consoles or show a single console
    Hosts(CloudHostsArgs),

    /// List sites across all accessible cloud consoles
    Sites(CloudSitesArgs),

    /// List cloud-managed devices, optionally scoped to one or more hosts
    Devices(CloudDevicesArgs),

    /// View ISP metrics through Site Manager
    Isp(CloudIspArgs),

    /// View SD-WAN configurations and deployment status
    Sdwan(CloudSdwanArgs),
}

#[derive(Debug, Args)]
pub struct CloudHostsArgs {
    #[command(subcommand)]
    pub command: Option<CloudHostsCommand>,
}

#[derive(Debug, Subcommand)]
pub enum CloudHostsCommand {
    /// Show a single console
    Get { id: String },
}

#[derive(Debug, Args, Default)]
pub struct CloudSitesArgs {}

#[derive(Debug, Args)]
pub struct CloudDevicesArgs {
    /// Restrict device results to one or more cloud hosts
    #[arg(long = "host")]
    pub hosts: Vec<String>,
}

#[derive(Debug, Args)]
pub struct CloudIspArgs {
    /// Metric interval (`5m` or `1h`)
    #[arg(long = "type", default_value = "5m", value_parser = ["5m", "1h"])]
    pub interval: String,

    #[command(subcommand)]
    pub command: Option<CloudIspCommand>,
}

#[derive(Debug, Subcommand)]
pub enum CloudIspCommand {
    /// Query ISP metrics for a specific set of sites
    Query {
        /// One or more cloud Site Manager site IDs
        #[arg(long = "sites", required = true, value_delimiter = ',')]
        sites: Vec<String>,
    },
}

#[derive(Debug, Args)]
pub struct CloudSdwanArgs {
    #[command(subcommand)]
    pub command: Option<CloudSdwanCommand>,
}

#[derive(Debug, Subcommand)]
pub enum CloudSdwanCommand {
    /// Show a single SD-WAN configuration
    Get { id: String },

    /// Show deployment status for an SD-WAN configuration
    Status { id: String },
}
