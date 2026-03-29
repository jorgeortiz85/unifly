use std::path::Path;

use crate::cli::error::CliError;

pub fn confirm(message: &str, yes_flag: bool) -> Result<bool, CliError> {
    if yes_flag {
        return Ok(true);
    }

    let confirmed = dialoguer::Confirm::new()
        .with_prompt(message)
        .default(false)
        .interact()
        .map_err(|error| CliError::Io(std::io::Error::other(error)))?;
    Ok(confirmed)
}

pub fn read_json_file(path: &Path) -> Result<serde_json::Value, CliError> {
    let contents = std::fs::read_to_string(path)?;
    serde_json::from_str(&contents).map_err(|error| CliError::Validation {
        field: "from-file".into(),
        reason: format!("invalid JSON: {error}"),
    })
}
