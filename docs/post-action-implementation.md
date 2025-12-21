# Post-Actions Implementation Plan

Detailed implementation guide for the Post-Actions feature.

---

## Phase 1: Dependencies & Data Model

### 1.1 Add Dependencies

**File**: `src-tauri/Cargo.toml`

Add after line 28 (`which = "7"`):

```toml
enigo = "0.3"
```

### 1.2 Update Rust Schema

**File**: `src-tauri/src/config/schema.rs`

Add the following types after `AppSettings` (line 68):

```rust
/// Trigger timing for post-actions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
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
```

Update `HotkeyConfig` struct (line 24-34) to add post_actions field:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyConfig {
    pub id: String,
    pub name: String,
    pub hotkey: HotkeyBinding,
    pub program: ProgramConfig,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub post_actions: PostActionsConfig,
}
```

### 1.3 Update TypeScript Types

**File**: `src/lib/types.ts`

Add after `AppConfig` interface (line 36):

```typescript
// Post-Action Types
export type PostActionTrigger = { type: 'onExit' } | { type: 'afterDelay'; delayMs: number };

export interface Keystroke {
  modifiers: string[];
  key: string;
}

export type PostActionType =
  | { type: 'pasteClipboard' }
  | { type: 'simulateKeystroke'; keystroke: Keystroke }
  | { type: 'delay'; delayMs: number };

export interface PostAction {
  id: string;
  actionType: PostActionType;
  enabled: boolean;
}

export interface PostActionsConfig {
  enabled: boolean;
  trigger: PostActionTrigger;
  actions: PostAction[];
}
```

Update `HotkeyConfig` interface to include postActions:

```typescript
export interface HotkeyConfig {
  id: string;
  name: string;
  hotkey: HotkeyBinding;
  program: ProgramConfig;
  enabled: boolean;
  createdAt: string;
  updatedAt: string;
  postActions?: PostActionsConfig; // NEW
}
```

---

## Phase 2: Backend - Post-Action Module

### 2.1 Create Module Structure

**File**: `src-tauri/src/postaction/mod.rs` (NEW)

```rust
//! Post-action execution module

mod executor;
mod input;

pub use executor::execute_with_post_actions;
```

### 2.2 Input Simulation

**File**: `src-tauri/src/postaction/input.rs` (NEW)

```rust
//! Keystroke simulation using enigo

use enigo::{Direction, Enigo, Key, Keyboard, Settings};

use crate::config::schema::Keystroke;
use crate::error::AppError;

pub struct InputSimulator {
    enigo: Enigo,
}

impl InputSimulator {
    pub fn new() -> Result<Self, AppError> {
        let enigo = Enigo::new(&Settings::default())
            .map_err(|e| AppError::PostAction(format!("Failed to create input simulator: {}", e)))?;
        Ok(Self { enigo })
    }

    /// Simulate a paste operation (Ctrl+V on Windows, Cmd+V on macOS)
    pub fn paste(&mut self) -> Result<(), AppError> {
        #[cfg(target_os = "macos")]
        let modifier = Key::Meta;

        #[cfg(not(target_os = "macos"))]
        let modifier = Key::Control;

        self.enigo.key(modifier, Direction::Press)
            .map_err(|e| AppError::PostAction(format!("Failed to press modifier: {}", e)))?;
        self.enigo.key(Key::Unicode('v'), Direction::Click)
            .map_err(|e| AppError::PostAction(format!("Failed to press V: {}", e)))?;
        self.enigo.key(modifier, Direction::Release)
            .map_err(|e| AppError::PostAction(format!("Failed to release modifier: {}", e)))?;

        Ok(())
    }

    /// Simulate a keystroke with modifiers
    pub fn simulate_keystroke(&mut self, keystroke: &Keystroke) -> Result<(), AppError> {
        // Press all modifiers
        for modifier in &keystroke.modifiers {
            let key = self.map_modifier(modifier)?;
            self.enigo.key(key, Direction::Press)
                .map_err(|e| AppError::PostAction(format!("Failed to press modifier: {}", e)))?;
        }

        // Press and release the key
        let key = self.map_key(&keystroke.key)?;
        self.enigo.key(key, Direction::Click)
            .map_err(|e| AppError::PostAction(format!("Failed to press key: {}", e)))?;

        // Release all modifiers (in reverse order)
        for modifier in keystroke.modifiers.iter().rev() {
            let key = self.map_modifier(modifier)?;
            self.enigo.key(key, Direction::Release)
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
```

### 2.3 Post-Action Executor

**File**: `src-tauri/src/postaction/executor.rs` (NEW)

```rust
//! Post-action execution logic

use std::thread;
use std::time::Duration;

use crate::config::schema::{PostAction, PostActionsConfig, PostActionTrigger, PostActionType, ProgramConfig};
use crate::error::AppError;
use crate::process;

use super::input::InputSimulator;

/// Execute a program with post-actions
pub fn execute_with_post_actions(
    program_config: &ProgramConfig,
    post_actions: &PostActionsConfig,
    hotkey_name: &str,
) -> Result<(), AppError> {
    // If no post-actions enabled, just launch normally
    if !post_actions.enabled || post_actions.actions.is_empty() {
        return process::spawner::launch(program_config);
    }

    match &post_actions.trigger {
        PostActionTrigger::OnExit => {
            // Launch and wait for process to exit
            let exit_code = launch_and_wait(program_config)?;

            if exit_code == 0 {
                execute_actions(&post_actions.actions, hotkey_name)?;
            } else {
                eprintln!(
                    "Hotkey '{}': process exited with code {}, skipping post-actions",
                    hotkey_name, exit_code
                );
            }
        }
        PostActionTrigger::AfterDelay { delay_ms } => {
            // Launch process (don't wait)
            process::spawner::launch(program_config)?;

            // Wait for delay then execute post-actions
            thread::sleep(Duration::from_millis(*delay_ms));
            execute_actions(&post_actions.actions, hotkey_name)?;
        }
    }

    Ok(())
}

/// Launch a program and wait for it to exit, returning the exit code
fn launch_and_wait(config: &ProgramConfig) -> Result<i32, AppError> {
    use std::process::Command;
    use crate::process::platform;

    let resolved_path = process::spawner::resolve_program(&config.path)
        .ok_or_else(|| AppError::Process(format!("Program not found: {}", config.path)))?;

    let mut command = Command::new(&resolved_path);

    // Add arguments
    for arg in &config.arguments {
        if !arg.is_empty() {
            command.arg(arg);
        }
    }

    // Set working directory
    if let Some(ref working_dir) = config.working_directory {
        if !working_dir.is_empty() {
            let dir = std::path::Path::new(working_dir);
            if dir.exists() && dir.is_dir() {
                command.current_dir(dir);
            }
        }
    }

    // Apply hidden mode if configured
    if config.hidden {
        platform::configure_hidden(&mut command);
    }

    // NOTE: Don't detach - we need to wait for this process
    let output = command.output().map_err(|e| {
        AppError::Process(format!("Failed to launch program '{}': {}", config.path, e))
    })?;

    Ok(output.status.code().unwrap_or(-1))
}

/// Execute a sequence of post-actions
fn execute_actions(actions: &[PostAction], hotkey_name: &str) -> Result<(), AppError> {
    let mut simulator = InputSimulator::new()?;

    for action in actions {
        if !action.enabled {
            continue;
        }

        // Small delay before simulating input to ensure window focus is stable
        thread::sleep(Duration::from_millis(50));

        match &action.action_type {
            PostActionType::PasteClipboard => {
                simulator.paste()?;
            }
            PostActionType::SimulateKeystroke { keystroke } => {
                simulator.simulate_keystroke(keystroke)?;
            }
            PostActionType::Delay { delay_ms } => {
                thread::sleep(Duration::from_millis(*delay_ms));
            }
        }
    }

    eprintln!("Hotkey '{}': post-actions completed", hotkey_name);
    Ok(())
}
```

### 2.4 Update Error Types

**File**: `src-tauri/src/error.rs`

Add new error variant:

```rust
#[derive(Error, Debug)]
pub enum AppError {
    // ... existing variants ...

    #[error("Post-action error: {0}")]
    PostAction(String),
}
```

### 2.5 Register Module

**File**: `src-tauri/src/lib.rs`

Add module declaration after existing mods:

```rust
mod postaction;
```

---

## Phase 3: Integrate with Hotkey Handler

### 3.1 Update Event Handler

**File**: `src-tauri/src/hotkey/manager.rs`

Update the `handle_event` function (lines 71-94):

```rust
/// Handle a hotkey event
fn handle_event(event: GlobalHotKeyEvent) {
    if event.state != HotKeyState::Pressed {
        return;
    }

    let registry = REGISTRY.read().unwrap();
    for (_, (hotkey_id, _, config)) in registry.iter() {
        if *hotkey_id == event.id {
            let program_config = config.program.clone();
            let post_actions = config.post_actions.clone();
            let hotkey_name = config.name.clone();

            // Spawn in a separate thread to avoid blocking the event loop
            std::thread::spawn(move || {
                // Check if post-actions are enabled
                if post_actions.enabled && !post_actions.actions.is_empty() {
                    if let Err(e) = crate::postaction::execute_with_post_actions(
                        &program_config,
                        &post_actions,
                        &hotkey_name,
                    ) {
                        eprintln!(
                            "Failed to execute hotkey '{}' with post-actions: {}",
                            hotkey_name, e
                        );
                    }
                } else {
                    // No post-actions, just launch normally
                    if let Err(e) = process::spawner::launch(&program_config) {
                        eprintln!(
                            "Failed to launch program for hotkey '{}': {}",
                            hotkey_name, e
                        );
                    }
                }
            });
            break;
        }
    }
}
```

### 3.2 Make resolve_program Public

**File**: `src-tauri/src/process/spawner.rs`

The `resolve_program` function is already public. Also make `platform` module accessible:

**File**: `src-tauri/src/process/mod.rs`

Ensure platform is public:

```rust
pub mod platform;
pub mod spawner;
```

---

## Phase 4: Frontend UI

### 4.1 Create PostActionEditor Component

**File**: `src/components/PostActionEditor.svelte` (NEW)

```svelte
<script lang="ts">
  import type { PostActionsConfig, PostAction, PostActionType, Keystroke } from '$lib/types';
  import HotkeyRecorder from './HotkeyRecorder.svelte';

  interface Props {
    value: PostActionsConfig;
    onChange: (config: PostActionsConfig) => void;
  }

  let { value, onChange }: Props = $props();

  function createDefaultConfig(): PostActionsConfig {
    return {
      enabled: false,
      trigger: { type: 'onExit' },
      actions: [],
    };
  }

  function addAction(type: 'pasteClipboard' | 'simulateKeystroke' | 'delay') {
    const config = value || createDefaultConfig();
    const newAction: PostAction = {
      id: crypto.randomUUID(),
      actionType: createActionType(type),
      enabled: true,
    };
    onChange({
      ...config,
      actions: [...config.actions, newAction],
    });
  }

  function createActionType(
    type: 'pasteClipboard' | 'simulateKeystroke' | 'delay'
  ): PostActionType {
    switch (type) {
      case 'pasteClipboard':
        return { type: 'pasteClipboard' };
      case 'simulateKeystroke':
        return { type: 'simulateKeystroke', keystroke: { modifiers: [], key: '' } };
      case 'delay':
        return { type: 'delay', delayMs: 500 };
    }
  }

  function updateAction(index: number, action: PostAction) {
    const newActions = [...value.actions];
    newActions[index] = action;
    onChange({ ...value, actions: newActions });
  }

  function removeAction(index: number) {
    onChange({
      ...value,
      actions: value.actions.filter((_, i) => i !== index),
    });
  }

  function moveAction(index: number, direction: 'up' | 'down') {
    const newIndex = direction === 'up' ? index - 1 : index + 1;
    if (newIndex < 0 || newIndex >= value.actions.length) return;

    const newActions = [...value.actions];
    [newActions[index], newActions[newIndex]] = [newActions[newIndex], newActions[index]];
    onChange({ ...value, actions: newActions });
  }

  function getActionLabel(actionType: PostActionType): string {
    switch (actionType.type) {
      case 'pasteClipboard':
        return 'Paste Clipboard';
      case 'simulateKeystroke':
        return 'Simulate Keystroke';
      case 'delay':
        return 'Delay';
    }
  }
</script>

<div class="space-y-4 border-t pt-4 mt-4">
  <div class="flex items-center justify-between">
    <h3 class="text-sm font-medium text-gray-700">Post-Actions</h3>
    <label class="flex items-center">
      <input
        type="checkbox"
        checked={value?.enabled || false}
        onchange={(e) =>
          onChange({ ...(value || createDefaultConfig()), enabled: e.currentTarget.checked })}
        class="h-4 w-4 rounded border-gray-300 text-primary-600 focus:ring-primary-500"
      />
      <span class="ml-2 text-sm text-gray-600">Enable</span>
    </label>
  </div>

  {#if value?.enabled}
    <!-- Trigger selection -->
    <div>
      <label class="block text-sm font-medium text-gray-700">Trigger</label>
      <select
        class="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2 text-sm shadow-sm focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
        value={value.trigger?.type || 'onExit'}
        onchange={(e) => {
          const type = e.currentTarget.value;
          onChange({
            ...value,
            trigger: type === 'onExit' ? { type: 'onExit' } : { type: 'afterDelay', delayMs: 1000 },
          });
        }}
      >
        <option value="onExit">After process exits (exit code 0)</option>
        <option value="afterDelay">After delay from launch</option>
      </select>

      {#if value.trigger?.type === 'afterDelay'}
        <div class="mt-2 flex items-center gap-2">
          <input
            type="number"
            min="0"
            step="100"
            value={value.trigger.delayMs}
            onchange={(e) =>
              onChange({
                ...value,
                trigger: { type: 'afterDelay', delayMs: parseInt(e.currentTarget.value) || 0 },
              })}
            class="w-24 rounded-md border border-gray-300 px-3 py-2 text-sm shadow-sm focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
          />
          <span class="text-sm text-gray-500">ms</span>
        </div>
      {/if}
    </div>

    <!-- Actions list -->
    <div class="space-y-2">
      <label class="block text-sm font-medium text-gray-700">Actions</label>

      {#each value.actions as action, index}
        <div class="flex items-center gap-2 rounded-md border border-gray-200 bg-gray-50 p-3">
          <!-- Reorder buttons -->
          <div class="flex flex-col gap-1">
            <button
              type="button"
              onclick={() => moveAction(index, 'up')}
              disabled={index === 0}
              class="p-1 text-gray-400 hover:text-gray-600 disabled:opacity-30"
            >
              <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M5 15l7-7 7 7"
                />
              </svg>
            </button>
            <button
              type="button"
              onclick={() => moveAction(index, 'down')}
              disabled={index === value.actions.length - 1}
              class="p-1 text-gray-400 hover:text-gray-600 disabled:opacity-30"
            >
              <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M19 9l-7 7-7-7"
                />
              </svg>
            </button>
          </div>

          <!-- Action configuration -->
          <div class="flex-1">
            {#if action.actionType.type === 'pasteClipboard'}
              <span class="text-sm font-medium">Paste Clipboard</span>
              <span class="text-xs text-gray-500 ml-2">(Ctrl/Cmd + V)</span>
            {:else if action.actionType.type === 'simulateKeystroke'}
              <div class="flex items-center gap-2">
                <span class="text-sm">Keystroke:</span>
                <HotkeyRecorder
                  value={{
                    modifiers: action.actionType.keystroke.modifiers,
                    key: action.actionType.keystroke.key,
                  }}
                  onCapture={(hk) =>
                    updateAction(index, {
                      ...action,
                      actionType: { type: 'simulateKeystroke', keystroke: hk },
                    })}
                />
              </div>
            {:else if action.actionType.type === 'delay'}
              <div class="flex items-center gap-2">
                <span class="text-sm">Wait</span>
                <input
                  type="number"
                  min="0"
                  step="100"
                  value={action.actionType.delayMs}
                  onchange={(e) =>
                    updateAction(index, {
                      ...action,
                      actionType: { type: 'delay', delayMs: parseInt(e.currentTarget.value) || 0 },
                    })}
                  class="w-20 rounded-md border border-gray-300 px-2 py-1 text-sm"
                />
                <span class="text-sm text-gray-500">ms</span>
              </div>
            {/if}
          </div>

          <!-- Enable/disable toggle -->
          <input
            type="checkbox"
            checked={action.enabled}
            onchange={(e) => updateAction(index, { ...action, enabled: e.currentTarget.checked })}
            class="h-4 w-4 rounded border-gray-300 text-primary-600"
            title="Enable/disable this action"
          />

          <!-- Remove button -->
          <button
            type="button"
            onclick={() => removeAction(index)}
            class="p-1 text-red-400 hover:text-red-600"
            title="Remove action"
          >
            <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          </button>
        </div>
      {/each}

      <!-- Add action buttons -->
      <div class="flex gap-2 pt-2">
        <button
          type="button"
          onclick={() => addAction('pasteClipboard')}
          class="rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm font-medium text-gray-700 hover:bg-gray-50"
        >
          + Paste
        </button>
        <button
          type="button"
          onclick={() => addAction('simulateKeystroke')}
          class="rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm font-medium text-gray-700 hover:bg-gray-50"
        >
          + Keystroke
        </button>
        <button
          type="button"
          onclick={() => addAction('delay')}
          class="rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm font-medium text-gray-700 hover:bg-gray-50"
        >
          + Delay
        </button>
      </div>
    </div>
  {/if}
</div>
```

### 4.2 Integrate into HotkeyDialog

**File**: `src/components/HotkeyDialog.svelte`

**Step 1**: Add import (after line 4):

```svelte
import PostActionEditor from './PostActionEditor.svelte';
```

**Step 2**: Add state variable (after line 23):

```typescript
let postActions = $state<PostActionsConfig | undefined>(undefined);
```

**Step 3**: Update $effect to initialize postActions (line 34-55):
Add inside the `if (hotkey)` block:

```typescript
postActions = hotkey.postActions || { enabled: false, trigger: { type: 'onExit' }, actions: [] };
```

And in the `else` block:

```typescript
postActions = { enabled: false, trigger: { type: 'onExit' }, actions: [] };
```

**Step 4**: Add PostActionEditor component (after line 265, before the Footer):

```svelte
<!-- Post-Actions -->
<PostActionEditor
  value={postActions || { enabled: false, trigger: { type: 'onExit' }, actions: [] }}
  onChange={(config) => (postActions = config)}
/>
```

**Step 5**: Update handleSubmit to include postActions (line 118-123):

```typescript
onSave({
  name: name.trim(),
  hotkey: hotkeyBinding,
  program,
  enabled,
  postActions, // NEW
});
```

**Step 6**: Update Props interface onSave type (line 10):

```typescript
onSave: (hotkey: Omit<HotkeyConfig, 'id' | 'createdAt' | 'updatedAt'>) => void;
```

---

## Phase 5: Testing Checklist

### 5.1 Build Verification

- [ ] `npm run check` passes
- [ ] `cargo build` succeeds
- [ ] `npm run tauri:dev` starts without errors

### 5.2 Functional Tests

- [ ] Create hotkey without post-actions (backward compatibility)
- [ ] Create hotkey with paste post-action
- [ ] Test OnExit trigger with CLI tool
- [ ] Test AfterDelay trigger with GUI app
- [ ] Test chained actions (delay + paste)
- [ ] Test enable/disable post-actions toggle
- [ ] Test reordering actions
- [ ] Test keystroke simulation

### 5.3 Platform Tests

- [ ] macOS: Accessibility permissions prompt
- [ ] macOS: Cmd+V paste works
- [ ] Windows: Ctrl+V paste works

---

## File Summary

| File                                     | Action | Description                               |
| ---------------------------------------- | ------ | ----------------------------------------- |
| `src-tauri/Cargo.toml`                   | MODIFY | Add `enigo = "0.3"`                       |
| `src-tauri/src/config/schema.rs`         | MODIFY | Add PostAction types, update HotkeyConfig |
| `src-tauri/src/error.rs`                 | MODIFY | Add PostAction error variant              |
| `src-tauri/src/lib.rs`                   | MODIFY | Add `mod postaction;`                     |
| `src-tauri/src/process/mod.rs`           | MODIFY | Ensure `pub mod platform;`                |
| `src-tauri/src/hotkey/manager.rs`        | MODIFY | Update handle_event()                     |
| `src-tauri/src/postaction/mod.rs`        | CREATE | Module declaration                        |
| `src-tauri/src/postaction/input.rs`      | CREATE | InputSimulator with enigo                 |
| `src-tauri/src/postaction/executor.rs`   | CREATE | Post-action execution logic               |
| `src/lib/types.ts`                       | MODIFY | Add PostAction interfaces                 |
| `src/components/PostActionEditor.svelte` | CREATE | Post-action configuration UI              |
| `src/components/HotkeyDialog.svelte`     | MODIFY | Integrate PostActionEditor                |
