<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { save, open } from '@tauri-apps/plugin-dialog';
  import type { HotkeyConfig, AiSettings } from '$lib/types';
  import {
    hotkeys,
    loading,
    error,
    loadHotkeys,
    addHotkey,
    updateHotkey,
    deleteHotkey,
    toggleHotkey,
  } from '$stores/hotkeys';
  import { exportConfig, importConfig, updateTrayMenu, getConfig, saveConfig } from '$lib/commands';
  import HotkeyList from '$components/HotkeyList.svelte';
  import HotkeyDialog from '$components/HotkeyDialog.svelte';
  import ConfirmDialog from '$components/ConfirmDialog.svelte';
  import AiSettingsComponent from '$components/AiSettings.svelte';

  // Tab state
  type Tab = 'hotkeys' | 'ai';
  let activeTab = $state<Tab>('hotkeys');

  // AI Settings state
  let aiSettings = $state<AiSettings>({ providers: [], roles: [] });

  // Dialog state
  let dialogOpen = $state(false);
  let editingHotkey = $state<HotkeyConfig | null>(null);

  // Delete confirmation state
  let deleteDialogOpen = $state(false);
  let deletingHotkey = $state<HotkeyConfig | null>(null);

  // Load hotkeys, AI settings, and set up tray event listeners on mount
  onMount(() => {
    loadHotkeys();

    // Load AI settings
    getConfig().then((config) => {
      if (config.settings.ai) {
        aiSettings = config.settings.ai;
      }
    });

    // Listen for tray export event
    const unlistenExport = listen('tray-export', async () => {
      try {
        const path = await save({
          defaultPath: 'global-hotkey-config.json',
          filters: [{ name: 'JSON', extensions: ['json'] }],
        });
        if (path) {
          await exportConfig(path);
        }
      } catch (e) {
        console.error('Failed to export:', e);
        error.set(e instanceof Error ? e.message : 'Failed to export configuration');
      }
    });

    // Listen for tray import event
    const unlistenImport = listen('tray-import', async () => {
      try {
        const path = await open({
          multiple: false,
          filters: [{ name: 'JSON', extensions: ['json'] }],
        });
        if (path && typeof path === 'string') {
          await importConfig(path);
          await loadHotkeys();
          await updateTrayMenu();
        }
      } catch (e) {
        console.error('Failed to import:', e);
        error.set(e instanceof Error ? e.message : 'Failed to import configuration');
      }
    });

    // Cleanup listeners on unmount
    return () => {
      unlistenExport.then((fn) => fn());
      unlistenImport.then((fn) => fn());
    };
  });

  function handleAddClick() {
    editingHotkey = null;
    dialogOpen = true;
  }

  function handleEdit(hotkey: HotkeyConfig) {
    editingHotkey = hotkey;
    dialogOpen = true;
  }

  function handleDeleteClick(hotkey: HotkeyConfig) {
    deletingHotkey = hotkey;
    deleteDialogOpen = true;
  }

  async function handleSave(data: Omit<HotkeyConfig, 'id' | 'createdAt' | 'updatedAt'>) {
    try {
      if (editingHotkey) {
        await updateHotkey({
          ...editingHotkey,
          ...data,
        });
      } else {
        await addHotkey(data);
      }
      dialogOpen = false;
      editingHotkey = null;
    } catch (e) {
      console.error('Failed to save hotkey:', e);
    }
  }

  async function handleConfirmDelete() {
    if (!deletingHotkey) return;

    try {
      await deleteHotkey(deletingHotkey.id);
      deleteDialogOpen = false;
      deletingHotkey = null;
    } catch (e) {
      console.error('Failed to delete hotkey:', e);
    }
  }

  async function handleToggle(hotkey: HotkeyConfig) {
    try {
      await toggleHotkey(hotkey.id);
    } catch (e) {
      console.error('Failed to toggle hotkey:', e);
    }
  }

  async function handleAiSettingsChange(newSettings: AiSettings) {
    aiSettings = newSettings;
    try {
      const config = await getConfig();
      config.settings.ai = newSettings;
      await saveConfig(config);
    } catch (e) {
      console.error('Failed to save AI settings:', e);
      error.set(e instanceof Error ? e.message : 'Failed to save AI settings');
    }
  }
</script>

<main class="min-h-screen bg-gray-100">
  <!-- Header -->
  <header class="bg-white shadow">
    <div class="mx-auto max-w-5xl px-6 py-4">
      <div class="flex items-center justify-between">
        <div>
          <h1 class="text-xl font-bold text-gray-900">Global Hotkey</h1>
          <p class="text-sm text-gray-500">Configure keyboard shortcuts to launch programs</p>
        </div>
        {#if activeTab === 'hotkeys'}
          <button
            onclick={handleAddClick}
            class="inline-flex items-center rounded-md bg-primary-600 px-4 py-2 text-sm font-medium text-white shadow-sm transition-colors hover:bg-primary-700 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2"
          >
            <svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M12 4v16m8-8H4"
              />
            </svg>
            Add Hotkey
          </button>
        {/if}
      </div>
      <!-- Tabs -->
      <div class="mt-4 border-b border-gray-200">
        <nav class="-mb-px flex space-x-8">
          <button
            onclick={() => (activeTab = 'hotkeys')}
            class="whitespace-nowrap border-b-2 py-2 px-1 text-sm font-medium {activeTab ===
            'hotkeys'
              ? 'border-primary-500 text-primary-600'
              : 'border-transparent text-gray-500 hover:border-gray-300 hover:text-gray-700'}"
          >
            Hotkeys
          </button>
          <button
            onclick={() => (activeTab = 'ai')}
            class="whitespace-nowrap border-b-2 py-2 px-1 text-sm font-medium {activeTab === 'ai'
              ? 'border-primary-500 text-primary-600'
              : 'border-transparent text-gray-500 hover:border-gray-300 hover:text-gray-700'}"
          >
            AI Settings
          </button>
        </nav>
      </div>
    </div>
  </header>

  <!-- Content -->
  <div class="mx-auto max-w-5xl px-6 py-6">
    <!-- Error Alert -->
    {#if $error}
      <div class="mb-4 rounded-md bg-red-50 p-4">
        <div class="flex">
          <div class="flex-shrink-0">
            <svg class="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
              <path
                fill-rule="evenodd"
                d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
                clip-rule="evenodd"
              />
            </svg>
          </div>
          <div class="ml-3">
            <p class="text-sm font-medium text-red-800">{$error}</p>
          </div>
          <div class="ml-auto pl-3">
            <button
              onclick={() => error.set(null)}
              class="inline-flex rounded-md bg-red-50 p-1.5 text-red-500 hover:bg-red-100"
              aria-label="Dismiss error"
            >
              <svg class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                <path
                  fill-rule="evenodd"
                  d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
                  clip-rule="evenodd"
                />
              </svg>
            </button>
          </div>
        </div>
      </div>
    {/if}

    {#if activeTab === 'hotkeys'}
      <!-- Loading State -->
      {#if $loading}
        <div class="flex items-center justify-center py-12">
          <svg class="h-8 w-8 animate-spin text-primary-600" viewBox="0 0 24 24">
            <circle
              class="opacity-25"
              cx="12"
              cy="12"
              r="10"
              stroke="currentColor"
              stroke-width="4"
              fill="none"
            />
            <path
              class="opacity-75"
              fill="currentColor"
              d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
            />
          </svg>
          <span class="ml-2 text-gray-600">Loading hotkeys...</span>
        </div>
      {:else}
        <!-- Hotkey List -->
        <HotkeyList
          hotkeys={$hotkeys}
          onEdit={handleEdit}
          onDelete={handleDeleteClick}
          onToggle={handleToggle}
        />

        <!-- Stats -->
        {#if $hotkeys.length > 0}
          <div class="mt-4 text-sm text-gray-500">
            {$hotkeys.length} hotkey{$hotkeys.length === 1 ? '' : 's'} configured ({$hotkeys.filter(
              (h) => h.enabled
            ).length} enabled)
          </div>
        {/if}
      {/if}
    {:else if activeTab === 'ai'}
      <div class="rounded-lg bg-white p-6 shadow">
        <AiSettingsComponent value={aiSettings} onChange={handleAiSettingsChange} />
      </div>
    {/if}
  </div>

  <!-- Hotkey Dialog -->
  <HotkeyDialog
    open={dialogOpen}
    hotkey={editingHotkey}
    roles={aiSettings.roles || []}
    onSave={handleSave}
    onClose={() => {
      dialogOpen = false;
      editingHotkey = null;
    }}
  />

  <!-- Delete Confirmation Dialog -->
  <ConfirmDialog
    open={deleteDialogOpen}
    title="Delete Hotkey"
    message={`Are you sure you want to delete "${deletingHotkey?.name}"? This action cannot be undone.`}
    confirmText="Delete"
    variant="danger"
    onConfirm={handleConfirmDelete}
    onCancel={() => {
      deleteDialogOpen = false;
      deletingHotkey = null;
    }}
  />
</main>
