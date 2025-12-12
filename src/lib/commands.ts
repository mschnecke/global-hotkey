/**
 * Tauri command wrappers
 */

import { invoke } from '@tauri-apps/api/core';
import type { AppConfig, HotkeyConfig, HotkeyBinding, ProgramConfig } from './types';

// ============================================================================
// Configuration Commands
// ============================================================================

/**
 * Get the current application configuration
 */
export async function getConfig(): Promise<AppConfig> {
  return invoke<AppConfig>('get_config');
}

/**
 * Save the application configuration
 */
export async function saveConfig(config: AppConfig): Promise<void> {
  return invoke('save_config', { config });
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
