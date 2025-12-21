# Post-Actions Feature PRD

## Overview

Extend Global Hotkey to support **post-actions** that execute after a hotkey-triggered process completes or after a configurable delay.

## Primary Use Case

> A hotkey triggers a CLI tool that copies data to the clipboard. After the process completes, a post-action automatically pastes the clipboard content at the current cursor position.

## Requirements (Confirmed)

| Requirement             | Decision                                                |
| ----------------------- | ------------------------------------------------------- |
| **Trigger timing**      | Both: wait for process exit OR after delay (per-hotkey) |
| **Action types**        | Paste, keystroke simulation, delay, chained actions     |
| **Execution condition** | Only on success (exit code 0)                           |

---

## Data Model

### Rust Types (`src-tauri/src/config/schema.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PostActionTrigger {
    OnExit,                          // Wait for process exit code 0
    AfterDelay { delay_ms: u64 },    // Execute after delay from launch
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum PostActionType {
    PasteClipboard,
    SimulateKeystroke { keystroke: Keystroke },
    Delay { delay_ms: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostAction {
    pub id: String,
    pub action_type: PostActionType,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PostActionsConfig {
    pub enabled: bool,
    pub trigger: Option<PostActionTrigger>,
    pub actions: Vec<PostAction>,
}
```

### TypeScript Types (`src/lib/types.ts`)

```typescript
export type PostActionTrigger = { type: 'onExit' } | { type: 'afterDelay'; delayMs: number };

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
  trigger?: PostActionTrigger;
  actions: PostAction[];
}
```

---

## Implementation Plan

### Phase 1: Data Model

1. Update `src-tauri/src/config/schema.rs` - Add PostAction types, extend HotkeyConfig
2. Update `src/lib/types.ts` - Add TypeScript interfaces

### Phase 2: Backend - Keystroke Simulation

1. Add `enigo = "0.6"` to `src-tauri/Cargo.toml`
2. Create `src-tauri/src/postaction/mod.rs` - Module declaration
3. Create `src-tauri/src/postaction/input.rs` - InputSimulator using enigo
4. Create `src-tauri/src/postaction/executor.rs` - Post-action execution logic

### Phase 3: Backend - Process Integration

1. Update `src-tauri/src/process/spawner.rs` - Add `launch_and_wait()` for exit code capture
2. Update `src-tauri/src/hotkey/manager.rs` - Modify `handle_event()` to execute post-actions
3. Update `src-tauri/src/lib.rs` - Add `postaction` module

### Phase 4: Frontend UI

1. Create `src/components/PostActionEditor.svelte` - UI for configuring post-actions
2. Update `src/components/HotkeyDialog.svelte` - Integrate PostActionEditor

### Phase 5: Platform Integration

1. Add macOS accessibility permission check (for keystroke simulation)
2. Test on both Windows and macOS

---

## Files to Modify

| File                                 | Changes                                   |
| ------------------------------------ | ----------------------------------------- |
| `src-tauri/Cargo.toml`               | Add `enigo = "0.6"` dependency            |
| `src-tauri/src/config/schema.rs`     | Add PostAction types, extend HotkeyConfig |
| `src-tauri/src/lib.rs`               | Add `mod postaction;`                     |
| `src-tauri/src/hotkey/manager.rs`    | Update `handle_event()`                   |
| `src-tauri/src/process/spawner.rs`   | Add `launch_and_wait()`                   |
| `src/lib/types.ts`                   | Add PostAction interfaces                 |
| `src/components/HotkeyDialog.svelte` | Integrate post-action UI                  |

## Files to Create

| File                                     | Purpose                              |
| ---------------------------------------- | ------------------------------------ |
| `src-tauri/src/postaction/mod.rs`        | Module declaration                   |
| `src-tauri/src/postaction/input.rs`      | Keystroke simulation (enigo wrapper) |
| `src-tauri/src/postaction/executor.rs`   | Post-action execution logic          |
| `src/components/PostActionEditor.svelte` | Post-action configuration UI         |

---

## Technical Notes

### Keystroke Simulation

- Use **enigo** crate for cross-platform input simulation
- macOS: Requires Accessibility permissions (detect + prompt user)
- Windows: Uses `SendInput` API, no special permissions

### Process Exit Handling

- `OnExit` trigger: Use `Command::output()` to wait and capture exit code
- `AfterDelay` trigger: Launch detached, sleep, then execute actions
- Only execute if exit code == 0 (for OnExit mode)

### Known Limitations

- GUI apps may spawn child processes and exit immediately (use AfterDelay)
- Paste requires target window to have focus
- Long-running processes will block thread in OnExit mode
