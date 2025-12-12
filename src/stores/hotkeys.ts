/**
 * Hotkey configuration store
 */

import { writable, derived, get } from 'svelte/store';
import type { HotkeyConfig } from '$lib/types';
import * as commands from '$lib/commands';
import { generateId, getTimestamp } from '$lib/utils';

// Main store for hotkeys
export const hotkeys = writable<HotkeyConfig[]>([]);

// Loading and error state
export const loading = writable<boolean>(false);
export const error = writable<string | null>(null);

// Derived stores
export const enabledHotkeys = derived(hotkeys, ($hotkeys) => $hotkeys.filter((h) => h.enabled));

export const hotkeyCount = derived(hotkeys, ($hotkeys) => $hotkeys.length);

/**
 * Load hotkeys from backend
 */
export async function loadHotkeys(): Promise<void> {
  loading.set(true);
  error.set(null);
  try {
    const config = await commands.getConfig();
    hotkeys.set(config.hotkeys);
  } catch (e) {
    const message = e instanceof Error ? e.message : 'Failed to load hotkeys';
    error.set(message);
    console.error('Failed to load hotkeys:', e);
  } finally {
    loading.set(false);
  }
}

/**
 * Save all hotkeys to backend and update tray menu
 */
async function saveAllHotkeys(): Promise<void> {
  try {
    const config = await commands.getConfig();
    config.hotkeys = get(hotkeys);
    await commands.saveConfig(config);

    // Update tray menu to reflect changes
    await commands.updateTrayMenu();
  } catch (e) {
    const message = e instanceof Error ? e.message : 'Failed to save hotkeys';
    error.set(message);
    throw e;
  }
}

/**
 * Add a new hotkey
 */
export async function addHotkey(
  config: Omit<HotkeyConfig, 'id' | 'createdAt' | 'updatedAt'>
): Promise<void> {
  const now = getTimestamp();
  const newHotkey: HotkeyConfig = {
    ...config,
    id: generateId(),
    createdAt: now,
    updatedAt: now,
  };

  try {
    // Register with system first
    if (newHotkey.enabled) {
      await commands.registerHotkey(newHotkey);
    }

    // Add to store
    hotkeys.update((list) => [...list, newHotkey]);

    // Save to disk
    await saveAllHotkeys();
  } catch (e) {
    const message = e instanceof Error ? e.message : 'Failed to add hotkey';
    error.set(message);
    throw e;
  }
}

/**
 * Update an existing hotkey
 */
export async function updateHotkey(config: HotkeyConfig): Promise<void> {
  const updated: HotkeyConfig = {
    ...config,
    updatedAt: getTimestamp(),
  };

  try {
    // Unregister old hotkey
    await commands.unregisterHotkey(config.id);

    // Register new hotkey if enabled
    if (updated.enabled) {
      await commands.registerHotkey(updated);
    }

    // Update store
    hotkeys.update((list) => list.map((h) => (h.id === config.id ? updated : h)));

    // Save to disk
    await saveAllHotkeys();
  } catch (e) {
    const message = e instanceof Error ? e.message : 'Failed to update hotkey';
    error.set(message);
    throw e;
  }
}

/**
 * Delete a hotkey
 */
export async function deleteHotkey(id: string): Promise<void> {
  try {
    // Unregister from system
    await commands.unregisterHotkey(id);

    // Remove from store
    hotkeys.update((list) => list.filter((h) => h.id !== id));

    // Save to disk
    await saveAllHotkeys();
  } catch (e) {
    const message = e instanceof Error ? e.message : 'Failed to delete hotkey';
    error.set(message);
    throw e;
  }
}

/**
 * Toggle hotkey enabled/disabled state
 */
export async function toggleHotkey(id: string): Promise<void> {
  const currentHotkeys = get(hotkeys);
  const hotkey = currentHotkeys.find((h) => h.id === id);

  if (!hotkey) return;

  const updated: HotkeyConfig = {
    ...hotkey,
    enabled: !hotkey.enabled,
    updatedAt: getTimestamp(),
  };

  try {
    if (updated.enabled) {
      // Register with system
      await commands.registerHotkey(updated);
    } else {
      // Unregister from system
      await commands.unregisterHotkey(id);
    }

    // Update store
    hotkeys.update((list) => list.map((h) => (h.id === id ? updated : h)));

    // Save to disk
    await saveAllHotkeys();
  } catch (e) {
    const message = e instanceof Error ? e.message : 'Failed to toggle hotkey';
    error.set(message);
    throw e;
  }
}

/**
 * Find a hotkey by ID
 */
export function findHotkey(id: string): HotkeyConfig | undefined {
  return get(hotkeys).find((h) => h.id === id);
}
