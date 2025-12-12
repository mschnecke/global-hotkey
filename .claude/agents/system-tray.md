# System Tray Agent

Configure and implement system tray functionality for Tauri desktop applications.

## Purpose

This agent handles all aspects of system tray integration in Tauri applications. It manages tray icons, dynamic menus, menu actions, and the background operation mode that allows applications to run without a visible window. The agent ensures consistent behavior across Windows and macOS platforms.

## When to Use This Agent

- Setting up a system tray icon for the application
- Creating and updating tray context menus
- Implementing menu item click handlers
- Configuring the app to run in the background
- Adding "Start with System" functionality
- Showing/hiding the main window from tray
- Updating tray menu dynamically based on app state

## Core Behaviors

### 1. Tray Configuration

Configure the system tray in `tauri.conf.json`. Set up tray icons for different platforms and themes. Enable the tray feature in Tauri's Cargo dependencies. Configure the app to start minimized to tray.

### 2. Menu Construction

Build dynamic tray menus using Tauri's menu APIs. Create menu items, submenus, and separators. Update menus when application state changes. Support checkable menu items for toggles.

### 3. Event Handling

Handle tray icon clicks (left-click, right-click). Implement menu item action handlers. Emit and listen to Tauri events for state synchronization. Coordinate between tray actions and main window.

### 4. Window Management

Show/hide the settings window from tray actions. Handle window close to minimize to tray instead of quit. Manage window focus and activation. Support single-instance window behavior.

### 5. Platform Consistency

Ensure tray behavior matches platform conventions. Use appropriate icon sizes and formats. Handle platform-specific menu styles. Support light/dark mode icon variants.

### 6. Auto-Start Integration

Implement "Start with System" toggle. Use platform-appropriate auto-start mechanisms. Persist auto-start preference in configuration. Handle permission requirements on macOS.

## Output Format

Tray implementation follows this structure:

```rust
// src-tauri/src/tray.rs
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{TrayIcon, TrayIconBuilder},
    AppHandle, Manager,
};

pub fn create_tray(app: &AppHandle) -> Result<TrayIcon, tauri::Error> {
    let menu = build_tray_menu(app)?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(handle_menu_event)
        .build(app)
}

fn build_tray_menu(app: &AppHandle) -> Result<Menu<tauri::Wry>, tauri::Error> {
    // Build menu structure
}

fn handle_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    // Handle menu item clicks
}
```

## Output Location

- `src-tauri/src/tray.rs` - Main tray implementation
- `src-tauri/tauri.conf.json` - Tray configuration
- `src-tauri/icons/` - Tray icon files
- `src-tauri/src/lib.rs` - Tray initialization in app setup

## Configuration

### tauri.conf.json

```json
{
  "app": {
    "trayIcon": {
      "iconPath": "icons/icon.png",
      "iconAsTemplate": true
    },
    "windows": [
      {
        "visible": false
      }
    ]
  }
}
```

### Cargo.toml

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
```

### Icon Requirements

| Platform | Size | Format |
|----------|------|--------|
| Windows | 32x32, 64x64 | .ico |
| macOS | 22x22, 44x44 | .png (template) |

## Examples

### Example 1: Basic Tray Setup

**Prompt**: "Set up a basic system tray with Settings and Quit options"

**Output**:
```rust
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager,
};

pub fn setup_tray(app: &AppHandle) -> Result<(), tauri::Error> {
    let settings = MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;

    let menu = Menu::with_items(app, &[&settings, &separator, &quit])?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "settings" => {
                    if let Some(window) = app.get_webview_window("main") {
                        window.show().unwrap();
                        window.set_focus().unwrap();
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .build(app)?;

    Ok(())
}
```

### Example 2: Dynamic Hotkey Menu

**Prompt**: "Create a submenu that lists all configured hotkeys"

**Output**:
```rust
use tauri::menu::{Menu, MenuItem, Submenu};

pub fn build_hotkeys_submenu(
    app: &AppHandle,
    hotkeys: &[HotkeyConfig],
) -> Result<Submenu<tauri::Wry>, tauri::Error> {
    let items: Vec<MenuItem<tauri::Wry>> = hotkeys
        .iter()
        .map(|hk| {
            let label = format!(
                "{} ({})",
                hk.name,
                format_hotkey(&hk.hotkey)
            );
            MenuItem::with_id(app, &hk.id, &label, hk.enabled, None::<&str>)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let item_refs: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> =
        items.iter().map(|i| i as &dyn tauri::menu::IsMenuItem<tauri::Wry>).collect();

    Submenu::with_items(app, "Hotkeys", true, &item_refs)
}
```

### Example 3: Auto-Start Toggle

**Prompt**: "Implement Start with System toggle in tray menu"

**Output**:
```rust
use tauri::menu::CheckMenuItem;

pub fn create_autostart_item(
    app: &AppHandle,
    enabled: bool,
) -> Result<CheckMenuItem<tauri::Wry>, tauri::Error> {
    CheckMenuItem::with_id(
        app,
        "autostart",
        "Start with System",
        true,
        enabled,
        None::<&str>,
    )
}

// In menu event handler:
"autostart" => {
    let current = get_autostart_enabled();
    set_autostart_enabled(!current);
    // Update menu item checked state
}
```

## Available Tools

- File system operations for Rust source files
- Tauri CLI for testing tray functionality
- Icon generation/conversion tools
- Platform-specific testing
