use clap::{Args, Subcommand};

use super::ListArgs;

#[derive(Debug, Args)]
pub struct VpnArgs {
    #[command(subcommand)]
    pub command: VpnCommand,
}

#[derive(Debug, Subcommand)]
pub enum VpnCommand {
    /// Manage VPN servers
    Servers(VpnServersArgs),

    /// Manage site-to-site VPN tunnels
    Tunnels(VpnTunnelsArgs),

    /// Show live IPsec tunnel status
    Status,

    /// Show VPN subsystem health
    Health,
}

#[derive(Debug, Args)]
pub struct VpnServersArgs {
    #[command(subcommand)]
    pub command: Option<VpnServersCommand>,

    #[command(flatten)]
    pub list: ListArgs,
}

#[derive(Debug, Subcommand)]
pub enum VpnServersCommand {
    /// Get full details of a VPN server
    Get {
        /// VPN server ID (UUID)
        id: String,
    },
}

#[derive(Debug, Args)]
pub struct VpnTunnelsArgs {
    #[command(subcommand)]
    pub command: Option<VpnTunnelsCommand>,

    #[command(flatten)]
    pub list: ListArgs,
}

#[derive(Debug, Subcommand)]
pub enum VpnTunnelsCommand {
    /// Get full details of a site-to-site VPN tunnel
    Get {
        /// VPN tunnel ID (UUID)
        id: String,
    },
}
