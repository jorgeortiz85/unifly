use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct SettingsArgs {
    #[command(subcommand)]
    pub command: SettingsCommand,
}

#[derive(Debug, Subcommand)]
pub enum SettingsCommand {
    /// List all site setting sections (session API)
    #[command(alias = "ls")]
    List,

    /// Get a specific setting section by key (session API)
    Get {
        /// Setting section key (e.g. "ips", "dpi", "usg", "radio_ai")
        key: String,
    },

    /// Update a field in a setting section (session API)
    Set(SettingsSetArgs),

    /// Export all settings as raw JSON (session API)
    Export,
}

#[derive(Debug, Args)]
pub struct SettingsSetArgs {
    /// Setting section key (e.g. "dpi", "ips", "usg")
    pub key: String,

    /// Field name within the section
    #[arg(required_unless_present = "data")]
    pub field: Option<String>,

    /// Field value (parsed as bool/number/string)
    #[arg(requires = "field")]
    pub value: Option<String>,

    /// Merge a JSON object into the section
    #[arg(long, conflicts_with_all = ["field", "value"])]
    pub data: Option<String>,
}
