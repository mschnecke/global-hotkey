//! Hotkey registration and management

use std::collections::HashMap;
use std::sync::RwLock;

use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
};
use once_cell::sync::Lazy;

use crate::config::schema::{HotkeyBinding, HotkeyConfig};
use crate::error::AppError;
use crate::process;

/// Registry entry containing hotkey ID, HotKey object, and configuration
type RegistryEntry = (u32, HotKey, HotkeyConfig);

/// Registry mapping hotkey IDs to their configurations and HotKey objects
/// We store the hotkey ID (u32) instead of the HotKey object because HotKey doesn't implement Clone
pub static REGISTRY: Lazy<RwLock<HashMap<String, RegistryEntry>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

// Thread-local hotkey manager - must be used on the main thread only
thread_local! {
    static MANAGER: std::cell::RefCell<Option<GlobalHotKeyManager>> = const { std::cell::RefCell::new(None) };
}

/// Flag to track if event loop is running
static EVENT_LOOP_RUNNING: Lazy<RwLock<bool>> = Lazy::new(|| RwLock::new(false));

/// Initialize the hotkey manager and start the event loop
/// Must be called from the main thread
pub fn init() -> Result<(), AppError> {
    MANAGER.with(|m| {
        let mut manager_ref = m.borrow_mut();
        if manager_ref.is_some() {
            return Ok::<(), AppError>(());
        }

        let manager = GlobalHotKeyManager::new()
            .map_err(|e| AppError::Hotkey(format!("Failed to create hotkey manager: {}", e)))?;

        *manager_ref = Some(manager);
        Ok::<(), AppError>(())
    })?;

    // Start event handler thread if not already running
    let mut running = EVENT_LOOP_RUNNING.write().unwrap();
    if !*running {
        *running = true;
        start_event_loop();
    }

    Ok(())
}

/// Start the hotkey event loop in a background thread
fn start_event_loop() {
    std::thread::spawn(|| {
        let receiver = GlobalHotKeyEvent::receiver();
        loop {
            if let Ok(event) = receiver.recv() {
                handle_event(event);
            }
        }
    });
}

/// Handle a hotkey event
fn handle_event(event: GlobalHotKeyEvent) {
    if event.state != HotKeyState::Pressed {
        return;
    }

    let registry = REGISTRY.read().unwrap();
    for (_, (hotkey_id, _, config)) in registry.iter() {
        if *hotkey_id == event.id {
            let program_config = config.program.clone();
            let hotkey_name = config.name.clone();

            // Spawn in a separate thread to avoid blocking the event loop
            std::thread::spawn(move || {
                if let Err(e) = process::spawner::launch(&program_config) {
                    eprintln!(
                        "Failed to launch program for hotkey '{}': {}",
                        hotkey_name, e
                    );
                }
            });
            break;
        }
    }
}

/// Register a hotkey - must be called from the main thread
pub fn register(config: &HotkeyConfig) -> Result<(), AppError> {
    // Parse the hotkey first
    let hotkey = parse_hotkey(&config.hotkey)?;
    let hotkey_id = hotkey.id();

    MANAGER.with(|m| {
        let mut manager_ref = m.borrow_mut();

        // Initialize manager if not already done
        if manager_ref.is_none() {
            let manager = GlobalHotKeyManager::new()
                .map_err(|e| AppError::Hotkey(format!("Failed to create hotkey manager: {}", e)))?;
            *manager_ref = Some(manager);
        }

        if let Some(manager) = manager_ref.as_ref() {
            manager
                .register(hotkey)
                .map_err(|e| AppError::Hotkey(format!("Failed to register hotkey: {}", e)))?;
        }

        Ok::<(), AppError>(())
    })?;

    // Store in registry
    let mut registry = REGISTRY.write().unwrap();
    registry.insert(config.id.clone(), (hotkey_id, hotkey, config.clone()));

    Ok(())
}

/// Unregister a hotkey - must be called from the main thread
pub fn unregister(id: &str) -> Result<(), AppError> {
    let mut registry = REGISTRY.write().unwrap();

    if let Some((_, hotkey, _)) = registry.remove(id) {
        MANAGER.with(|m| {
            let manager_ref = m.borrow();
            if let Some(manager) = manager_ref.as_ref() {
                manager
                    .unregister(hotkey)
                    .map_err(|e| AppError::Hotkey(format!("Failed to unregister hotkey: {}", e)))?;
            }
            Ok::<(), AppError>(())
        })?;
    }

    Ok(())
}

/// Unregister all hotkeys - must be called from the main thread
#[allow(dead_code)]
pub fn unregister_all() -> Result<(), AppError> {
    let mut registry = REGISTRY.write().unwrap();
    let hotkeys: Vec<HotKey> = registry.values().map(|(_, h, _)| *h).collect();

    if !hotkeys.is_empty() {
        MANAGER.with(|m| {
            let manager_ref = m.borrow();
            if let Some(manager) = manager_ref.as_ref() {
                manager.unregister_all(&hotkeys).map_err(|e| {
                    AppError::Hotkey(format!("Failed to unregister hotkeys: {}", e))
                })?;
            }
            Ok::<(), AppError>(())
        })?;
    }

    registry.clear();
    Ok(())
}

/// Get list of registered hotkey IDs
pub fn get_registered_ids() -> Vec<String> {
    let registry = REGISTRY.read().unwrap();
    registry.keys().cloned().collect()
}

/// Check if a hotkey ID is registered
#[allow(dead_code)]
pub fn is_registered(id: &str) -> bool {
    let registry = REGISTRY.read().unwrap();
    registry.contains_key(id)
}

/// Parse a HotkeyBinding into a global_hotkey HotKey
fn parse_hotkey(binding: &HotkeyBinding) -> Result<HotKey, AppError> {
    let modifiers = parse_modifiers(&binding.modifiers)?;
    let code = parse_code(&binding.key)?;

    // HotKey::new returns HotKey directly, not a Result
    let hotkey = if modifiers.is_empty() {
        HotKey::new(None, code)
    } else {
        HotKey::new(Some(modifiers), code)
    };

    Ok(hotkey)
}

/// Parse modifier strings into Modifiers flags
fn parse_modifiers(mods: &[String]) -> Result<Modifiers, AppError> {
    let mut modifiers = Modifiers::empty();

    for m in mods {
        match m.to_lowercase().as_str() {
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            "alt" => modifiers |= Modifiers::ALT,
            "shift" => modifiers |= Modifiers::SHIFT,
            "meta" | "super" | "win" | "cmd" | "command" => modifiers |= Modifiers::META,
            "" => {} // Ignore empty strings
            other => return Err(AppError::Hotkey(format!("Unknown modifier: {}", other))),
        }
    }

    Ok(modifiers)
}

/// Parse a key string into a Code
fn parse_code(key: &str) -> Result<Code, AppError> {
    let code = match key.to_uppercase().as_str() {
        // Letters
        "A" => Code::KeyA,
        "B" => Code::KeyB,
        "C" => Code::KeyC,
        "D" => Code::KeyD,
        "E" => Code::KeyE,
        "F" => Code::KeyF,
        "G" => Code::KeyG,
        "H" => Code::KeyH,
        "I" => Code::KeyI,
        "J" => Code::KeyJ,
        "K" => Code::KeyK,
        "L" => Code::KeyL,
        "M" => Code::KeyM,
        "N" => Code::KeyN,
        "O" => Code::KeyO,
        "P" => Code::KeyP,
        "Q" => Code::KeyQ,
        "R" => Code::KeyR,
        "S" => Code::KeyS,
        "T" => Code::KeyT,
        "U" => Code::KeyU,
        "V" => Code::KeyV,
        "W" => Code::KeyW,
        "X" => Code::KeyX,
        "Y" => Code::KeyY,
        "Z" => Code::KeyZ,

        // Numbers
        "0" | "DIGIT0" => Code::Digit0,
        "1" | "DIGIT1" => Code::Digit1,
        "2" | "DIGIT2" => Code::Digit2,
        "3" | "DIGIT3" => Code::Digit3,
        "4" | "DIGIT4" => Code::Digit4,
        "5" | "DIGIT5" => Code::Digit5,
        "6" | "DIGIT6" => Code::Digit6,
        "7" | "DIGIT7" => Code::Digit7,
        "8" | "DIGIT8" => Code::Digit8,
        "9" | "DIGIT9" => Code::Digit9,

        // Function keys
        "F1" => Code::F1,
        "F2" => Code::F2,
        "F3" => Code::F3,
        "F4" => Code::F4,
        "F5" => Code::F5,
        "F6" => Code::F6,
        "F7" => Code::F7,
        "F8" => Code::F8,
        "F9" => Code::F9,
        "F10" => Code::F10,
        "F11" => Code::F11,
        "F12" => Code::F12,

        // Special keys
        "SPACE" | " " => Code::Space,
        "ENTER" | "RETURN" => Code::Enter,
        "TAB" => Code::Tab,
        "ESCAPE" | "ESC" => Code::Escape,
        "BACKSPACE" => Code::Backspace,
        "DELETE" | "DEL" => Code::Delete,
        "INSERT" | "INS" => Code::Insert,
        "HOME" => Code::Home,
        "END" => Code::End,
        "PAGEUP" | "PGUP" => Code::PageUp,
        "PAGEDOWN" | "PGDN" => Code::PageDown,

        // Arrow keys
        "UP" | "ARROWUP" => Code::ArrowUp,
        "DOWN" | "ARROWDOWN" => Code::ArrowDown,
        "LEFT" | "ARROWLEFT" => Code::ArrowLeft,
        "RIGHT" | "ARROWRIGHT" => Code::ArrowRight,

        // Punctuation and symbols
        "MINUS" | "-" => Code::Minus,
        "EQUAL" | "=" => Code::Equal,
        "BRACKETLEFT" | "[" => Code::BracketLeft,
        "BRACKETRIGHT" | "]" => Code::BracketRight,
        "BACKSLASH" | "\\" => Code::Backslash,
        "SEMICOLON" | ";" => Code::Semicolon,
        "QUOTE" | "'" => Code::Quote,
        "BACKQUOTE" | "`" => Code::Backquote,
        "COMMA" | "," => Code::Comma,
        "PERIOD" | "." => Code::Period,
        "SLASH" | "/" => Code::Slash,

        // Numpad
        "NUMPAD0" => Code::Numpad0,
        "NUMPAD1" => Code::Numpad1,
        "NUMPAD2" => Code::Numpad2,
        "NUMPAD3" => Code::Numpad3,
        "NUMPAD4" => Code::Numpad4,
        "NUMPAD5" => Code::Numpad5,
        "NUMPAD6" => Code::Numpad6,
        "NUMPAD7" => Code::Numpad7,
        "NUMPAD8" => Code::Numpad8,
        "NUMPAD9" => Code::Numpad9,
        "NUMPADADD" | "NUMPAD+" => Code::NumpadAdd,
        "NUMPADSUBTRACT" | "NUMPAD-" => Code::NumpadSubtract,
        "NUMPADMULTIPLY" | "NUMPAD*" => Code::NumpadMultiply,
        "NUMPADDIVIDE" | "NUMPAD/" => Code::NumpadDivide,
        "NUMPADDECIMAL" | "NUMPAD." => Code::NumpadDecimal,
        "NUMPADENTER" => Code::NumpadEnter,

        other => return Err(AppError::Hotkey(format!("Unknown key: {}", other))),
    };

    Ok(code)
}

/// Format a hotkey binding for display
pub fn format_hotkey(binding: &HotkeyBinding) -> String {
    let mut parts: Vec<&str> = binding.modifiers.iter().map(|s| s.as_str()).collect();
    parts.push(&binding.key);
    parts.join(" + ")
}
