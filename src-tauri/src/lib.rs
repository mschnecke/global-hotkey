//! Global Hotkey - Cross-platform keystroke-summoned program launcher
//!
//! This library provides the Tauri backend for the Global Hotkey application.

mod config;
mod error;
mod hotkey;
mod postaction;
mod process;
mod tray;

pub use config::schema::{AppConfig, AppSettings, HotkeyBinding, HotkeyConfig, ProgramConfig};
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
// Application Entry Point
// ============================================================================

/// Run the Tauri application
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
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

            // Keep tray alive for the lifetime of the app
            std::mem::forget(tray);

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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
