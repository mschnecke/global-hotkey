<script lang="ts">
  import { onMount } from 'svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import { getVersion } from '@tauri-apps/api/app';
  import {
    getAutostart,
    setAutostart,
    getConfigLocation,
    changeConfigLocation,
  } from '$lib/commands';

  let launchAtStartup = $state(false);
  let loading = $state(true);
  let version = $state('');
  let configLocationPath = $state('');
  let changingLocation = $state(false);

  onMount(async () => {
    try {
      [launchAtStartup, version, configLocationPath] = await Promise.all([
        getAutostart(),
        getVersion(),
        getConfigLocation(),
      ]);
    } catch (e) {
      console.error('Failed to load settings:', e);
    } finally {
      loading = false;
    }
  });

  async function handleToggleAutostart() {
    const newValue = !launchAtStartup;
    try {
      await setAutostart(newValue);
      launchAtStartup = newValue;
    } catch (e) {
      console.error('Failed to toggle autostart:', e);
    }
  }

  async function handleChangeConfigLocation() {
    try {
      const path = await open({
        directory: true,
        title: 'Select Configuration Location',
      });
      if (path && typeof path === 'string') {
        changingLocation = true;
        await changeConfigLocation(path);
        configLocationPath = await getConfigLocation();
      }
    } catch (e) {
      console.error('Failed to change config location:', e);
    } finally {
      changingLocation = false;
    }
  }

  async function handleResetConfigLocation() {
    try {
      changingLocation = true;
      await changeConfigLocation(undefined);
      configLocationPath = await getConfigLocation();
    } catch (e) {
      console.error('Failed to reset config location:', e);
    } finally {
      changingLocation = false;
    }
  }
</script>

<div class="space-y-6">
  <div>
    <h3 class="text-lg font-medium text-gray-900">General Settings</h3>
    <p class="mt-1 text-sm text-gray-500">Configure application behavior</p>
  </div>

  <div class="space-y-4">
    <!-- Launch at Startup -->
    <div class="flex items-center justify-between">
      <div>
        <div class="text-sm font-medium text-gray-700">Launch at startup</div>
        <div class="text-xs text-gray-500">Start Global Hotkey when you log in</div>
      </div>
      <button
        onclick={handleToggleAutostart}
        disabled={loading}
        class="relative w-11 h-6 rounded-full transition-colors disabled:opacity-50 {launchAtStartup
          ? 'bg-primary-500'
          : 'bg-gray-300'}"
        aria-label="Toggle launch at startup"
      >
        <span
          class="absolute top-1 left-1 w-4 h-4 bg-white rounded-full transition-transform shadow {launchAtStartup
            ? 'translate-x-5'
            : ''}"
        ></span>
      </button>
    </div>

    <!-- Config Location -->
    <div class="pt-4 border-t border-gray-100">
      <div class="text-sm font-medium text-gray-700">Configuration location</div>
      <div class="text-xs text-gray-500 mb-2">Where hotkeys and AI settings are stored</div>
      <div class="flex items-center gap-2">
        <input
          type="text"
          readonly
          value={configLocationPath}
          class="flex-1 px-3 py-2 text-sm bg-gray-50 border border-gray-200 rounded-md text-gray-600"
        />
        <button
          onclick={handleChangeConfigLocation}
          disabled={loading || changingLocation}
          class="px-3 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 disabled:opacity-50"
        >
          {changingLocation ? 'Changing...' : 'Change'}
        </button>
        <button
          onclick={handleResetConfigLocation}
          disabled={loading || changingLocation}
          class="px-3 py-2 text-sm font-medium text-gray-500 bg-white border border-gray-300 rounded-md hover:bg-gray-50 disabled:opacity-50"
          title="Reset to default location"
        >
          Reset
        </button>
      </div>
    </div>
  </div>

  <!-- About -->
  <div class="pt-6 border-t border-gray-200">
    <div>
      <h3 class="text-lg font-medium text-gray-900">About</h3>
    </div>
    <div class="mt-2 text-sm text-gray-500">
      Global Hotkey v{version}
    </div>
  </div>
</div>
