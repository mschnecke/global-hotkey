# PRD: Post-Actions for Hotkey Triggers

## Overview

Add configurable post-actions that execute automatically after a hotkey-triggered process completes. The primary use case is pasting clipboard content to the current cursor position after a program places results in the clipboard.

## Problem Statement

Users trigger programs via global hotkeys that produce output in the clipboard (e.g., screenshot tools, text transformers, AI assistants). Currently, users must manually paste (Ctrl+V / Cmd+V) after the program completes. This breaks the seamless workflow that global hotkeys are designed to provide.

## Use Cases

1. **Clipboard Paste**: A CLI tool processes selected text and puts the result in the clipboard. The user wants it automatically pasted at the cursor position.
2. **Delayed Paste**: A program takes time to process. Wait for it to complete, then paste.
3. **Notification**: Show a system notification after a background task completes.
4. **Chained Hotkeys**: Trigger another hotkey after the first completes.

## Proposed Solution

### New Data Structure: `PostAction`

Add a `post_action` field to `HotkeyConfig`:

```rust
// src-tauri/src/config/schema.rs

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PostAction {
    None,
    Paste {
        #[serde(default)]
        delay_ms: u64,  // Delay before pasting (default: 0)
    },
    TypeText {
        text: String,
        #[serde(default)]
        delay_ms: u64,
    },
    Notify {
        title: Option<String>,
        message: String,
    },
    // Future: TriggerHotkey { hotkey_id: String }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HotkeyConfig {
    pub id: String,
    pub name: String,
    pub hotkey: HotkeyBinding,
    pub program: ProgramConfig,
    pub enabled: bool,
    #[serde(default)]
    pub post_action: Option<PostAction>,  // NEW FIELD
    pub created_at: String,
    pub updated_at: String,
}
```

### TypeScript Types

```typescript
// src/lib/types.ts

export type PostAction =
  | { type: 'none' }
  | { type: 'paste'; delay_ms?: number }
  | { type: 'type_text'; text: string; delay_ms?: number }
  | { type: 'notify'; title?: string; message: string };

export interface HotkeyConfig {
  id: string;
  name: string;
  hotkey: HotkeyBinding;
  program: ProgramConfig;
  enabled: boolean;
  postAction?: PostAction;  // NEW FIELD
  createdAt: string;
  updatedAt: string;
}
```

## Implementation Details

### 1. Paste Action Implementation

The paste action requires simulating keyboard input at the OS level.

**Recommended Approach**: Use the `enigo` crate for cross-platform keyboard simulation.

```rust
// src-tauri/src/post_action/executor.rs

use enigo::{Enigo, Key, KeyboardControllable};

pub async fn execute_post_action(action: &PostAction) -> Result<(), String> {
    match action {
        PostAction::None => Ok(()),
        PostAction::Paste { delay_ms } => {
            if *delay_ms > 0 {
                tokio::time::sleep(Duration::from_millis(*delay_ms)).await;
            }
            simulate_paste()
        }
        PostAction::TypeText { text, delay_ms } => {
            if *delay_ms > 0 {
                tokio::time::sleep(Duration::from_millis(*delay_ms)).await;
            }
            simulate_type(text)
        }
        PostAction::Notify { title, message } => {
            show_notification(title.as_deref(), message)
        }
    }
}

fn simulate_paste() -> Result<(), String> {
    let mut enigo = Enigo::new();

    #[cfg(target_os = "macos")]
    {
        enigo.key_down(Key::Meta);
        enigo.key_click(Key::Layout('v'));
        enigo.key_up(Key::Meta);
    }

    #[cfg(target_os = "windows")]
    {
        enigo.key_down(Key::Control);
        enigo.key_click(Key::Layout('v'));
        enigo.key_up(Key::Control);
    }

    Ok(())
}
```

### 2. Process Completion Detection

**Challenge**: Currently, processes are spawned detached and we don't wait for them.

**Options**:

| Option | Pros | Cons |
|--------|------|------|
| A. Wait for process exit | Accurate completion detection | Blocks if process hangs; changes current behavior |
| B. Fixed delay | Simple implementation | Unreliable; may paste too early or wait too long |
| C. Clipboard change detection | Accurate for clipboard use cases | Complex; doesn't work for all post-actions |
| D. User-configured delay | User controls timing | Requires user to know how long process takes |

**Recommended**: Option A with timeout + user-configured delay as fallback.

```rust
pub struct PostActionConfig {
    pub action: PostAction,
    pub wait_for_exit: bool,      // Wait for process to complete
    pub timeout_ms: Option<u64>,  // Max wait time (default: 30000)
}
```

### 3. Modified Event Flow

```
User presses hotkey
    → OS sends GlobalHotKeyEvent
    → handle_event() matches hotkey
    → Spawn async task:
        1. Launch process
        2. If wait_for_exit: wait with timeout
        3. Apply post_action delay_ms
        4. Execute post_action
```

### 4. UI Changes

**HotkeyDialog.svelte** additions:

```
┌─────────────────────────────────────────────────┐
│ Post Action                                     │
│ ┌─────────────────────────────────────────────┐ │
│ │ ▼ None                                      │ │
│ │   Paste clipboard                           │ │
│ │   Type text                                 │ │
│ │   Show notification                         │ │
│ └─────────────────────────────────────────────┘ │
│                                                 │
│ (When "Paste clipboard" selected:)              │
│ ┌─────────────────────────────────────────────┐ │
│ │ ☑ Wait for program to exit                  │ │
│ │   Timeout: [30000] ms                       │ │
│ │   Additional delay: [0] ms                  │ │
│ └─────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────┘
```

## Technical Requirements

### Dependencies

Add to `src-tauri/Cargo.toml`:

```toml
[dependencies]
enigo = "0.1"  # Cross-platform keyboard/mouse simulation
```

### Permissions (macOS)

macOS requires Accessibility permissions to simulate keyboard input. The app must:
1. Be added to System Preferences → Security & Privacy → Privacy → Accessibility
2. Show a prompt guiding users to enable this permission

### Permissions (Windows)

No special permissions required for keyboard simulation on Windows.

## Migration

### Config Version Bump

Increment config version from `1.0.0` to `1.1.0`.

### Backward Compatibility

- `post_action` field is `Option<PostAction>` with `#[serde(default)]`
- Existing configs without `post_action` deserialize as `None`
- No migration script needed

## Security Considerations

1. **Keyboard Simulation**: Can be abused for keylogging or unauthorized input
   - Mitigated: Only executes user-configured actions on user-triggered hotkeys

2. **Clipboard Access**: Already implicit in paste functionality
   - No additional risk beyond what user intends

3. **Process Wait**: Malicious or hung processes could delay post-action indefinitely
   - Mitigated: Configurable timeout with sensible default (30s)

## Testing Plan

1. **Unit Tests**
   - PostAction serialization/deserialization
   - Config migration (old config loads correctly)

2. **Integration Tests**
   - Paste action simulates correct key combo per platform
   - Wait-for-exit correctly detects process completion
   - Timeout triggers correctly

3. **Manual Tests**
   - Create hotkey with paste post-action
   - Trigger hotkey, verify clipboard is pasted
   - Test with slow-completing programs
   - Test timeout behavior

## Implementation Phases

### Phase 1: Core Infrastructure (MVP)
- Add `PostAction` enum with `Paste` variant only
- Add `post_action` field to `HotkeyConfig`
- Implement keyboard simulation with `enigo`
- Modify event handler to execute post-action
- Update UI with post-action dropdown

### Phase 2: Process Completion Detection
- Add wait-for-exit option
- Implement timeout handling
- Add configurable delay

### Phase 3: Additional Actions
- `TypeText` action
- `Notify` action
- Future: `TriggerHotkey` action

## Open Questions

1. **Should paste work with any app or only specific apps?**
   - Recommendation: Any app (current foreground app at time of paste)

2. **What if the original window loses focus during process execution?**
   - Consider: Re-focus original window before paste?
   - Recommendation: Paste to current foreground (simpler, matches user expectation)

3. **Should there be a global "enable post-actions" toggle?**
   - Use case: Quickly disable all post-actions for debugging
   - Recommendation: Add to settings in a future phase

4. **Multiple post-actions per hotkey?**
   - Use case: Paste AND show notification
   - Recommendation: Start with single action, consider array in future version

## Success Metrics

- Users can configure paste post-action without manual intervention
- Paste executes within 100ms of process completion (when wait-for-exit enabled)
- No accessibility permission prompts on Windows
- Clear macOS permission guidance in documentation

## References

- [enigo crate](https://crates.io/crates/enigo) - Keyboard simulation
- [tauri-plugin-clipboard](https://github.com/prabhuignoto/tauri-plugin-clipboard) - Alternative clipboard approach
- Current hotkey handling: `src-tauri/src/hotkey/manager.rs`
