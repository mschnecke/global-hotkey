# Product Requirements Document: Global Hotkey Launcher

## 1. Product Overview

### 1.1 Product Name
**Global Hotkey** - A cross-platform keystroke-summoned program launcher

### 1.2 Vision
A lightweight, hidden desktop application that allows users to configure global keyboard shortcuts to instantly launch programs, including CLI applications that need to run in the background.

### 1.3 Target Platforms
- Windows 10/11
- macOS 10.15+ (Catalina and later)

## 2. User Stories

### 2.1 Core User Stories

| ID | As a... | I want to... | So that... |
|----|---------|--------------|------------|
| US-01 | Power user | Register a global hotkey to launch any program | I can quickly access my frequently used applications |
| US-02 | Developer | Launch CLI programs in hidden mode | Terminal windows don't clutter my workspace |
| US-03 | User | Edit existing hotkey configurations | I can update paths or change key combinations |
| US-04 | User | Delete hotkey configurations | I can remove shortcuts I no longer need |
| US-05 | User | Pass arguments to launched programs | I can customize how programs start |
| US-06 | User | Set working directory for programs | Programs start in the correct context |
| US-07 | User | Export my configurations | I can backup or share my setup |
| US-08 | User | Import configurations | I can restore or apply shared setups |
| US-09 | User | Access settings from system tray | I can manage hotkeys without a visible window |

## 3. Functional Requirements

### 3.1 Hotkey Management

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-01 | System shall allow users to add new hotkey-program mappings | Must Have |
| FR-02 | System shall allow users to edit existing hotkey configurations | Must Have |
| FR-03 | System shall allow users to delete hotkey configurations | Must Have |
| FR-04 | System shall validate that hotkeys don't conflict with existing system shortcuts | Must Have |
| FR-05 | System shall validate that hotkeys don't conflict with other configured hotkeys | Must Have |
| FR-06 | System shall display all configured hotkeys in a flat list | Must Have |

### 3.2 Program Launching

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-07 | System shall launch programs when their registered hotkey is pressed | Must Have |
| FR-08 | System shall support launching GUI applications | Must Have |
| FR-09 | System shall support launching CLI applications in hidden/background mode | Must Have |
| FR-10 | System shall support passing command-line arguments to launched programs | Must Have |
| FR-11 | System shall support configuring working directory for each program | Must Have |
| FR-12 | System shall provide file browser for selecting program executables | Should Have |

### 3.3 Configuration Management

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-13 | System shall export all configurations to a JSON file | Must Have |
| FR-14 | System shall import configurations from a JSON file | Must Have |
| FR-15 | System shall merge imported configs with existing (with conflict resolution) | Should Have |
| FR-16 | System shall persist configurations across application restarts | Must Have |

### 3.4 System Tray Integration

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-17 | Application shall run as a hidden/background process | Must Have |
| FR-18 | Application shall display icon in system tray | Must Have |
| FR-19 | System tray menu shall provide access to settings window | Must Have |
| FR-20 | System tray menu shall provide quick list of configured hotkeys | Should Have |
| FR-21 | System tray menu shall provide quit option | Must Have |
| FR-22 | Application shall start minimized to tray on launch | Must Have |

### 3.5 Startup Behavior

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-23 | Application shall optionally start with system boot | Should Have |
| FR-24 | Application shall register all hotkeys on startup | Must Have |

## 4. Non-Functional Requirements

### 4.1 Performance

| ID | Requirement |
|----|-------------|
| NFR-01 | Hotkey response time shall be < 100ms from keypress to program launch initiation |
| NFR-02 | Application memory footprint shall be < 50MB when idle |
| NFR-03 | Application shall have minimal CPU usage when idle (< 1%) |

### 4.2 Reliability

| ID | Requirement |
|----|-------------|
| NFR-04 | Application shall gracefully handle invalid program paths |
| NFR-05 | Application shall recover from crashes and restore hotkey registrations |
| NFR-06 | Configuration file corruption shall not prevent application startup |

### 4.3 Usability

| ID | Requirement |
|----|-------------|
| NFR-07 | Settings UI shall be intuitive and require no documentation for basic operations |
| NFR-08 | Hotkey recording shall use press-to-record interaction pattern |
| NFR-09 | Error messages shall be clear and actionable |

### 4.4 Security

| ID | Requirement |
|----|-------------|
| NFR-10 | Application shall not require elevated/admin privileges for normal operation |
| NFR-11 | Configuration files shall be stored in user-specific directories |

## 5. Technical Architecture

### 5.1 Technology Stack

| Layer | Technology | Rationale |
|-------|------------|-----------|
| Framework | Tauri 2.x | Cross-platform, lightweight, Rust backend for performance |
| Frontend | Svelte + TypeScript | Lightweight, reactive, excellent Tauri integration |
| Styling | Tailwind CSS | Utility-first, consistent styling |
| State Management | Svelte Stores | Native Svelte reactivity, no external dependencies |
| Backend | Rust | System integration, global hotkey handling |
| Config Storage | JSON | Human-readable, easy import/export |

### 5.2 Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    System Tray                          │
│                  (Right-click menu)                     │
└─────────────────────┬───────────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────────┐
│                  Tauri Core (Rust)                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐  │
│  │   Global    │  │   Process   │  │   Config        │  │
│  │   Hotkey    │  │   Spawner   │  │   Manager       │  │
│  │   Manager   │  │             │  │                 │  │
│  └─────────────┘  └─────────────┘  └─────────────────┘  │
└─────────────────────┬───────────────────────────────────┘
                      │ IPC Commands
┌─────────────────────▼───────────────────────────────────┐
│               Svelte Frontend                           │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐  │
│  │  Settings   │  │   Hotkey    │  │   Import/       │  │
│  │  Window     │  │   List      │  │   Export        │  │
│  └─────────────┘  └─────────────┘  └─────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

### 5.3 Component Responsibilities

#### Rust Backend (src-tauri/)
- **Global Hotkey Manager**: Register/unregister system-wide hotkeys, handle key events
- **Process Spawner**: Launch programs with arguments, handle hidden mode for CLI apps
- **Config Manager**: Load/save/validate configuration, handle import/export

#### Svelte Frontend (src/)
- **Settings Window**: Main UI for managing hotkey configurations
- **Hotkey List**: Display and manage all configured hotkeys
- **Import/Export**: File dialogs and merge conflict resolution UI

## 6. Data Model

### 6.1 Configuration Schema

```json
{
  "version": "1.0",
  "hotkeys": [
    {
      "id": "uuid-string",
      "name": "Launch VS Code",
      "hotkey": {
        "modifiers": ["ctrl", "alt"],
        "key": "c"
      },
      "program": {
        "path": "C:\\Program Files\\Microsoft VS Code\\Code.exe",
        "arguments": ["--new-window"],
        "workingDirectory": "C:\\Projects",
        "hidden": false
      },
      "enabled": true,
      "createdAt": "2024-01-15T10:30:00Z",
      "updatedAt": "2024-01-15T10:30:00Z"
    }
  ],
  "settings": {
    "startWithSystem": true,
    "showTrayNotifications": true
  }
}
```

### 6.2 Hotkey Object Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | string (UUID) | Yes | Unique identifier |
| name | string | Yes | User-friendly display name |
| hotkey.modifiers | string[] | Yes | Modifier keys: ctrl, alt, shift, meta/cmd |
| hotkey.key | string | Yes | Primary key (a-z, 0-9, F1-F12, etc.) |
| program.path | string | Yes | Absolute path to executable |
| program.arguments | string[] | No | Command-line arguments |
| program.workingDirectory | string | No | Working directory for process |
| program.hidden | boolean | No | Launch in hidden mode (default: false) |
| enabled | boolean | Yes | Whether hotkey is active |
| createdAt | string (ISO) | Yes | Creation timestamp |
| updatedAt | string (ISO) | Yes | Last modification timestamp |

## 7. UI/UX Specifications

### 7.1 System Tray Menu Structure

```
[App Icon]
├── Hotkeys
│   ├── Launch VS Code (Ctrl+Alt+C)
│   ├── Open Terminal (Ctrl+Alt+T)
│   └── ... (list of all hotkeys)
├── ─────────────
├── Settings...
├── Import/Export
│   ├── Export Configuration...
│   └── Import Configuration...
├── ─────────────
├── Start with System [✓]
└── Quit
```

### 7.2 Settings Window

**Hotkey List View:**
- Table/list showing: Name, Hotkey combination, Program path, Status (enabled/disabled)
- Actions per row: Edit, Delete, Enable/Disable toggle
- Add button for new hotkey

**Add/Edit Hotkey Dialog:**
- Name input field
- Hotkey recorder (click to record, press keys)
- Program path with file browser button
- Arguments input (comma or space separated)
- Working directory with folder browser button
- Hidden mode checkbox (for CLI programs)
- Save/Cancel buttons

### 7.3 Visual Design Guidelines
- Follow system theme (light/dark mode)
- Minimal, clean interface
- Use native OS dialog styles where possible
- Keyboard navigable

## 8. Configuration Storage

### 8.1 Storage Locations

Use the user's home directory as the root for configuration files:

| Platform | Config Directory |
|----------|------------------|
| Windows | `~/global-hotkey/` |
| macOS | `~/global-hotkey/` |

Resolve `~` to the user's home directory via Tauri's path API: `path.homeDir()` (Rust) or `@tauri-apps/api/path` (frontend).

### 8.2 Files

All files are stored relative to `~/global-hotkey/`:

| File | Path | Purpose |
|------|------|---------|
| `config.json` | `~/global-hotkey/config.json` | Main configuration file |
| `config.backup.json` | `~/global-hotkey/config.backup.json` | Automatic backup before modifications |

## 9. Platform-Specific Considerations

### 9.1 Windows
- Use Windows API for global hotkey registration
- Handle UAC for programs requiring elevation
- Support both `.exe` and `.bat`/`.cmd` files
- Hidden mode uses `CREATE_NO_WINDOW` process flag

### 9.2 macOS
- Require Accessibility permissions for global hotkeys
- Guide user through System Preferences for permissions
- Support `.app` bundles and Unix executables
- Hidden mode uses background process launching

## 10. Future Considerations (Out of Scope for v1.0)

- Categories/folders for organizing hotkeys
- Hotkey sequences (chord combinations)
- Conditional launching (based on active window/app)
- Plugin system for custom actions
- Cloud sync for configurations
- Multiple configuration profiles
- Spotlight-style search launcher

## 11. Success Metrics

| Metric | Target |
|--------|--------|
| Hotkey registration success rate | > 99% |
| Program launch success rate | > 99% |
| Application crash rate | < 0.1% |
| Memory usage (idle) | < 50MB |

## 12. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2024-12-12 | - | Initial PRD |
