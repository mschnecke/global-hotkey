//! Configuration file management
//!
//! Configuration is split into two files:
//! 1. Settings file (~/.global-hotkey-settings.json) - Fixed location, contains app preferences
//! 2. Config file (default ~/.global-hotkey/config.json) - Configurable location, contains hotkeys and AI settings

use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;

use crate::error::AppError;

use super::schema::{AppConfig, AppSettings, FullConfig, LegacyAppConfig};
use super::validation;

const SETTINGS_FILE_NAME: &str = ".global-hotkey-settings.json";
const DEFAULT_CONFIG_DIR: &str = ".global-hotkey";
const CONFIG_FILE_NAME: &str = "config.json";
const BACKUP_FILE_NAME: &str = "config.backup.json";
const LEGACY_CONFIG_FILE: &str = ".global-hotkey.json";
const LEGACY_BACKUP_FILE: &str = ".global-hotkey.backup.json";

/// Cached settings for determining config location
static CACHED_SETTINGS: RwLock<Option<AppSettings>> = RwLock::new(None);

/// Get the user's home directory
fn get_home_dir() -> Result<PathBuf, AppError> {
    dirs::home_dir().ok_or_else(|| AppError::Config("Cannot find home directory".into()))
}

/// Get the settings file path (always in home directory)
fn get_settings_path() -> Result<PathBuf, AppError> {
    Ok(get_home_dir()?.join(SETTINGS_FILE_NAME))
}

/// Get the default config directory path
fn get_default_config_dir() -> Result<PathBuf, AppError> {
    Ok(get_home_dir()?.join(DEFAULT_CONFIG_DIR))
}

/// Get the config directory path (from settings or default)
pub fn get_config_dir() -> Result<PathBuf, AppError> {
    let settings = CACHED_SETTINGS.read().unwrap();
    if let Some(ref s) = *settings {
        if let Some(ref custom_path) = s.config_location {
            return Ok(PathBuf::from(custom_path));
        }
    }
    get_default_config_dir()
}

/// Get the main configuration file path
fn get_config_path() -> Result<PathBuf, AppError> {
    Ok(get_config_dir()?.join(CONFIG_FILE_NAME))
}

/// Get the backup configuration file path
fn get_backup_path() -> Result<PathBuf, AppError> {
    Ok(get_config_dir()?.join(BACKUP_FILE_NAME))
}

/// Get the legacy configuration file path (for migration)
fn get_legacy_config_path() -> Result<PathBuf, AppError> {
    Ok(get_home_dir()?.join(LEGACY_CONFIG_FILE))
}

/// Get the legacy backup file path (for migration)
fn get_legacy_backup_path() -> Result<PathBuf, AppError> {
    Ok(get_home_dir()?.join(LEGACY_BACKUP_FILE))
}

/// Migrate from legacy single-file format to new dual-file format
fn migrate_from_legacy() -> Result<bool, AppError> {
    let legacy_path = get_legacy_config_path()?;
    let settings_path = get_settings_path()?;

    // Only migrate if legacy exists and new settings don't
    if !legacy_path.exists() || settings_path.exists() {
        return Ok(false);
    }

    eprintln!("Migrating from legacy config format...");

    // Read the legacy config
    let content = fs::read_to_string(&legacy_path)?;
    let legacy: LegacyAppConfig = serde_json::from_str(&content).map_err(|e| {
        AppError::Config(format!("Failed to parse legacy config: {}", e))
    })?;

    // Create new settings
    let settings = AppSettings {
        start_with_system: legacy.settings.start_with_system,
        show_tray_notifications: legacy.settings.show_tray_notifications,
        config_location: None, // Use default location
    };

    // Create new config
    let config = AppConfig {
        version: legacy.version,
        hotkeys: legacy.hotkeys,
        ai: legacy.settings.ai,
    };

    // Save new settings file
    save_settings(&settings)?;

    // Cache the settings
    {
        let mut cached = CACHED_SETTINGS.write().unwrap();
        *cached = Some(settings);
    }

    // Ensure config directory exists
    let config_dir = get_config_dir()?;
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }

    // Save new config file
    save_config(&config)?;

    // Optionally copy legacy backup
    let legacy_backup = get_legacy_backup_path()?;
    if legacy_backup.exists() {
        // We can't easily migrate the backup since it's in old format,
        // but we keep it as-is for safety
        eprintln!("Legacy backup preserved at: {:?}", legacy_backup);
    }

    eprintln!("Migration complete. New config at: {:?}", get_config_path()?);
    Ok(true)
}

/// Initialize the configuration system
pub fn init() -> Result<(), AppError> {
    // First, load settings (or create defaults)
    let settings = load_settings()?;

    // Cache settings for path resolution
    {
        let mut cached = CACHED_SETTINGS.write().unwrap();
        *cached = Some(settings);
    }

    // Try to migrate from legacy format
    migrate_from_legacy()?;

    // Ensure config directory exists
    let config_dir = get_config_dir()?;
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

/// Load settings from file
pub fn load_settings() -> Result<AppSettings, AppError> {
    let settings_path = get_settings_path()?;

    if settings_path.exists() {
        let content = fs::read_to_string(&settings_path)?;
        match serde_json::from_str::<AppSettings>(&content) {
            Ok(settings) => {
                // Update cache
                let mut cached = CACHED_SETTINGS.write().unwrap();
                *cached = Some(settings.clone());
                return Ok(settings);
            }
            Err(e) => {
                eprintln!("Settings file corrupted: {}", e);
                // Fall through to create default
            }
        }
    }

    // Return default settings
    let settings = AppSettings::default();
    // Update cache
    {
        let mut cached = CACHED_SETTINGS.write().unwrap();
        *cached = Some(settings.clone());
    }
    Ok(settings)
}

/// Save settings to file
pub fn save_settings(settings: &AppSettings) -> Result<(), AppError> {
    let settings_path = get_settings_path()?;
    let content = serde_json::to_string_pretty(settings)?;
    fs::write(&settings_path, content)?;

    // Update cache
    let mut cached = CACHED_SETTINGS.write().unwrap();
    *cached = Some(settings.clone());

    Ok(())
}

/// Load configuration from file
pub fn load_config() -> Result<AppConfig, AppError> {
    let config_path = get_config_path()?;

    // Try loading main config
    if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        match serde_json::from_str::<AppConfig>(&content) {
            Ok(mut config) => {
                // Initialize built-in roles if empty
                if config.ai.roles.is_empty() {
                    config.ai.roles = crate::ai::get_builtin_roles();
                }
                validation::validate_config(&config)?;
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
    let mut config = AppConfig::default();
    config.ai.roles = crate::ai::get_builtin_roles();
    Ok(config)
}

/// Load configuration from backup file
fn load_backup() -> Result<AppConfig, AppError> {
    let backup_path = get_backup_path()?;

    if backup_path.exists() {
        let content = fs::read_to_string(&backup_path)?;
        let config: AppConfig = serde_json::from_str(&content)?;
        validation::validate_config(&config)?;

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
    validation::validate_config(config)?;

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

/// Load full config (both settings and config)
pub fn load_full_config() -> Result<FullConfig, AppError> {
    let settings = load_settings()?;
    let config = load_config()?;
    Ok(FullConfig { settings, config })
}

/// Save full config (both settings and config)
pub fn save_full_config(full: &FullConfig) -> Result<(), AppError> {
    save_settings(&full.settings)?;
    save_config(&full.config)?;
    Ok(())
}

/// Change the config location
/// This will copy existing config to the new location
pub fn change_config_location(new_path: Option<String>) -> Result<(), AppError> {
    let old_config_dir = get_config_dir()?;
    let old_config_path = get_config_path()?;
    let old_backup_path = get_backup_path()?;

    // Determine new directory
    let new_config_dir = match &new_path {
        Some(p) => PathBuf::from(p),
        None => get_default_config_dir()?,
    };

    // Don't do anything if paths are the same
    if old_config_dir == new_config_dir {
        return Ok(());
    }

    // Create new directory if needed
    if !new_config_dir.exists() {
        fs::create_dir_all(&new_config_dir)?;
    }

    // Copy config file if it exists
    if old_config_path.exists() {
        let new_config_path = new_config_dir.join(CONFIG_FILE_NAME);
        fs::copy(&old_config_path, &new_config_path)?;
    }

    // Copy backup file if it exists
    if old_backup_path.exists() {
        let new_backup_path = new_config_dir.join(BACKUP_FILE_NAME);
        fs::copy(&old_backup_path, &new_backup_path)?;
    }

    // Update settings with new location
    let mut settings = load_settings()?;
    settings.config_location = new_path;
    save_settings(&settings)?;

    Ok(())
}

/// Get the current config directory path (for UI display)
pub fn get_config_location() -> Result<String, AppError> {
    let config_dir = get_config_dir()?;
    Ok(config_dir.to_string_lossy().to_string())
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
    validation::validate_config(&config)?;
    Ok(config)
}
