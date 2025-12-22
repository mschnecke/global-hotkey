# PRD: AI Module for Global Hotkey

## Overview

Add an AI module that sends text and audio prompts to AI models and returns AI-generated responses. The module enables seamless AI-assisted workflows triggered by global hotkeys, with the primary use case being: record German audio prompt → transcribe → translate → beautify → save to clipboard.

## Problem Statement

Users need to quickly interact with AI models via global hotkeys. The current manual workflow (record audio, upload to AI service, copy response, paste) is inefficient and breaks the seamless experience that global hotkeys are designed to provide.

**Target Users**: Power users who want to integrate AI capabilities into their hotkey-driven workflows for tasks like:

- Voice-to-text transcription
- Real-time translation
- Text beautification and formatting
- AI-assisted response generation

## Use Cases

| Use Case             | Description                                               | Example                                                       |
| -------------------- | --------------------------------------------------------- | ------------------------------------------------------------- |
| Audio Transcription  | Record speech and get transcribed text                    | Record German meeting notes → get text transcript             |
| Audio Translation    | Record speech in one language, get translation in another | Record German voice → get English translation                 |
| Text Beautification  | Transform raw/informal text into polished output          | Rough notes → professional email                              |
| Custom Pipelines     | Chain multiple AI tasks together                          | Audio → transcribe → translate → format as email → clipboard  |
| Clipboard Processing | Send clipboard content to AI with custom instructions     | Select text → hotkey → AI improves/summarizes → new clipboard |

## Proposed Solution

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        Global Hotkey App                         │
├─────────────────────────────────────────────────────────────────┤
│  PostAction System                                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
│  │ PasteClip   │  │ Keystroke   │  │ CallAi      │ ← NEW        │
│  └─────────────┘  └─────────────┘  └─────────────┘              │
├─────────────────────────────────────────────────────────────────┤
│  AI Module (NEW)                                                 │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │ Input Sources    │ Provider Layer      │ Output Actions     ││
│  │ ┌─────────────┐  │ ┌───────────────┐   │ ┌───────────────┐  ││
│  │ │ Clipboard   │  │ │ Gemini        │   │ │ Clipboard     │  ││
│  │ │ Audio Rec.  │  │ │ OpenAI (fut.) │   │ │ Paste         │  ││
│  │ │ Process Out │  │ │ Anthropic     │   │ │ File          │  ││
│  │ └─────────────┘  │ │ Ollama        │   │ └───────────────┘  ││
│  │                  │ └───────────────┘   │                    ││
│  └─────────────────────────────────────────────────────────────┘│
│  ┌─────────────────────────────────────────────────────────────┐│
│  │ Role System (Configurable Prompts)                          ││
│  │ • DE-Transcribe  • DE-EN-Translate  • Beautify  • Custom... ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

### New Data Structures

#### Rust Backend (`src-tauri/src/config/schema.rs`)

```rust
/// AI Provider type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AiProviderType {
    Gemini,
    // Future: OpenAi, Anthropic, Ollama
}

/// AI Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiProviderConfig {
    pub id: String,
    pub provider_type: AiProviderType,
    pub api_key: String,
    pub model: Option<String>,
    pub base_url: Option<String>,  // For Ollama/custom endpoints
    pub enabled: bool,
}

/// Output format for AI responses
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum OutputFormat {
    #[default]
    Plain,
    Markdown,
    Json,
}

/// Configurable AI Role/Task
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiRole {
    pub id: String,
    pub name: String,
    pub system_prompt: String,
    pub output_format: OutputFormat,
    pub is_builtin: bool,
}

/// AI Settings (added to AppSettings)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AiSettings {
    pub providers: Vec<AiProviderConfig>,
    pub default_provider_id: Option<String>,
    pub roles: Vec<AiRole>,
}

/// Audio format for recording
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum AudioFormat {
    #[default]
    Opus,  // Preferred: ~10x smaller files
    Wav,   // Fallback for compatibility
}

/// Input source for AI actions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum AiInputSource {
    /// Read text from clipboard
    Clipboard,
    /// Record audio from microphone
    RecordAudio {
        max_duration_ms: u64,
        format: AudioFormat,
    },
    /// Use stdout from launched process
    ProcessOutput,
}

/// New PostActionType variant
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum PostActionType {
    PasteClipboard,
    SimulateKeystroke { keystroke: Keystroke },
    Delay { delay_ms: u64 },
    /// NEW: Call AI with input and save response
    CallAi {
        role_id: String,
        input_source: AiInputSource,
        provider_id: Option<String>,  // Uses default if None
    },
}
```

#### TypeScript Frontend (`src/lib/types.ts`)

```typescript
// AI Provider types
export type AiProviderType = 'gemini' | 'openai' | 'anthropic' | 'ollama';

export interface AiProviderConfig {
  id: string;
  providerType: AiProviderType;
  apiKey: string;
  model?: string;
  baseUrl?: string;
  enabled: boolean;
}

export type OutputFormat = 'plain' | 'markdown' | 'json';

export interface AiRole {
  id: string;
  name: string;
  systemPrompt: string;
  outputFormat: OutputFormat;
  isBuiltin: boolean;
}

export interface AiSettings {
  providers: AiProviderConfig[];
  defaultProviderId?: string;
  roles: AiRole[];
}

export type AudioFormat = 'opus' | 'wav'; // Opus preferred (smaller files)

export type AiInputSource =
  | { type: 'clipboard' }
  | { type: 'recordAudio'; maxDurationMs: number; format: AudioFormat }
  | { type: 'processOutput' };

// Extended PostActionType
export type PostActionType =
  | { type: 'pasteClipboard' }
  | { type: 'simulateKeystroke'; keystroke: Keystroke }
  | { type: 'delay'; delayMs: number }
  | { type: 'callAi'; roleId: string; inputSource: AiInputSource; providerId?: string };
```

### Default Built-in Roles

| Role ID           | Name                  | System Prompt                                                                                                        |
| ----------------- | --------------------- | -------------------------------------------------------------------------------------------------------------------- |
| `de-transcribe`   | DE Transcribe         | "Transcribe the following German audio accurately. Output only the transcription without any additional commentary." |
| `de-en-translate` | DE→EN Translate       | "Translate the following German text to English. Maintain the original meaning and tone."                            |
| `beautify`        | Beautify Text         | "Improve the formatting, grammar, and clarity of this text while preserving its meaning."                            |
| `ai-response`     | Format as AI Response | "Format this text as a professional, well-structured response suitable for an AI assistant."                         |

## Technical Requirements

### Dependencies

Add to `src-tauri/Cargo.toml`:

```toml
[dependencies]
# Audio Recording
cpal = "0.15"           # Cross-platform audio I/O
ogg = "0.9"             # Ogg container format
opus = "0.3"            # Opus codec encoding
hound = "3.5"           # WAV file handling (fallback)

# HTTP Client for AI APIs
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }

# Async utilities
tokio = { version = "1", features = ["full"] }

# Base64 encoding for audio data
base64 = "0.22"
```

### Gemini API Integration

**Endpoint**: `https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent`

**Supported Models**:

- `gemini-1.5-flash` (default, fast, cost-effective)
- `gemini-1.5-pro` (higher quality)
- `gemini-2.0-flash-exp` (experimental, latest)

**Audio Support**: Gemini 1.5+ models support audio input natively via base64-encoded data.

**API Key Validation**:

- Format: 39 characters, starts with "AIza"
- Multiple keys supported (pipe-separated: `key1|key2|key3`) for rate limit fallback

**Request Format**:

```json
{
  "contents": [
    {
      "parts": [
        {
          "text": "Transcribe this German audio:"
        },
        {
          "inline_data": {
            "mime_type": "audio/ogg",
            "data": "<base64-encoded-opus-audio>"
          }
        }
      ]
    }
  ],
  "generationConfig": {
    "temperature": 0.1,
    "maxOutputTokens": 8192
  }
}
```

**Supported MIME types**: `audio/ogg` (Opus), `audio/wav`, `audio/mp3`, `audio/aac`

### Audio Recording Specifications

| Parameter    | Value                     | Notes                                 |
| ------------ | ------------------------- | ------------------------------------- |
| Sample Rate  | 16000 Hz                  | Optimal for speech recognition        |
| Channels     | Mono                      | Single channel sufficient             |
| Bit Depth    | 16-bit                    | Standard quality                      |
| Format       | Opus (default) or WAV     | Opus preferred for ~10x smaller files |
| Max Duration | 30 seconds (configurable) | Prevents accidental long recordings   |
| Device       | System default            | Future: configurable selection        |

**Why Opus over WAV:**

- 30s of 16kHz mono WAV ≈ 960 KB
- 30s of 16kHz mono Opus ≈ 60-100 KB (10x smaller)
- Gemini API accepts both formats natively
- Faster upload times, lower bandwidth usage
- WAV available as fallback for compatibility

### New Tauri Commands

```rust
// AI Provider commands
#[tauri::command]
async fn test_ai_provider(provider_id: String) -> Result<bool, String>;

#[tauri::command]
async fn get_ai_models(provider_type: AiProviderType) -> Result<Vec<String>, String>;

// AI execution commands
#[tauri::command]
async fn send_to_ai(
    input: String,
    role_id: String,
    provider_id: Option<String>
) -> Result<String, String>;

#[tauri::command]
async fn send_audio_to_ai(
    audio_path: String,
    role_id: String,
    provider_id: Option<String>
) -> Result<String, String>;

// Audio recording commands
#[tauri::command]
async fn start_audio_recording(max_duration_ms: u64) -> Result<String, String>;

#[tauri::command]
async fn stop_audio_recording() -> Result<String, String>;

#[tauri::command]
fn get_audio_devices() -> Result<Vec<String>, String>;

// Role management commands
#[tauri::command]
fn get_builtin_roles() -> Vec<AiRole>;
```

## UI Changes

### Settings Page - AI Configuration Tab

```
┌─────────────────────────────────────────────────────────────────┐
│ Settings                                          [General] [AI]│
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│ AI Providers                                                    │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ ● Gemini                                        [Default]   │ │
│ │   API Key: [••••••••••••••••••••••••••••] [Show] [Test]     │ │
│ │   Model:   [gemini-1.5-flash           ▼]                   │ │
│ │   Status:  ✓ Connected                                      │ │
│ ├─────────────────────────────────────────────────────────────┤ │
│ │ ○ OpenAI                                        [Disabled]  │ │
│ │   (Coming soon)                                             │ │
│ └─────────────────────────────────────────────────────────────┘ │
│                                                                 │
│ [+ Add Provider]                                                │
│                                                                 │
│ ─────────────────────────────────────────────────────────────── │
│                                                                 │
│ AI Roles                                                        │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ Built-in Roles                                              │ │
│ │  • DE Transcribe         - Transcribe German audio          │ │
│ │  • DE→EN Translate       - Translate German to English      │ │
│ │  • Beautify Text         - Improve text formatting          │ │
│ │  • Format as AI Response - Professional AI response format  │ │
│ ├─────────────────────────────────────────────────────────────┤ │
│ │ Custom Roles                                                │ │
│ │  • My Summary Role                        [Edit] [Delete]   │ │
│ │  • Code Review Helper                     [Edit] [Delete]   │ │
│ └─────────────────────────────────────────────────────────────┘ │
│                                                                 │
│ [+ Add Custom Role]                                             │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Post-Action Editor - Call AI Action

```
┌─────────────────────────────────────────────────────────────────┐
│ Post-Actions                                         [Enabled ✓]│
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│ 1. [Call AI              ▼]                          [×]        │
│    ┌─────────────────────────────────────────────────────────┐  │
│    │ Input Source: [Record Audio     ▼]                      │  │
│    │               Max Duration: [30    ] seconds            │  │
│    │               Format: [WAV ▼]                           │  │
│    │                                                         │  │
│    │ AI Role:      [DE→EN Translate  ▼]                      │  │
│    │                                                         │  │
│    │ Provider:     [Default (Gemini) ▼]                      │  │
│    └─────────────────────────────────────────────────────────┘  │
│                                                                 │
│ 2. [Paste Clipboard      ▼]                          [×]        │
│                                                                 │
│ [+ Add Action]                                                  │
│                                                                 │
│ Trigger: [After Delay ▼] [500] ms                               │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Custom Role Editor Dialog

```
┌─────────────────────────────────────────────────────────────────┐
│ Edit AI Role                                              [×]   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│ Name:                                                           │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ My Custom Role                                              │ │
│ └─────────────────────────────────────────────────────────────┘ │
│                                                                 │
│ System Prompt:                                                  │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ You are a helpful assistant. Please summarize the          │ │
│ │ following text in 3 bullet points, focusing on the         │ │
│ │ main takeaways.                                             │ │
│ │                                                             │ │
│ │                                                             │ │
│ └─────────────────────────────────────────────────────────────┘ │
│                                                                 │
│ Output Format: [Plain Text ▼]                                   │
│                                                                 │
│                                         [Cancel]  [Save]        │
└─────────────────────────────────────────────────────────────────┘
```

## Event Flow

### Audio Recording + AI Processing Flow

```
User presses configured hotkey
    │
    ▼
PostAction executor starts
    │
    ├─► CallAi action detected
    │       │
    │       ▼
    │   Input Source = RecordAudio?
    │       │
    │       ├─► YES: Start audio recording
    │       │       │
    │       │       ▼
    │       │   Show recording indicator (tray/notification)
    │       │       │
    │       │       ▼
    │       │   Wait for: stop signal OR max duration
    │       │       │
    │       │       ▼
    │       │   Encode to Opus format (~10x smaller than WAV)
    │       │       │
    │       │       ▼
    │       │   Save audio to temp file (.ogg)
    │       │       │
    │       │       ▼
    │       │   Encode audio as base64
    │       │
    │       ├─► NO (Clipboard): Read clipboard content
    │       │
    │       ├─► NO (ProcessOutput): Capture process stdout
    │       │
    │       ▼
    │   Load role configuration
    │       │
    │       ▼
    │   Build API request with:
    │   - System prompt from role
    │   - Input content (text or audio)
    │       │
    │       ▼
    │   Send to AI provider (Gemini)
    │       │
    │       ▼
    │   Receive response
    │       │
    │       ▼
    │   Save response to clipboard
    │       │
    │       ▼
    │   Clean up temp audio file
    │
    ▼
Continue with next PostAction (e.g., PasteClipboard)
```

## Security Considerations

| Concern           | Mitigation                                                            |
| ----------------- | --------------------------------------------------------------------- |
| API Key Storage   | Store in config file (Phase 1); Future: OS keychain/encrypted storage |
| API Key Exposure  | Mask in UI, never log, exclude from exports                           |
| Audio Data        | Delete temp files immediately after processing                        |
| Network Security  | HTTPS-only for all API calls                                          |
| Microphone Access | macOS: Request permission via system dialog                           |
| Rate Limiting     | Support multiple API keys with automatic fallback                     |

## Platform-Specific Requirements

### macOS

- **Microphone Permission**: App must request access via `NSMicrophoneUsageDescription` in Info.plist
- **Accessibility Permission**: Already required for hotkey capture
- **Notification**: Use existing tray notification system for recording status

### Windows

- **Microphone Permission**: Windows will prompt user on first use
- **No additional permissions required** for audio recording

## Migration

### Config Version

Increment config version from `1.0.0` to `1.1.0`.

### Backward Compatibility

```rust
// AppSettings extension
pub struct AppSettings {
    pub start_with_system: bool,
    pub show_tray_notifications: bool,
    #[serde(default)]  // NEW - defaults to empty AiSettings
    pub ai: AiSettings,
}
```

- `ai` field uses `#[serde(default)]` - existing configs load with empty AI settings
- No migration script needed
- Built-in roles generated on first load if `ai.roles` is empty

## Testing Plan

### Unit Tests

- [ ] AiProviderConfig serialization/deserialization
- [ ] AiRole serialization/deserialization
- [ ] AiInputSource enum variants
- [ ] API key validation (format, multi-key parsing)
- [ ] Config migration (old config loads with default AI settings)

### Integration Tests

- [ ] Gemini API client connects and authenticates
- [ ] Audio recording produces valid WAV file
- [ ] Base64 encoding of audio data
- [ ] Full flow: record → encode → send → receive → clipboard

### Manual Tests

- [ ] Configure Gemini API key in settings
- [ ] Test API connection button works
- [ ] Create custom role
- [ ] Configure hotkey with CallAi post-action
- [ ] Trigger hotkey, record audio, verify response in clipboard
- [ ] Test with different input sources (clipboard, audio)
- [ ] Test error handling (invalid API key, network failure)

## Implementation Phases

### Phase 1: Core Infrastructure (MVP)

**Goal**: API key configuration and basic Gemini text API

- [ ] Add `AiSettings` to `AppConfig` schema
- [ ] Create `src-tauri/src/ai/` module structure
- [ ] Implement Gemini API client (text-only)
- [ ] Add Tauri commands: `test_ai_provider`, `send_to_ai`
- [ ] Settings UI: API key input with test button
- [ ] API key validation

**Files to modify/create**:

- `src-tauri/src/config/schema.rs` - Add AI types
- `src-tauri/src/ai/mod.rs` - New module
- `src-tauri/src/ai/gemini.rs` - Gemini client
- `src-tauri/src/ai/provider.rs` - Provider trait
- `src-tauri/src/lib.rs` - Register commands
- `src/lib/types.ts` - TypeScript types
- `src/components/SettingsDialog.svelte` - AI settings tab

### Phase 2: Audio Recording

**Goal**: Record audio from microphone with Opus encoding

- [ ] Add `cpal`, `ogg`, `opus`, and `hound` dependencies
- [ ] Implement audio recording module with Opus encoding
- [ ] Add Tauri commands: `start_audio_recording`, `stop_audio_recording`
- [ ] Recording status indicator (tray icon change or notification)
- [ ] Temp file management

**Files to modify/create**:

- `src-tauri/Cargo.toml` - Add dependencies
- `src-tauri/src/audio/mod.rs` - New module
- `src-tauri/src/audio/recorder.rs` - Recording logic
- `src-tauri/src/audio/encoder.rs` - Opus encoding
- `src-tauri/src/lib.rs` - Register commands

### Phase 3: Post-Action Integration

**Goal**: CallAi as a post-action type

- [ ] Add `CallAi` variant to `PostActionType`
- [ ] Implement `AiInputSource` handling in executor
- [ ] Integrate audio → Gemini API flow
- [ ] Update PostActionEditor UI with CallAi option
- [ ] Input source selection UI

**Files to modify**:

- `src-tauri/src/config/schema.rs` - Add CallAi variant
- `src-tauri/src/postaction/executor.rs` - Handle CallAi
- `src/lib/types.ts` - Update PostActionType
- `src/components/PostActionEditor.svelte` - CallAi UI

### Phase 4: Configurable Roles

**Goal**: Built-in and custom roles system

- [ ] Define built-in roles with default prompts
- [ ] Role CRUD operations
- [ ] Role editor dialog UI
- [ ] Role selection dropdown in post-action editor

**Files to modify/create**:

- `src-tauri/src/ai/roles.rs` - Role management
- `src/components/RoleEditor.svelte` - New component
- `src/components/PostActionEditor.svelte` - Role selection

### Phase 5: Multi-Provider Support (Future)

**Goal**: Abstract provider layer for multiple AI services

- [ ] Define `AiProvider` trait
- [ ] Refactor Gemini into provider implementation
- [ ] Add OpenAI provider
- [ ] Add Anthropic provider
- [ ] Add Ollama (local) provider
- [ ] Provider selection UI

## Open Questions

1. **Recording stop mechanism**: Should recording stop on:
   - Second hotkey press? (same key)
   - Any key press?
   - Fixed timeout only?
   - **Recommendation**: Fixed timeout with option to stop early via same hotkey

2. **Recording feedback**: How to indicate recording is active?
   - Tray icon change?
   - System notification?
   - Both?
   - **Recommendation**: Tray icon change (red dot overlay)

3. **Error handling UI**: How to show API errors to user?
   - System notification?
   - Tray menu status?
   - Log file?
   - **Recommendation**: System notification for user-facing errors, log file for debugging

4. **Multiple API keys**: Support pipe-separated keys for rate limit fallback?
   - **Recommendation**: Yes, follow gia pattern for robustness

## Success Metrics

- [ ] API key can be configured and validated in under 30 seconds
- [ ] Audio recording works reliably on macOS and Windows
- [ ] Gemini transcription returns results within 5 seconds for 30-second audio
- [ ] Response automatically saved to clipboard
- [ ] Custom roles can be created and used
- [ ] Error states are clearly communicated to user

## References

- [Google Gemini API Documentation](https://ai.google.dev/docs)
- [cpal crate](https://crates.io/crates/cpal) - Cross-platform audio I/O
- [hound crate](https://crates.io/crates/hound) - WAV file handling
- [reqwest crate](https://crates.io/crates/reqwest) - HTTP client
- [github-gia](https://github.com/mschnecke/github-gia) - Reference implementation inspiration
- Current post-action implementation: `src-tauri/src/postaction/executor.rs`
