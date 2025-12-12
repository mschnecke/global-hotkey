/**
 * App settings store
 */

import { writable } from 'svelte/store';
import type { AppSettings } from '$lib/types';
import * as commands from '$lib/commands';

const defaultSettings: AppSettings = {
  startWithSystem: false,
  showTrayNotifications: true,
};

export const settings = writable<AppSettings>(defaultSettings);
export const settingsLoading = writable<boolean>(false);
export const settingsError = writable<string | null>(null);

/**
 * Load settings from backend
 */
export async function loadSettings(): Promise<void> {
  settingsLoading.set(true);
  settingsError.set(null);
  try {
    const config = await commands.getConfig();
    settings.set(config.settings);
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
    commands
      .getConfig()
      .then((config) => {
        return commands.saveConfig({
          ...config,
          settings: updated,
        });
      })
      .catch((e) => {
        settingsError.set(e instanceof Error ? e.message : 'Failed to save settings');
      });

    return updated;
  });
}
