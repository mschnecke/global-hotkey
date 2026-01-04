/**
 * Tauri command wrappers
 */

import { invoke } from '@tauri-apps/api/core';
import type {
  AppConfig,
  AppSettings,
  FullConfig,
  HotkeyConfig,
  HotkeyBinding,
  ProgramConfig,
  AiRole,
} from './types';

// ============================================================================
// Configuration Commands
// ============================================================================

/**
 * Get the full configuration (settings + config)
 */
export async function getFullConfig(): Promise<FullConfig> {
  return invoke<FullConfig>('get_full_config');
}

/**
 * Save the full configuration (settings + config)
 */
export async function saveFullConfig(fullConfig: FullConfig): Promise<void> {
  return invoke('save_full_config', { fullConfig });
}

/**
 * Get the current application configuration (hotkeys + AI)
 */
export async function getConfig(): Promise<AppConfig> {
  return invoke<AppConfig>('get_config');
}

/**
 * Save the application configuration (hotkeys + AI)
 */
export async function saveConfig(config: AppConfig): Promise<void> {
  return invoke('save_config', { config });
}

/**
 * Get the current application settings
 */
export async function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>('get_settings');
}

/**
 * Save the application settings
 */
export async function saveSettings(settings: AppSettings): Promise<void> {
  return invoke('save_settings', { settings });
}

/**
 * Get the current config location path
 */
export async function getConfigLocation(): Promise<string> {
  return invoke<string>('get_config_location');
}

/**
 * Change the config location (copies existing config to new location)
 */
export async function changeConfigLocation(newPath?: string): Promise<void> {
  return invoke('change_config_location', { newPath });
}

/**
 * Export configuration to a file
 */
export async function exportConfig(path: string): Promise<void> {
  return invoke('export_config', { path });
}

/**
 * Import configuration from a file
 */
export async function importConfig(path: string): Promise<AppConfig> {
  return invoke<AppConfig>('import_config', { path });
}

// ============================================================================
// Hotkey Commands
// ============================================================================

/**
 * Register a hotkey with the system
 */
export async function registerHotkey(config: HotkeyConfig): Promise<void> {
  return invoke('register_hotkey', { config });
}

/**
 * Unregister a hotkey from the system
 */
export async function unregisterHotkey(id: string): Promise<void> {
  return invoke('unregister_hotkey', { id });
}

/**
 * Check if a hotkey binding conflicts with existing hotkeys
 */
export async function checkConflict(binding: HotkeyBinding): Promise<boolean> {
  return invoke<boolean>('check_conflict', { binding });
}

/**
 * Check if a hotkey binding conflicts with system hotkeys
 */
export async function checkSystemConflict(binding: HotkeyBinding): Promise<boolean> {
  return invoke<boolean>('check_system_conflict', { binding });
}

/**
 * Get list of currently registered hotkey IDs
 */
export async function getRegisteredHotkeys(): Promise<string[]> {
  return invoke<string[]>('get_registered_hotkeys');
}

// ============================================================================
// Process Commands
// ============================================================================

/**
 * Launch a program
 */
export async function launchProgram(config: ProgramConfig): Promise<void> {
  return invoke('launch_program', { config });
}

/**
 * Validate that a program path exists and is executable
 */
export async function validateProgramPath(path: string): Promise<boolean> {
  return invoke<boolean>('validate_program_path', { path });
}

/**
 * Get executable file extensions for the current platform
 */
export async function getExecutableExtensions(): Promise<string[]> {
  return invoke<string[]>('get_executable_extensions');
}

// ============================================================================
// System Tray Commands
// ============================================================================

/**
 * Update the system tray menu with current hotkeys
 */
export async function updateTrayMenu(): Promise<void> {
  return invoke('update_tray_menu');
}

/**
 * Check if autostart is enabled
 */
export async function getAutostart(): Promise<boolean> {
  return invoke<boolean>('get_autostart');
}

/**
 * Set autostart state
 */
export async function setAutostart(enabled: boolean): Promise<void> {
  return invoke('set_autostart', { enabled });
}

// ============================================================================
// AI Commands
// ============================================================================

/**
 * Test an AI provider connection
 */
export async function testAiProvider(apiKey: string, model?: string): Promise<boolean> {
  return invoke('test_ai_provider', { apiKey, model });
}

/**
 * Send text to AI and get response
 */
export async function sendToAi(
  apiKey: string,
  systemPrompt: string,
  userInput: string,
  model?: string
): Promise<string> {
  return invoke('send_to_ai', { apiKey, model, systemPrompt, userInput });
}

/**
 * Get built-in AI roles
 */
export async function getBuiltinRoles(): Promise<AiRole[]> {
  return invoke('get_builtin_roles');
}

/**
 * Save an AI role (create or update)
 */
export async function saveAiRole(role: AiRole): Promise<void> {
  return invoke('save_ai_role', { role });
}

/**
 * Delete an AI role
 */
export async function deleteAiRole(roleId: string): Promise<void> {
  return invoke('delete_ai_role', { roleId });
}

// ============================================================================
// Audio Commands
// ============================================================================

/**
 * Start audio recording from microphone
 */
export async function startAudioRecording(): Promise<void> {
  return invoke('start_audio_recording');
}

/**
 * Audio recording result with data and mime type
 */
export interface AudioRecordingResult {
  /** Base64-encoded audio data */
  data: string;
  /** MIME type (audio/ogg for Opus, audio/wav for fallback) */
  mime_type: string;
}

/**
 * Stop audio recording and get audio data as base64 with mime type
 * Uses Opus encoding by default (with WAV fallback)
 */
export async function stopAudioRecording(): Promise<AudioRecordingResult> {
  return invoke('stop_audio_recording');
}

/**
 * Check if currently recording audio
 */
export async function isAudioRecording(): Promise<boolean> {
  return invoke('is_audio_recording');
}

/**
 * Send audio to AI for transcription/processing
 */
export async function sendAudioToAi(
  apiKey: string,
  systemPrompt: string,
  audioBase64: string,
  mimeType?: string,
  model?: string
): Promise<string> {
  return invoke('send_audio_to_ai', { apiKey, model, systemPrompt, audioBase64, mimeType });
}
