# Global Hotkey - Implementation Plan

## Overview

This document outlines the implementation plan for Global Hotkey, a cross-platform keystroke-summoned program launcher.

### Technology Stack

| Layer | Technology | Rationale |
|-------|------------|-----------|
| Framework | Tauri 2.x | Cross-platform, lightweight, Rust backend |
| Frontend | Svelte + TypeScript | Lightweight, reactive, excellent Tauri integration |
| Styling | Tailwind CSS | Utility-first, consistent styling |
| State | Svelte Stores | Native reactivity, no external dependencies |
| Backend | Rust | System integration, performance |
| Config | JSON | Human-readable, easy import/export |

### Target Platforms

- Windows 10/11
- macOS 10.15+ (Apple Silicon only)

---

## Phase 1: Project Scaffolding

**Goal**: Set up the complete development environment and project structure

### Tasks

1. Initialize Tauri 2.x project with Svelte template
2. Configure TypeScript for frontend
3. Set up Tailwind CSS
4. Configure ESLint + Prettier
5. Set up Husky + lint-staged for git hooks
6. Create initial folder structure

### Files to Create

| File | Purpose |
|------|---------|
| `package.json` | Node.js dependencies and scripts |
| `package-lock.json` | Locked dependency versions |
| `tsconfig.json` | TypeScript configuration |
| `svelte.config.js` | Svelte configuration |
| `vite.config.ts` | Vite bundler configuration |
| `tailwind.config.js` | Tailwind CSS configuration |
| `postcss.config.js` | PostCSS for Tailwind |
| `.eslintrc.cjs` | ESLint rules |
| `.prettierrc` | Prettier formatting rules |
| `.husky/pre-commit` | Git pre-commit hook |
| `src-tauri/Cargo.toml` | Rust dependencies |
| `src-tauri/tauri.conf.json` | Tauri app configuration |
| `src-tauri/build.rs` | Tauri build script |
| `src-tauri/src/main.rs` | Rust entry point |
| `src-tauri/src/lib.rs` | Rust library module |

### Directory Structure

```
global-hotkey/
├── .husky/
│   └── pre-commit
├── src/
│   ├── app.css
│   ├── App.svelte
│   ├── main.ts
│   ├── vite-env.d.ts
│   ├── components/
│   ├── lib/
│   └── stores/
├── src-tauri/
│   ├── icons/
│   ├── src/
│   │   ├── main.rs
│   │   └── lib.rs
│   ├── Cargo.toml
│   ├── build.rs
│   └── tauri.conf.json
├── package.json
├── tsconfig.json
├── vite.config.ts
├── svelte.config.js
├── tailwind.config.js
└── postcss.config.js
```

### Key Dependencies

**Node.js (package.json)**:
- `@tauri-apps/api` ^2.0.0
- `svelte` ^5.0.0
- `typescript` ^5.0.0
- `vite` ^6.0.0
- `@sveltejs/vite-plugin-svelte` ^5.0.0
- `tailwindcss` ^3.4.0
- `eslint` ^9.0.0
- `prettier` ^3.0.0
- `husky` ^9.0.0
- `lint-staged` ^16.0.0

**Rust (Cargo.toml)**:
- `tauri` 2.x with features: tray-icon, shell-open
- `serde` + `serde_json` - JSON serialization
- `global-hotkey` - Cross-platform hotkey handling
- `uuid` - Unique identifiers
- `chrono` - Timestamps
- `dirs` - Home directory resolution

---

## Phase 2: Rust Backend Core

**Goal**: Implement the three core Rust modules

### 2.1 Config Manager

**Location**: `src-tauri/src/config/`

**Files**:
- `mod.rs` - Module exports
- `manager.rs` - ConfigManager struct and methods
- `schema.rs` - Configuration data structures
- `validation.rs` - Config validation logic

**Responsibilities**:
- Load configuration from `~/global-hotkey/config.json`
- Save configuration with automatic backup
- Validate configuration schema
- Handle import/export operations
- Recover from corrupted config files

**Data Structures**:

```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub version: String,
    pub hotkeys: Vec<HotkeyConfig>,
    pub settings: AppSettings,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HotkeyConfig {
    pub id: String,
    pub name: String,
    pub hotkey: HotkeyBinding,
    pub program: ProgramConfig,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HotkeyBinding {
    pub modifiers: Vec<String>,
    pub key: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProgramConfig {
    pub path: String,
    pub arguments: Vec<String>,
    pub working_directory: Option<String>,
    pub hidden: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub start_with_system: bool,
    pub show_tray_notifications: bool,
}
```

**Tauri Commands**:
- `get_config() -> AppConfig`
- `save_config(config: AppConfig) -> Result<(), String>`
- `export_config(path: String) -> Result<(), String>`
- `import_config(path: String) -> Result<AppConfig, String>`

### 2.2 Global Hotkey Manager

**Location**: `src-tauri/src/hotkey/`

**Files**:
- `mod.rs` - Module exports
- `manager.rs` - HotkeyManager struct
- `handler.rs` - Key event handling
- `conflict.rs` - Conflict detection

**Responsibilities**:
- Register system-wide hotkeys
- Unregister hotkeys on removal/disable
- Handle key press events
- Dispatch to process spawner
- Detect conflicts with system/existing hotkeys

**Key Implementation**:
- Use `global-hotkey` crate for cross-platform support
- Maintain registry of active hotkeys
- Event loop integration with Tauri

**Tauri Commands**:
- `register_hotkey(config: HotkeyConfig) -> Result<(), String>`
- `unregister_hotkey(id: String) -> Result<(), String>`
- `check_conflict(binding: HotkeyBinding) -> Result<bool, String>`
- `get_registered_hotkeys() -> Vec<String>`

### 2.3 Process Spawner

**Location**: `src-tauri/src/process/`

**Files**:
- `mod.rs` - Module exports
- `spawner.rs` - ProcessSpawner struct
- `platform.rs` - Platform-specific code

**Responsibilities**:
- Launch GUI applications
- Launch CLI apps in hidden mode
- Pass command-line arguments
- Set working directory
- Handle launch errors gracefully

**Platform-Specific**:

| Platform | Hidden Mode | Notes |
|----------|-------------|-------|
| Windows | `CREATE_NO_WINDOW` flag | Use `std::os::windows::process::CommandExt` |
| macOS | Background launch | Use `open -g` or NSWorkspace |

**Tauri Commands**:
- `launch_program(config: ProgramConfig) -> Result<(), String>`
- `validate_program_path(path: String) -> Result<bool, String>`

---

## Phase 3: System Tray Integration

**Goal**: Implement system tray icon and menu

### Files

- `src-tauri/src/tray.rs` - Tray logic
- `src-tauri/icons/` - Tray icons (32x32, various formats)

### Menu Structure

```
[App Icon]
├── Hotkeys
│   ├── Launch VS Code (Ctrl+Alt+C)
│   ├── Open Terminal (Ctrl+Alt+T)
│   └── ...
├── ─────────────
├── Settings...
├── Import/Export
│   ├── Export Configuration...
│   └── Import Configuration...
├── ─────────────
├── Start with System [✓]
└── Quit
```

### Implementation

1. Configure tray in `tauri.conf.json`
2. Build dynamic menu from hotkey list
3. Handle menu item clicks via Tauri events
4. Update menu when configuration changes
5. Show/hide settings window from tray

### Tauri Configuration

```json
{
  "app": {
    "trayIcon": {
      "iconPath": "icons/icon.png",
      "iconAsTemplate": true
    }
  }
}
```

---

## Phase 4: Svelte Frontend

**Goal**: Build the settings window UI

### Directory Structure

```
src/
├── App.svelte           # Main app component
├── main.ts              # Entry point
├── app.css              # Global styles + Tailwind
├── vite-env.d.ts        # Vite type declarations
├── components/
│   ├── HotkeyList.svelte      # Table of hotkeys
│   ├── HotkeyDialog.svelte    # Add/Edit modal
│   ├── HotkeyRecorder.svelte  # Key capture component
│   ├── FileBrowser.svelte     # File picker button
│   └── ConfirmDialog.svelte   # Delete confirmation
├── lib/
│   ├── commands.ts      # Tauri command wrappers
│   ├── types.ts         # TypeScript interfaces
│   └── utils.ts         # Helper functions
└── stores/
    ├── hotkeys.ts       # Hotkey configuration store
    └── settings.ts      # App settings store
```

### Components

#### HotkeyList.svelte
- Table displaying: Name, Hotkey, Program, Status
- Row actions: Edit, Delete, Enable/Disable toggle
- Add button for new hotkey
- Empty state when no hotkeys configured

#### HotkeyDialog.svelte
- Modal dialog for add/edit
- Fields:
  - Name (text input)
  - Hotkey (HotkeyRecorder component)
  - Program path (FileBrowser component)
  - Arguments (text input)
  - Working directory (FileBrowser component)
  - Hidden mode (checkbox)
- Validation before save
- Save/Cancel buttons

#### HotkeyRecorder.svelte
- Click to activate recording mode
- Capture modifier keys + primary key
- Display formatted hotkey string
- Clear button to reset

#### FileBrowser.svelte
- Button that opens native file dialog
- Uses Tauri's `dialog.open()`
- Filter by executable types per platform

### TypeScript Interfaces

```typescript
interface HotkeyConfig {
  id: string;
  name: string;
  hotkey: {
    modifiers: string[];
    key: string;
  };
  program: {
    path: string;
    arguments: string[];
    workingDirectory?: string;
    hidden: boolean;
  };
  enabled: boolean;
  createdAt: string;
  updatedAt: string;
}

interface AppSettings {
  startWithSystem: boolean;
  showTrayNotifications: boolean;
}

interface AppConfig {
  version: string;
  hotkeys: HotkeyConfig[];
  settings: AppSettings;
}
```

### Svelte Stores

```typescript
// stores/hotkeys.ts
import { writable } from 'svelte/store';
import type { HotkeyConfig } from '$lib/types';

export const hotkeys = writable<HotkeyConfig[]>([]);

export async function loadHotkeys() { /* ... */ }
export async function addHotkey(config: HotkeyConfig) { /* ... */ }
export async function updateHotkey(config: HotkeyConfig) { /* ... */ }
export async function deleteHotkey(id: string) { /* ... */ }
export async function toggleHotkey(id: string) { /* ... */ }
```

---

## Phase 5: Import/Export & Backup

**Goal**: Configuration portability and reliability

### Features

1. **Export**: Save all hotkeys to user-selected JSON file
2. **Import**: Load hotkeys from JSON file with validation
3. **Merge**: Option to merge or replace existing config
4. **Backup**: Auto-create `config.backup.json` before changes
5. **Recovery**: Load from backup if main config corrupted

### Import Conflict Resolution

When importing, if hotkey ID or key binding already exists:
1. Show conflict dialog
2. Options: Skip, Replace, Keep Both (rename)
3. Apply user choice per-item or for all

### File Dialogs

Use Tauri's native dialog APIs:
- `dialog.save()` for export
- `dialog.open()` for import
- Filter: `*.json` files

---

## Phase 6: Platform-Specific Features

### Windows

| Feature | Implementation |
|---------|----------------|
| Hidden CLI | `CREATE_NO_WINDOW` process creation flag |
| Auto-start | Registry key or Startup folder shortcut |
| Executable types | `.exe`, `.bat`, `.cmd`, `.ps1` |
| UAC | Detect and warn for elevated programs |

**Registry for Auto-start**:
```
HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run
```

### macOS

| Feature | Implementation |
|---------|----------------|
| Hidden CLI | Launch as background process |
| Auto-start | LaunchAgent plist in `~/Library/LaunchAgents/` |
| Executable types | `.app` bundles, Unix executables |
| Permissions | Request Accessibility permission |

**Accessibility Permission**:
- Required for global hotkey capture
- Guide user through System Preferences
- Check permission status on startup

**LaunchAgent for Auto-start**:
```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "...">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>com.globalhotkey.app</string>
  <key>ProgramArguments</key>
  <array>
    <string>/Applications/Global Hotkey.app/Contents/MacOS/Global Hotkey</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
</dict>
</plist>
```

---

## Phase 7: CI/CD & Deployment

### GitHub Actions

#### ci.yml (Continuous Integration)

**Triggers**: Push to `main`/`develop`, PRs to `main`

**Jobs**:
1. `lint-and-test` (Ubuntu)
   - Install Node.js dependencies
   - Run ESLint
   - Run Prettier check
   - Run frontend tests
2. `build` (Matrix: macOS + Windows)
   - Install dependencies
   - Install Rust toolchain
   - Build frontend
   - Build Tauri app

#### release.yml (Release Automation)

**Triggers**: Tag push `v*`, manual dispatch

**Jobs**:
1. `bump-version` - Update version in config files
2. `create-release` - Create GitHub draft release
3. `build-tauri` - Build installers (parallel macOS/Windows)
4. `publish-release` - Publish release
5. `update-homebrew` - Update Homebrew tap
6. `update-chocolatey` - Update Chocolatey package

### Git Hooks (Husky + lint-staged)

**Pre-commit**:
```bash
npx lint-staged
```

**lint-staged config**:
```json
{
  "src/**/*.{ts,svelte}": ["prettier --write", "eslint --fix --max-warnings 0"],
  "src/**/*.css": ["prettier --write"]
}
```

### Package Distribution

#### Chocolatey (Windows)

**Location**: `packages/chocolatey/`

**Files**:
- `global-hotkey.nuspec` - Package manifest
- `tools/chocolateyinstall.ps1` - Install script
- `tools/chocolateyuninstall.ps1` - Uninstall script

**Install**: `choco install global-hotkey`

#### Homebrew (macOS)

**Tap Repository**: `homebrew-global-hotkey`

**Cask file**: `Casks/global-hotkey.rb`

**Install**:
```bash
brew tap mschnecke/global-hotkey
brew install --cask global-hotkey
```

---

## Implementation Order

| Order | Phase | Description | Dependencies |
|-------|-------|-------------|--------------|
| 1 | 1 | Project Scaffolding | None |
| 2 | 2.1 | Config Manager | Phase 1 |
| 3 | 4 (partial) | Basic Frontend Shell | Phase 1 |
| 4 | 2.2 | Hotkey Manager | Phase 2.1 |
| 5 | 2.3 | Process Spawner | Phase 2.1 |
| 6 | 4 (complete) | Full Frontend | Phases 2.x |
| 7 | 3 | System Tray | Phases 2.x, 4 |
| 8 | 5 | Import/Export | Phases 2.1, 4 |
| 9 | 6 | Platform Features | All core phases |
| 10 | 7 | CI/CD | All phases |

---

## Success Criteria

| Metric | Target |
|--------|--------|
| Hotkey registration success | > 99% |
| Program launch success | > 99% |
| Application crash rate | < 0.1% |
| Memory usage (idle) | < 50MB |
| Hotkey response time | < 100ms |
| CPU usage (idle) | < 1% |

---

## Agents for Implementation

Six specialized agents are defined in `.claude/agents/`:

1. **tauri-scaffolding** - Project initialization and setup
2. **rust-backend** - Rust module development
3. **svelte-frontend** - UI component development
4. **system-tray** - Tray functionality
5. **ci-cd-setup** - GitHub Actions and deployment
6. **platform-integration** - OS-specific implementations

See agent files for detailed specifications and usage triggers.
