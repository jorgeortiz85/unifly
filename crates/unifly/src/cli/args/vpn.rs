use std::path::PathBuf;

use clap::{Args, Subcommand, ValueEnum};

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

    /// Manage session site-to-site VPNs
    #[command(alias = "s2s")]
    SiteToSite(SiteToSiteVpnArgs),

    /// Manage session remote-access VPN servers
    #[command(alias = "ra")]
    RemoteAccess(RemoteAccessVpnArgs),

    /// Manage configured session VPN clients
    Clients(VpnClientsArgs),

    /// View and restart session VPN client connections
    Connections(VpnConnectionsArgs),

    /// Manage WireGuard peers on session remote-access servers
    #[command(alias = "peer")]
    Peers(VpnPeersArgs),

    /// Inspect session magic site-to-site VPN configs
    #[command(alias = "magic-site-to-site-vpn", alias = "magic-s2s")]
    MagicSiteToSite(MagicSiteToSiteVpnArgs),

    /// Manage session VPN-related site settings
    Settings(VpnSettingsArgs),
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

#[derive(Debug, Args)]
pub struct SiteToSiteVpnArgs {
    #[command(subcommand)]
    pub command: SiteToSiteVpnCommand,
}

#[derive(Debug, Subcommand)]
pub enum SiteToSiteVpnCommand {
    /// List site-to-site VPN records
    #[command(alias = "ls")]
    List(ListArgs),

    /// Get a site-to-site VPN record
    Get { id: String },

    /// Create a site-to-site VPN from a JSON file
    Create {
        #[arg(long, short = 'F')]
        from_file: PathBuf,
    },

    /// Update a site-to-site VPN from a JSON file
    Update {
        id: String,

        #[arg(long, short = 'F')]
        from_file: PathBuf,
    },

    /// Delete a site-to-site VPN record
    Delete { id: String },
}

#[derive(Debug, Args)]
pub struct RemoteAccessVpnArgs {
    #[command(subcommand)]
    pub command: RemoteAccessVpnCommand,
}

#[derive(Debug, Subcommand)]
pub enum RemoteAccessVpnCommand {
    /// List remote-access VPN server records
    #[command(alias = "ls")]
    List(ListArgs),

    /// Get a remote-access VPN server record
    Get { id: String },

    /// Create a remote-access VPN server from a JSON file
    Create {
        #[arg(long, short = 'F')]
        from_file: PathBuf,
    },

    /// Update a remote-access VPN server from a JSON file
    Update {
        id: String,

        #[arg(long, short = 'F')]
        from_file: PathBuf,
    },

    /// Suggest available OpenVPN ports for a remote-access server
    SuggestPort,

    /// Download an OpenVPN client configuration for a remote-access server
    DownloadConfig {
        id: String,

        #[arg(long = "path")]
        path: Option<PathBuf>,
    },

    /// Delete a remote-access VPN server record
    Delete { id: String },
}

#[derive(Debug, Args)]
pub struct VpnClientsArgs {
    #[command(subcommand)]
    pub command: VpnClientsCommand,
}

#[derive(Debug, Subcommand)]
pub enum VpnClientsCommand {
    /// List configured VPN client records
    #[command(alias = "ls")]
    List(ListArgs),

    /// Get a configured VPN client record
    Get { id: String },

    /// Create a configured VPN client from a JSON file
    Create {
        #[arg(long, short = 'F')]
        from_file: PathBuf,
    },

    /// Update a configured VPN client from a JSON file
    Update {
        id: String,

        #[arg(long, short = 'F')]
        from_file: PathBuf,
    },

    /// Delete a configured VPN client record
    Delete { id: String },
}

#[derive(Debug, Args)]
pub struct VpnConnectionsArgs {
    #[command(subcommand)]
    pub command: VpnConnectionsCommand,
}

#[derive(Debug, Subcommand)]
pub enum VpnConnectionsCommand {
    /// List VPN client connections
    #[command(alias = "ls")]
    List(ListArgs),

    /// Get a VPN client connection
    Get { id: String },

    /// Restart a VPN client connection
    Restart { id: String },
}

#[derive(Debug, Args)]
pub struct VpnPeersArgs {
    #[command(subcommand)]
    pub command: VpnPeersCommand,
}

#[derive(Debug, Subcommand)]
pub enum VpnPeersCommand {
    /// List WireGuard peers, optionally scoped to one server
    #[command(alias = "ls")]
    List {
        server_id: Option<String>,

        #[command(flatten)]
        list: ListArgs,
    },

    /// Get a WireGuard peer from a remote-access server
    Get { server_id: String, id: String },

    /// Create a WireGuard peer from a JSON file
    Create {
        server_id: String,

        #[arg(long, short = 'F')]
        from_file: PathBuf,
    },

    /// Update a WireGuard peer from a JSON file
    Update {
        server_id: String,
        id: String,

        #[arg(long, short = 'F')]
        from_file: PathBuf,
    },

    /// Delete a WireGuard peer from a remote-access server
    Delete { server_id: String, id: String },

    /// List subnets already consumed by WireGuard peers
    #[command(alias = "existing-subnets")]
    Subnets,
}

#[derive(Debug, Args)]
pub struct MagicSiteToSiteVpnArgs {
    #[command(subcommand)]
    pub command: MagicSiteToSiteVpnCommand,
}

#[derive(Debug, Subcommand)]
pub enum MagicSiteToSiteVpnCommand {
    /// List magic site-to-site VPN configs
    #[command(alias = "ls")]
    List(ListArgs),

    /// Get a magic site-to-site VPN config
    Get { id: String },
}

#[derive(Debug, Args)]
pub struct VpnSettingsArgs {
    #[command(subcommand)]
    pub command: VpnSettingsCommand,
}

#[derive(Debug, Subcommand)]
pub enum VpnSettingsCommand {
    /// List discovered VPN-related site settings
    #[command(alias = "ls")]
    List(ListArgs),

    /// Get a specific VPN setting
    Get {
        #[arg(value_enum)]
        key: VpnSettingKey,
    },

    /// Toggle a VPN setting via session site settings
    Set {
        #[arg(value_enum)]
        key: VpnSettingKey,

        #[arg(long, action = clap::ArgAction::Set)]
        enabled: bool,
    },

    /// Patch a VPN setting from a JSON file
    Patch {
        #[arg(value_enum)]
        key: VpnSettingKey,

        #[arg(long, short = 'F')]
        from_file: PathBuf,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum VpnSettingKey {
    Teleport,
    MagicSiteToSiteVpn,
    Openvpn,
    PeerToPeer,
}
