# Implementation Plan: AI Module

This document provides step-by-step implementation instructions for the AI Module feature as defined in [PRD-ai-module.md](./PRD-ai-module.md).

---

## Prerequisites

Before starting implementation, ensure:

- [ ] Rust toolchain installed (1.70+)
- [ ] Node.js 18+ installed
- [ ] Gemini API key obtained from [Google AI Studio](https://aistudio.google.com/apikey)

---

## Phase 1: Core Infrastructure (MVP)

**Goal**: API key configuration and basic Gemini text API integration.

### Step 1.1: Add Rust Dependencies

**File**: `src-tauri/Cargo.toml`

Add the following dependencies:

```toml
[dependencies]
# ... existing dependencies ...

# AI Module - HTTP Client
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }

# Base64 encoding for audio data
base64 = "0.22"
```

**Verification**: Run `cargo check` in `src-tauri/` directory.

---

### Step 1.2: Add AI Types to Schema

**File**: `src-tauri/src/config/schema.rs`

Add the following types after the existing `PostActionsConfig` struct:

```rust
// ============================================================================
// AI Module Types
// ============================================================================

/// AI Provider type
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum AiProviderType {
    #[default]
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
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

/// Output format for AI responses
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
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
    #[serde(default)]
    pub output_format: OutputFormat,
    #[serde(default)]
    pub is_builtin: bool,
}

/// AI Settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AiSettings {
    #[serde(default)]
    pub providers: Vec<AiProviderConfig>,
    #[serde(default)]
    pub default_provider_id: Option<String>,
    #[serde(default)]
    pub roles: Vec<AiRole>,
}

/// Audio format for recording
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum AudioFormat {
    #[default]
    Opus,
    Wav,
}

/// Input source for AI actions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum AiInputSource {
    Clipboard,
    RecordAudio {
        #[serde(default = "default_max_duration")]
        max_duration_ms: u64,
        #[serde(default)]
        format: AudioFormat,
    },
    ProcessOutput,
}

fn default_max_duration() -> u64 {
    30000 // 30 seconds
}
```

**Update `AppSettings`** to include AI settings:

```rust
/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub start_with_system: bool,
    pub show_tray_notifications: bool,
    #[serde(default)]
    pub ai: AiSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            start_with_system: false,
            show_tray_notifications: true,
            ai: AiSettings::default(),
        }
    }
}
```

**Verification**: Run `cargo check`.

---

### Step 1.3: Create AI Module Structure

Create the following file structure:

```
src-tauri/src/
├── ai/
│   ├── mod.rs          # Module exports
│   ├── provider.rs     # Provider trait
│   ├── gemini.rs       # Gemini implementation
│   └── roles.rs        # Built-in roles
```

**File**: `src-tauri/src/ai/mod.rs`

```rust
//! AI Module - Handles AI provider integrations

pub mod gemini;
pub mod provider;
pub mod roles;

pub use provider::AiProvider;
pub use gemini::GeminiProvider;
pub use roles::get_builtin_roles;
```

**File**: `src-tauri/src/ai/provider.rs`

```rust
//! AI Provider trait and common types

use crate::error::AppError;

/// Response from an AI provider
#[derive(Debug, Clone)]
pub struct AiResponse {
    pub text: String,
    pub finish_reason: Option<String>,
}

/// Trait for AI providers
pub trait AiProvider: Send + Sync {
    /// Send a text prompt to the AI
    fn send_text(
        &self,
        system_prompt: &str,
        user_input: &str,
    ) -> impl std::future::Future<Output = Result<AiResponse, AppError>> + Send;

    /// Send audio data to the AI
    fn send_audio(
        &self,
        system_prompt: &str,
        audio_data: &[u8],
        mime_type: &str,
    ) -> impl std::future::Future<Output = Result<AiResponse, AppError>> + Send;

    /// Test the connection/API key
    fn test_connection(
        &self,
    ) -> impl std::future::Future<Output = Result<bool, AppError>> + Send;
}
```

**File**: `src-tauri/src/ai/gemini.rs`

```rust
//! Gemini API client implementation

use base64::Engine;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use super::provider::{AiProvider, AiResponse};

const GEMINI_API_BASE: &str = "https://generativelanguage.googleapis.com/v1beta";
const DEFAULT_MODEL: &str = "gemini-1.5-flash";

pub struct GeminiProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl GeminiProvider {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model: model.unwrap_or_else(|| DEFAULT_MODEL.to_string()),
        }
    }

    fn endpoint(&self) -> String {
        format!(
            "{}/models/{}:generateContent?key={}",
            GEMINI_API_BASE, self.model, self.api_key
        )
    }
}

// Request/Response types
#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
    #[serde(rename = "generationConfig", skip_serializing_if = "Option::is_none")]
    generation_config: Option<GenerationConfig>,
}

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize)]
#[serde(untagged)]
enum Part {
    Text { text: String },
    InlineData { inline_data: InlineData },
}

#[derive(Serialize)]
struct InlineData {
    mime_type: String,
    data: String,
}

#[derive(Serialize)]
struct GenerationConfig {
    temperature: f32,
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: u32,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<Candidate>>,
    error: Option<GeminiError>,
}

#[derive(Deserialize)]
struct Candidate {
    content: CandidateContent,
    #[serde(rename = "finishReason")]
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct CandidateContent {
    parts: Vec<ResponsePart>,
}

#[derive(Deserialize)]
struct ResponsePart {
    text: Option<String>,
}

#[derive(Deserialize)]
struct GeminiError {
    message: String,
}

impl AiProvider for GeminiProvider {
    async fn send_text(
        &self,
        system_prompt: &str,
        user_input: &str,
    ) -> Result<AiResponse, AppError> {
        let combined_prompt = format!("{}\n\n{}", system_prompt, user_input);

        let request = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part::Text { text: combined_prompt }],
            }],
            generation_config: Some(GenerationConfig {
                temperature: 0.1,
                max_output_tokens: 8192,
            }),
        };

        let response = self
            .client
            .post(&self.endpoint())
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Ai(format!("Request failed: {}", e)))?;

        let gemini_response: GeminiResponse = response
            .json()
            .await
            .map_err(|e| AppError::Ai(format!("Failed to parse response: {}", e)))?;

        if let Some(error) = gemini_response.error {
            return Err(AppError::Ai(error.message));
        }

        let text = gemini_response
            .candidates
            .and_then(|c| c.into_iter().next())
            .and_then(|c| c.content.parts.into_iter().next())
            .and_then(|p| p.text)
            .unwrap_or_default();

        Ok(AiResponse {
            text,
            finish_reason: None,
        })
    }

    async fn send_audio(
        &self,
        system_prompt: &str,
        audio_data: &[u8],
        mime_type: &str,
    ) -> Result<AiResponse, AppError> {
        let audio_base64 = base64::engine::general_purpose::STANDARD.encode(audio_data);

        let request = GeminiRequest {
            contents: vec![Content {
                parts: vec![
                    Part::Text { text: system_prompt.to_string() },
                    Part::InlineData {
                        inline_data: InlineData {
                            mime_type: mime_type.to_string(),
                            data: audio_base64,
                        },
                    },
                ],
            }],
            generation_config: Some(GenerationConfig {
                temperature: 0.1,
                max_output_tokens: 8192,
            }),
        };

        let response = self
            .client
            .post(&self.endpoint())
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Ai(format!("Request failed: {}", e)))?;

        let gemini_response: GeminiResponse = response
            .json()
            .await
            .map_err(|e| AppError::Ai(format!("Failed to parse response: {}", e)))?;

        if let Some(error) = gemini_response.error {
            return Err(AppError::Ai(error.message));
        }

        let text = gemini_response
            .candidates
            .and_then(|c| c.into_iter().next())
            .and_then(|c| c.content.parts.into_iter().next())
            .and_then(|p| p.text)
            .unwrap_or_default();

        Ok(AiResponse {
            text,
            finish_reason: None,
        })
    }

    async fn test_connection(&self) -> Result<bool, AppError> {
        let result = self.send_text("Respond with only: OK", "Test").await;
        match result {
            Ok(_) => Ok(true),
            Err(e) => Err(e),
        }
    }
}
```

**File**: `src-tauri/src/ai/roles.rs`

```rust
//! Built-in AI roles

use crate::config::schema::{AiRole, OutputFormat};

/// Get the default built-in roles
pub fn get_builtin_roles() -> Vec<AiRole> {
    vec![
        AiRole {
            id: "de-transcribe".to_string(),
            name: "DE Transcribe".to_string(),
            system_prompt: "Transcribe the following German audio accurately. Output only the transcription without any additional commentary.".to_string(),
            output_format: OutputFormat::Plain,
            is_builtin: true,
        },
        AiRole {
            id: "de-en-translate".to_string(),
            name: "DE→EN Translate".to_string(),
            system_prompt: "Translate the following German text to English. Maintain the original meaning and tone.".to_string(),
            output_format: OutputFormat::Plain,
            is_builtin: true,
        },
        AiRole {
            id: "beautify".to_string(),
            name: "Beautify Text".to_string(),
            system_prompt: "Improve the formatting, grammar, and clarity of this text while preserving its meaning.".to_string(),
            output_format: OutputFormat::Plain,
            is_builtin: true,
        },
        AiRole {
            id: "ai-response".to_string(),
            name: "Format as AI Response".to_string(),
            system_prompt: "Format this text as a professional, well-structured response suitable for an AI assistant.".to_string(),
            output_format: OutputFormat::Plain,
            is_builtin: true,
        },
    ]
}
```

---

### Step 1.4: Add AI Error Variant

**File**: `src-tauri/src/error.rs`

Add the `Ai` variant to `AppError`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    // ... existing variants ...

    #[error("AI error: {0}")]
    Ai(String),
}
```

---

### Step 1.5: Register AI Module

**File**: `src-tauri/src/lib.rs`

Add module declaration at the top:

```rust
mod ai;  // Add this line
mod config;
mod error;
// ...
```

Add the Tauri commands:

```rust
// ============================================================================
// Tauri Commands - AI Module
// ============================================================================

/// Test an AI provider connection
#[tauri::command]
async fn test_ai_provider(api_key: String, model: Option<String>) -> Result<bool, String> {
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
```

Register the commands in `invoke_handler`:

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    // AI commands
    test_ai_provider,
    send_to_ai,
    get_builtin_roles,
])
```

---

### Step 1.6: Add TypeScript Types

**File**: `src/lib/types.ts`

Add the following types:

```typescript
// ============================================================================
// AI Module Types
// ============================================================================

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

export type AudioFormat = 'opus' | 'wav';

export type AiInputSource =
  | { type: 'clipboard' }
  | { type: 'recordAudio'; maxDurationMs: number; format: AudioFormat }
  | { type: 'processOutput' };
```

Update `AppSettings`:

```typescript
export interface AppSettings {
  startWithSystem: boolean;
  showTrayNotifications: boolean;
  ai?: AiSettings;
}
```

---

### Step 1.7: Add Command Wrappers

**File**: `src/lib/commands.ts`

Add the following functions:

```typescript
// ============================================================================
// AI Commands
// ============================================================================

export async function testAiProvider(apiKey: string, model?: string): Promise<boolean> {
  return invoke('test_ai_provider', { apiKey, model });
}

export async function sendToAi(
  apiKey: string,
  systemPrompt: string,
  userInput: string,
  model?: string
): Promise<string> {
  return invoke('send_to_ai', { apiKey, model, systemPrompt, userInput });
}

export async function getBuiltinRoles(): Promise<AiRole[]> {
  return invoke('get_builtin_roles');
}
```

---

### Step 1.8: Create AI Settings Component

**File**: `src/components/AiSettings.svelte`

```svelte
<script lang="ts">
  import type { AiSettings, AiProviderConfig, AiRole } from '$lib/types';
  import { testAiProvider, getBuiltinRoles } from '$lib/commands';

  interface Props {
    value: AiSettings;
    onChange: (settings: AiSettings) => void;
  }

  let { value, onChange }: Props = $props();

  let testStatus: 'idle' | 'testing' | 'success' | 'error' = $state('idle');
  let testError: string = $state('');
  let showApiKey: boolean = $state(false);

  // Initialize with defaults if empty
  $effect(() => {
    if (!value.providers || value.providers.length === 0) {
      onChange({
        ...value,
        providers: [
          {
            id: crypto.randomUUID(),
            providerType: 'gemini',
            apiKey: '',
            model: 'gemini-1.5-flash',
            enabled: true,
          },
        ],
      });
    }
  });

  async function handleTest() {
    const provider = value.providers?.[0];
    if (!provider?.apiKey) {
      testError = 'Please enter an API key';
      testStatus = 'error';
      return;
    }

    testStatus = 'testing';
    testError = '';

    try {
      await testAiProvider(provider.apiKey, provider.model);
      testStatus = 'success';
    } catch (e) {
      testStatus = 'error';
      testError = String(e);
    }
  }

  function updateProvider(updates: Partial<AiProviderConfig>) {
    const providers = [...(value.providers || [])];
    if (providers.length === 0) {
      providers.push({
        id: crypto.randomUUID(),
        providerType: 'gemini',
        apiKey: '',
        enabled: true,
      });
    }
    providers[0] = { ...providers[0], ...updates };
    onChange({ ...value, providers });
  }
</script>

<div class="space-y-6">
  <div>
    <h3 class="text-lg font-medium text-gray-900 mb-4">AI Provider</h3>

    <div class="space-y-4 p-4 bg-gray-50 rounded-lg">
      <div class="flex items-center gap-2">
        <span class="font-medium text-gray-700">Gemini</span>
        {#if testStatus === 'success'}
          <span class="text-green-600 text-sm">✓ Connected</span>
        {/if}
      </div>

      <div>
        <label for="api-key" class="block text-sm font-medium text-gray-700"> API Key </label>
        <div class="mt-1 flex gap-2">
          <input
            id="api-key"
            type={showApiKey ? 'text' : 'password'}
            value={value.providers?.[0]?.apiKey || ''}
            oninput={(e) => updateProvider({ apiKey: e.currentTarget.value })}
            placeholder="AIza..."
            class="flex-1 rounded-md border border-gray-300 px-3 py-2 text-sm"
          />
          <button
            type="button"
            onclick={() => (showApiKey = !showApiKey)}
            class="px-3 py-2 text-sm border rounded-md hover:bg-gray-100"
          >
            {showApiKey ? 'Hide' : 'Show'}
          </button>
          <button
            type="button"
            onclick={handleTest}
            disabled={testStatus === 'testing'}
            class="px-3 py-2 text-sm bg-primary-600 text-white rounded-md hover:bg-primary-700 disabled:opacity-50"
          >
            {testStatus === 'testing' ? 'Testing...' : 'Test'}
          </button>
        </div>
        {#if testError}
          <p class="mt-1 text-sm text-red-600">{testError}</p>
        {/if}
      </div>

      <div>
        <label for="model" class="block text-sm font-medium text-gray-700"> Model </label>
        <select
          id="model"
          value={value.providers?.[0]?.model || 'gemini-1.5-flash'}
          onchange={(e) => updateProvider({ model: e.currentTarget.value })}
          class="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2 text-sm"
        >
          <option value="gemini-1.5-flash">gemini-1.5-flash (Fast)</option>
          <option value="gemini-1.5-pro">gemini-1.5-pro (Quality)</option>
          <option value="gemini-2.0-flash-exp">gemini-2.0-flash-exp (Experimental)</option>
        </select>
      </div>
    </div>
  </div>

  <div>
    <h3 class="text-lg font-medium text-gray-900 mb-4">AI Roles</h3>
    <p class="text-sm text-gray-500 mb-2">
      Built-in roles for common AI tasks. Custom roles coming soon.
    </p>
    <div class="space-y-2">
      {#each value.roles || [] as role}
        <div class="p-3 bg-gray-50 rounded-md">
          <div class="font-medium text-sm">{role.name}</div>
          <div class="text-xs text-gray-500 truncate">{role.systemPrompt}</div>
        </div>
      {/each}
    </div>
  </div>
</div>
```

---

### Phase 1 Verification Checklist

- [ ] `cargo check` passes without errors
- [ ] `npm run check` passes without errors
- [ ] API key can be entered in settings
- [ ] "Test" button connects to Gemini API
- [ ] Built-in roles display in settings

---

## Phase 2: Audio Recording

**Goal**: Record audio from microphone with Opus encoding.

### Step 2.1: Add Audio Dependencies

**File**: `src-tauri/Cargo.toml`

```toml
[dependencies]
# ... existing ...

# Audio Recording
cpal = "0.15"           # Cross-platform audio I/O
ogg = "0.9"             # Ogg container format
opus = "0.3"            # Opus codec
hound = "3.5"           # WAV fallback
```

**Verification**: `cargo check`

---

### Step 2.2: Create Audio Module

**File structure**:

```
src-tauri/src/
├── audio/
│   ├── mod.rs
│   ├── recorder.rs
│   └── encoder.rs
```

**File**: `src-tauri/src/audio/mod.rs`

```rust
//! Audio recording module

pub mod encoder;
pub mod recorder;

pub use recorder::AudioRecorder;
```

**File**: `src-tauri/src/audio/recorder.rs`

```rust
//! Audio recording implementation using cpal

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crate::error::AppError;

pub struct AudioRecorder {
    samples: Arc<Mutex<Vec<f32>>>,
    is_recording: Arc<AtomicBool>,
    sample_rate: u32,
}

impl AudioRecorder {
    pub fn new() -> Result<Self, AppError> {
        Ok(Self {
            samples: Arc::new(Mutex::new(Vec::new())),
            is_recording: Arc::new(AtomicBool::new(false)),
            sample_rate: 16000, // Target sample rate for speech
        })
    }

    pub fn start(&self) -> Result<(), AppError> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| AppError::Audio("No input device found".to_string()))?;

        let config = device
            .default_input_config()
            .map_err(|e| AppError::Audio(format!("Failed to get config: {}", e)))?;

        self.is_recording.store(true, Ordering::SeqCst);
        let samples = Arc::clone(&self.samples);
        let is_recording = Arc::clone(&self.is_recording);

        let stream = device
            .build_input_stream(
                &config.into(),
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if is_recording.load(Ordering::SeqCst) {
                        if let Ok(mut samples) = samples.lock() {
                            samples.extend_from_slice(data);
                        }
                    }
                },
                |err| eprintln!("Audio stream error: {}", err),
                None,
            )
            .map_err(|e| AppError::Audio(format!("Failed to build stream: {}", e)))?;

        stream.play().map_err(|e| AppError::Audio(format!("Failed to start: {}", e)))?;

        // Keep stream alive - in real implementation, store in struct
        std::mem::forget(stream);

        Ok(())
    }

    pub fn stop(&self) -> Result<Vec<f32>, AppError> {
        self.is_recording.store(false, Ordering::SeqCst);

        let samples = self.samples.lock()
            .map_err(|_| AppError::Audio("Failed to lock samples".to_string()))?
            .clone();

        Ok(samples)
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}
```

**File**: `src-tauri/src/audio/encoder.rs`

```rust
//! Audio encoding to Opus format

use crate::error::AppError;

/// Encode PCM samples to Opus in Ogg container
pub fn encode_to_opus(samples: &[f32], sample_rate: u32) -> Result<Vec<u8>, AppError> {
    // Convert f32 to i16
    let samples_i16: Vec<i16> = samples
        .iter()
        .map(|&s| (s * 32767.0) as i16)
        .collect();

    // Create Opus encoder
    let mut encoder = opus::Encoder::new(
        sample_rate,
        opus::Channels::Mono,
        opus::Application::Voip,
    )
    .map_err(|e| AppError::Audio(format!("Failed to create encoder: {}", e)))?;

    // Encode in frames (20ms at 16kHz = 320 samples)
    let frame_size = (sample_rate / 50) as usize; // 20ms frames
    let mut output = Vec::new();
    let mut buffer = vec![0u8; 4000]; // Max opus frame size

    for chunk in samples_i16.chunks(frame_size) {
        if chunk.len() < frame_size {
            break; // Skip incomplete frame
        }

        let encoded_len = encoder
            .encode(chunk, &mut buffer)
            .map_err(|e| AppError::Audio(format!("Encoding failed: {}", e)))?;

        output.extend_from_slice(&buffer[..encoded_len]);
    }

    Ok(output)
}

/// Encode PCM samples to WAV format (fallback)
pub fn encode_to_wav(samples: &[f32], sample_rate: u32) -> Result<Vec<u8>, AppError> {
    use std::io::Cursor;

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut cursor = Cursor::new(Vec::new());
    {
        let mut writer = hound::WavWriter::new(&mut cursor, spec)
            .map_err(|e| AppError::Audio(format!("Failed to create WAV writer: {}", e)))?;

        for &sample in samples {
            let sample_i16 = (sample * 32767.0) as i16;
            writer.write_sample(sample_i16)
                .map_err(|e| AppError::Audio(format!("Failed to write sample: {}", e)))?;
        }

        writer.finalize()
            .map_err(|e| AppError::Audio(format!("Failed to finalize WAV: {}", e)))?;
    }

    Ok(cursor.into_inner())
}
```

---

### Step 2.3: Add Audio Error Variant

**File**: `src-tauri/src/error.rs`

```rust
#[error("Audio error: {0}")]
Audio(String),
```

---

### Step 2.4: Add Audio Commands

**File**: `src-tauri/src/lib.rs`

```rust
mod audio;  // Add module declaration

// Add state for recorder
use std::sync::Mutex;
use once_cell::sync::Lazy;

static AUDIO_RECORDER: Lazy<Mutex<Option<audio::AudioRecorder>>> =
    Lazy::new(|| Mutex::new(None));

#[tauri::command]
async fn start_audio_recording() -> Result<(), String> {
    let recorder = audio::AudioRecorder::new().map_err(|e| e.to_string())?;
    recorder.start().map_err(|e| e.to_string())?;

    let mut guard = AUDIO_RECORDER.lock().map_err(|e| e.to_string())?;
    *guard = Some(recorder);

    Ok(())
}

#[tauri::command]
async fn stop_audio_recording(format: String) -> Result<Vec<u8>, String> {
    let mut guard = AUDIO_RECORDER.lock().map_err(|e| e.to_string())?;
    let recorder = guard.take()
        .ok_or_else(|| "No active recording".to_string())?;

    let samples = recorder.stop().map_err(|e| e.to_string())?;
    let sample_rate = recorder.sample_rate();

    let encoded = match format.as_str() {
        "opus" => audio::encoder::encode_to_opus(&samples, sample_rate),
        "wav" => audio::encoder::encode_to_wav(&samples, sample_rate),
        _ => audio::encoder::encode_to_opus(&samples, sample_rate),
    }.map_err(|e| e.to_string())?;

    Ok(encoded)
}
```

Register commands:

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing ...
    start_audio_recording,
    stop_audio_recording,
])
```

---

### Phase 2 Verification Checklist

- [ ] `cargo check` passes
- [ ] Audio recording starts without error
- [ ] Audio stops and returns encoded data
- [ ] Opus encoding produces smaller files than WAV

---

## Phase 3: Post-Action Integration

**Goal**: Add `CallAi` as a post-action type.

### Step 3.1: Add CallAi PostActionType Variant

**File**: `src-tauri/src/config/schema.rs`

Update `PostActionType` enum:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum PostActionType {
    PasteClipboard,
    SimulateKeystroke { keystroke: Keystroke },
    Delay { delay_ms: u64 },
    /// Call AI with input and save response to clipboard
    CallAi {
        role_id: String,
        input_source: AiInputSource,
        #[serde(default)]
        provider_id: Option<String>,
    },
}
```

---

### Step 3.2: Update PostAction Executor

**File**: `src-tauri/src/postaction/executor.rs`

Add handling for `CallAi`:

```rust
use crate::ai::{GeminiProvider, AiProvider};
use crate::config::schema::AiInputSource;
use arboard::Clipboard;

// In execute_actions function:
PostActionType::CallAi { role_id, input_source, provider_id } => {
    execute_ai_action(role_id, input_source, provider_id).await?;
}

async fn execute_ai_action(
    role_id: &str,
    input_source: &AiInputSource,
    _provider_id: &Option<String>,
) -> Result<(), AppError> {
    // Load config to get API key and role
    let config = crate::config::manager::load_config()?;
    let ai_settings = &config.settings.ai;

    let provider = ai_settings.providers.first()
        .ok_or_else(|| AppError::Ai("No AI provider configured".to_string()))?;

    let role = ai_settings.roles.iter()
        .find(|r| r.id == role_id)
        .or_else(|| crate::ai::get_builtin_roles().iter().find(|r| r.id == role_id).cloned())
        .ok_or_else(|| AppError::Ai(format!("Role not found: {}", role_id)))?;

    let gemini = GeminiProvider::new(
        provider.api_key.clone(),
        provider.model.clone(),
    );

    let response = match input_source {
        AiInputSource::Clipboard => {
            let mut clipboard = Clipboard::new()
                .map_err(|e| AppError::Ai(format!("Clipboard error: {}", e)))?;
            let text = clipboard.get_text()
                .map_err(|e| AppError::Ai(format!("Failed to read clipboard: {}", e)))?;
            gemini.send_text(&role.system_prompt, &text).await?
        }
        AiInputSource::RecordAudio { max_duration_ms, format } => {
            // TODO: Implement audio recording flow
            return Err(AppError::Ai("Audio recording not yet implemented in post-actions".to_string()));
        }
        AiInputSource::ProcessOutput => {
            return Err(AppError::Ai("Process output not yet implemented".to_string()));
        }
    };

    // Save response to clipboard
    let mut clipboard = Clipboard::new()
        .map_err(|e| AppError::Ai(format!("Clipboard error: {}", e)))?;
    clipboard.set_text(&response.text)
        .map_err(|e| AppError::Ai(format!("Failed to set clipboard: {}", e)))?;

    Ok(())
}
```

---

### Step 3.3: Update TypeScript Types

**File**: `src/lib/types.ts`

Update `PostActionType`:

```typescript
export type PostActionType =
  | { type: 'pasteClipboard' }
  | { type: 'simulateKeystroke'; keystroke: Keystroke }
  | { type: 'delay'; delayMs: number }
  | { type: 'callAi'; roleId: string; inputSource: AiInputSource; providerId?: string };
```

---

### Step 3.4: Update PostActionEditor UI

**File**: `src/components/PostActionEditor.svelte`

Add `callAi` support to the component (extend existing `addAction` and rendering logic).

---

### Phase 3 Verification Checklist

- [ ] CallAi action can be added to post-actions
- [ ] Clipboard input source works
- [ ] AI response is saved to clipboard
- [ ] Error handling works for missing API key

---

## Phase 4: Configurable Roles

**Goal**: Built-in and custom roles system.

### Step 4.1: Initialize Built-in Roles

On app startup, merge built-in roles with user config:

```rust
// In config loading logic
if config.settings.ai.roles.is_empty() {
    config.settings.ai.roles = ai::get_builtin_roles();
}
```

### Step 4.2: Create Role Editor Component

**File**: `src/components/RoleEditor.svelte`

Create a dialog for editing custom roles with:

- Name input
- System prompt textarea
- Output format selector
- Save/Cancel buttons

### Step 4.3: Add Role CRUD Commands

```rust
#[tauri::command]
async fn save_ai_role(role: AiRole) -> Result<(), String>;

#[tauri::command]
async fn delete_ai_role(role_id: String) -> Result<(), String>;
```

---

### Phase 4 Verification Checklist

- [ ] Built-in roles appear on first load
- [ ] Custom roles can be created
- [ ] Roles can be selected in post-action editor
- [ ] Custom roles persist across restarts

---

## Testing Checkpoints

### Unit Tests

Create `src-tauri/src/ai/tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_roles_not_empty() {
        let roles = get_builtin_roles();
        assert!(!roles.is_empty());
        assert!(roles.iter().all(|r| r.is_builtin));
    }

    #[test]
    fn test_ai_settings_default() {
        let settings = AiSettings::default();
        assert!(settings.providers.is_empty());
        assert!(settings.roles.is_empty());
    }
}
```

### Integration Tests

1. Test Gemini API connection with valid key
2. Test audio recording produces valid output
3. Test CallAi post-action end-to-end

### Manual Testing

1. Configure API key → Test connection → Verify success
2. Create hotkey with CallAi action → Trigger → Verify clipboard
3. Test with different roles → Verify correct prompts used

---

## File Summary

### New Files

| File                               | Description          |
| ---------------------------------- | -------------------- |
| `src-tauri/src/ai/mod.rs`          | AI module exports    |
| `src-tauri/src/ai/provider.rs`     | Provider trait       |
| `src-tauri/src/ai/gemini.rs`       | Gemini client        |
| `src-tauri/src/ai/roles.rs`        | Built-in roles       |
| `src-tauri/src/audio/mod.rs`       | Audio module exports |
| `src-tauri/src/audio/recorder.rs`  | Recording logic      |
| `src-tauri/src/audio/encoder.rs`   | Opus/WAV encoding    |
| `src/components/AiSettings.svelte` | AI settings UI       |
| `src/components/RoleEditor.svelte` | Role editor dialog   |

### Modified Files

| File                                     | Changes               |
| ---------------------------------------- | --------------------- |
| `src-tauri/Cargo.toml`                   | Add dependencies      |
| `src-tauri/src/lib.rs`                   | Add modules, commands |
| `src-tauri/src/config/schema.rs`         | Add AI types          |
| `src-tauri/src/error.rs`                 | Add error variants    |
| `src-tauri/src/postaction/executor.rs`   | Handle CallAi         |
| `src/lib/types.ts`                       | Add AI types          |
| `src/lib/commands.ts`                    | Add AI commands       |
| `src/components/PostActionEditor.svelte` | Add CallAi option     |

---

## Dependencies Summary

```toml
# New dependencies for AI module
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
base64 = "0.22"
cpal = "0.15"
ogg = "0.9"
opus = "0.3"
hound = "3.5"
```
