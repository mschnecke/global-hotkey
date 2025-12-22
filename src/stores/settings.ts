/**
 * App settings store
 */

import { writable } from 'svelte/store';
import type { AppSettings } from '$lib/types';
import * as commands from '$lib/commands';

const defaultSettings: AppSettings = {
  startWithSystem: false,
  showTrayNotifications: true,
  configLocation: undefined,
};

export const settings = writable<AppSettings>(defaultSettings);
export const settingsLoading = writable<boolean>(false);
export const settingsError = writable<string | null>(null);
export const configLocation = writable<string>('');

/**
 * Load settings from backend
 */
export async function loadSettings(): Promise<void> {
  settingsLoading.set(true);
  settingsError.set(null);
  try {
    const loadedSettings = await commands.getSettings();
    settings.set(loadedSettings);

    // Also load the actual config location path
    const location = await commands.getConfigLocation();
    configLocation.set(location);
  } catch (e) {
    settingsError.set(e instanceof Error ? e.message : 'Failed to load settings');
  } finally {
    settingsLoading.set(false);
  }
}

/**
 * Update settings
 */
export async function updateSettings(newSettings: Partial<AppSettings>): Promise<void> {
  settings.update((current) => {
    const updated = { ...current, ...newSettings };

    // Save to backend asynchronously
    commands.saveSettings(updated).catch((e) => {
      settingsError.set(e instanceof Error ? e.message : 'Failed to save settings');
    });

    return updated;
  });
}

/**
 * Change the config location
 */
export async function changeConfigLocation(newPath?: string): Promise<void> {
  try {
    await commands.changeConfigLocation(newPath);
    // Reload settings and config location
    await loadSettings();
  } catch (e) {
    settingsError.set(e instanceof Error ? e.message : 'Failed to change config location');
    throw e;
  }
}
