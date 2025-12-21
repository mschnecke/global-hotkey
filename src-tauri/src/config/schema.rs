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
    #[serde(default)]
    pub post_actions: PostActionsConfig,
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

/// Trigger timing for post-actions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum PostActionTrigger {
    /// Execute after process exits with code 0
    #[default]
    OnExit,
    /// Execute after a delay (milliseconds) from process start
    AfterDelay { delay_ms: u64 },
}

/// Keystroke for simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Keystroke {
    pub modifiers: Vec<String>,
    pub key: String,
}

/// Types of post-actions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum PostActionType {
    /// Simulate Ctrl+V (Cmd+V on macOS) to paste clipboard
    PasteClipboard,
    /// Simulate a custom keystroke combination
    SimulateKeystroke { keystroke: Keystroke },
    /// Wait for a specified duration before next action
    Delay { delay_ms: u64 },
}

/// A single post-action
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostAction {
    pub id: String,
    pub action_type: PostActionType,
    pub enabled: bool,
}

/// Post-action configuration for a hotkey
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PostActionsConfig {
    /// Whether post-actions are enabled for this hotkey
    pub enabled: bool,
    /// When to trigger post-actions
    #[serde(default)]
    pub trigger: PostActionTrigger,
    /// Ordered list of actions to execute
    #[serde(default)]
    pub actions: Vec<PostAction>,
}
