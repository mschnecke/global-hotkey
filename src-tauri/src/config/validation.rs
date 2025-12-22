//! Configuration validation

use crate::error::AppError;

use super::schema::{AppConfig, HotkeyAction};

/// Validate the main configuration
pub fn validate_config(config: &AppConfig) -> Result<(), AppError> {
    // Check version format
    if config.version.is_empty() {
        return Err(AppError::Config("Version cannot be empty".into()));
    }

    // Validate each hotkey
    for hotkey in &config.hotkeys {
        validate_hotkey(hotkey)?;
    }

    Ok(())
}

/// Validate a single hotkey configuration
fn validate_hotkey(hotkey: &super::schema::HotkeyConfig) -> Result<(), AppError> {
    // Check ID
    if hotkey.id.is_empty() {
        return Err(AppError::Config("Hotkey ID cannot be empty".into()));
    }

    // Check name
    if hotkey.name.is_empty() {
        return Err(AppError::Config("Hotkey name cannot be empty".into()));
    }

    // Check hotkey binding
    if hotkey.hotkey.key.is_empty() {
        return Err(AppError::Config("Hotkey key cannot be empty".into()));
    }

    // Validate action based on type
    match &hotkey.action {
        HotkeyAction::LaunchProgram { program } => {
            if program.path.is_empty() {
                return Err(AppError::Config("Program path cannot be empty".into()));
            }
        }
        HotkeyAction::CallAi { role_id, .. } => {
            if role_id.is_empty() {
                return Err(AppError::Config("AI role ID cannot be empty".into()));
            }
        }
    }

    Ok(())
}
