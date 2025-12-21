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

/// Main action type for a hotkey
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum HotkeyAction {
    /// Launch a program
    LaunchProgram { program: ProgramConfig },
    /// Call AI with input and save response to clipboard
    CallAi {
        #[serde(rename = "roleId")]
        role_id: String,
        #[serde(rename = "inputSource")]
        input_source: AiInputSource,
        #[serde(default, rename = "providerId")]
        provider_id: Option<String>,
    },
}

/// Configuration for a single hotkey
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyConfig {
    pub id: String,
    pub name: String,
    pub hotkey: HotkeyBinding,
    /// The main action to perform - either launch program or call AI
    pub action: HotkeyAction,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub post_actions: PostActionsConfig,
}

// Custom deserializer to handle both old format (program field) and new format (action field)
impl<'de> serde::Deserialize<'de> for HotkeyConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct HotkeyConfigHelper {
            id: String,
            name: String,
            hotkey: HotkeyBinding,
            // New format
            action: Option<HotkeyAction>,
            // Old format (for backward compatibility)
            program: Option<ProgramConfig>,
            enabled: bool,
            created_at: String,
            updated_at: String,
            #[serde(default)]
            post_actions: PostActionsConfig,
        }

        let helper = HotkeyConfigHelper::deserialize(deserializer)?;

        // Determine action from either new `action` field or old `program` field
        let action = if let Some(action) = helper.action {
            action
        } else if let Some(program) = helper.program {
            // Migrate old format to new format
            HotkeyAction::LaunchProgram { program }
        } else {
            return Err(D::Error::custom(
                "Either 'action' or 'program' field is required",
            ));
        };

        Ok(HotkeyConfig {
            id: helper.id,
            name: helper.name,
            hotkey: helper.hotkey,
            action,
            enabled: helper.enabled,
            created_at: helper.created_at,
            updated_at: helper.updated_at,
            post_actions: helper.post_actions,
        })
    }
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
    #[serde(default)]
    pub ai: AiSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            start_with_system: false,
            show_tray_notifications: true,
            ai: AiSettings::default(),
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

// ============================================================================
// AI Module Types
// ============================================================================

/// AI Provider type
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum AiProviderType {
    #[default]
    Gemini,
    // Future: OpenAi, Anthropic, Ollama
}

/// AI Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiProviderConfig {
    pub id: String,
    pub provider_type: AiProviderType,
    pub api_key: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

/// Output format for AI responses
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum OutputFormat {
    #[default]
    Plain,
    Markdown,
    Json,
}

/// Configurable AI Role/Task
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiRole {
    pub id: String,
    pub name: String,
    pub system_prompt: String,
    #[serde(default)]
    pub output_format: OutputFormat,
    #[serde(default)]
    pub is_builtin: bool,
}

/// AI Settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AiSettings {
    #[serde(default)]
    pub providers: Vec<AiProviderConfig>,
    #[serde(default)]
    pub default_provider_id: Option<String>,
    #[serde(default)]
    pub roles: Vec<AiRole>,
}

/// Audio format for recording
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum AudioFormat {
    #[default]
    Opus,
    Wav,
}

/// Input source for AI actions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum AiInputSource {
    Clipboard,
    RecordAudio {
        #[serde(default = "default_max_duration")]
        max_duration_ms: u64,
        #[serde(default)]
        format: AudioFormat,
    },
    ProcessOutput,
}

fn default_max_duration() -> u64 {
    30000 // 30 seconds
}
