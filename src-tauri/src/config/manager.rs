//! Configuration file management

use std::fs;
use std::path::PathBuf;

use crate::error::AppError;

use super::schema::AppConfig;
use super::validation;

/// Get the configuration directory path
fn get_config_dir() -> Result<PathBuf, AppError> {
    let home =
        dirs::home_dir().ok_or_else(|| AppError::Config("Cannot find home directory".into()))?;
    Ok(home.join("global-hotkey"))
}

/// Get the main configuration file path
fn get_config_path() -> Result<PathBuf, AppError> {
    Ok(get_config_dir()?.join("config.json"))
}

/// Get the backup configuration file path
fn get_backup_path() -> Result<PathBuf, AppError> {
    Ok(get_config_dir()?.join("config.backup.json"))
}

/// Initialize the configuration system
pub fn init() -> Result<(), AppError> {
    let config_dir = get_config_dir()?;

    // Create config directory if it doesn't exist
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }

    // Create default config if it doesn't exist
    let config_path = get_config_path()?;
    if !config_path.exists() {
        let default_config = AppConfig::default();
        save_config(&default_config)?;
    }

    Ok(())
}

/// Load configuration from file
pub fn load_config() -> Result<AppConfig, AppError> {
    let config_path = get_config_path()?;

    // Try loading main config
    if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        match serde_json::from_str::<AppConfig>(&content) {
            Ok(config) => {
                validation::validate(&config)?;
                return Ok(config);
            }
            Err(e) => {
                eprintln!("Main config corrupted: {}", e);
                // Try loading backup
                return load_backup();
            }
        }
    }

    // Return default config if no file exists
    Ok(AppConfig::default())
}

/// Load configuration from backup file
fn load_backup() -> Result<AppConfig, AppError> {
    let backup_path = get_backup_path()?;

    if backup_path.exists() {
        let content = fs::read_to_string(&backup_path)?;
        let config: AppConfig = serde_json::from_str(&content)?;
        validation::validate(&config)?;

        // Restore backup to main config
        let config_path = get_config_path()?;
        fs::copy(&backup_path, &config_path)?;

        Ok(config)
    } else {
        Ok(AppConfig::default())
    }
}

/// Save configuration to file
pub fn save_config(config: &AppConfig) -> Result<(), AppError> {
    validation::validate(config)?;

    let config_path = get_config_path()?;
    let backup_path = get_backup_path()?;

    // Create backup of existing config
    if config_path.exists() {
        fs::copy(&config_path, &backup_path)?;
    }

    // Write new config
    let content = serde_json::to_string_pretty(config)?;
    fs::write(&config_path, content)?;

    Ok(())
}

/// Export configuration to a user-specified file
pub fn export_config(config: &AppConfig, path: &str) -> Result<(), AppError> {
    let content = serde_json::to_string_pretty(config)?;
    fs::write(path, content)?;
    Ok(())
}

/// Import configuration from a user-specified file
pub fn import_config(path: &str) -> Result<AppConfig, AppError> {
    let content = fs::read_to_string(path)?;
    let config: AppConfig = serde_json::from_str(&content)?;
    validation::validate(&config)?;
    Ok(config)
}
