//! System tray functionality with dynamic menu
#![allow(dead_code)] // Some functions reserved for future dynamic menu updates

use image::GenericImageView;
use once_cell::sync::Lazy;
use std::path::Path;
use std::sync::RwLock;
use tauri::{
    image::Image,
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{TrayIcon, TrayIconBuilder},
    AppHandle, Emitter, Manager, Wry,
};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_notification::NotificationExt;

use crate::config::schema::HotkeyConfig;
use crate::error::AppError;
use crate::hotkey;

/// Tray icon state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrayIconState {
    Normal,
    Active, // Recording or processing
}

/// Global app handle for tray/notification access from hotkey manager
pub static APP_HANDLE: Lazy<RwLock<Option<AppHandle>>> = Lazy::new(|| RwLock::new(None));

/// Current tray icon state
static TRAY_STATE: Lazy<RwLock<TrayIconState>> = Lazy::new(|| RwLock::new(TrayIconState::Normal));

// ============================================================================
// Theme Detection (Windows)
// ============================================================================

/// Check if the system is using dark mode (Windows only)
#[cfg(target_os = "windows")]
pub fn is_dark_mode() -> bool {
    use std::ptr;
    use windows::core::PCWSTR;
    use windows::Win32::System::Registry::{
        RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY_CURRENT_USER, KEY_READ, REG_VALUE_TYPE,
    };

    unsafe {
        let subkey: Vec<u16> =
            "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize\0"
                .encode_utf16()
                .collect();
        let value_name: Vec<u16> = "AppsUseLightTheme\0".encode_utf16().collect();

        let mut hkey = std::mem::zeroed();
        let result = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey.as_ptr()),
            0,
            KEY_READ,
            &mut hkey,
        );

        if result.is_err() {
            return false; // Default to light mode if can't read
        }

        let mut data: u32 = 1;
        let mut data_size: u32 = std::mem::size_of::<u32>() as u32;
        let mut data_type = REG_VALUE_TYPE::default();

        let result = RegQueryValueExW(
            hkey,
            PCWSTR(value_name.as_ptr()),
            Some(ptr::null_mut()),
            Some(&mut data_type),
            Some(&mut data as *mut u32 as *mut u8),
            Some(&mut data_size),
        );

        let _ = RegCloseKey(hkey);

        if result.is_ok() {
            // AppsUseLightTheme: 0 = dark mode, 1 = light mode
            data == 0
        } else {
            false // Default to light mode
        }
    }
}

/// macOS handles theme automatically with iconAsTemplate, so always return false
#[cfg(target_os = "macos")]
pub fn is_dark_mode() -> bool {
    false // macOS uses template icons that auto-adapt
}

/// Linux - default to light mode for now
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn is_dark_mode() -> bool {
    false
}

/// Get the appropriate tray icon path based on system theme
fn get_tray_icon_path() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        if is_dark_mode() {
            "icons/tray-icon-dark.png" // White icon for dark backgrounds
        } else {
            "icons/tray-icon-light.png" // Gray icon for light backgrounds
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        "icons/tray-icon.png" // macOS uses template icon
    }
}

/// Get the full path to an icon file, handling both dev and production modes
fn get_icon_full_path(app: &AppHandle, icon_path: &str) -> Result<std::path::PathBuf, AppError> {
    // First try the resource directory (production mode)
    if let Ok(resource_dir) = app.path().resource_dir() {
        let full_path = resource_dir.join(icon_path);
        println!("[TRAY DEBUG] Trying resource_dir path: {:?}", full_path);
        if full_path.exists() {
            println!("[TRAY DEBUG] Found icon at resource_dir: {:?}", full_path);
            return Ok(full_path);
        }
    }

    // Fallback for dev mode: try relative to the executable
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            // In dev mode, icons might be in src-tauri/icons relative to project root
            // Try going up from target/debug to find src-tauri/icons
            let mut current = exe_dir.to_path_buf();
            for _ in 0..5 {
                let dev_path = current.join("src-tauri").join(icon_path);
                println!("[TRAY DEBUG] Trying dev path: {:?}", dev_path);
                if dev_path.exists() {
                    println!("[TRAY DEBUG] Found icon at dev path: {:?}", dev_path);
                    return Ok(dev_path);
                }
                if !current.pop() {
                    break;
                }
            }
        }
    }

    // Last resort: try current working directory
    let cwd_path = std::env::current_dir()
        .map_err(|e| AppError::Tray(format!("Failed to get current dir: {}", e)))?
        .join("src-tauri")
        .join(icon_path);

    println!("[TRAY DEBUG] Trying cwd path: {:?}", cwd_path);
    if cwd_path.exists() {
        println!("[TRAY DEBUG] Found icon at cwd path: {:?}", cwd_path);
        return Ok(cwd_path);
    }

    println!("[TRAY DEBUG] Icon not found anywhere!");
    Err(AppError::Tray(format!(
        "Could not find icon file: {}",
        icon_path
    )))
}

/// Load a PNG image file and convert to Tauri Image format
fn load_icon_from_path<P: AsRef<Path>>(path: P) -> Result<Image<'static>, AppError> {
    let img = image::open(path.as_ref())
        .map_err(|e| AppError::Tray(format!("Failed to open icon: {}", e)))?;

    let (width, height) = img.dimensions();
    let rgba = img.into_rgba8().into_raw();

    Ok(Image::new_owned(rgba, width, height))
}

/// Store a reference to the tray icon for menu updates
pub static TRAY: Lazy<RwLock<Option<TrayIcon>>> = Lazy::new(|| RwLock::new(None));

/// Set up the system tray
pub fn setup(app: &AppHandle) -> Result<(), AppError> {
    let tray = build_tray(app, &[])?;

    // Keep the tray icon alive for the lifetime of the app
    // Without this, the tray icon is dropped and disappears
    std::mem::forget(tray);

    Ok(())
}

/// Build the tray icon with current hotkey list
fn build_tray(app: &AppHandle, hotkeys: &[HotkeyConfig]) -> Result<TrayIcon, AppError> {
    println!("[TRAY DEBUG] Building tray icon...");
    let menu = build_menu(app, hotkeys)?;

    // Load icon using include_bytes! for reliable embedding
    #[cfg(target_os = "macos")]
    let icon = Image::from_bytes(include_bytes!("../icons/tray-icon@2x.png"))
        .map_err(|e| AppError::Tray(format!("Failed to load tray icon: {}", e)))?;

    #[cfg(target_os = "windows")]
    let icon = {
        if is_dark_mode() {
            Image::from_bytes(include_bytes!("../icons/tray-icon-dark.png"))
        } else {
            Image::from_bytes(include_bytes!("../icons/tray-icon-light.png"))
        }
        .map_err(|e| AppError::Tray(format!("Failed to load tray icon: {}", e)))?
    };

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let icon = Image::from_bytes(include_bytes!("../icons/tray-icon.png"))
        .map_err(|e| AppError::Tray(format!("Failed to load tray icon: {}", e)))?;

    println!("[TRAY DEBUG] Icon loaded successfully");

    let tray = TrayIconBuilder::new()
        .icon(icon)
        .icon_as_template(cfg!(target_os = "macos"))
        .menu(&menu)
        .show_menu_on_left_click(true)
        .tooltip("Global Hotkey")
        .on_menu_event(move |app, event| {
            handle_menu_event(app, event.id.as_ref());
        })
        .build(app)
        .map_err(|e| AppError::Tray(format!("Failed to build tray icon: {}", e)))?;

    println!("[TRAY DEBUG] Tray icon built successfully!");
    Ok(tray)
}

/// Build the complete menu structure
fn build_menu(app: &AppHandle, hotkeys: &[HotkeyConfig]) -> Result<Menu<Wry>, AppError> {
    // Build hotkeys submenu
    let hotkeys_submenu = build_hotkeys_submenu(app, hotkeys)?;

    // Settings item
    let settings_item = MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)
        .map_err(|e| AppError::Tray(format!("Failed to create settings item: {}", e)))?;

    // Import/Export submenu
    let import_export_submenu = build_import_export_submenu(app)?;

    // Check if autostart is enabled
    let autostart_enabled = app.autolaunch().is_enabled().unwrap_or(false);

    // Start with System checkbox
    let autostart_item = CheckMenuItem::with_id(
        app,
        "autostart",
        "Start with System",
        true,
        autostart_enabled,
        None::<&str>,
    )
    .map_err(|e| AppError::Tray(format!("Failed to create autostart item: {}", e)))?;

    // Quit item
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)
        .map_err(|e| AppError::Tray(format!("Failed to create quit item: {}", e)))?;

    // Separators
    let sep1 = PredefinedMenuItem::separator(app)
        .map_err(|e| AppError::Tray(format!("Failed to create separator: {}", e)))?;
    let sep2 = PredefinedMenuItem::separator(app)
        .map_err(|e| AppError::Tray(format!("Failed to create separator: {}", e)))?;

    // Build complete menu
    Menu::with_items(
        app,
        &[
            &hotkeys_submenu,
            &sep1,
            &settings_item,
            &import_export_submenu,
            &sep2,
            &autostart_item,
            &quit_item,
        ],
    )
    .map_err(|e| AppError::Tray(format!("Failed to create menu: {}", e)))
}

/// Build the hotkeys submenu
fn build_hotkeys_submenu(
    app: &AppHandle,
    hotkeys: &[HotkeyConfig],
) -> Result<Submenu<Wry>, AppError> {
    let enabled_hotkeys: Vec<&HotkeyConfig> = hotkeys.iter().filter(|h| h.enabled).collect();

    if enabled_hotkeys.is_empty() {
        // Show placeholder when no hotkeys configured
        let no_hotkeys = MenuItem::with_id(
            app,
            "no_hotkeys",
            "(No hotkeys configured)",
            false,
            None::<&str>,
        )
        .map_err(|e| AppError::Tray(format!("Failed to create no_hotkeys item: {}", e)))?;

        Submenu::with_items(app, "Hotkeys", true, &[&no_hotkeys])
            .map_err(|e| AppError::Tray(format!("Failed to create hotkeys submenu: {}", e)))
    } else {
        // Build menu items for each hotkey
        let mut items: Vec<MenuItem<Wry>> = Vec::new();

        for hk in enabled_hotkeys {
            let label = format!(
                "{} ({})",
                hk.name,
                hotkey::manager::format_hotkey(&hk.hotkey)
            );
            let item_id = format!("hotkey_{}", hk.id);

            let item = MenuItem::with_id(app, &item_id, &label, true, None::<&str>)
                .map_err(|e| AppError::Tray(format!("Failed to create hotkey item: {}", e)))?;

            items.push(item);
        }

        // Create submenu - we need to convert to references
        let item_refs: Vec<&dyn tauri::menu::IsMenuItem<Wry>> = items
            .iter()
            .map(|i| i as &dyn tauri::menu::IsMenuItem<Wry>)
            .collect();

        Submenu::with_items(app, "Hotkeys", true, &item_refs)
            .map_err(|e| AppError::Tray(format!("Failed to create hotkeys submenu: {}", e)))
    }
}

/// Build the Import/Export submenu
fn build_import_export_submenu(app: &AppHandle) -> Result<Submenu<Wry>, AppError> {
    let export_item =
        MenuItem::with_id(app, "export", "Export Configuration...", true, None::<&str>)
            .map_err(|e| AppError::Tray(format!("Failed to create export item: {}", e)))?;

    let import_item =
        MenuItem::with_id(app, "import", "Import Configuration...", true, None::<&str>)
            .map_err(|e| AppError::Tray(format!("Failed to create import item: {}", e)))?;

    Submenu::with_items(app, "Import/Export", true, &[&export_item, &import_item])
        .map_err(|e| AppError::Tray(format!("Failed to create import/export submenu: {}", e)))
}

/// Handle tray menu events
fn handle_menu_event(app: &AppHandle, id: &str) {
    match id {
        "settings" => {
            // Show the settings window
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "export" => {
            // Emit event to frontend for export dialog
            let _ = app.emit("tray-export", ());
        }
        "import" => {
            // Emit event to frontend for import dialog
            let _ = app.emit("tray-import", ());
        }
        "autostart" => {
            // Toggle autostart
            if let Ok(autolaunch) = app.autolaunch().is_enabled() {
                let result = if autolaunch {
                    app.autolaunch().disable()
                } else {
                    app.autolaunch().enable()
                };

                if let Err(e) = result {
                    eprintln!("Failed to toggle autostart: {}", e);
                }

                // Update the menu to reflect new state
                if let Ok(config) = crate::config::manager::load_config() {
                    let _ = update_menu(app, &config.hotkeys);
                }
            }
        }
        "quit" => {
            // Exit the application
            app.exit(0);
        }
        id if id.starts_with("hotkey_") => {
            // Execute hotkey's program
            let hotkey_id = &id[7..]; // Remove "hotkey_" prefix
            execute_hotkey_program(hotkey_id);
        }
        _ => {}
    }
}

/// Execute a program associated with a hotkey ID
fn execute_hotkey_program(id: &str) {
    use crate::config::schema::HotkeyAction;

    let registry = hotkey::manager::REGISTRY.read().unwrap();
    if let Some((_, _, config)) = registry.get(id) {
        let action = config.action.clone();
        let hotkey_name = config.name.clone();

        std::thread::spawn(move || {
            if let HotkeyAction::LaunchProgram { program } = action {
                if let Err(e) = crate::process::spawner::launch(&program) {
                    eprintln!(
                        "Failed to launch program for hotkey '{}': {}",
                        hotkey_name, e
                    );
                }
            }
            // Note: CallAi actions are not supported from tray menu
        });
    } else {
        // Hotkey not in registry (maybe disabled), try to find in config
        if let Ok(config) = crate::config::manager::load_config() {
            if let Some(hk) = config.hotkeys.iter().find(|h| h.id == id) {
                let action = hk.action.clone();
                let hotkey_name = hk.name.clone();

                std::thread::spawn(move || {
                    if let HotkeyAction::LaunchProgram { program } = action {
                        if let Err(e) = crate::process::spawner::launch(&program) {
                            eprintln!(
                                "Failed to launch program for hotkey '{}': {}",
                                hotkey_name, e
                            );
                        }
                    }
                });
            }
        }
    }
}

/// Update the tray menu with current hotkeys
pub fn update_menu(app: &AppHandle, hotkeys: &[HotkeyConfig]) -> Result<(), AppError> {
    let tray_ref = TRAY.read().unwrap();

    if let Some(tray) = tray_ref.as_ref() {
        let menu = build_menu(app, hotkeys)?;
        tray.set_menu(Some(menu))
            .map_err(|e| AppError::Tray(format!("Failed to update tray menu: {}", e)))?;
    }

    Ok(())
}

/// Check if autostart is enabled
pub fn is_autostart_enabled(app: &AppHandle) -> bool {
    app.autolaunch().is_enabled().unwrap_or(false)
}

/// Set autostart state
pub fn set_autostart(app: &AppHandle, enabled: bool) -> Result<(), AppError> {
    let result = if enabled {
        app.autolaunch().enable()
    } else {
        app.autolaunch().disable()
    };

    result.map_err(|e| AppError::Tray(format!("Failed to set autostart: {}", e)))
}

/// Update the tray icon based on current system theme
/// Call this when the system theme changes
pub fn update_tray_icon(app: &AppHandle) -> Result<(), AppError> {
    let tray_ref = TRAY.read().unwrap();

    if let Some(tray) = tray_ref.as_ref() {
        let icon_path = get_tray_icon_path();
        let full_path = get_icon_full_path(app, icon_path)?;

        let icon = load_icon_from_path(&full_path)?;

        tray.set_icon(Some(icon))
            .map_err(|e| AppError::Tray(format!("Failed to update tray icon: {}", e)))?;
    }

    Ok(())
}

// ============================================================================
// Icon State Management
// ============================================================================

/// Store the app handle for global access
pub fn set_app_handle(app: AppHandle) {
    let mut handle = APP_HANDLE.write().unwrap();
    *handle = Some(app);
}

/// Set the tray icon state (normal or active)
pub fn set_icon_state(state: TrayIconState) {
    // Update state tracking
    {
        let mut current_state = TRAY_STATE.write().unwrap();
        if *current_state == state {
            return; // No change needed
        }
        *current_state = state;
    }

    // Get app handle
    let app_handle = {
        let handle = APP_HANDLE.read().unwrap();
        handle.clone()
    };

    if let Some(app) = app_handle {
        // Run on main thread - required for macOS UI operations
        let _ = app.run_on_main_thread(move || {
            let tray_ref = TRAY.read().unwrap();
            if let Some(tray) = tray_ref.as_ref() {
                match state {
                    TrayIconState::Normal => {
                        // Use normal icon (template on macOS)
                        #[cfg(target_os = "macos")]
                        {
                            let _ = tray.set_icon_as_template(true);
                        }
                        if let Ok(icon) = Image::from_bytes(include_bytes!("../icons/tray-icon@2x.png")) {
                            let _ = tray.set_icon(Some(icon));
                        }
                    }
                    TrayIconState::Active => {
                        // Use active icon (colored, not template)
                        #[cfg(target_os = "macos")]
                        {
                            let _ = tray.set_icon_as_template(false);
                        }
                        if let Ok(icon) = Image::from_bytes(include_bytes!("../icons/32x32.png")) {
                            let _ = tray.set_icon(Some(icon));
                        }
                    }
                }
            }
        });
    }
}

/// Send a system notification
pub fn send_notification(title: &str, body: &str) {
    let app_handle = {
        let handle = APP_HANDLE.read().unwrap();
        handle.clone()
    };

    if let Some(app) = app_handle {
        // Check if notifications are enabled in settings
        let notifications_enabled = crate::config::manager::load_config()
            .map(|c| c.settings.show_tray_notifications)
            .unwrap_or(true);

        if notifications_enabled {
            if let Err(e) = app.notification()
                .builder()
                .title(title)
                .body(body)
                .show()
            {
                eprintln!("Failed to send notification: {}", e);
            }
        }
    }
}
