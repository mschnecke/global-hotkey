//! Global Hotkey - Cross-platform keystroke-summoned program launcher
//!
//! This library provides the Tauri backend for the Global Hotkey application.

mod ai;
mod audio;
mod config;
mod error;
mod hotkey;
mod postaction;
mod process;
mod tray;

use once_cell::sync::Lazy;
use std::sync::Mutex;

// Global audio recorder state
static AUDIO_RECORDER: Lazy<Mutex<Option<audio::AudioRecorderHandle>>> =
    Lazy::new(|| Mutex::new(None));

pub use config::schema::{AppConfig, AppSettings, HotkeyAction, HotkeyBinding, HotkeyConfig, ProgramConfig};
pub use error::AppError;

// ============================================================================
// Tauri Commands - Configuration
// ============================================================================

/// Get the current application configuration
#[tauri::command]
async fn get_config() -> Result<AppConfig, String> {
    config::manager::load_config().map_err(|e| e.to_string())
}

/// Save the application configuration
#[tauri::command]
async fn save_config(config: AppConfig) -> Result<(), String> {
    config::manager::save_config(&config).map_err(|e| e.to_string())
}

/// Export configuration to a user-specified file
#[tauri::command]
async fn export_config(path: String) -> Result<(), String> {
    let config = config::manager::load_config().map_err(|e| e.to_string())?;
    config::manager::export_config(&config, &path).map_err(|e| e.to_string())
}

/// Import configuration from a user-specified file
#[tauri::command]
async fn import_config(path: String) -> Result<AppConfig, String> {
    config::manager::import_config(&path).map_err(|e| e.to_string())
}

// ============================================================================
// Tauri Commands - Hotkey Management
// ============================================================================

/// Register a hotkey with the system
#[tauri::command]
async fn register_hotkey(app: tauri::AppHandle, config: HotkeyConfig) -> Result<(), String> {
    // Must run on main thread because GlobalHotKeyManager uses thread-local storage
    let (tx, rx) = std::sync::mpsc::channel();
    app.run_on_main_thread(move || {
        let result = hotkey::manager::register(&config);
        let _ = tx.send(result);
    })
    .map_err(|e| e.to_string())?;

    rx.recv()
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

/// Unregister a hotkey from the system
#[tauri::command]
async fn unregister_hotkey(app: tauri::AppHandle, id: String) -> Result<(), String> {
    // Must run on main thread because GlobalHotKeyManager uses thread-local storage
    let (tx, rx) = std::sync::mpsc::channel();
    app.run_on_main_thread(move || {
        let result = hotkey::manager::unregister(&id);
        let _ = tx.send(result);
    })
    .map_err(|e| e.to_string())?;

    rx.recv()
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

/// Check if a hotkey binding conflicts with existing hotkeys
#[tauri::command]
async fn check_conflict(binding: HotkeyBinding) -> Result<bool, String> {
    Ok(hotkey::conflict::check_conflict(&binding))
}

/// Check if a hotkey conflicts with system hotkeys
#[tauri::command]
async fn check_system_conflict(binding: HotkeyBinding) -> Result<bool, String> {
    Ok(hotkey::conflict::conflicts_with_system(&binding))
}

/// Get list of currently registered hotkey IDs
#[tauri::command]
async fn get_registered_hotkeys() -> Vec<String> {
    hotkey::manager::get_registered_ids()
}

// ============================================================================
// Tauri Commands - Process Management
// ============================================================================

/// Launch a program with the given configuration
#[tauri::command]
async fn launch_program(config: ProgramConfig) -> Result<(), String> {
    process::spawner::launch(&config).map_err(|e| e.to_string())
}

/// Validate that a program path exists and is executable
#[tauri::command]
async fn validate_program_path(path: String) -> Result<bool, String> {
    Ok(process::spawner::validate_path(&path))
}

/// Get executable file extensions for the current platform
#[tauri::command]
async fn get_executable_extensions() -> Vec<&'static str> {
    process::spawner::get_executable_extensions()
}

// ============================================================================
// Tauri Commands - System Tray
// ============================================================================

/// Update the tray menu with current hotkeys
#[tauri::command]
async fn update_tray_menu(app: tauri::AppHandle) -> Result<(), String> {
    let config = config::manager::load_config().map_err(|e| e.to_string())?;
    tray::update_menu(&app, &config.hotkeys).map_err(|e| e.to_string())
}

/// Update the tray icon based on system theme
#[tauri::command]
async fn update_tray_icon(app: tauri::AppHandle) -> Result<(), String> {
    tray::update_tray_icon(&app).map_err(|e| e.to_string())
}

/// Check if system is using dark mode
#[tauri::command]
async fn is_dark_mode() -> bool {
    tray::is_dark_mode()
}

/// Check if autostart is enabled
#[tauri::command]
async fn get_autostart(app: tauri::AppHandle) -> bool {
    tray::is_autostart_enabled(&app)
}

/// Set autostart state
#[tauri::command]
async fn set_autostart(app: tauri::AppHandle, enabled: bool) -> Result<(), String> {
    tray::set_autostart(&app, enabled).map_err(|e| e.to_string())
}

// ============================================================================
// Tauri Commands - AI Module
// ============================================================================

/// Test an AI provider connection
#[tauri::command]
async fn test_ai_provider(api_key: String, model: Option<String>) -> Result<bool, String> {
    use ai::AiProvider;
    let provider = ai::GeminiProvider::new(api_key, model);
    provider.test_connection().await.map_err(|e| e.to_string())
}

/// Send text to AI and get response
#[tauri::command]
async fn send_to_ai(
    api_key: String,
    model: Option<String>,
    system_prompt: String,
    user_input: String,
) -> Result<String, String> {
    use ai::AiProvider;
    let provider = ai::GeminiProvider::new(api_key, model);
    let response = provider
        .send_text(&system_prompt, &user_input)
        .await
        .map_err(|e| e.to_string())?;
    Ok(response.text)
}

/// Get built-in AI roles
#[tauri::command]
fn get_builtin_roles() -> Vec<config::schema::AiRole> {
    ai::get_builtin_roles()
}

/// Save an AI role (create or update)
#[tauri::command]
async fn save_ai_role(role: config::schema::AiRole) -> Result<(), String> {
    let mut app_config = config::manager::load_config().map_err(|e| e.to_string())?;

    // Find existing role index
    let existing_idx = app_config
        .settings
        .ai
        .roles
        .iter()
        .position(|r| r.id == role.id);

    if let Some(idx) = existing_idx {
        // Update existing role (only if not builtin)
        if app_config.settings.ai.roles[idx].is_builtin {
            return Err("Cannot modify built-in roles".to_string());
        }
        app_config.settings.ai.roles[idx] = role;
    } else {
        // Add new role
        app_config.settings.ai.roles.push(role);
    }

    config::manager::save_config(&app_config).map_err(|e| e.to_string())
}

/// Delete an AI role
#[tauri::command]
async fn delete_ai_role(role_id: String) -> Result<(), String> {
    let mut app_config = config::manager::load_config().map_err(|e| e.to_string())?;

    // Find the role
    let role_idx = app_config
        .settings
        .ai
        .roles
        .iter()
        .position(|r| r.id == role_id);

    if let Some(idx) = role_idx {
        // Check if it's a built-in role
        if app_config.settings.ai.roles[idx].is_builtin {
            return Err("Cannot delete built-in roles".to_string());
        }
        app_config.settings.ai.roles.remove(idx);
        config::manager::save_config(&app_config).map_err(|e| e.to_string())
    } else {
        Err("Role not found".to_string())
    }
}

// ============================================================================
// Tauri Commands - Audio Recording
// ============================================================================

/// Start audio recording
#[tauri::command]
async fn start_audio_recording() -> Result<(), String> {
    let recorder = audio::AudioRecorderHandle::start().map_err(|e| e.to_string())?;

    let mut guard = AUDIO_RECORDER
        .lock()
        .map_err(|e| format!("Failed to lock recorder: {}", e))?;
    *guard = Some(recorder);

    Ok(())
}

/// Stop audio recording and return WAV data as base64
#[tauri::command]
async fn stop_audio_recording() -> Result<String, String> {
    let mut guard = AUDIO_RECORDER
        .lock()
        .map_err(|e| format!("Failed to lock recorder: {}", e))?;

    let recorder = guard
        .take()
        .ok_or_else(|| "No active recording".to_string())?;

    let (samples, sample_rate, channels) = recorder.stop().map_err(|e| e.to_string())?;

    let wav_data =
        audio::encode_to_wav(&samples, sample_rate, channels).map_err(|e| e.to_string())?;

    // Return as base64 for easy transfer to frontend
    use base64::Engine;
    Ok(base64::engine::general_purpose::STANDARD.encode(&wav_data))
}

/// Check if currently recording
#[tauri::command]
async fn is_audio_recording() -> Result<bool, String> {
    let guard = AUDIO_RECORDER
        .lock()
        .map_err(|e| format!("Failed to lock recorder: {}", e))?;

    Ok(guard.as_ref().map(|r| r.is_recording()).unwrap_or(false))
}

/// Send audio to AI for transcription/processing
#[tauri::command]
async fn send_audio_to_ai(
    api_key: String,
    model: Option<String>,
    system_prompt: String,
    audio_base64: String,
) -> Result<String, String> {
    use ai::AiProvider;
    use base64::Engine;

    let audio_data = base64::engine::general_purpose::STANDARD
        .decode(&audio_base64)
        .map_err(|e| format!("Failed to decode audio: {}", e))?;

    let provider = ai::GeminiProvider::new(api_key, model);
    let response = provider
        .send_audio(&system_prompt, &audio_data, "audio/wav")
        .await
        .map_err(|e| e.to_string())?;

    Ok(response.text)
}

// ============================================================================
// Application Entry Point
// ============================================================================

/// Run the Tauri application
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .setup(|app| {
            use tauri::menu::{Menu, MenuItem};
            use tauri::tray::TrayIconBuilder;
            use tauri::Manager;

            // Hide dock icon on macOS - this is a menu bar app
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Store app handle for global access (tray icon changes, notifications)
            tray::set_app_handle(app.handle().clone());

            // Initialize configuration directory and files
            if let Err(e) = config::manager::init() {
                eprintln!("Failed to initialize config: {}", e);
            }

            // Initialize hotkey manager
            if let Err(e) = hotkey::manager::init() {
                eprintln!("Failed to initialize hotkey manager: {}", e);
            }

            // Load saved configuration
            let loaded_config = config::manager::load_config().ok();

            // Set up system tray
            let settings_item =
                MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&settings_item, &quit_item])?;

            // Load icon - embedded at compile time for reliability
            let tray_icon =
                tauri::image::Image::from_bytes(include_bytes!("../icons/tray-icon@2x.png"))
                    .expect("Failed to load tray icon");

            let tray = TrayIconBuilder::new()
                .icon(tray_icon)
                .icon_as_template(cfg!(target_os = "macos"))
                .tooltip("Global Hotkey")
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "settings" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            // Store tray icon reference for later updates
            {
                let mut tray_ref = tray::TRAY.write().unwrap();
                *tray_ref = Some(tray);
            }

            // Register saved hotkeys
            if let Some(cfg) = loaded_config {
                for hk in cfg.hotkeys.iter().filter(|h| h.enabled) {
                    if let Err(e) = hotkey::manager::register(hk) {
                        eprintln!("Failed to register hotkey '{}': {}", hk.name, e);
                    }
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            // Hide window instead of closing when close button is clicked
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                window.hide().unwrap_or_default();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            // Config commands
            get_config,
            save_config,
            export_config,
            import_config,
            // Hotkey commands
            register_hotkey,
            unregister_hotkey,
            check_conflict,
            check_system_conflict,
            get_registered_hotkeys,
            // Process commands
            launch_program,
            validate_program_path,
            get_executable_extensions,
            // Tray commands
            update_tray_menu,
            update_tray_icon,
            is_dark_mode,
            get_autostart,
            set_autostart,
            // AI commands
            test_ai_provider,
            send_to_ai,
            get_builtin_roles,
            save_ai_role,
            delete_ai_role,
            // Audio commands
            start_audio_recording,
            stop_audio_recording,
            is_audio_recording,
            send_audio_to_ai,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
