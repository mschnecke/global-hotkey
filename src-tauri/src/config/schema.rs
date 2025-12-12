//! Configuration data structures

use serde::{Deserialize, Serialize};

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub version: String,
    pub hotkeys: Vec<HotkeyConfig>,
    pub settings: AppSettings,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            hotkeys: Vec::new(),
            settings: AppSettings::default(),
        }
    }
}

/// Configuration for a single hotkey
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyConfig {
    pub id: String,
    pub name: String,
    pub hotkey: HotkeyBinding,
    pub program: ProgramConfig,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Hotkey binding (modifiers + key)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyBinding {
    pub modifiers: Vec<String>,
    pub key: String,
}

/// Program launch configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgramConfig {
    pub path: String,
    pub arguments: Vec<String>,
    pub working_directory: Option<String>,
    pub hidden: bool,
}

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub start_with_system: bool,
    pub show_tray_notifications: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            start_with_system: false,
            show_tray_notifications: true,
        }
    }
}
