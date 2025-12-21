//! Application error types

use thiserror::Error;

/// Application-wide error type
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Hotkey error: {0}")]
    Hotkey(String),

    #[error("Process error: {0}")]
    Process(String),

    #[error("Tray error: {0}")]
    Tray(String),

    #[error("Post-action error: {0}")]
    PostAction(String),

    #[error("AI error: {0}")]
    Ai(String),

    #[error("Audio error: {0}")]
    Audio(String),
}

impl From<AppError> for String {
    fn from(error: AppError) -> Self {
        error.to_string()
    }
}
