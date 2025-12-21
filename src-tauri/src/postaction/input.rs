//! Keystroke simulation using enigo

use enigo::{Direction, Enigo, Key, Keyboard, Settings};

use crate::config::schema::Keystroke;
use crate::error::AppError;

pub struct InputSimulator {
    enigo: Enigo,
}

impl InputSimulator {
    pub fn new() -> Result<Self, AppError> {
        let enigo = Enigo::new(&Settings::default()).map_err(|e| {
            AppError::PostAction(format!("Failed to create input simulator: {}", e))
        })?;
        Ok(Self { enigo })
    }

    /// Simulate a paste operation (Ctrl+V on Windows, Cmd+V on macOS)
    pub fn paste(&mut self) -> Result<(), AppError> {
        #[cfg(target_os = "macos")]
        let modifier = Key::Meta;

        #[cfg(not(target_os = "macos"))]
        let modifier = Key::Control;

        self.enigo
            .key(modifier, Direction::Press)
            .map_err(|e| AppError::PostAction(format!("Failed to press modifier: {}", e)))?;
        self.enigo
            .key(Key::Unicode('v'), Direction::Click)
            .map_err(|e| AppError::PostAction(format!("Failed to press V: {}", e)))?;
        self.enigo
            .key(modifier, Direction::Release)
            .map_err(|e| AppError::PostAction(format!("Failed to release modifier: {}", e)))?;

        Ok(())
    }

    /// Simulate a keystroke with modifiers
    pub fn simulate_keystroke(&mut self, keystroke: &Keystroke) -> Result<(), AppError> {
        // Press all modifiers
        for modifier in &keystroke.modifiers {
            let key = self.map_modifier(modifier)?;
            self.enigo
                .key(key, Direction::Press)
                .map_err(|e| AppError::PostAction(format!("Failed to press modifier: {}", e)))?;
        }

        // Press and release the key
        let key = self.map_key(&keystroke.key)?;
        self.enigo
            .key(key, Direction::Click)
            .map_err(|e| AppError::PostAction(format!("Failed to press key: {}", e)))?;

        // Release all modifiers (in reverse order)
        for modifier in keystroke.modifiers.iter().rev() {
            let key = self.map_modifier(modifier)?;
            self.enigo
                .key(key, Direction::Release)
                .map_err(|e| AppError::PostAction(format!("Failed to release modifier: {}", e)))?;
        }

        Ok(())
    }

    fn map_modifier(&self, modifier: &str) -> Result<Key, AppError> {
        match modifier.to_lowercase().as_str() {
            "ctrl" | "control" => Ok(Key::Control),
            "alt" => Ok(Key::Alt),
            "shift" => Ok(Key::Shift),
            "meta" | "cmd" | "command" | "win" | "super" => Ok(Key::Meta),
            other => Err(AppError::PostAction(format!("Unknown modifier: {}", other))),
        }
    }

    fn map_key(&self, key: &str) -> Result<Key, AppError> {
        // Handle single character keys
        if key.len() == 1 {
            let c = key.chars().next().unwrap();
            return Ok(Key::Unicode(c.to_ascii_lowercase()));
        }

        // Handle special keys
        match key.to_uppercase().as_str() {
            "ENTER" | "RETURN" => Ok(Key::Return),
            "TAB" => Ok(Key::Tab),
            "SPACE" => Ok(Key::Space),
            "BACKSPACE" => Ok(Key::Backspace),
            "DELETE" => Ok(Key::Delete),
            "ESCAPE" | "ESC" => Ok(Key::Escape),
            "UP" | "ARROWUP" => Ok(Key::UpArrow),
            "DOWN" | "ARROWDOWN" => Ok(Key::DownArrow),
            "LEFT" | "ARROWLEFT" => Ok(Key::LeftArrow),
            "RIGHT" | "ARROWRIGHT" => Ok(Key::RightArrow),
            "HOME" => Ok(Key::Home),
            "END" => Ok(Key::End),
            "PAGEUP" => Ok(Key::PageUp),
            "PAGEDOWN" => Ok(Key::PageDown),
            "F1" => Ok(Key::F1),
            "F2" => Ok(Key::F2),
            "F3" => Ok(Key::F3),
            "F4" => Ok(Key::F4),
            "F5" => Ok(Key::F5),
            "F6" => Ok(Key::F6),
            "F7" => Ok(Key::F7),
            "F8" => Ok(Key::F8),
            "F9" => Ok(Key::F9),
            "F10" => Ok(Key::F10),
            "F11" => Ok(Key::F11),
            "F12" => Ok(Key::F12),
            _ => Err(AppError::PostAction(format!("Unknown key: {}", key))),
        }
    }
}
