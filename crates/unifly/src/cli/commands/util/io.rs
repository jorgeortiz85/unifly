use std::io::IsTerminal;
use std::path::Path;

use crate::cli::error::CliError;

pub fn confirm(message: &str, yes_flag: bool) -> Result<bool, CliError> {
    if yes_flag {
        return Ok(true);
    }

    if !std::io::stdin().is_terminal() {
        return Err(CliError::NonInteractiveRequiresYes {
            action: message.into(),
        });
    }

    let confirmed = dialoguer::Confirm::new()
        .with_prompt(message)
        .default(false)
        .interact()
        .map_err(|error| CliError::Io(std::io::Error::other(error)))?;
    Ok(confirmed)
}

pub fn read_json_file(path: &Path) -> Result<serde_json::Value, CliError> {
    let file = std::fs::File::open(path)?;
    let reader = json_comments::StripComments::new(std::io::BufReader::new(file));
    serde_json::from_reader(reader).map_err(|error| CliError::Validation {
        field: "from-file".into(),
        reason: format!("invalid JSON: {error}"),
    })
}
